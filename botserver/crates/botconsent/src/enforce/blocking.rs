//! Blocking consent requests: the caller (an agent action in the person's
//! virtual machine) waits on a per-request channel that [`super::resolve`]
//! wakes, instead of the model retrying after a "continue" message.
//!
//! A blocking request carries `"blocking": true` in its detail, so the
//! interface knows not to send a continuation and an "allow once" is never
//! parked for [`super::take_approved_match`]: it covers the one waiting call.

use std::sync::Arc;
use std::time::Instant;

use serde_json::Value;
use tokio::sync::oneshot;
use uuid::Uuid;

use crate::models::{is_known_action_class, is_outside_vm_class, is_vm_class, Decision};
use crate::store;

use super::service::{ConsentContext, ConsentService, PendingRequest};

/// Detail key a caller uses for the redacted copy kept in `consent_audit`.
pub const AUDIT_KEY: &str = "audit";

/// Whether a pending request was raised by a caller that waits for it.
#[must_use]
pub fn is_blocking(req: &PendingRequest) -> bool {
    req.detail.get("blocking").and_then(Value::as_bool) == Some(true)
}

/// The audit copy of a detail: its redacted `audit` view when the caller
/// supplied one, the detail itself otherwise.
#[must_use]
pub fn audit_view(detail: &Value) -> Value {
    detail.get(AUDIT_KEY).cloned().unwrap_or_else(|| detail.clone())
}

/// Rejects a decision the request does not offer, before anything changes.
/// VM requests offer once / this conversation / everything (in-VM only and
/// never for a protected action) / deny; the conversation choices need a
/// thread.
pub(crate) fn check_offer(req: &PendingRequest, decision: Decision) -> Result<(), String> {
    let vm = is_vm_class(&req.action_class);
    let has_thread = req.thread_id.as_deref().is_some_and(|t| !t.is_empty());
    let protected = req.detail.get("protected").and_then(Value::as_bool) == Some(true);
    match decision {
        Decision::AllowOnce | Decision::Deny => Ok(()),
        Decision::AllowConversation if !has_thread => {
            Err("this choice needs a conversation".to_string())
        }
        Decision::AllowConversation if vm && protected => {
            Err("this action is asked every time".to_string())
        }
        Decision::AllowConversation => Ok(()),
        Decision::AllowAll
            if vm && has_thread && !protected && !is_outside_vm_class(&req.action_class) =>
        {
            Ok(())
        }
        Decision::AllowAll => Err("\"allow all\" is not offered for this request".to_string()),
        Decision::AllowRun | Decision::AllowWorkspace | Decision::Always if vm => {
            Err("this choice is not offered for virtual machine actions".to_string())
        }
        Decision::AllowRun | Decision::AllowWorkspace | Decision::Always => Ok(()),
    }
}

/// Raises a pending request that the caller waits for, without looking up a
/// stored grant (the caller already did, with its own rules). Returns the
/// request and the receiver woken by [`super::resolve`]. Unknown classes and
/// a request without a thread fail closed.
pub async fn raise_blocking(
    state: &Arc<ConsentService>,
    user_id: Uuid,
    app_id: &str,
    class: &str,
    mut detail: Value,
    context: &ConsentContext,
) -> Result<(PendingRequest, oneshot::Receiver<Decision>), String> {
    if !is_known_action_class(class) {
        return Err(format!("unknown action class '{class}'"));
    }
    if context.thread_id.as_deref().map_or(true, str::is_empty) {
        return Err("a blocking request needs a conversation".to_string());
    }
    if let Some(map) = detail.as_object_mut() {
        map.insert("blocking".to_string(), Value::Bool(true));
    }
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
    let (tx, rx) = oneshot::channel();
    state.waiters.lock().await.insert(req.request_id.clone(), tx);
    state
        .pending
        .lock()
        .await
        .insert(req.request_id.clone(), (req.clone(), Instant::now()));
    match state.pool.get() {
        Ok(mut conn) => {
            let audit = serde_json::json!({
                "request_id": req.request_id,
                "app_id": app_id,
                "action_class": class,
                "thread_id": context.thread_id,
                "run_id": context.run_id,
                "detail": audit_view(&req.detail),
            });
            store::audit(&mut conn, None, Some(user_id), &audit, store::OUTCOME_PENDING, None);
        }
        Err(e) => tracing::error!("consent blocking audit: pool unavailable: {e}"),
    }
    Ok((req, rx))
}

/// Withdraws a blocking request nobody answered (time limit, cancelled run):
/// the card can no longer be resolved and the waiter is dropped.
pub async fn withdraw(state: &Arc<ConsentService>, request_id: &str) -> bool {
    state.waiters.lock().await.remove(request_id);
    state.pending.lock().await.remove(request_id).is_some()
}

/// Withdraws every request of a conversation that was deleted: its cards can
/// no longer be answered, and each action still waiting on one is told no
/// (its waiter is dropped). Returns how many were pending.
pub async fn withdraw_thread(state: &Arc<ConsentService>, thread_id: &str) -> usize {
    let in_thread = |req: &PendingRequest| req.thread_id.as_deref() == Some(thread_id);
    let ids: Vec<String> = {
        let mut pend = state.pending.lock().await;
        let ids: Vec<String> = pend
            .iter()
            .filter(|(_, (req, _))| in_thread(req))
            .map(|(id, _)| id.clone())
            .collect();
        for id in &ids {
            pend.remove(id);
        }
        let mut appr = state.approved_once.lock().await;
        appr.retain(|_, (req, _)| !in_thread(req));
        ids
    };
    let mut waiters = state.waiters.lock().await;
    for id in &ids {
        waiters.remove(id);
    }
    ids.len()
}

/// Whether a blocking request outlived the turn that raised it, so it must be
/// withdrawn and its card removed. The rule: a request stays answerable while
/// its own turn runs, including when the person sends another message in the
/// meantime (that message is queued for the running turn). It ends when the
/// turn is stopped (`cancelled_now`, ignoring a stop flag that was already
/// set when the request was raised, which belongs to an earlier turn) or when
/// another turn became the conversation's current one (`current_run` differs
/// from `raised_in`: the person's new message replaced the turn).
#[must_use]
pub fn turn_superseded(
    raised_in: Option<&str>,
    current_run: Option<&str>,
    cancelled_now: bool,
    cancelled_at_raise: bool,
) -> bool {
    raised_in != current_run || (cancelled_now && !cancelled_at_raise)
}

/// Wakes the waiter of a resolved blocking request, if one is waiting.
pub(crate) async fn wake(state: &Arc<ConsentService>, request_id: &str, decision: Decision) {
    if let Some(tx) = state.waiters.lock().await.remove(request_id) {
        if tx.send(decision).is_err() {
            tracing::debug!("consent: waiter for {request_id} was already gone");
        }
    }
}
