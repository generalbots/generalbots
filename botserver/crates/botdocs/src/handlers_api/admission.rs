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

    fn gate(limit_mib: u64) -> MemoryAdmission {
        let ceiling = CgroupMemory {
            limit_bytes: limit_mib * MIB,
            current_bytes: None,
            source: CgroupSource::V2,
        };
        MemoryAdmission::new(MemoryPolicy::from_ceiling(ceiling))
    }

    #[test]
    fn over_upload_cap_maps_to_413() {
        let gate = gate(4096);
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
        let gate = gate(1024);
        let result = gate.try_admit(WorkClass::SheetsImport, 25 * MIB);
        assert!(result.is_err(), "600 MiB peak must exceed a 384 MiB budget");
        if let Err(error) = result {
            let rejection = reject(error);
            assert_eq!(rejection.0, StatusCode::INSUFFICIENT_STORAGE);
        }
    }

    #[test]
    fn busy_class_maps_to_429_with_retry_hint() {
        let gate = gate(4096);
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
        let gate = gate(4096);
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
        let gate = gate(4096);
        let admitted = admit_on(&gate, WorkClass::DocsImport, 4 * MIB, Some("tenant-1")).await;
        assert!(admitted.is_ok());
        if let Ok(permit) = admitted {
            assert_eq!(permit.class(), WorkClass::DocsImport);
            assert_eq!(gate.snapshot().active[0], 1);
        }
        let refused = admit_on(&gate, WorkClass::DocsImport, 26 * MIB, Some("tenant-1")).await;
        assert!(refused.is_err());
        if let Err(rejection) = refused {
            assert_eq!(rejection.0, StatusCode::PAYLOAD_TOO_LARGE);
            let body = rejection.1 .0.to_string();
            assert!(body.contains("office_payload_too_large"));
            assert!(!body.contains("confidential-salary-sheet.docx"));
        }
    }
}
