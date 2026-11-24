//! #2530 — per-request memory admission and import budget for Slides.
//!
//! Slides previously had no gate at all: a PPTX was opened as a zip and every
//! slide part was read into a `String` synchronously on the request path. This
//! module mirrors the Docs/Sheets bridge so all three apps share the same
//! cgroup-derived gate and the same pure, env-configurable import budget.

use axum::{http::StatusCode, Json};
use botcore::{
    AdmissionError, AdmissionPermit, AdmissionSnapshot, ImportBudget, MemoryAdmission, WorkClass,
};
use std::sync::OnceLock;

type Rejection = (StatusCode, Json<serde_json::Value>);

static ADMISSION: OnceLock<MemoryAdmission> = OnceLock::new();

pub fn admission() -> &'static MemoryAdmission {
    ADMISSION.get_or_init(MemoryAdmission::from_system)
}

/// Content-free snapshot for a health/status route (#2530 Phase 3).
pub fn admission_snapshot() -> AdmissionSnapshot {
    admission().snapshot()
}

fn reject(error: AdmissionError) -> Rejection {
    (error.status_code(), Json(error.json_body()))
}

/// Queuing admission; the permit is released by RAII on every exit path.
pub async fn admit(
    class: WorkClass,
    payload_bytes: u64,
    tenant: Option<&str>,
) -> Result<AdmissionPermit, Rejection> {
    admission()
        .admit_for(class, payload_bytes, tenant)
        .await
        .map_err(reject)
}

/// Resolves the shared, env-configurable import budget for `class` (#2530).
/// The byte cap starts from the endpoint's own upload limit and can only be
/// tightened by configuration, so the default preserves existing behaviour.
#[must_use]
pub fn import_budget(class: WorkClass) -> ImportBudget {
    ImportBudget::from_env(admission().policy().for_class(class).max_input_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use botcore::{CgroupMemory, CgroupSource, MemoryPolicy};

    fn gate(mib: u64) -> MemoryAdmission {
        MemoryAdmission::new(MemoryPolicy::from_ceiling(CgroupMemory {
            limit_bytes: mib * 1024 * 1024,
            current_bytes: None,
            source: CgroupSource::V2,
        }))
    }

    #[test]
    fn over_the_slides_upload_cap_maps_to_413() {
        let result = gate(4096).try_admit(WorkClass::SlidesImport, 51 * 1024 * 1024);
        assert!(result.is_err(), "51 MiB must exceed the 50 MiB Slides cap");
        if let Err(error) = result {
            let rejection = reject(error);
            assert_eq!(rejection.0, StatusCode::PAYLOAD_TOO_LARGE);
            assert_eq!(rejection.1 .0["error"], "office_payload_too_large");
        }
    }

    #[test]
    fn import_budget_never_exceeds_the_endpoint_cap() {
        let cap = admission()
            .policy()
            .for_class(WorkClass::SlidesImport)
            .max_input_bytes;
        assert!(import_budget(WorkClass::SlidesImport).max_bytes <= cap);
        assert!(admission_snapshot().request_budget_bytes > 0);
    }
}
