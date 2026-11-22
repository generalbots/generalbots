use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use uuid::Uuid;

use crate::models::{ConsentScope, Decision};
use crate::store;

use super::blocking::{audit_view, check_offer, is_blocking, wake};
use super::service::{sweep_maps, ttl, ConsentService, PendingMap, PendingRequest, ResolvedOutcome};

/// Pure map-level transition shared by [`resolve`] and unit tests.
pub(crate) fn resolve_in_memory(
    pending: &mut PendingMap,
    approved_once: &mut PendingMap,
    request_id: &str,
    decision: Decision,
    now: Instant,
) -> Result<ResolvedOutcome, String> {
    if let Some((req, _)) = pending.get(request_id) {
        check_offer(req, decision)?;
    }
    let (req, born) = pending
        .remove(request_id)
        .ok_or_else(|| format!("unknown consent request '{request_id}'"))?;
    if now.duration_since(born) >= ttl() {
        return Err(format!("consent request '{request_id}' expired"));
    }
    match decision {
        Decision::AllowOnce => {
            // A blocking request is answered through its waiter; parking it
            // would let another call of the same class spend this approval.
            if !is_blocking(&req) {
                approved_once.insert(request_id.to_string(), (req.clone(), now));
            }
            Ok(ResolvedOutcome::ConsumeOnce(req))
        }
        Decision::AllowConversation => {
            Ok(ResolvedOutcome::PersistGrant(req, ConsentScope::Conversation))
        }
        Decision::AllowAll => {
            let mut all = req;
            all.action_class = crate::models::VM_ALL_CLASS.to_string();
            Ok(ResolvedOutcome::PersistGrant(all, ConsentScope::Conversation))
        }
        Decision::AllowRun => Ok(ResolvedOutcome::PersistGrant(req, ConsentScope::Run)),
        Decision::AllowWorkspace => {
            Ok(ResolvedOutcome::PersistGrant(req, ConsentScope::Workspace))
        }
        Decision::Always => Ok(ResolvedOutcome::PersistGrant(req, ConsentScope::Always)),
        Decision::Deny => Ok(ResolvedOutcome::RecordDenial(req)),
    }
}

/// Marker in the error of a decision refused because the caller does not own
/// the request; the HTTP route answers 403 for it.
pub const NOT_OWNER: &str = "not yours to answer";

/// Who answers a card: the signed-in person and, when the interface sends
/// it, the conversation the card is shown in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Resolver<'a> {
    pub user_id: Uuid,
    pub thread_id: Option<&'a str>,
}

/// Only the person a request belongs to may answer it, and a decision that
/// stores a conversation grant ("this conversation", "allow all") only from
/// the conversation that asked, when the caller names one. A refused caller
/// changes nothing: the request stays pending for its owner.
pub(crate) fn check_owner(
    req: &PendingRequest,
    decision: Decision,
    caller: Resolver<'_>,
) -> Result<(), String> {
    if req.user_id != caller.user_id {
        return Err(format!("consent request '{}' is {NOT_OWNER}", req.request_id));
    }
    let conversation_scoped = matches!(decision, Decision::AllowConversation | Decision::AllowAll);
    if let (true, Some(asked_in), Some(answered_in)) =
        (conversation_scoped, req.thread_id.as_deref(), caller.thread_id)
    {
        if asked_in != answered_in {
            return Err(format!(
                "consent request '{}' is {NOT_OWNER} from another conversation",
                req.request_id
            ));
        }
    }
    Ok(())
}

/// Applies the person's decision to one pending request. Ownership is
/// checked first ([`check_owner`]); a refusal leaves the request pending.
pub async fn resolve(
    state: &Arc<ConsentService>,
    request_id: &str,
    decision: Decision,
    caller: Resolver<'_>,
) -> Result<ResolvedOutcome, String> {
    let user_id = caller.user_id;
    let outcome = {
        let mut pend = state.pending.lock().await;
        if let Some((req, _)) = pend.get(request_id) {
            if let Err(refused) = check_owner(req, decision, caller) {
                tracing::warn!("consent: refused a decision on request {request_id}: not the owner");
                return Err(refused);
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
                // The grant belongs to the person whose agent asked: the gate
                // looks it up under that id.
                let spec = store::GrantSpec {
                    user_id: req.user_id,
                    app_id: &req.app_id,
                    action_class: &req.action_class,
                    scope: &scope_json,
                    granted_via: "prompt",
                    expires_at,
                };
                match store::grant(&mut conn, spec) {
                    Ok(row) => store::audit(
                        &mut conn, Some(row.id), Some(req.user_id), &audit_view(&req.detail),
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

    wake(state, request_id, decision).await;
    Ok(outcome)
}

/// Executor handoff for `allow_once` when the caller does not know the
/// request id (the chat gate re-checks on the model's retry): removes the
/// first approval matching this `(user, app, action_class)` so the gated
/// operation runs exactly once. Without this an "Allow once" click was
/// stored and never consumed, so every retry prompted again (#2203).
pub async fn take_approved_match(
    state: &Arc<ConsentService>,
    user_id: Uuid,
    app_id: &str,
    action_class: &str,
) -> Option<PendingRequest> {
    let mut appr = state.approved_once.lock().await;
    sweep_maps(&mut HashMap::new(), &mut appr, Instant::now());
    let key = appr
        .iter()
        .find(|(_, (req, _))| {
            req.user_id == user_id
                && req.app_id == app_id
                && req.action_class == action_class
        })
        .map(|(key, _)| key.clone());
    key.and_then(|key| appr.remove(&key).map(|(req, _)| req))
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
