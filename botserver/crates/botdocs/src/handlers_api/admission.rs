//! #2530 — per-request memory admission for Docs import/export routes.
//!
//! Thin crate-local bridge to `botcore::memory_admission`. The gate is built
//! once from the boot-time cgroup snapshot and shared by every Docs route;
//! permits are RAII, so a dropped/disconnected request releases its slot
//! without any daemon task. Typed refusals are mapped to the stable 413/507/429
//! JSON contract here so handlers never invent their own limits.

use axum::{http::StatusCode, Json};
use botcore::{
    AdmissionError, AdmissionPermit, AdmissionSnapshot, CgroupMemory, CgroupSource,
    MemoryAdmission, MemoryPolicy, WorkClass,
};
use std::sync::OnceLock;

pub use botcore::WorkClass as DocsWorkClass;

const MIB: u64 = 1024 * 1024;

/// Process-wide gate. `from_system` is a one-shot cgroup read with a
/// conservative fallback; it never panics and never starts a background task.
static ADMISSION: OnceLock<MemoryAdmission> = OnceLock::new();

pub fn admission() -> &'static MemoryAdmission {
    ADMISSION.get_or_init(MemoryAdmission::from_system)
}

/// Content-free snapshot for a health/status route (#2530 Phase 3). A caller in
/// the API crate can aggregate this with the Sheets/Slides gates.
pub fn admission_snapshot() -> AdmissionSnapshot {
    admission().snapshot()
}

type Rejection = (StatusCode, Json<serde_json::Value>);

/// Maps a typed refusal to its documented HTTP status and JSON body.
fn reject(error: AdmissionError) -> Rejection {
    (error.status_code(), Json(error.json_body()))
}

/// Admits one operation against an explicit gate. `admit` passes the
/// process-wide gate; tests pass a synthetic one.
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

    fn make_gate(limit_mib: u64) -> MemoryAdmission {
        let ceiling = CgroupMemory {
            limit_bytes: limit_mib * MIB,
            current_bytes: None,
            source: CgroupSource::V2,
        };
        MemoryAdmission::new(MemoryPolicy::from_ceiling(ceiling))
    }

    #[test]
    fn over_upload_cap_maps_to_413() {
        let big_gate = make_gate(4096);
        let result = gate.try_admit(WorkClass::DocsImport, 26 * MIB);
        assert!(result.is_err(), "26 MiB must exceed the 25 MiB Docs cap");
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
        assert!(result.is_err(), "600 MiB peak must exceed a 384 MiB budget");
        if let Err(error) = result {
            let rejection = reject(error);
            assert_eq!(rejection.0, StatusCode::INSUFFICIENT_STORAGE);
        }
    }

    #[test]
    fn busy_class_maps_to_429_with_retry_hint() {
        let big_gate = make_gate(4096);
        let first = gate.try_admit(WorkClass::OfficeExport, MIB);
        assert!(first.is_ok());
        if let Ok(permit) = first {
            let second = gate.try_admit(WorkClass::OfficeExport, MIB);
            assert!(second.is_err());
            if let Err(error) = second {
                assert_eq!(error.status_code(), StatusCode::TOO_MANY_REQUESTS);
            }
            assert_eq!(permit.available_permits(), 0);
        }
    }

    #[test]
    fn snapshot_exposes_ceiling_and_queue_state() {
        let big_gate = make_gate(4096);
        let snapshot = gate.snapshot();
        assert_eq!(snapshot.cgroup_limit_bytes, 4096 * MIB);
        assert_eq!(snapshot.request_budget_bytes, 1536 * MIB);
        assert_eq!(snapshot.queued, [0; 6]);
        // The health accessor must be callable without a running server.
        let live = admission_snapshot();
        assert!(live.cgroup_limit_bytes > 0);
    }

    #[tokio::test]
    async fn async_admission_maps_errors_and_holds_a_permit() {
        let big_gate = make_gate(4096);
        let admitted = admit_on(&big_gate, WorkClass::DocsImport, 4 * MIB, Some("tenant-1")).await;
        assert!(admitted.is_ok());
        if let Ok(permit) = admitted {
            assert_eq!(permit.class(), WorkClass::DocsImport);
            assert_eq!(gate.snapshot().active[0], 1);
        }
        let refused = admit_on(&big_gate, WorkClass::DocsImport, 26 * MIB, Some("tenant-1")).await;
        assert!(refused.is_err());
        if let Err(rejection) = refused {
            assert_eq!(rejection.0, StatusCode::PAYLOAD_TOO_LARGE);
            let body = rejection.1 .0.to_string();
            assert!(body.contains("office_payload_too_large"));
            assert!(!body.contains("confidential-salary-sheet.docx"));
        }
    }

    /// Every variant must be mapped by `reject()` (not just by
    /// `error.status_code()`) to its documented status and stable error code.
    #[test]
    fn reject_maps_each_typed_error_to_413_507_429() {
        let cases = [
            (
                AdmissionError::PayloadTooLarge {
                    class: WorkClass::DocsImport,
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

    /// The tenant is passed into `admit_on` for the structured log only. None
    /// of the three refusal bodies may carry the tenant id, a document name or
    /// any document text. Only the allow-listed size/class keys may appear.
    #[tokio::test]
    async fn reject_bodies_never_carry_filename_body_or_tenant() {
        const SECRET_FILENAME: &str = "confidential-salary-sheet.docx";
        const SECRET_BODY: &str = "employee salaries are confidential";
        const SECRET_TENANT: &str = "tenant-acme-corp-007";

        let big_gate = make_gate(4096);
        let small_gate = make_gate(1024);

        let too_large = admit_on(&big_gate, WorkClass::DocsImport, 26 * MIB, Some(SECRET_TENANT)).await;
        let out_of_budget = admit_on(
            &small_gate,
            WorkClass::SheetsImport,
            25 * MIB,
            Some(SECRET_TENANT),
        )
        .await;
        // Hold the single OfficeExport slot so the second attempt is Busy.
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
                "document body leaked: {serialized}"
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
        // Keep the permit alive through the Busy attempt, then release it.
        drop(held);
    }

    /// The health snapshot must expose a positive, cgroup-derived budget: the
    /// gate is built once from the boot-time cgroup ceiling, never a constant.
    #[test]
    fn admission_snapshot_budget_is_positive_and_cgroup_derived() {
        let live = admission_snapshot();
        assert!(
            live.cgroup_limit_bytes > 0,
            "cgroup limit must be detected or fall back to a positive ceiling"
        );
        assert!(
            live.request_budget_bytes > 0,
            "request budget must be derived from a positive cgroup ceiling"
        );
        assert!(
            live.request_budget_bytes <= live.cgroup_limit_bytes,
            "a per-request budget cannot exceed the process ceiling"
        );
    }
}
