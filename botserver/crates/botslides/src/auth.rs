//! Slides-scoped owner identity (#2137).
//!
//! Every deck lives under `users/{owner}/presentations/` in the bot's Drive
//! bucket, so the owner id is the only thing that decides which decks a caller
//! can list, load, save or delete. Before #2137 the owner was the literal
//! `default-user`, which meant every caller shared one namespace and could
//! read and delete anybody's presentations.
//!
//! The host crate resolves the platform `AuthenticatedUser` into a
//! [`SlidesUser`] extension (`botserver/src/slides/user_middleware.rs`).
//! Handlers take the [`SlidesUser`] extractor, so a request that carries no
//! verified identity is answered with `401 Unauthorized` instead of falling
//! back to a shared account.

use axum::extract::FromRequestParts;
use axum::http::{request::Parts, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;

/// Verified owner of a Slides request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlidesUser {
    /// Owner id used as the `users/{owner}` storage segment.
    pub id: String,
    /// Display name, for logs and presence only.
    pub name: String,
}

impl SlidesUser {
    /// Builds an identity from a server-verified principal.
    ///
    /// Returns `None` for an id that must never own a namespace — empty or one
    /// of the legacy shared owners (`default-user`) — so the caller answers
    /// `401` instead of re-creating the shared prefix (#2137).
    pub fn verified(id: impl Into<String>, name: impl Into<String>) -> Option<Self> {
        let id = id.into().trim().to_string();
        if id.is_empty() || id == "default-user" {
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

/// Rendered when a Slides request has no verified identity.
///
/// The response is a plain `401` with a JSON body: the API clients and the
/// fragment shell both treat it as "sign in again", never as an empty deck.
#[derive(Debug, Clone, Copy)]
pub struct SlidesUnauthorized;

impl IntoResponse for SlidesUnauthorized {
    fn into_response(self) -> Response {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Authentication required" })),
        )
            .into_response()
    }
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for SlidesUser
where
    S: Send + Sync,
{
    type Rejection = SlidesUnauthorized;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<SlidesUser>()
            .cloned()
            .ok_or(SlidesUnauthorized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;

    fn parts() -> Parts {
        let (parts, _) = Request::builder()
            .uri("/api/slides/list")
            .body(())
            .expect("request");
        parts
    }

    #[tokio::test]
    async fn missing_identity_is_unauthorized() {
        let mut parts = parts();
        let result = SlidesUser::from_request_parts(&mut parts, &()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn verified_identity_is_the_owner() {
        let mut parts = parts();
        parts
            .extensions
            .insert(SlidesUser::new("user-a", "Alice"));
        let user = SlidesUser::from_request_parts(&mut parts, &())
            .await
            .expect("verified identity");
        assert_eq!(user.owner_id(), "user-a");
    }

    #[test]
    fn legacy_shared_owner_is_never_verified() {
        assert!(SlidesUser::verified("", "Alice").is_none());
        assert!(SlidesUser::verified("default-user", "Anonymous").is_none());
        assert!(SlidesUser::verified("user-a", "").is_some());
    }
}
