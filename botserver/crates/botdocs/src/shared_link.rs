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
//!   records — a link grants a view, not an editor session;
//! * #2452 — the body is role-scoped: only roles that actually carry source
//!   capability receive the editable document body. A `viewer` link gets the
//!   document metadata and an empty body, never the full HTML source.
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

/// Roles whose capability includes the editable document body. The Share
/// dialog mints `viewer`, `commenter` and `editor`; `owner` is accepted for
/// legacy rows. A `viewer` link grants a read-only view, not the source, and
/// an unrecognised role fails closed.
fn role_allows_source(role: &str) -> bool {
    matches!(
        role.trim().to_ascii_lowercase().as_str(),
        "commenter" | "editor" | "owner"
    )
}

/// The single decision for releasing source bytes across the capability
/// boundary. Every gate fails closed: a malformed token, an expired link or a
/// role without source capability denies the body. The handler rejects the
/// first two up front with the generic `not_found()`; this function also
/// feeds the body field so the role check cannot be bypassed later.
fn body_for_link<'a>(
    token: &str,
    role: &str,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
    now: chrono::DateTime<chrono::Utc>,
    source: &'a str,
) -> &'a str {
    let allowed =
        is_valid_token(token) && !link_is_expired(expires_at, now) && role_allows_source(role);
    if allowed { source } else { "" }
}

/// Unknown, expired, revoked and wrong-type links are indistinguishable: the
/// caller learns `404`, never whether a token ever existed.
fn not_found() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "Link not found" })),
    )
}

/// #2452 — the single redemption decision for link expiry.
///
/// Kept pure so the deadline boundary is unit-testable without a database:
/// `None` means a link that never expires (the backward-compatible default),
/// and an equality at the deadline is already expired. The caller turns every
/// `true` into the same generic `not_found()` as an unknown token, so an
/// expired link cannot be distinguished from one that never existed and the
/// token/document are never disclosed.
fn link_is_expired(
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
    now: chrono::DateTime<chrono::Utc>,
) -> bool {
    expires_at.is_some_and(|deadline| now >= deadline)
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
    // #2452 — refuse an expired link through the same generic error used for
    // an unknown/wrong-type token: no existence hint, no token, no document.
    if link_is_expired(link.expires_at, chrono::Utc::now()) {
        return Err(not_found());
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

    // #2452 — only a link that actually resolved to a document is a "view".
    // The token and the body stay out of the audit payload.
    crate::access_audit::record_shared_view(&state, &link.resource_id);

    // #2452 — the capability is scoped to the role recorded on the link. A
    // viewer sees only the metadata shell; the editable HTML source (and
    // anything embedded in it) never leaves the server on a viewer link.
    // The response shape is unchanged: only `content` is role-dependent.
    let content = body_for_link(
        &token,
        &link.role,
        link.expires_at,
        chrono::Utc::now(),
        &document.content,
    )
    .to_string();

    Ok(Json(SharedDocument {
        id: document.id,
        title: document.title,
        content,
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
    use super::{body_for_link, is_valid_token, link_is_expired};

    fn minted_token() -> String {
        format!(
            "{}{}",
            uuid::Uuid::new_v4().simple(),
            uuid::Uuid::new_v4().simple()
        )
    }

    const SOURCE: &str = "<h1>Shared heading</h1><p>UAT_SECRET_MARKER</p>";

    #[test]
    fn the_minted_token_shape_is_accepted() {
        assert!(is_valid_token(&minted_token()));
    }

    #[test]
    fn anything_else_is_rejected() {
        assert!(!is_valid_token(""));
        assert!(!is_valid_token("abc"));
        assert!(!is_valid_token(&"g".repeat(64)));
        assert!(!is_valid_token(&"a".repeat(63)));
        assert!(!is_valid_token(&"a".repeat(65)));
    }

    #[test]
    fn expired_deadlines_are_refused_including_the_boundary() {
        let now = chrono::Utc::now();
        assert!(link_is_expired(
            Some(now - chrono::Duration::seconds(1)),
            now
        ));
        assert!(link_is_expired(Some(now), now));
    }

    #[test]
    fn future_and_open_ended_links_stay_redeemable() {
        let now = chrono::Utc::now();
        assert!(!link_is_expired(
            Some(now + chrono::Duration::hours(1)),
            now
        ));
        assert!(!link_is_expired(None, now));
    }

    #[test]
    fn viewer_link_does_not_release_the_editable_source() {
        let now = chrono::Utc::now();
        let token = minted_token();
        assert!(!SOURCE.is_empty() && SOURCE.contains("UAT_SECRET_MARKER"));
        for role in ["viewer", "Viewer", "", "guest"] {
            assert_eq!(body_for_link(&token, role, None, now, SOURCE), "");
        }
    }

    #[test]
    fn owner_and_higher_role_links_still_release_the_source() {
        let now = chrono::Utc::now();
        let token = minted_token();
        for role in ["commenter", "editor", "owner", "EDITOR"] {
            assert_eq!(body_for_link(&token, role, None, now, SOURCE), SOURCE);
        }
    }

    #[test]
    fn bad_token_and_expired_link_fail_closed_without_body_bytes() {
        let now = chrono::Utc::now();
        let token = minted_token();
        assert_eq!(
            body_for_link("g".repeat(64).as_str(), "editor", None, now, SOURCE),
            ""
        );
        assert_eq!(body_for_link(&token, "editor", Some(now), now, SOURCE), "");
        assert_eq!(
            body_for_link(
                &token,
                "editor",
                Some(now - chrono::Duration::seconds(1)),
                now,
                SOURCE
            ),
            ""
        );
        assert_eq!(
            body_for_link(
                &token,
                "editor",
                Some(now + chrono::Duration::hours(1)),
                now,
                SOURCE
            ),
            SOURCE
        );
    }
}
