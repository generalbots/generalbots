//! Read side of a Docs share link (#2253).
//!
//! The Share dialog mints a capability token with `POST /api/collab/links`
//! (`resource_links`, `resource_type = 'docs'`). Until this module existed
//! nothing consumed that token: the link opened the editor shell, which
//! required a session, so an anonymous reader saw a login redirect (or, worse,
//! their own empty document) instead of the shared one.
//!
//! This endpoint is the counterpart of the mint, and it is read-only by
//! construction:
//!
//! * the route is declared a capability in the platform allowlist
//!   (`botsecurity_auth::auth_api::public_routes`), which refuses a
//!   state-changing method on that entry;
//! * every mutating Docs route keeps taking a [`crate::auth::DocUser`], and an
//!   anonymous request has none (`401`), so a link session cannot save;
//! * the response always carries `read_only: true`, whatever role the link
//!   records — a link grants a view, not an editor session.
//!
//! The owning namespace is captured on the link row when it is minted
//! (`owner_id`), because documents live under `users/{owner}/docs` (#2135) and
//! an anonymous reader has no identity to derive it from.

use crate::state::DocState;
use crate::storage::load_document_from_drive;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use diesel::prelude::*;
use diesel::sql_types::{Nullable, Text, Timestamptz};
use serde::Serialize;
use std::sync::Arc;

/// Length of the token minted by `collab_routes::create_link`: two simple
/// UUIDs concatenated, i.e. 64 lowercase hex characters.
const TOKEN_LEN: usize = 64;

/// Resource type the share dialog uses for a Docs document.
const DOCS_RESOURCE: &str = "docs";

/// One shared document, as an anonymous reader may see it.
#[derive(Debug, Serialize)]
pub struct SharedDocument {
    pub id: String,
    pub title: String,
    pub content: String,
    /// Always `true`: redeeming a link never opens an editing session.
    pub read_only: bool,
    /// Role recorded on the link (`viewer`, `commenter`, `editor`).
    pub role: String,
    pub updated_at: String,
}

#[derive(Debug, QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct LinkRow {
    #[diesel(sql_type = Text)]
    resource_type: String,
    #[diesel(sql_type = Text)]
    resource_id: String,
    #[diesel(sql_type = Text)]
    role: String,
    #[diesel(sql_type = Nullable<Text>)]
    owner_id: Option<String>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// A token is usable only when it has the minted shape: a 64-character
/// lowercase hex string. Anything else is rejected before the database is
/// touched.
pub fn is_valid_token(token: &str) -> bool {
    token.len() == TOKEN_LEN && token.chars().all(|c| c.is_ascii_hexdigit())
}

/// Unknown, expired, revoked and wrong-type links are indistinguishable: the
/// caller learns `404`, never whether a token ever existed.
fn not_found() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "Link not found" })),
    )
}

fn internal<E: std::fmt::Display>(
    error: &E,
    context: &str,
) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({
            "error": botsecurity_core::error_sanitizer::log_and_sanitize_json(error, context, None)
        })),
    )
}

/// `GET /api/docs/shared/:token` — read a document through its share link.
///
/// Anonymous by design (capability token). The response is the document body
/// plus the read-only marker the viewer renders; nothing here writes, and the
/// token is never echoed back.
pub async fn handle_shared_document(
    State(state): State<Arc<DocState>>,
    Path(token): Path<String>,
) -> Result<Json<SharedDocument>, (StatusCode, Json<serde_json::Value>)> {
    let token = token.trim().to_lowercase();
    if !is_valid_token(&token) {
        return Err(not_found());
    }

    let link = load_link(&state, &token).await?;
    if link.resource_type != DOCS_RESOURCE {
        return Err(not_found());
    }
    if let Some(expires_at) = link.expires_at {
        if expires_at < chrono::Utc::now() {
            return Err(not_found());
        }
    }

    let Some(owner_id) = resolve_owner(&state, &link).await else {
        log::warn!(
            "docs: share link for {} has no owner namespace; regenerate the link",
            link.resource_id
        );
        return Err(not_found());
    };

    let document = load_document_from_drive(&state, &owner_id, &link.resource_id)
        .await
        .map_err(|e| internal(&e, "botdocs::shared_link"))?
        .ok_or_else(not_found)?;

    Ok(Json(SharedDocument {
        id: document.id,
        title: document.title,
        content: document.content,
        read_only: true,
        role: link.role,
        updated_at: document.updated_at.to_rfc3339(),
    }))
}

async fn load_link(
    state: &DocState,
    token: &str,
) -> Result<LinkRow, (StatusCode, Json<serde_json::Value>)> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| internal(&e, "botdocs::shared_link"))?;

    diesel::sql_query(
        "SELECT resource_type, resource_id, role, owner_id, expires_at \
         FROM resource_links WHERE token = $1",
    )
    .bind::<Text, _>(token)
    .load::<LinkRow>(&mut conn)
    .map_err(|e| internal(&e, "botdocs::shared_link"))?
    .into_iter()
    .next()
    .ok_or_else(not_found)
}

/// Owner namespace of the linked document.
///
/// Normally recorded on the link row. Links minted before `owner_id` existed
/// have none; for those the namespace is recovered from the only key that can
/// hold the document (`users/{owner}/docs/{id}.html`), so an old link keeps
/// working instead of silently 404ing.
async fn resolve_owner(state: &DocState, link: &LinkRow) -> Option<String> {
    if let Some(owner_id) = link.owner_id.as_ref().filter(|o| !o.trim().is_empty()) {
        return Some(owner_id.clone());
    }

    let keys = state
        .drive
        .list_objects(&state.bucket_name, Some("users/"))
        .await
        .ok()?;
    let suffix = format!("/docs/{}.html", link.resource_id);
    keys.into_iter().find_map(|key| {
        key.strip_prefix("users/")
            .and_then(|rest| rest.strip_suffix(&suffix))
            .filter(|owner| !owner.is_empty())
            .map(str::to_string)
    })
}

#[cfg(test)]
mod tests {
    use super::is_valid_token;

    #[test]
    fn the_minted_token_shape_is_accepted() {
        let token = format!(
            "{}{}",
            uuid::Uuid::new_v4().simple(),
            uuid::Uuid::new_v4().simple()
        );
        assert!(is_valid_token(&token));
    }

    #[test]
    fn anything_else_is_rejected() {
        assert!(!is_valid_token(""));
        assert!(!is_valid_token("abc"));
        assert!(!is_valid_token(&"g".repeat(64)));
        assert!(!is_valid_token(&"a".repeat(63)));
        assert!(!is_valid_token(&"a".repeat(65)));
    }
}
