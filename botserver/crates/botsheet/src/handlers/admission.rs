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

    fn gate(limit_mib: u64) -> MemoryAdmission {
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
        let gate = gate(4096);
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
        let gate = gate(1024);
        let result = gate.try_admit(WorkClass::SheetsImport, 25 * MIB);
        assert!(result.is_err());
        if let Err(error) = result {
            assert_eq!(reject(error).0, StatusCode::INSUFFICIENT_STORAGE);
        }
    }

    #[test]
    fn snapshot_exposes_ceiling_and_queue_state() {
        let gate = gate(4096);
        let snapshot = gate.snapshot();
        assert_eq!(snapshot.cgroup_limit_bytes, 4096 * MIB);
        assert_eq!(snapshot.request_budget_bytes, 1536 * MIB);
        assert!(admission_snapshot().cgroup_limit_bytes > 0);
    }

    #[tokio::test]
    async fn async_admission_maps_errors_without_leaking_content() {
        let gate = gate(4096);
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
}
