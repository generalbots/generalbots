use crate::auth::DocUser;
use crate::state::DocState;
use crate::storage::{load_document_from_drive, save_document};
use crate::types::{
    AcceptRejectAllRequest, AcceptRejectChangeRequest, EnableTrackChangesRequest,
    ListTrackChangesResponse,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use botcore::shared::schema::{rbac_groups, rbac_user_groups};
use chrono::Utc;
use diesel::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

// #2448 criterion 5 — `docs.track_changes.accept_all` is flagged `admin_only`
// in the command catalog, but its HTTP route reaches this handler directly and
// therefore bypasses the catalog gate. The role is resolved server-side from
// the SAME RBAC group membership the canonical `resolve_user_role` reads —
// never from a request body, a header or a caller-supplied role claim. The
// canonical `is_admin_group_name` lives in `botsecurity-auth`, which this
// crate cannot name without a Cargo.toml change (outside this fix's ownership),
// so its exact, full-string name set is mirrored below. Keep both in sync.
const ADMIN_GROUP_NAMES: &[&str] = &[
    "admin",
    "administrator",
    "superadmin",
    "super_admin",
    "super-admin",
    "super",
];

fn is_admin_group_name(name: &str) -> bool {
    let candidate = name.trim();
    ADMIN_GROUP_NAMES
        .iter()
        .any(|admin| candidate.eq_ignore_ascii_case(admin))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AdminAccess {
    Allowed,
    NonAdmin,
    Unresolved,
}

impl AdminAccess {
    /// Pure decision table over the resolved DB fact. `None` means the role
    /// could not be resolved and always denies; `Some(false)` is a regular
    /// non-admin caller.
    fn from_resolved(is_admin: Option<bool>) -> Self {
        match is_admin {
            Some(true) => Self::Allowed,
            Some(false) => Self::NonAdmin,
            None => Self::Unresolved,
        }
    }
}

/// Resolves admin membership from `rbac_user_groups` → `rbac_groups`, the role
/// source used by `resolve_user_role`. `None` means the lookup could not be
/// completed and the caller must be denied. `DocUser.id()` is the verified
/// principal id the auth middleware derived, not a client-supplied value.
fn resolve_admin(state: &DocState, user_id: &str) -> Option<bool> {
    let user_id = Uuid::parse_str(user_id).ok()?;
    let mut conn = state.pool.get().ok()?;
    let group_ids: Vec<Uuid> = rbac_user_groups::table
        .filter(rbac_user_groups::user_id.eq(user_id))
        .select(rbac_user_groups::group_id)
        .load(&mut conn)
        .ok()?;
    let names: Vec<String> = rbac_groups::table
        .filter(rbac_groups::id.eq_any(&group_ids))
        .filter(rbac_groups::is_active.eq(true))
        .select(rbac_groups::name)
        .load(&mut conn)
        .ok()?;
    Some(names.iter().any(|name| is_admin_group_name(name)))
}

/// The typed refusal already used by the Docs AST and export gates: a stable
/// machine-readable code and reason, never the document id, owner or a raw
/// error from the role lookup.
fn admin_denial(reason: &'static str) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::FORBIDDEN,
        Json(serde_json::json!({
            "error": "Document not found or access denied",
            "code": "docs_acl_denied",
            "reason": reason,
        })),
    )
}

pub async fn handle_enable_track_changes(
    State(state): State<Arc<DocState>>,
    user: DocUser,
    Json(req): Json<EnableTrackChangesRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = user.id().to_string();
    let mut doc = match load_document_from_drive(&state, &user_id, &req.doc_id).await {
        Ok(Some(d)) => d,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Document not found" })),
            ));
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    serde_json::json!({ "error": botsecurity_core::error_sanitizer::log_and_sanitize_json(&e, "botdocs::track_changes", None)}),
                ),
            ));
        }
    };

    doc.track_changes_enabled = req.enabled;
    doc.updated_at = Utc::now();

    if let Err(e) = save_document(&state, &user_id, &doc).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(
                serde_json::json!({ "error": botsecurity_core::error_sanitizer::log_and_sanitize_json(&e, "botdocs::track_changes", None)}),
            ),
        ));
    }

    Ok(Json(
        serde_json::json!({ "success": true, "enabled": req.enabled }),
    ))
}

pub async fn handle_accept_reject_change(
    State(state): State<Arc<DocState>>,
    user: DocUser,
    Json(req): Json<AcceptRejectChangeRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = user.id().to_string();
    let mut doc = match load_document_from_drive(&state, &user_id, &req.doc_id).await {
        Ok(Some(d)) => d,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Document not found" })),
            ));
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    serde_json::json!({ "error": botsecurity_core::error_sanitizer::log_and_sanitize_json(&e, "botdocs::track_changes", None)}),
                ),
            ));
        }
    };

    if let Some(changes) = &mut doc.track_changes {
        for change in changes.iter_mut() {
            if change.id == req.change_id {
                change.accepted = Some(req.accept);
                break;
            }
        }
    }

    doc.updated_at = Utc::now();
    if let Err(e) = save_document(&state, &user_id, &doc).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(
                serde_json::json!({ "error": botsecurity_core::error_sanitizer::log_and_sanitize_json(&e, "botdocs::track_changes", None)}),
            ),
        ));
    }

    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn handle_accept_reject_all(
    State(state): State<Arc<DocState>>,
    user: DocUser,
    Json(req): Json<AcceptRejectAllRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // #2448 criterion 5 — authorize BEFORE any document lookup. Previously a
    // non-admin reached the loader and got the same 404 as the owner, which
    // both leaked existence and never enforced the catalog's admin-only flag.
    // Resolving from RBAC also keeps the chat catalog path working: its signed
    // loopback assertion carries no role claim, but it maps to the same user.
    match AdminAccess::from_resolved(resolve_admin(&state, user.id())) {
        AdminAccess::Allowed => {}
        AdminAccess::NonAdmin => {
            log::warn!("docs track-changes accept-all denied: insufficient role");
            return Err(admin_denial("insufficient-role"));
        }
        AdminAccess::Unresolved => {
            log::warn!("docs track-changes accept-all denied: caller role unresolved");
            return Err(admin_denial("no-identity"));
        }
    }
    let user_id = user.id().to_string();
    let mut doc = match load_document_from_drive(&state, &user_id, &req.doc_id).await {
        Ok(Some(d)) => d,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Document not found" })),
            ));
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    serde_json::json!({ "error": botsecurity_core::error_sanitizer::log_and_sanitize_json(&e, "botdocs::track_changes", None)}),
                ),
            ));
        }
    };

    if let Some(changes) = &mut doc.track_changes {
        for change in changes.iter_mut() {
            change.accepted = Some(req.accept);
        }
    }

    doc.updated_at = Utc::now();
    if let Err(e) = save_document(&state, &user_id, &doc).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(
                serde_json::json!({ "error": botsecurity_core::error_sanitizer::log_and_sanitize_json(&e, "botdocs::track_changes", None)}),
            ),
        ));
    }

    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn handle_list_track_changes(
    State(state): State<Arc<DocState>>,
    user: DocUser,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ListTrackChangesResponse>, (StatusCode, Json<serde_json::Value>)> {
    let doc_id = match params.get("doc_id") {
        Some(value) => value.clone(),
        None => {
            log::warn!("[docs] track-changes list request without doc_id");
            String::new()
        }
    };
    let user_id = user.id().to_string();
    let doc = match load_document_from_drive(&state, &user_id, &doc_id).await {
        Ok(Some(d)) => d,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Document not found" })),
            ));
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    serde_json::json!({ "error": botsecurity_core::error_sanitizer::log_and_sanitize_json(&e, "botdocs::track_changes", None)}),
                ),
            ));
        }
    };

    let changes = match doc.track_changes {
        Some(changes) => changes,
        None => Vec::new(),
    };
    Ok(Json(ListTrackChangesResponse {
        changes,
        enabled: doc.track_changes_enabled,
    }))
}

#[cfg(test)]
mod admin_gate_tests {
    use super::*;

    /// An admin keeps the existing success path; the gate is not a blanket deny.
    /// Near misses must never acquire admin from the mirrored name set.
    #[test]
    fn admin_group_is_allowed_and_near_miss_is_not() {
        assert_eq!(
            AdminAccess::from_resolved(Some(true)),
            AdminAccess::Allowed
        );
        for name in ADMIN_GROUP_NAMES {
            assert!(is_admin_group_name(name), "admin name '{name}' must match");
        }
        for name in ["notadmin", "admintrainee", "admin-readonly", "user", ""] {
            assert!(
                !is_admin_group_name(name),
                "near-miss name '{name}' must not grant admin"
            );
        }
    }

    /// A non-admin gets the same typed 403 shape as the other Docs gates,
    /// never the loader's 404 and never a document id in the body.
    #[test]
    fn non_admin_gets_typed_denial() {
        assert_eq!(
            AdminAccess::from_resolved(Some(false)),
            AdminAccess::NonAdmin
        );
        let (status, body) = admin_denial("insufficient-role");
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(body.0["code"], "docs_acl_denied");
        assert_eq!(body.0["reason"], "insufficient-role");
        assert!(body.0.get("doc_id").is_none());
    }

    /// A failed or unparseable role lookup denies instead of falling back to
    /// "user" and leaking the document-existence distinction.
    #[test]
    fn unresolved_role_fails_closed() {
        assert_eq!(AdminAccess::from_resolved(None), AdminAccess::Unresolved);
        let (status, body) = admin_denial("no-identity");
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(body.0["code"], "docs_acl_denied");
        assert_eq!(body.0["reason"], "no-identity");
    }
}
