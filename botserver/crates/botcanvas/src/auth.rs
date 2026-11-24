//! Canvas-scoped caller identity (#2735).
//!
//! The host (`botserver/src/canvas/user_middleware.rs`) bridges the verified
//! platform `AuthenticatedUser` extension into this crate's [`CanvasUser`].
//! Handlers take the extractor, so a request that reaches a collaboration
//! route without a verified principal is answered `401` before any canvas row
//! is read or written. No identity is ever taken from a request body or a
//! client-supplied header: that is what made the grantee forgeable.

use axum::extract::FromRequestParts;
use axum::http::{request::Parts, StatusCode};
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

/// A principal the host auth middleware has already verified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanvasUser {
    pub user_id: Uuid,
}

impl CanvasUser {
    /// Builds an identity from a server-verified principal.
    pub fn new(user_id: Uuid) -> Self {
        Self { user_id }
    }

    /// A nil id is the anonymous principal and can never own or edit a canvas.
    pub fn is_verified(&self) -> bool {
        !self.user_id.is_nil()
    }
}

/// Rendered when a canvas request carries no verified identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanvasUnauthorized;

impl IntoResponse for CanvasUnauthorized {
    fn into_response(self) -> Response {
        (
            StatusCode::UNAUTHORIZED,
            "Authentication required".to_string(),
        )
            .into_response()
    }
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for CanvasUser
where
    S: Send + Sync,
{
    type Rejection = CanvasUnauthorized;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<CanvasUser>()
            .cloned()
            .filter(CanvasUser::is_verified)
            .ok_or(CanvasUnauthorized)
    }
}

#[cfg(test)]
#[path = "auth_tests.rs"]
mod tests;
