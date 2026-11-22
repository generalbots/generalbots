//! Bounded, per-class concurrency gate with RAII permits and observable queue.
//!
//! The gate never starts a task and never blocks on I/O: `try_admit` is for
//! request paths that must fail fast with 429, `admit_for` waits in a bounded
//! queue. Every refusal emits one structured event with only sizes, class,
//! reason and tenant id — never document content.

use super::cgroup::{CgroupMemory, CgroupSource};
use super::error::AdmissionError;
use super::policy::{MemoryPolicy, WorkClass};
use serde::Serialize;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

#[derive(Debug, Clone, Serialize)]
pub struct AdmissionSnapshot {
    pub cgroup_limit_bytes: u64,
    pub cgroup_source: CgroupSource,
    pub cgroup_current_bytes: Option<u64>,
    pub request_budget_bytes: u64,
    pub active: [usize; 6],
    pub queued: [usize; 6],
    pub admitted_total: u64,
    pub rejected_total: u64,
}

#[derive(Debug)]
struct Gate {
    semaphores: [Arc<Semaphore>; 6],
    active: [AtomicUsize; 6],
    queued: [AtomicUsize; 6],
    admitted_total: AtomicU64,
    rejected_total: AtomicU64,
}

/// Cancellation-safe queue accounting: a waiting future that is dropped (for
/// example when the client disconnects) must not leave a ghost in `queued`.
#[derive(Debug)]
struct QueueGuard {
    gate: Arc<Gate>,
    index: usize,
}

impl Drop for QueueGuard {
    fn drop(&mut self) {
        self.gate.queued[self.index].fetch_sub(1, Ordering::Relaxed);
    }
}

#[derive(Debug, Clone)]
pub struct MemoryAdmission {
    policy: MemoryPolicy,
    gate: Arc<Gate>,
}

impl MemoryAdmission {
    pub fn new(policy: MemoryPolicy) -> Self {
        let gate = Gate {
            semaphores: std::array::from_fn(|index| {
                Arc::new(Semaphore::new(policy.classes()[index].max_concurrency))
            }),
            active: std::array::from_fn(|_| AtomicUsize::new(0)),
            queued: std::array::from_fn(|_| AtomicUsize::new(0)),
            admitted_total: AtomicU64::new(0),
            rejected_total: AtomicU64::new(0),
        };
        Self {
            policy,
            gate: Arc::new(gate),
        }
    }

    /// Builds the guard from the boot-time cgroup snapshot.
    pub fn from_system() -> Self {
        Self::new(MemoryPolicy::from_ceiling(CgroupMemory::detect()))
    }

    pub fn policy(&self) -> &MemoryPolicy {
        &self.policy
    }

    /// Fail-fast admission. Use on handlers that cannot wait; a full class
    /// returns 429 with a `Retry-After` hint instead of piling up in memory.
    pub fn try_admit(
        &self,
        class: WorkClass,
        payload_bytes: u64,
    ) -> Result<AdmissionPermit, AdmissionError> {
        self.try_admit_for(class, payload_bytes, None)
    }

    pub fn try_admit_for(
        &self,
        class: WorkClass,
        payload_bytes: u64,
        tenant: Option<&str>,
    ) -> Result<AdmissionPermit, AdmissionError> {
        self.check_limits(class, payload_bytes, tenant)?;
        let index = class.index();
        let acquired = self.gate.semaphores[index].clone().try_acquire_owned();
        let permit = match acquired {
            Ok(permit) => permit,
            Err(_) => {
                let error = AdmissionError::Busy {
                    class,
                    queued: self.queued_for(class),
                };
                self.emit_rejection(&error, tenant);
                return Err(error);
            }
        };
        Ok(self.build_permit(class, permit))
    }

    /// Queuing admission. The queue depth is visible through `snapshot()` and
    /// `queued_for()` while callers wait, so a busy app can show its position.
    pub async fn admit_for(
        &self,
        class: WorkClass,
        payload_bytes: u64,
        tenant: Option<&str>,
    ) -> Result<AdmissionPermit, AdmissionError> {
        self.check_limits(class, payload_bytes, tenant)?;
        let index = class.index();
        self.gate.queued[index].fetch_add(1, Ordering::Relaxed);
        let queue_guard = QueueGuard {
            gate: Arc::clone(&self.gate),
            index,
        };
        let acquired = self.gate.semaphores[index].clone().acquire_owned().await;
        drop(queue_guard);
        let permit = match acquired {
            Ok(permit) => permit,
            Err(_) => {
                let error = AdmissionError::Busy {
                    class,
                    queued: self.queued_for(class),
                };
                self.emit_rejection(&error, tenant);
                return Err(error);
            }
        };
        Ok(self.build_permit(class, permit))
    }

    pub fn queued_for(&self, class: WorkClass) -> usize {
        self.gate.queued[class.index()].load(Ordering::Relaxed)
    }

    pub fn snapshot(&self) -> AdmissionSnapshot {
        AdmissionSnapshot {
            cgroup_limit_bytes: self.policy.ceiling.limit_bytes,
            cgroup_source: self.policy.ceiling.source,
            cgroup_current_bytes: self.policy.ceiling.current_bytes,
            request_budget_bytes: self.policy.budget_bytes,
            active: std::array::from_fn(|index| self.gate.active[index].load(Ordering::Relaxed)),
            queued: std::array::from_fn(|index| self.gate.queued[index].load(Ordering::Relaxed)),
            admitted_total: self.gate.admitted_total.load(Ordering::Relaxed),
            rejected_total: self.gate.rejected_total.load(Ordering::Relaxed),
        }
    }

    fn build_permit(&self, class: WorkClass, permit: OwnedSemaphorePermit) -> AdmissionPermit {
        let index = class.index();
        self.gate.active[index].fetch_add(1, Ordering::Relaxed);
        self.gate.admitted_total.fetch_add(1, Ordering::Relaxed);
        AdmissionPermit {
            class,
            gate: Arc::clone(&self.gate),
            started_at: Instant::now(),
            permit,
        }
    }

    fn check_limits(
        &self,
        class: WorkClass,
        payload_bytes: u64,
        tenant: Option<&str>,
    ) -> Result<(), AdmissionError> {
        let entry = self.policy.for_class(class);
        if payload_bytes > entry.max_input_bytes {
            let error = AdmissionError::PayloadTooLarge {
                class,
                bytes: payload_bytes,
                limit_bytes: entry.max_input_bytes,
            };
            self.emit_rejection(&error, tenant);
            return Err(error);
        }
        let estimated_peak = entry.estimated_peak_bytes(payload_bytes);
        if estimated_peak > self.policy.budget_bytes {
            let error = AdmissionError::BudgetExceeded {
                class,
                estimated_peak_bytes: estimated_peak,
                budget_bytes: self.policy.budget_bytes,
            };
            self.emit_rejection(&error, tenant);
            return Err(error);
        }
        Ok(())
    }

    fn emit_rejection(&self, error: &AdmissionError, tenant: Option<&str>) {
        self.gate.rejected_total.fetch_add(1, Ordering::Relaxed);
        let requested_bytes = match error {
            AdmissionError::PayloadTooLarge { bytes, .. } => *bytes,
            AdmissionError::BudgetExceeded {
                estimated_peak_bytes,
                ..
            } => *estimated_peak_bytes,
            AdmissionError::Busy { .. } => 0,
        };
        let tenant = match tenant {
            Some(value) => value,
            None => "unknown",
        };
        log::warn!(
            target: "gb.admission",
            "event=office_request_rejected class={} reason={} requested_bytes={} tenant={} — Office request refused by the per-request memory guard",
            error.class().as_str(),
            error.code(),
            requested_bytes,
            tenant
        );
    }
}

/// RAII handle for one admitted operation. Dropping it releases the semaphore
/// slot and records a content-free duration event.
#[derive(Debug)]
pub struct AdmissionPermit {
    class: WorkClass,
    gate: Arc<Gate>,
    started_at: Instant,
    permit: OwnedSemaphorePermit,
}

impl AdmissionPermit {
    pub fn class(&self) -> WorkClass {
        self.class
    }

    pub fn available_permits(&self) -> usize {
        self.permit.semaphore().available_permits()
    }
}

impl Drop for AdmissionPermit {
    fn drop(&mut self) {
        let index = self.class.index();
        self.gate.active[index].fetch_sub(1, Ordering::Relaxed);
        log::debug!(
            target: "gb.admission",
            "event=office_request_finished class={} duration_ms={} — admission permit released",
            self.class.as_str(),
            self.started_at.elapsed().as_millis()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::memory_admission::cgroup::FALLBACK_LIMIT_BYTES;
    use crate::shared::memory_admission::policy::MIB;
    use std::time::Duration;

    fn four_gib() -> MemoryAdmission {
        let ceiling = CgroupMemory {
            limit_bytes: 4 * 1024 * MIB,
            current_bytes: None,
            source: CgroupSource::V2,
        };
        MemoryAdmission::new(MemoryPolicy::from_ceiling(ceiling))
    }

    fn fallback() -> MemoryAdmission {
        let ceiling = CgroupMemory {
            limit_bytes: FALLBACK_LIMIT_BYTES,
            current_bytes: None,
            source: CgroupSource::Fallback,
        };
        MemoryAdmission::new(MemoryPolicy::from_ceiling(ceiling))
    }

    #[test]
    fn over_upload_cap_returns_413() {
        let admission = four_gib();
        let over = 26 * MIB;
        let result = admission.try_admit(WorkClass::DocsImport, over);
        assert!(result.is_err(), "26 MiB must exceed the 25 MiB Docs cap");
        if let Err(error) = result {
            assert_eq!(error.status_code().as_u16(), 413);
        }
        assert_eq!(admission.snapshot().rejected_total, 1);
        assert_eq!(admission.snapshot().admitted_total, 0);
    }

    #[test]
    fn small_ceiling_refuses_a_peak_that_a_laptop_admits() {
        let laptop = four_gib();
        assert!(laptop.try_admit(WorkClass::SheetsImport, 25 * MIB).is_ok());

        let small = fallback();
        let result = small.try_admit(WorkClass::SheetsImport, 25 * MIB);
        assert!(result.is_err(), "600 MiB peak must exceed a 384 MiB budget");
        if let Err(error) = result {
            assert_eq!(
                error,
                AdmissionError::BudgetExceeded {
                    class: WorkClass::SheetsImport,
                    estimated_peak_bytes: 25 * MIB * 24,
                    budget_bytes: 384 * MIB,
                }
            );
            assert_eq!(error.status_code().as_u16(), 507);
        }
    }

    #[test]
    fn busy_class_returns_429_then_recovers_on_drop() {
        let admission = four_gib();
        let first = admission.try_admit(WorkClass::SlidesImport, 10 * MIB);
        assert!(first.is_ok(), "first Slides import must be admitted");
        if let Ok(permit) = first {
            assert_eq!(permit.available_permits(), 0);
            let second = admission.try_admit(WorkClass::SlidesImport, 10 * MIB);
            assert!(second.is_err(), "second Slides import must be busy");
            if let Err(error) = second {
                assert_eq!(error.status_code().as_u16(), 429);
            }
            assert_eq!(admission.snapshot().active[2], 1);
            drop(permit);
        }
        assert_eq!(admission.snapshot().active[2], 0);
        assert!(admission
            .try_admit(WorkClass::SlidesImport, 10 * MIB)
            .is_ok());
    }

    #[test]
    fn classes_do_not_share_permits() {
        let admission = four_gib();
        let docs = admission.try_admit(WorkClass::DocsImport, 10 * MIB);
        assert!(docs.is_ok(), "Docs import must be admitted");
        assert!(admission
            .try_admit(WorkClass::SheetsImport, 10 * MIB)
            .is_ok());
        drop(docs);
    }

    #[test]
    fn snapshot_exposes_cgroup_and_queue_state() {
        let admission = fallback();
        let snapshot = admission.snapshot();
        assert_eq!(snapshot.cgroup_limit_bytes, FALLBACK_LIMIT_BYTES);
        assert_eq!(snapshot.cgroup_source, CgroupSource::Fallback);
        assert_eq!(snapshot.request_budget_bytes, 384 * MIB);
        assert_eq!(snapshot.queued, [0; 6]);
        assert_eq!(snapshot.active, [0; 6]);
    }

    #[tokio::test]
    async fn queued_waiter_is_observable_and_cancellation_safe() {
        let ceiling = CgroupMemory {
            limit_bytes: 4 * 1024 * MIB,
            current_bytes: None,
            source: CgroupSource::V2,
        };
        let mut policy = MemoryPolicy::from_ceiling(ceiling);
        let index = WorkClass::DocsImport.index();
        policy.classes[index].max_concurrency = 0;
        let admission = Arc::new(MemoryAdmission::new(policy));

        let waiting = {
            let admission = Arc::clone(&admission);
            tokio::spawn(async move { admission.admit_for(WorkClass::DocsImport, 1, None).await })
        };
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert_eq!(admission.queued_for(WorkClass::DocsImport), 1);
        assert_eq!(admission.snapshot().queued[0], 1);

        waiting.abort();
        match waiting.await {
            Ok(_) => {}
            Err(_) => {}
        }
        assert_eq!(admission.queued_for(WorkClass::DocsImport), 0);
        assert_eq!(admission.snapshot().rejected_total, 0);
    }

    #[tokio::test]
    async fn async_admission_succeeds_when_a_slot_is_free() {
        let admission = four_gib();
        let result = admission
            .admit_for(WorkClass::OfficeJsonSave, MIB, Some("tenant-1"))
            .await;
        assert!(result.is_ok(), "free JSON save slot must be admitted");
        if let Ok(permit) = result {
            assert_eq!(permit.class(), WorkClass::OfficeJsonSave);
            assert_eq!(admission.snapshot().active[4], 1);
            drop(permit);
        }
        assert_eq!(admission.snapshot().active[4], 0);
    }
}
