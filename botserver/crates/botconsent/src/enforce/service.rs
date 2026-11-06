use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

use crate::models::ConsentScope;
use crate::DbPool;

/// How long a pending (or approved allow-once) request stays in memory.
pub const PENDING_TTL_SECS: u64 = 300;

/// The run/thread a tool call belongs to. Used to scope "this run" grants and
/// to correlate the audit trail with `run_events`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConsentContext {
    pub thread_id: Option<String>,
    pub run_id: Option<String>,
    pub workspace_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PendingRequest {
    pub request_id: String,
    pub user_id: Uuid,
    pub app_id: String,
    pub action_class: String,
    pub detail: Value,
    #[serde(default)]
    pub thread_id: Option<String>,
    #[serde(default)]
    pub run_id: Option<String>,
    #[serde(default)]
    pub risk_tier: String,
}

/// Pending requests keyed by request id, each paired with the instant it was
/// raised so the sweeper can expire it.
pub type PendingMap = HashMap<String, (PendingRequest, Instant)>;

pub enum ConsentDecision {
    Granted(crate::models::AppPermissionRow),
    Pending(PendingRequest),
    Denied,
}

/// What [`resolve_in_memory`] decided; the caller persists DB side effects.
#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedOutcome {
    ConsumeOnce(PendingRequest),
    PersistGrant(PendingRequest, ConsentScope),
    RecordDenial(PendingRequest),
}

pub struct ConsentService {
    pub pool: DbPool,
    pub(crate) pending: tokio::sync::Mutex<PendingMap>,
    pub(crate) approved_once: tokio::sync::Mutex<PendingMap>,
    sweeper_started: Arc<AtomicBool>,
}

impl ConsentService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            pool,
            pending: tokio::sync::Mutex::new(HashMap::new()),
            approved_once: tokio::sync::Mutex::new(HashMap::new()),
            sweeper_started: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Starts the background TTL sweeper exactly once per service instance.
    pub fn ensure_sweeper(self: &Arc<Self>) {
        if self.sweeper_started.swap(true, Ordering::SeqCst) {
            return;
        }
        let svc = Arc::clone(self);
        tokio::spawn(async move {
            let mut tick = tokio::time::interval(Duration::from_secs(60));
            loop {
                tick.tick().await;
                let mut pend = svc.pending.lock().await;
                let mut appr = svc.approved_once.lock().await;
                sweep_maps(&mut pend, &mut appr, Instant::now());
            }
        });
    }
}

pub(super) fn ttl() -> Duration {
    Duration::from_secs(PENDING_TTL_SECS)
}

pub(crate) fn sweep_maps(
    pending: &mut PendingMap,
    approved: &mut PendingMap,
    now: Instant,
) {
    let ttl = ttl();
    pending.retain(|_, (_, born)| now.duration_since(*born) < ttl);
    approved.retain(|_, (_, born)| now.duration_since(*born) < ttl);
}
