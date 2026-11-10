//! Paper-scoped owner identity (#2137).
//!
//! Every document lives under `users/{owner}/papers/` in the bot's Drive
//! bucket, so the owner id decides which documents a caller can list, open,
//! save or delete. Before #2137 a request without a session fell back to a
//! find-or-create `anonymous@local` account, which made every unauthenticated
//! caller share one document store; the raw `x-session-id` header was also
//! accepted as a bearer token.
//!
//! Identity now comes only from the platform auth middleware (cookie or
//! `Authorization`, verified there) through the host bridge
//! (`botserver/src/paper/user_middleware.rs`). Handlers take the [`PaperUser`]
//! extractor, so a request without a verified identity is answered with
//! `401 Unauthorized` and no document is touched.

use axum::extract::FromRequestParts;
use axum::http::{request::Parts, StatusCode};
use axum::response::{Html, IntoResponse, Response};

/// Verified owner of a Paper request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaperUser {
    /// Owner id used as the `users/{owner}` storage segment.
    pub id: String,
    /// Display name, for logs only.
    pub name: String,
}

impl PaperUser {
    /// Builds an identity from a server-verified principal.
    ///
    /// Returns `None` for an id that must never own a namespace — empty or one
    /// of the legacy shared owners (`anonymous@local`) — so the caller answers
    /// `401` instead of re-creating the shared store (#2137).
    pub fn verified(id: impl Into<String>, name: impl Into<String>) -> Option<Self> {
        let id = id.into().trim().to_string();
        if id.is_empty() || id == "anonymous@local" {
            return None;
        }
        let name = name.into().trim().to_string();
        Some(Self {
            id: id.clone(),
            name: if name.is_empty() { id } else { name },
        })
    }

    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
        }
    }

    /// Owner id used by every storage call.
    pub fn owner_id(&self) -> &str {
        &self.id
    }
}

/// Rendered when a Paper request has no verified identity.
#[derive(Debug, Clone, Copy)]
pub struct PaperUnauthorized;

impl IntoResponse for PaperUnauthorized {
    fn into_response(self) -> Response {
        (
            StatusCode::UNAUTHORIZED,
            Html(crate::utils::format_error("Authentication required")),
        )
            .into_response()
    }
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for PaperUser
where
    S: Send + Sync,
{
    type Rejection = PaperUnauthorized;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<PaperUser>()
            .cloned()
            .ok_or(PaperUnauthorized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;

    fn parts() -> Parts {
        let request = Request::builder()
            .uri("/api/ui/paper/list")
            .body(())
            .expect("request");
        request.into_parts().0
    }

    #[tokio::test]
    async fn missing_identity_is_unauthorized() {
        let mut parts = parts();
        let result = PaperUser::from_request_parts(&mut parts, &()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn verified_identity_is_the_owner() {
        let mut parts = parts();
        parts.extensions.insert(PaperUser::new("user-a", "Alice"));
        let user = PaperUser::from_request_parts(&mut parts, &())
            .await
            .expect("verified identity");
        assert_eq!(user.owner_id(), "user-a");
    }

    #[test]
    fn legacy_shared_owner_is_never_verified() {
        assert!(PaperUser::verified("", "Alice").is_none());
        assert!(PaperUser::verified("anonymous@local", "Anonymous").is_none());
        assert!(PaperUser::verified("user-a", "").is_some());
    }
}
