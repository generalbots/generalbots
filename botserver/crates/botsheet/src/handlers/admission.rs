//! #2530 — per-request memory admission for Sheets import/export routes.
//!
//! Thin crate-local bridge to `botcore::memory_admission`. One process-wide
//! gate is shared by every Sheets route; permits are RAII, so a disconnect
//! releases the slot. Typed refusals map to the stable 413/507/429 JSON body
//! here, never in the handler.

use axum::{http::StatusCode, Json};
use botcore::{
    AdmissionError, AdmissionPermit, AdmissionSnapshot, CgroupMemory, CgroupSource,
    MemoryAdmission, MemoryPolicy, WorkClass,
};
use botsheet_core::types::Spreadsheet;
use std::sync::OnceLock;

pub use botcore::WorkClass as SheetWorkClass;

const MIB: u64 = 1024 * 1024;

static ADMISSION: OnceLock<MemoryAdmission> = OnceLock::new();

pub fn admission() -> &'static MemoryAdmission {
    ADMISSION.get_or_init(MemoryAdmission::from_system)
}

/// Content-free snapshot for a health/status route (#2530 Phase 3).
pub fn admission_snapshot() -> AdmissionSnapshot {
    admission().snapshot()
}

type Rejection = (StatusCode, Json<serde_json::Value>);

fn reject(error: AdmissionError) -> Rejection {
    (error.status_code(), Json(error.json_body()))
}

/// Cheap in-memory estimate for an export: populated cells plus any retained
/// original package, both already resident in the session.
pub fn sheet_payload_bytes(sheet: &Spreadsheet) -> u64 {
    let cells: usize = sheet.worksheets.iter().map(|sheet| sheet.data.len()).sum();
    let retained = sheet.source_bytes.as_ref().map_or(0, |bytes| bytes.len());
    (cells as u64)
        .saturating_mul(128)
        .saturating_add(retained as u64)
}

async fn admit_on(
    gate: &MemoryAdmission,
    class: WorkClass,
    payload_bytes: u64,
    tenant: Option<&str>,
) -> Result<AdmissionPermit, Rejection> {
    gate.admit_for(class, payload_bytes, tenant)
        .await
        .map_err(reject)
}

pub async fn admit(
    class: WorkClass,
    payload_bytes: u64,
    tenant: Option<&str>,
) -> Result<AdmissionPermit, Rejection> {
    admit_on(admission(), class, payload_bytes, tenant).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use botsheet_core::types::Worksheet;

    fn make_gate(limit_mib: u64) -> MemoryAdmission {
        let ceiling = CgroupMemory {
            limit_bytes: limit_mib * MIB,
            current_bytes: None,
            source: CgroupSource::V2,
        };
        MemoryAdmission::new(MemoryPolicy::from_ceiling(ceiling))
    }

    fn sheet(cell_count: usize, retained: usize) -> Spreadsheet {
        let mut sheet = Spreadsheet {
            id: "sheet-1".to_string(),
            name: "Budget".to_string(),
            owner_id: "user-1".to_string(),
            worksheets: vec![Worksheet::default()],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            named_ranges: None,
            external_links: None,
            source_bucket: None,
            source_path: None,
            source_bytes: None,
            acl: std::collections::HashMap::new(),
        };
        for index in 0..cell_count {
            sheet.worksheets[0]
                .data
                .insert(format!("A{index}"), Default::default());
        }
        if retained > 0 {
            sheet.source_bytes = Some(vec![0; retained]);
        }
        sheet
    }

    #[test]
    fn estimate_counts_cells_and_retained_package() {
        assert_eq!(sheet_payload_bytes(&sheet(10, 0)), 1280);
        assert_eq!(sheet_payload_bytes(&sheet(10, 512)), 1792);
    }

    #[test]
    fn over_upload_cap_maps_to_413() {
        let gate = make_gate(4096);
        let result = gate.try_admit(WorkClass::SheetsImport, 26 * MIB);
        assert!(result.is_err());
        if let Err(error) = result {
            let rejection = reject(error);
            assert_eq!(rejection.0, StatusCode::PAYLOAD_TOO_LARGE);
            assert_eq!(rejection.1 .0["error"], "office_payload_too_large");
        }
    }

    #[test]
    fn small_ceiling_maps_peak_estimate_to_507() {
        let gate = make_gate(1024);
        let result = gate.try_admit(WorkClass::SheetsImport, 25 * MIB);
        assert!(result.is_err());
        if let Err(error) = result {
            assert_eq!(reject(error).0, StatusCode::INSUFFICIENT_STORAGE);
        }
    }

    #[test]
    fn snapshot_exposes_ceiling_and_queue_state() {
        let gate = make_gate(4096);
        let snapshot = gate.snapshot();
        assert_eq!(snapshot.cgroup_limit_bytes, 4096 * MIB);
        assert_eq!(snapshot.request_budget_bytes, 1536 * MIB);
        assert!(admission_snapshot().cgroup_limit_bytes > 0);
    }

    #[tokio::test]
    async fn async_admission_maps_errors_without_leaking_content() {
        let gate = make_gate(4096);
        let admitted = admit_on(&gate, WorkClass::OfficeExport, 4 * MIB, Some("tenant-1")).await;
        assert!(admitted.is_ok());
        if let Ok(permit) = admitted {
            assert_eq!(permit.class(), WorkClass::OfficeExport);
            assert_eq!(gate.snapshot().active[5], 1);
        }
        let refused = admit_on(&gate, WorkClass::SheetsImport, 26 * MIB, Some("tenant-1")).await;
        assert!(refused.is_err());
        if let Err(rejection) = refused {
            assert_eq!(rejection.0, StatusCode::PAYLOAD_TOO_LARGE);
            let body = rejection.1 .0.to_string();
            assert!(body.contains("office_payload_too_large"));
            assert!(!body.contains("salaries.xlsx"));
        }
    }

    /// Every typed refusal must be mapped by `reject()` — not merely by
    /// `error.status_code()` — to its documented HTTP status and stable code.
    /// This pins the 413 / 507 / 429 contract for Sheets callers in one place.
    #[test]
    fn reject_maps_each_typed_error_to_413_507_429() {
        let cases = [
            (
                AdmissionError::PayloadTooLarge {
                    class: WorkClass::SheetsImport,
                    bytes: 26 * MIB,
                    limit_bytes: 25 * MIB,
                },
                413,
                "office_payload_too_large",
            ),
            (
                AdmissionError::BudgetExceeded {
                    class: WorkClass::SheetsImport,
                    estimated_peak_bytes: 600 * MIB,
                    budget_bytes: 384 * MIB,
                },
                507,
                "office_memory_budget_exceeded",
            ),
            (
                AdmissionError::Busy {
                    class: WorkClass::OfficeExport,
                    queued: 1,
                },
                429,
                "office_busy",
            ),
        ];
        for (error, expected_status, expected_code) in cases {
            let rejection = reject(error);
            assert_eq!(rejection.0.as_u16(), expected_status);
            assert_eq!(rejection.1 .0["error"], expected_code);
            assert!(rejection.1 .0.get("i18n_key").is_some());
            assert!(rejection.1 .0.get("class").is_some());
        }
    }

    /// Holding the single export slot must produce a typed Busy refusal that
    /// `reject` maps to 429, and dropping the permit must release the slot.
    #[test]
    fn busy_export_maps_to_429_and_recovers_on_drop() {
        let gate = make_gate(4096);
        let held = gate.try_admit(WorkClass::OfficeExport, MIB);
        assert!(held.is_ok(), "first export must be admitted");
        if let Ok(permit) = held {
            assert_eq!(permit.available_permits(), 0);
            let second = gate.try_admit(WorkClass::OfficeExport, MIB);
            assert!(second.is_err(), "second export must be busy");
            if let Err(error) = second {
                let rejection = reject(error);
                assert_eq!(rejection.0, StatusCode::TOO_MANY_REQUESTS);
                assert_eq!(rejection.1 .0["error"], "office_busy");
                assert_eq!(rejection.1 .0["queued"], 0);
            }
            drop(permit);
        }
        assert!(gate.try_admit(WorkClass::OfficeExport, MIB).is_ok());
    }

    /// `sheet_payload_bytes` must grow linearly with the number of populated
    /// cells (128 bytes each) plus the retained source package, so a bigger
    /// sheet can never be underestimated.
    #[test]
    fn sheet_payload_estimate_scales_with_cell_count() {
        let empty = sheet_payload_bytes(&sheet(0, 0));
        let ten = sheet_payload_bytes(&sheet(10, 0));
        let twenty = sheet_payload_bytes(&sheet(20, 0));
        let forty = sheet_payload_bytes(&sheet(40, 0));
        assert_eq!(empty, 0);
        assert_eq!(ten, 10 * 128);
        assert_eq!(twenty, 2 * ten, "doubling cells must double the estimate");
        assert_eq!(forty, 2 * twenty);
        let with_package = sheet_payload_bytes(&sheet(40, 512));
        assert_eq!(with_package, forty + 512);
    }

    /// The health snapshot is content-free: it carries only numeric/enum
    /// counters. It must never expose a filename, tenant id or cell data, even
    /// when the caller passed a secret tenant to the gate.
    #[test]
    fn admission_snapshot_is_content_free() {
        const SECRET_FILENAME: &str = "confidential-salary-sheet.xlsx";
        const SECRET_TENANT: &str = "tenant-acme-corp-007";
        const SECRET_CELL: &str = "employee-salary-cell-value";

        let gate = make_gate(4096);
        let _ = gate.try_admit_for(WorkClass::SheetsImport, 4 * MIB, Some(SECRET_TENANT));
        let serialized = serde_json::to_value(&gate.snapshot()).expect("snapshot serializes");
        let text = serialized.to_string();
        for secret in [SECRET_FILENAME, SECRET_TENANT, SECRET_CELL] {
            assert!(!text.contains(secret), "snapshot leaked {secret}: {text}");
        }
        if let Some(object) = serialized.as_object() {
            for key in object.keys() {
                assert!(
                    matches!(
                        key.as_str(),
                        "cgroup_limit_bytes"
                            | "cgroup_source"
                            | "cgroup_current_bytes"
                            | "request_budget_bytes"
                            | "active"
                            | "queued"
                            | "admitted_total"
                            | "rejected_total"
                    ),
                    "unexpected snapshot field {key}"
                );
            }
        }
        // The public health accessor must also work without a running server.
        assert!(admission_snapshot().cgroup_limit_bytes > 0);
    }

    /// The tenant is passed into `admit_on` for the structured log only. None
    /// of the three refusal bodies may carry a filename, cell content or the
    /// tenant id — only the allow-listed size/class keys may appear.
    #[tokio::test]
    async fn reject_bodies_never_carry_filename_body_or_tenant() {
        const SECRET_FILENAME: &str = "confidential-salary-sheet.xlsx";
        const SECRET_BODY: &str = "employee salaries are confidential";
        const SECRET_TENANT: &str = "tenant-acme-corp-007";

        let gate = make_gate(4096);
        let small = make_gate(1024);
        let too_large = admit_on(
            &gate,
            WorkClass::SheetsImport,
            26 * MIB,
            Some(SECRET_TENANT),
        )
        .await;
        let out_of_budget = admit_on(
            &small,
            WorkClass::SheetsImport,
            25 * MIB,
            Some(SECRET_TENANT),
        )
        .await;
        let held = admit_on(&gate, WorkClass::OfficeExport, MIB, Some(SECRET_TENANT)).await;
        let busy = admit_on(&gate, WorkClass::OfficeExport, MIB, Some(SECRET_TENANT)).await;

        for result in [too_large, out_of_budget, busy] {
            let rejection = match result {
                Err(rejection) => rejection,
                Ok(_) => continue,
            };
            let body = rejection.1 .0;
            let serialized = body.to_string();
            assert!(
                !serialized.contains(SECRET_FILENAME),
                "filename leaked: {serialized}"
            );
            assert!(
                !serialized.contains(SECRET_BODY),
                "cell body leaked: {serialized}"
            );
            assert!(
                !serialized.contains(SECRET_TENANT),
                "tenant id leaked: {serialized}"
            );
            if let Some(object) = body.as_object() {
                for key in object.keys() {
                    assert!(
                        matches!(
                            key.as_str(),
                            "error"
                                | "i18n_key"
                                | "message"
                                | "class"
                                | "bytes"
                                | "limit_bytes"
                                | "estimated_peak_bytes"
                                | "budget_bytes"
                                | "queued"
                        ),
                        "unexpected field {key}"
                    );
                }
            }
        }
        drop(held);
    }
}
