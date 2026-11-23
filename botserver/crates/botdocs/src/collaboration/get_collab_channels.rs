//! Presence/collaboration HTTP + WebSocket handlers (#2733).
//!
//! The WebSocket used to verify only `?token=` and then join whatever
//! `doc_id` the caller put in the path. That proves *who* connects, never
//! *which document* they may join, so any authenticated user could read every
//! presence/edit broadcast and inject edits into a foreign document. Every
//! upgrade now goes through [`authorize_doc_ws`] first.

use super::*;
use crate::auth::DocUser;
use crate::legacy::is_legacy_owner;
use crate::state::DocState;
use axum::http::StatusCode;
use diesel::{sql_query, sql_types::Text, QueryableByName, RunQueryDsl};

pub fn get_collab_channels() -> &'static CollaborationChannels {
    COLLAB_CHANNELS.get_or_init(|| Arc::new(tokio::sync::RwLock::new(HashMap::new())))
}
pub fn get_presence() -> &'static PresenceMap {
    PRESENCE.get_or_init(|| Arc::new(tokio::sync::RwLock::new(HashMap::new())))
}
pub fn get_typing() -> &'static TypingMap {
    TYPING.get_or_init(|| Arc::new(tokio::sync::RwLock::new(HashMap::new())))
}
pub fn get_selections() -> &'static SelectionMap {
    SELECTIONS.get_or_init(|| Arc::new(tokio::sync::RwLock::new(HashMap::new())))
}
pub fn get_mentions() -> &'static MentionMap {
    MENTIONS.get_or_init(|| Arc::new(tokio::sync::RwLock::new(HashMap::new())))
}
pub async fn handle_get_collaborators(Path(doc_id): Path<String>) -> impl IntoResponse {
    let presence = get_presence().read().await;
    let users: Vec<&UserPresence> = presence
        .get(&doc_id)
        .map(|v| v.iter().collect())
        .unwrap_or_default();

    Json(serde_json::json!({
        "count": users.len(),
        "users": users
    }))
}
pub async fn handle_get_presence(Path(doc_id): Path<String>) -> impl IntoResponse {
    let presence = get_presence().read().await;
    let users = presence.get(&doc_id).cloned().unwrap_or_default();
    Json(serde_json::json!({ "users": users }))
}
pub async fn handle_get_typing(Path(doc_id): Path<String>) -> impl IntoResponse {
    let typing = get_typing().read().await;
    let indicators = typing.get(&doc_id).cloned().unwrap_or_default();

    let now = Utc::now();
    let active: Vec<&TypingIndicator> = indicators
        .iter()
        .filter(|t| (now - t.started_at).num_seconds() < 5)
        .collect();

    Json(serde_json::json!({ "typing": active }))
}
pub async fn handle_get_selections(Path(doc_id): Path<String>) -> impl IntoResponse {
    let selections = get_selections().read().await;
    let sels = selections.get(&doc_id).cloned().unwrap_or_default();
    Json(serde_json::json!({ "selections": sels }))
}
pub async fn handle_get_mentions(Path(user_id): Path<String>) -> impl IntoResponse {
    let mentions = get_mentions().read().await;
    let user_mentions = mentions.get(&user_id).cloned().unwrap_or_default();
    Json(serde_json::json!({ "mentions": user_mentions }))
}

/// #2733 — why a document collaboration upgrade was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DocAccessError {
    /// The path segment cannot name a document object.
    InvalidDocId,
    /// The verified token carries no identity that can own a namespace.
    InvalidIdentity,
    /// The identity is real but has no relationship to this document.
    Forbidden,
}

impl DocAccessError {
    fn code(self) -> &'static str {
        match self {
            Self::InvalidDocId => "invalid_doc_id",
            Self::InvalidIdentity => "invalid_identity",
            Self::Forbidden => "document_access_denied",
        }
    }
}

/// Pure access predicate over the two facts authorization can establish:
/// a stored collaboration grant, or ownership of the document object.
///
/// `None` grant + `false` ownership means the document is foreign or unknown,
/// which is never access. Unknown/legacy roles fail closed too.
pub(super) fn grants_doc_access(role: Option<&str>, owner: bool) -> bool {
    owner || matches!(role, Some("viewer" | "commenter" | "editor" | "owner"))
}

/// A document id must be a single path segment, of a size the storage layer
/// accepts. `current` is reserved by the save path and cannot be persisted.
pub(super) fn is_addressable_doc_id(doc_id: &str) -> bool {
    let id = doc_id.trim();
    !id.is_empty()
        && id == doc_id
        && id != "current"
        && id.len() <= 255
        && !id.contains('/')
        && !id.contains('\\')
        && !id.contains("..")
}

/// Guest/legacy ids cannot own a `users/{owner}/docs` namespace, even when a
/// token happened to carry one as its subject.
pub(super) fn is_usable_ws_identity(user_id: &str) -> bool {
    let id = user_id.trim();
    if id.is_empty()
        || id != user_id
        || is_legacy_owner(id)
        || matches!(
            id.to_ascii_lowercase().as_str(),
            "anon" | "guest" | "default-user" | "current"
        )
    {
        return false;
    }
    DocUser::verified(id, id).is_some()
}

#[derive(QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct RoleRow {
    #[diesel(sql_type = Text)]
    role: String,
}

/// Role granted to this verified identity on the document, if any. Matches
/// the user id and the email carried by the token, then the email domain —
/// the same lookup the collab REST layer uses. A store failure is logged and
/// treated as "no grant"; the owner probe still gets a say.
fn resolve_doc_role(
    state: &DocState,
    doc_id: &str,
    user_id: &str,
    user_name: &str,
) -> Option<String> {
    let mut conn = match state.pool.get() {
        Ok(conn) => conn,
        Err(_) => {
            log::warn!("docs ws: permission store unavailable for document {doc_id}");
            return None;
        }
    };

    let rows = match sql_query(
        "SELECT role FROM resource_permissions \
         WHERE resource_type = 'docs' AND resource_id = $1 AND grantee_type = 'user' \
           AND (grantee_id = $2 OR grantee_id = $3) \
         ORDER BY CASE role WHEN 'owner' THEN 0 ELSE 1 END LIMIT 1",
    )
    .bind::<Text, _>(doc_id)
    .bind::<Text, _>(user_id)
    .bind::<Text, _>(user_name)
    .load::<RoleRow>(&mut conn)
    {
        Ok(rows) => rows,
        Err(_) => {
            log::warn!("docs ws: permission lookup failed for document {doc_id}");
            return None;
        }
    };
    if let Some(row) = rows.into_iter().next() {
        return Some(row.role);
    }

    let domain = match user_name.split_once('@') {
        Some((_, domain)) if !domain.is_empty() => domain,
        _ => return None,
    };
    let with_at = format!("@{domain}");
    match sql_query(
        "SELECT role FROM resource_permissions \
         WHERE resource_type = 'docs' AND resource_id = $1 AND grantee_type = 'domain' \
           AND (grantee_id = $2 OR grantee_id = $3) LIMIT 1",
    )
    .bind::<Text, _>(doc_id)
    .bind::<Text, _>(domain)
    .bind::<Text, _>(&with_at)
    .load::<RoleRow>(&mut conn)
    {
        Ok(rows) => rows.into_iter().next().map(|row| row.role),
        Err(_) => {
            log::warn!("docs ws: domain permission lookup failed for document {doc_id}");
            None
        }
    }
}

/// Whether the document really exists in the caller's own namespace. Docs REST
/// handlers scope every read/write this way, so the same path is the owner
/// check. Errors fail closed (no ownership) and are logged without details
/// that could carry storage endpoints.
async fn owns_doc(state: &DocState, owner: &DocUser, doc_id: &str) -> bool {
    let key = format!("{}/{doc_id}.html", owner.docs_path());
    match state.drive.object_exists(&state.bucket_name, &key).await {
        Ok(exists) => exists,
        Err(_) => {
            log::warn!("docs ws: owner probe failed for document {doc_id}");
            false
        }
    }
}

/// The single authorization decision taken before the WebSocket upgrade.
pub(super) async fn authorize_doc_ws(
    state: &DocState,
    doc_id: &str,
    (user_id, user_name): &(String, String),
) -> Result<(), DocAccessError> {
    if !is_addressable_doc_id(doc_id) {
        return Err(DocAccessError::InvalidDocId);
    }
    if !is_usable_ws_identity(user_id) {
        return Err(DocAccessError::InvalidIdentity);
    }
    let Some(owner) = DocUser::verified(user_id, user_name) else {
        return Err(DocAccessError::InvalidIdentity);
    };
    let role = resolve_doc_role(state, doc_id, user_id, user_name);
    let is_owner = owns_doc(state, &owner, doc_id).await;
    if grants_doc_access(role.as_deref(), is_owner) {
        Ok(())
    } else {
        Err(DocAccessError::Forbidden)
    }
}

/// A refusal carries a machine-readable reason, never the document body or
/// another tenant's id, and never upgrades the connection.
fn forbidden(reason: DocAccessError) -> Response {
    let status = match reason {
        DocAccessError::InvalidDocId => StatusCode::BAD_REQUEST,
        DocAccessError::InvalidIdentity => StatusCode::UNAUTHORIZED,
        DocAccessError::Forbidden => StatusCode::FORBIDDEN,
    };
    let code = reason.code();
    (
        status,
        [
            ("x-auth-error", "forbidden_resource"),
            ("x-auth-reason", code),
        ],
        Json(serde_json::json!({
            "error": "document_access_denied",
            "code": "docs_ws_forbidden",
            "reason": code,
        })),
    )
        .into_response()
}

pub async fn handle_docs_websocket(
    ws: WebSocketUpgrade,
    Path(doc_id): Path<String>,
    Query(q): Query<WsAuthQuery>,
    State(state): State<Arc<DocState>>,
) -> Response {
    let identity = match extract_user_from_token(&q.token) {
        Ok(identity) => identity,
        Err(reason) => {
            log::warn!(
                "docs ws refused for document {doc_id}: {}",
                reason_code(&reason)
            );
            return unauthorized(reason);
        }
    };

    match authorize_doc_ws(&state, &doc_id, &identity).await {
        Ok(()) => ws
            .on_upgrade(move |socket| handle_docs_connection(socket, doc_id, identity))
            .into_response(),
        Err(reason) => {
            log::warn!("docs ws refused for document {doc_id}: {}", reason.code());
            forbidden(reason)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{grants_doc_access, is_addressable_doc_id, is_usable_ws_identity};

    #[test]
    fn owned_document_and_explicit_grants_are_accepted() {
        assert!(grants_doc_access(None, true));
        for role in ["viewer", "commenter", "editor", "owner"] {
            assert!(grants_doc_access(Some(role), false), "role {role}");
        }
        assert!(grants_doc_access(Some("owner"), true));
    }

    #[test]
    fn foreign_unknown_and_unknown_roles_are_denied() {
        for role in [None, Some(""), Some("guest"), Some("admin"), Some("none")] {
            assert!(!grants_doc_access(role, false), "role {role:?}");
        }
    }

    #[test]
    fn doc_id_must_be_a_single_safe_segment() {
        const ID: &str = "8f14e45f-ceea-467f-a1a5-c4d1d2f4a9d6";
        let too_long = "x".repeat(256);
        assert!(is_addressable_doc_id(ID));
        assert!(is_addressable_doc_id("report_v2"));
        for id in [
            "",
            "   ",
            " current ",
            "current",
            "../other",
            "a/b",
            "a\\b",
            "a..b",
            too_long.as_str(),
        ] {
            assert!(!is_addressable_doc_id(id), "id {id:?}");
        }
    }

    #[test]
    fn guest_and_legacy_identities_cannot_own_a_namespace() {
        for id in ["", " anon", "anon", "Guest", "default-user", "current"] {
            assert!(!is_usable_ws_identity(id), "id {id:?}");
        }
        assert!(is_usable_ws_identity(
            "8f14e45f-ceea-467f-a1a5-c4d1d2f4a9d6"
        ));
    }
}
