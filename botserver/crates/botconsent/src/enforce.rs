//! Enforcement core: decides between a stored grant, a fresh prompt and a
//! denial for every `(user, app, action_class)` attempt, and manages pending
//! consent requests with a five-minute TTL plus the allow-once handoff used
//! by command executors.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

use crate::models::{
    is_destructive_class, is_known_action_class, ConsentScope, Decision,
};
use crate::store;
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

fn ttl() -> Duration {
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

/// Entry point for executors: checks the effective grant and either allows,
/// creates a five-minute prompt or denies. Both audited paths land in
/// `consent_audit`. Unknown action classes fail closed.
pub async fn authorize(
    state: &Arc<ConsentService>,
    user_id: Uuid,
    app_id: &str,
    class: &str,
    detail: Value,
) -> ConsentDecision {
    authorize_with_context(
        state,
        user_id,
        app_id,
        class,
        detail,
        &ConsentContext::default(),
    )
    .await
}

/// Entry point with run/thread context. Destructive classes always prompt;
/// stored grants are checked against their scope.
pub async fn authorize_with_context(
    state: &Arc<ConsentService>,
    user_id: Uuid,
    app_id: &str,
    class: &str,
    detail: Value,
    context: &ConsentContext,
) -> ConsentDecision {
    if !is_known_action_class(class) {
        return deny_audited(state, user_id, app_id, class, &detail, context);
    }

    let request_json = audit_request(app_id, class, &detail, context);

    // A destructive action is consent-per-call by design: an "always" grant
    // never covers a delete or a payment.
    let destructive = is_destructive_class(class);
    let lookup = if destructive {
        Ok(None)
    } else {
        match state.pool.get() {
            Ok(mut conn) => {
                store::effective_grant_for(&mut conn, user_id, app_id, class, context)
            }
            Err(e) => {
                tracing::error!("consent authorize: pool unavailable: {e}");
                return ConsentDecision::Denied;
            }
        }
    };

    match lookup {
        Ok(Some(row)) => {
            let mut conn = match state.pool.get() {
                Ok(c) => c,
                Err(e) => {
                    tracing::error!("consent authorize audit: pool unavailable: {e}");
                    return ConsentDecision::Granted(row);
                }
            };
            store::audit(&mut conn, Some(row.id), Some(user_id), &request_json, store::OUTCOME_GRANTED, None);
            ConsentDecision::Granted(row)
        }
        Ok(None) => {
            let req = PendingRequest {
                request_id: Uuid::new_v4().to_string(),
                user_id,
                app_id: app_id.to_string(),
                action_class: class.to_string(),
                detail,
                thread_id: context.thread_id.clone(),
                run_id: context.run_id.clone(),
                risk_tier: crate::models::risk_tier(class).to_string(),
            };
            let mut pend = state.pending.lock().await;
            sweep_maps(&mut pend, &mut HashMap::new(), Instant::now());
            pend.insert(req.request_id.clone(), (req.clone(), Instant::now()));
            drop(pend);
            if let Ok(mut conn) = state.pool.get() {
                store::audit(
                    &mut conn,
                    None,
                    Some(user_id),
                    &request_json,
                    if destructive { store::OUTCOME_PENDING_DESTRUCTIVE } else { store::OUTCOME_PENDING },
                    None,
                );
            } else {
                tracing::error!("consent authorize audit: pool unavailable");
            }
            ConsentDecision::Pending(req)
        }
        Err(e) => {
            tracing::error!("consent authorize lookup failed: {e}");
            deny_audited(state, user_id, app_id, class, &detail, context)
        }
    }
}

fn audit_request(
    app_id: &str,
    class: &str,
    detail: &Value,
    context: &ConsentContext,
) -> Value {
    serde_json::json!({
        "app_id": app_id,
        "action_class": class,
        "risk_tier": crate::models::risk_tier(class),
        "app_hash": hash_text(app_id),
        "argument_hash": hash_text(&detail.to_string()),
        "thread_id": context.thread_id,
        "run_id": context.run_id,
        "workspace_id": context.workspace_id,
        "detail": detail,
    })
}

fn hash_text(text: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn deny_audited(
    state: &ConsentService,
    user_id: Uuid,
    app_id: &str,
    class: &str,
    detail: &Value,
    context: &ConsentContext,
) -> ConsentDecision {
    let request_json = audit_request(app_id, class, detail, context);
    if let Ok(mut conn) = state.pool.get() {
        store::audit(&mut conn, None, Some(user_id), &request_json, store::OUTCOME_DENIED, None);
    } else {
        tracing::error!("consent deny audit: pool unavailable");
    }
    ConsentDecision::Denied
}

/// Pure map-level transition shared by [`resolve`] and unit tests.
pub(crate) fn resolve_in_memory(
    pending: &mut PendingMap,
    approved_once: &mut PendingMap,
    request_id: &str,
    decision: Decision,
    now: Instant,
) -> Result<ResolvedOutcome, String> {
    let (req, born) = pending
        .remove(request_id)
        .ok_or_else(|| format!("unknown consent request '{request_id}'"))?;
    if now.duration_since(born) >= ttl() {
        return Err(format!("consent request '{request_id}' expired"));
    }
    match decision {
        Decision::AllowOnce => {
            approved_once.insert(request_id.to_string(), (req.clone(), now));
            Ok(ResolvedOutcome::ConsumeOnce(req))
        }
        Decision::AllowRun => Ok(ResolvedOutcome::PersistGrant(req, ConsentScope::Run)),
        Decision::AllowWorkspace => {
            Ok(ResolvedOutcome::PersistGrant(req, ConsentScope::Workspace))
        }
        Decision::Always => Ok(ResolvedOutcome::PersistGrant(req, ConsentScope::Always)),
        Decision::Deny => Ok(ResolvedOutcome::RecordDenial(req)),
    }
}

/// Applies the user decision to one pending request. The caller must own the
/// request; ownership is verified before any transition.
pub async fn resolve(
    state: &Arc<ConsentService>,
    request_id: &str,
    decision: Decision,
    user_id: Uuid,
) -> Result<ResolvedOutcome, String> {
    let outcome = {
        let mut pend = state.pending.lock().await;
        if let Some((req, _)) = pend.get(request_id) {
            if req.user_id != user_id {
                return Err(format!("unknown consent request '{request_id}'"));
            }
        }
        let mut appr = state.approved_once.lock().await;
        resolve_in_memory(&mut pend, &mut appr, request_id, decision, Instant::now())?
    };

    if let Ok(mut conn) = state.pool.get() {
        match &outcome {
            ResolvedOutcome::PersistGrant(req, scope) => {
                let expires_at = scope
                    .lifetime_secs()
                    .map(|seconds| chrono::Utc::now() + chrono::Duration::seconds(seconds));
                let scope_json = serde_json::json!({
                    "kind": scope.as_str(),
                    "thread_id": req.thread_id,
                    "run_id": req.run_id,
                });
                let spec = store::GrantSpec {
                    user_id,
                    app_id: &req.app_id,
                    action_class: &req.action_class,
                    scope: &scope_json,
                    granted_via: "prompt",
                    expires_at,
                };
                match store::grant(&mut conn, spec) {
                    Ok(row) => store::audit(
                        &mut conn, Some(row.id), Some(user_id), &req.detail,
                        store::OUTCOME_GRANTED, Some(user_id),
                    ),
                    Err(e) => tracing::error!("consent resolve grant failed: {e}"),
                }
            }
            ResolvedOutcome::ConsumeOnce(_) | ResolvedOutcome::RecordDenial(_) => {}
        }
        let outcome_label = match &outcome {
            ResolvedOutcome::PersistGrant(_, _) | ResolvedOutcome::ConsumeOnce(_) => store::OUTCOME_GRANTED,
            ResolvedOutcome::RecordDenial(_) => store::OUTCOME_DENIED,
        };
        store::audit(
            &mut conn, None, Some(user_id),
            &serde_json::json!({ "request_id": request_id }),
            outcome_label, Some(user_id),
        );
    } else {
        tracing::error!("consent resolve persistence: pool unavailable");
    }

    Ok(outcome)
}

/// Executor handoff for `allow_once`: removes and returns the approved
/// payload so the gated operation can run exactly one time.
pub async fn take_pending_approved(
    state: &Arc<ConsentService>,
    request_id: &str,
) -> Option<PendingRequest> {
    let mut appr = state.approved_once.lock().await;
    sweep_maps(&mut HashMap::new(), &mut appr, Instant::now());
    appr.remove(request_id).map(|(req, _)| req)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req(id: &str) -> PendingRequest {
        PendingRequest {
            request_id: id.to_string(),
            user_id: Uuid::nil(),
            app_id: "crm".to_string(),
            action_class: "create".to_string(),
            detail: serde_json::json!({ "name": "cmd" }),
            thread_id: None,
            run_id: None,
            risk_tier: "medium".to_string(),
        }
    }

    fn maps() -> (PendingMap, PendingMap) {
        (HashMap::new(), HashMap::new())
    }

    #[test]
    fn allow_once_marks_consumed_exactly_once() {
        let (mut p, mut a) = maps();
        p.insert("r1".into(), (req("r1"), Instant::now()));
        let out = resolve_in_memory(&mut p, &mut a, "r1", Decision::AllowOnce, Instant::now())
            .expect("transition ok");
        assert!(matches!(out, ResolvedOutcome::ConsumeOnce(_)));
        assert!(!p.contains_key("r1"));
        assert_eq!(
            take_from(&mut a, "r1").map(|r| r.request_id),
            Some("r1".to_string())
        );
        assert!(take_from(&mut a, "r1").is_none());
    }

    #[test]
    fn always_does_not_populate_approved_once() {
        let (mut p, mut a) = maps();
        p.insert("r2".into(), (req("r2"), Instant::now()));
        let out = resolve_in_memory(&mut p, &mut a, "r2", Decision::Always, Instant::now())
            .expect("transition ok");
        assert_eq!(out, ResolvedOutcome::PersistGrant(req("r2"), ConsentScope::Always));
        assert!(a.is_empty());
        assert!(p.is_empty());
    }

    #[test]
    fn deny_records_without_approval() {
        let (mut p, mut a) = maps();
        p.insert("r3".into(), (req("r3"), Instant::now()));
        let out = resolve_in_memory(&mut p, &mut a, "r3", Decision::Deny, Instant::now())
            .expect("transition ok");
        assert!(matches!(out, ResolvedOutcome::RecordDenial(_)));
        assert!(a.is_empty());
        assert!(p.is_empty());
    }

    #[test]
    fn unknown_and_expired_requests_are_rejected() {
        let (mut p, mut a) = maps();
        assert!(resolve_in_memory(&mut p, &mut a, "ghost", Decision::AllowOnce, Instant::now()).is_err());

        let stale_born = Instant::now().checked_sub(ttl()).unwrap_or_else(Instant::now);
        p.insert("old".into(), (req("old"), stale_born));
        assert!(resolve_in_memory(&mut p, &mut a, "old", Decision::AllowOnce, Instant::now()).is_err());
    }

    #[test]
    fn sweeper_drops_expired_entries_only() {
        let (mut p, mut a) = maps();
        let fresh = Instant::now();
        let stale = fresh.checked_sub(ttl()).unwrap_or(fresh);
        p.insert("fresh".into(), (req("fresh"), fresh));
        p.insert("stale".into(), (req("stale"), stale));
        a.insert("astale".into(), (req("astale"), stale));
        sweep_maps(&mut p, &mut a, fresh);
        assert!(p.contains_key("fresh"));
        assert!(!p.contains_key("stale"));
        assert!(!a.contains_key("astale"));
    }

    fn take_from(
        a: &mut PendingMap,
        id: &str,
    ) -> Option<PendingRequest> {
        a.remove(id).map(|(req, _)| req)
    }
}
