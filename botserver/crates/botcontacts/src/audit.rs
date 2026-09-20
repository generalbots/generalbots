//! #1441 P2 — CRM audit trail writer.
//!
//! Every destructive or state-changing CRM operation (delete, bulk action,
//! stage transition, lead conversion) appends one row to `crm_audit_logs`
//! carrying the actor (JWT email claim), the entity, the action, and optional
//! before/after snapshots. Writes are best-effort: an audit failure is logged
//! and never blocks the business operation it describes.

use axum::http::HeaderMap;
use diesel::prelude::*;
use std::sync::Arc;

use crate::models::CrmAuditLog;
use crate::schema::crm_audit_logs;
use crate::scope::{email_from_jwt, email_from_session};
use crate::CrateState;

/// Actor identity resolved from the request token. The suite authenticates
/// with either a JWT (carrying an `email` claim) or an opaque session token
/// (resolved through the session cache by `email_from_session`); anonymous
/// callers are recorded as `anonymous`.
pub fn actor_from_headers(headers: &HeaderMap) -> Option<String> {
    email_from_jwt(headers).or_else(|| email_from_session(headers))
}

/// Inserts one audit row. Errors are logged, never propagated — the calling
/// handler has usually already committed the business change, and losing the
/// response to an audit failure would hide the real outcome from the user.
pub fn record(
    state: &Arc<CrateState>,
    headers: &HeaderMap,
    branch_id: uuid::Uuid,
    entity: &str,
    entity_id: Option<uuid::Uuid>,
    action: &str,
    before: Option<serde_json::Value>,
    after: Option<serde_json::Value>,
    detail: Option<serde_json::Value>,
) {
    let Ok(mut conn) = state.db_pool.get() else {
        log::warn!("[crm-audit] pool unavailable; audit row for {entity}/{action} dropped");
        return;
    };
    let row = CrmAuditLog {
        id: uuid::Uuid::new_v4(),
        branch_id,
        entity: entity.to_string(),
        entity_id,
        action: action.to_string(),
        actor_email: actor_from_headers(headers),
        before,
        after,
        detail,
        created_at: chrono::Utc::now(),
    };
    if let Err(e) = diesel::insert_into(crm_audit_logs::table)
        .values(&row)
        .execute(&mut conn)
    {
        log::warn!("[crm-audit] insert failed for {entity}/{action}: {e}");
    }
}
