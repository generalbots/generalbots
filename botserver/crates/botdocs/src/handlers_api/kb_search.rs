//! Read-only, citation-only view of the in-memory KB index (issue #2499).
//!
//! Saves project structured chunks into `DocState::kb_index`; this handler is
//! the HTTP read side of that projection. It answers a permission-aware search
//! with citation metadata only — the index never stored document content, so
//! neither can this response.

use crate::auth::DocUser;
use crate::state::DocState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use botcore::kb::permissions::UserContext;
use botcore::kb::structured::SourceRef;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

const DEFAULT_LIMIT: usize = 10;
const MAX_LIMIT: usize = 50;

#[derive(Debug, Deserialize)]
pub struct KbSearchQuery {
    pub q: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedQuery {
    pub text: String,
    pub limit: usize,
}

/// Fixed-message validation errors. No caller or internal error text is ever
/// echoed: the client gets a stable code-shaped message, details go to logs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KbSearchError {
    MissingQuery,
    InvalidLimit,
    InvalidPrincipal,
}

impl IntoResponse for KbSearchError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::MissingQuery => (StatusCode::BAD_REQUEST, "query is required"),
            Self::InvalidLimit => (StatusCode::BAD_REQUEST, "limit must be between 1 and 50"),
            Self::InvalidPrincipal => (StatusCode::UNAUTHORIZED, "authentication required"),
        };
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

/// Pure request validation, shared by the handler and the unit tests.
pub fn validate(query: Option<&str>, limit: Option<usize>) -> Result<ValidatedQuery, KbSearchError> {
    let text = query.unwrap_or("").trim();
    if text.is_empty() {
        return Err(KbSearchError::MissingQuery);
    }
    let limit = limit.unwrap_or(DEFAULT_LIMIT);
    if limit == 0 || limit > MAX_LIMIT {
        return Err(KbSearchError::InvalidLimit);
    }
    Ok(ValidatedQuery {
        text: text.to_string(),
        limit,
    })
}

/// Parse the verified owner id. Directory users carry a bare UUID; a
/// single-user desktop install prefixes it with `local-` (the same string the
/// save projected into the ACL's `allowed_users`).
fn parse_owner_id(raw: &str) -> Option<Uuid> {
    Uuid::parse_str(raw.strip_prefix("local-").unwrap_or(raw)).ok()
}

/// The ACL principal comes from the server-verified `DocUser` and the server's
/// own bucket, never from the query string. The bucket is the tenant the
/// projection stamped on every point, so a caller cannot widen the scope.
fn principal_for(state: &DocState, user: &DocUser) -> Result<UserContext, KbSearchError> {
    let user_id = match parse_owner_id(user.id()) {
        Some(id) => id,
        None => {
            log::warn!("botdocs: KB search rejected an unverifiable principal");
            return Err(KbSearchError::InvalidPrincipal);
        }
    };
    // `project_to_kb` stored the raw verified id in the ACL's `allowed_users`;
    // pass it as the alternate identifier so desktop-local `local-<uuid>`
    // owners match it exactly (a bare-UUID user also matches via `user_id`).
    let owner_id = Some(user.id().to_string());
    // A non-UUID bucket cannot match any projected tenant, so failing to parse
    // it leaves `organization_id` empty and the index returns no hits.
    // Same deterministic mapping the projector uses: a named bucket
    // (`default.gbai`) must still match the tenant stamped on every point.
    let organization_id = Some(crate::state::DocState::tenant_uuid(&state.bucket_name));
    Ok(UserContext::authenticated(user_id, owner_id, organization_id))
}

/// Citation metadata only: no `content`, no payload, no raw error text.
#[derive(Debug, Serialize)]
pub struct KbSearchHitDto {
    pub id: String,
    pub source_ref: SourceRef,
    pub section: Option<String>,
    pub score: f32,
}

#[derive(Debug, Serialize)]
pub struct KbSearchResponse {
    pub hits: Vec<KbSearchHitDto>,
}

/// `GET /api/docs/kb/search?q=<text>&limit=<n>`
pub async fn handle_kb_search(
    State(state): State<Arc<DocState>>,
    user: DocUser,
    Query(params): Query<KbSearchQuery>,
) -> Result<Json<KbSearchResponse>, KbSearchError> {
    let validated = validate(params.q.as_deref(), params.limit)?;
    let principal = principal_for(&state, &user)?;
    let hits = state.search_text(&validated.text, &principal, validated.limit);
    Ok(Json(KbSearchResponse {
        hits: hits
            .into_iter()
            .map(|hit| KbSearchHitDto {
                id: hit.id,
                source_ref: hit.source,
                section: hit.section,
                score: hit.score,
            })
            .collect(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn empty_and_whitespace_queries_are_rejected() {
        assert_eq!(validate(None, None), Err(KbSearchError::MissingQuery));
        assert_eq!(validate(Some(""), None), Err(KbSearchError::MissingQuery));
        assert_eq!(validate(Some("   \t "), None), Err(KbSearchError::MissingQuery));
    }

    #[test]
    fn out_of_range_limits_are_rejected() {
        assert_eq!(
            validate(Some("clause"), Some(0)),
            Err(KbSearchError::InvalidLimit)
        );
        assert_eq!(
            validate(Some("clause"), Some(MAX_LIMIT + 1)),
            Err(KbSearchError::InvalidLimit)
        );
    }

    #[test]
    fn verified_owner_ids_are_parsed_with_desktop_prefix() {
        const UUID: &str = "33333333-3333-3333-3333-333333333333";
        assert_eq!(parse_owner_id(UUID), Uuid::parse_str(UUID).ok());
        assert_eq!(
            parse_owner_id(&format!("local-{UUID}")),
            Uuid::parse_str(UUID).ok()
        );
        assert_eq!(parse_owner_id("not-a-principal"), None);
    }

    #[test]
    fn default_and_boundary_limits_are_accepted() {
        let default = validate(Some("clause"), None);
        assert!(default.is_ok());
        if let Ok(valid) = default {
            assert_eq!(valid.limit, DEFAULT_LIMIT);
            assert_eq!(valid.text, "clause");
        }
        let maximum = validate(Some("  clause  "), Some(MAX_LIMIT));
        assert!(maximum.is_ok());
        if let Ok(valid) = maximum {
            assert_eq!(valid.limit, MAX_LIMIT);
            assert_eq!(valid.text, "clause");
        }
    }
}
