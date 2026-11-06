use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use serde_json::Value;
use uuid::Uuid;

use crate::models::{is_destructive_class, is_known_action_class};
use crate::store;

use super::service::{sweep_maps, ConsentContext, ConsentDecision, ConsentService, PendingRequest};

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
