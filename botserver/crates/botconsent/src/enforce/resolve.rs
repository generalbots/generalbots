use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use uuid::Uuid;

use crate::models::{ConsentScope, Decision};
use crate::store;

use super::service::{sweep_maps, ttl, ConsentService, PendingMap, PendingRequest, ResolvedOutcome};

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
