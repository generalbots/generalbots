//! #1286 — per-project edit locks for parallel multi-chat sessions.
//!
//! Two chat tabs bound to the same project can both start LLM "modify"
//! runs. Without coordination their tool writes interleave (agent A
//! overwrites the file agent B just wrote) producing silent corruption.
//! This registry keeps ONE exclusive write slot per project:
//!
//! - The first modifying run acquires the slot and holds it for the whole
//!   run (held at spawn, released at terminal state).
//! - Further modifying runs WAIT FIFO (semaphore permit order) with a
//!   bounded timeout — they stay `pending` and observable while queued,
//!   never dropped silently (the #1275 lesson).
//! - Read-only/other-project runs are untouched: zero locking, fully
//!   parallel — that is the feature's core value.
//!
//! Lock timeouts are bounded and logged; contention never panics.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore, TryAcquireError};
use uuid::Uuid;

const LOG_TARGET: &str = "botvibe::project_locks";

/// How long a queued run may wait for the project's write slot before it
/// is failed with an explicit error (never silently dropped).
pub const LOCK_WAIT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(900);

#[derive(Default)]
pub struct ProjectLockRegistry {
    locks: Mutex<HashMap<String, Arc<ProjectLock>>>,
}

struct ProjectLock {
    /// One permit = the project's single write slot. FIFO fairness is
    /// guaranteed by tokio semaphores (waiters are woken in acquire order).
    /// Arc so guards can own an `OwnedSemaphorePermit` (no borrow of self).
    write_slot: Arc<Semaphore>,
    /// Who currently holds the slot (run_id), for introspection/telemetry.
    holder: Mutex<Option<String>>,
}

impl ProjectLock {
    fn new() -> Self {
        Self {
            write_slot: Arc::new(Semaphore::new(1)),
            holder: Mutex::new(None),
        }
    }
}

impl ProjectLockRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    fn lock_for(&self, locks: &mut HashMap<String, Arc<ProjectLock>>, project_id: &str) -> Arc<ProjectLock> {
        Arc::clone(locks.entry(project_id.to_string()).or_insert_with(|| Arc::new(ProjectLock::new())))
    }

    /// Wait (FIFO, bounded) for the project's exclusive write slot.
    /// Returns a guard that releases the slot on drop.
    pub async fn acquire(
        &self,
        project_id: &str,
        run_id: Uuid,
    ) -> Result<ProjectLockGuard, LockError> {
        if project_id.is_empty() {
            return Err(LockError::InvalidProject);
        }
        let lock = {
            let mut locks = self.locks.lock().await;
            self.lock_for(&mut locks, project_id)
        };
        ProjectLockGuard::acquire(lock, run_id).await
    }
    /// Whether a modifying run currently holds the project's write slot.
    pub async fn holder(&self, project_id: &str) -> Option<String> {
        let locks = self.locks.lock().await;
        let lock = locks.get(project_id)?;
        let holder = lock.holder.lock().await;
        holder.clone()
    }

    /// True when no run currently holds the project's write slot.
    pub async fn is_free(&self, project_id: &str) -> bool {
        self.holder(project_id).await.is_none()
    }
}

#[derive(Debug)]
pub enum LockError {
    /// Bounded wait exceeded — the run is failed explicitly, never dropped.
    Timeout,
    /// Registry closed while waiting (shutdown).
    Closed,
    /// Empty or invalid project key.
    InvalidProject,
}

impl std::fmt::Display for LockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LockError::Timeout => write!(f, "timed out waiting for the project edit lock"),
            LockError::Closed => write!(f, "lock registry closed"),
            LockError::InvalidProject => write!(f, "invalid project id"),
        }
    }
}

/// RAII guard: owns the project's write permit; dropping it releases the
/// slot and wakes the next FIFO waiter.
pub struct ProjectLockGuard {
    lock: Arc<ProjectLock>,
    permit: Option<tokio::sync::OwnedSemaphorePermit>,
    held_run: Option<Uuid>,
}

impl ProjectLockGuard {
    async fn acquire(lock: Arc<ProjectLock>, run_id: Uuid) -> Result<Self, LockError> {
        // try_acquire first so an uncontended path never touches the timer.
        let permit = match lock.write_slot.clone().try_acquire_owned() {
            Ok(p) => p,
            Err(TryAcquireError::NoPermits) => {
                log::info!(
                    target: LOG_TARGET,
                    "run {run_id} queued: another session is modifying this project",
                );
                let acquired = tokio::time::timeout(
                    LOCK_WAIT_TIMEOUT,
                    lock.write_slot.clone().acquire_owned(),
                )
                .await;
                match acquired {
                    Ok(Ok(p)) => p,
                    Ok(Err(_)) => return Err(LockError::Closed),
                    Err(_) => return Err(LockError::Timeout),
                }
            }
            Err(TryAcquireError::Closed) => return Err(LockError::Closed),
        };
        {
            let mut holder = lock.holder.lock().await;
            *holder = Some(run_id.to_string());
        }
        Ok(Self {
            lock,
            permit: Some(permit),
            held_run: Some(run_id),
        })
    }

    /// Run id currently holding the slot (introspection for tests/logs).
    pub fn held_run(&self) -> Option<Uuid> {
        self.held_run
    }
}

impl Drop for ProjectLockGuard {
    fn drop(&mut self) {
        // Releasing the permit first wakes the next FIFO waiter; the holder
        // bookkeeping is cleared under its lock (drop of the permit field
        // happens automatically when self is dropped).
        self.permit = None;
        let lock = Arc::clone(&self.lock);
        if let Some(run) = self.held_run.take() {
            tokio::spawn(async move {
                let mut holder = lock.holder.lock().await;
                if holder.as_deref() == Some(run.to_string().as_str()) {
                    *holder = None;
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn uncontended_acquire_is_immediate() {
        let reg = ProjectLockRegistry::new();
        let g = reg.acquire("proj-a", Uuid::new_v4()).await;
        assert!(g.is_ok());
    }

    #[tokio::test]
    async fn second_writer_queues_until_first_releases() {
        let reg = Arc::new(ProjectLockRegistry::new());
        let g1 = reg.acquire("proj", Uuid::new_v4()).await.expect("first hold");
        let reg2 = Arc::clone(&reg);
        let handle = tokio::spawn(async move {
            reg2.acquire("proj", Uuid::new_v4()).await.expect("second hold")
        });
        // Second is queued — give it a moment to block on the slot.
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        assert!(!handle.is_finished(), "queued run must wait for the slot");
        drop(g1);
        let g2 = tokio::time::timeout(std::time::Duration::from_secs(2), handle)
            .await
            .expect("release unblocks the queue")
            .expect("queued acquire succeeds");
        assert!(g2.held_run().is_some());
    }

    #[tokio::test]
    async fn different_projects_never_block_each_other() {
        let reg = ProjectLockRegistry::new();
        let _g1 = reg.acquire("proj-a", Uuid::new_v4()).await.expect("a");
        let _g2 = reg.acquire("proj-b", Uuid::new_v4()).await.expect("b");
    }

    #[tokio::test]
    async fn holder_reports_current_run() {
        let reg = ProjectLockRegistry::new();
        let run_id = Uuid::new_v4();
        let g = reg.acquire("proj", run_id).await.expect("hold");
        assert_eq!(reg.holder("proj").await, Some(run_id.to_string()));
        drop(g);
        // The release is spawned; poll briefly.
        for _ in 0..50 {
            if reg.holder("proj").await.is_none() {
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        panic!("holder not cleared after drop");
    }

    #[tokio::test]
    async fn empty_project_id_is_rejected() {
        let reg = ProjectLockRegistry::new();
        assert!(matches!(
            reg.acquire("", Uuid::new_v4()).await,
            Err(LockError::InvalidProject)
        ));
    }
}
