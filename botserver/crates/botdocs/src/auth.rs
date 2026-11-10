//! Docs-scoped caller identity (#2135).
//!
//! The host crate (`botserver/src/docs/user_middleware.rs`) converts the
//! platform `AuthenticatedUser` extension into a [`DocUser`] on every Docs
//! request. Handlers then take `user: DocUser` and derive the storage namespace
//! from it, so a handler cannot compile without an owner.
//!
//! There is deliberately no anonymous constructor: the namespace
//! `users/{owner}/docs` is per user, and a caller whose identity was not
//! verified has no namespace of its own. Such a request is rejected with `401`
//! instead of being folded into a shared fallback — that fallback is exactly
//! what made every account read every other account's documents before #2135.
//!
//! # Storage boundary
//!
//! * `users/{owner}/docs` — the per-user boundary. `owner` is the directory
//!   user id the auth middleware verified (a UUID), never a display name and
//!   never a caller-supplied string.
//! * The bucket (`DocState::bucket_name`) — the tenant/bot boundary. One bot
//!   writes to its own bucket, so two users of different tenants never meet
//!   even if their ids collided.
//! * a per-install UUID — the single-user desktop profile. The host resolves it
//!   once per install (`security::office_owner::office_install_owner`), so two
//!   installs never share a namespace.
//!
//! Sharing a document between users is therefore explicit and out of scope
//! here: collaborators are recorded inside the document, not by widening the
//! path.

use axum::extract::FromRequestParts;
use axum::http::{Extensions, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::legacy::is_legacy_owner;
use crate::storage_core::{get_user_docs_path, sanitize_owner};

/// A caller whose identity the server verified, scoped to the Docs app.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocUser {
    id: String,
    display_name: String,
}

impl DocUser {
    /// Builds an identity from a server-verified principal.
    ///
    /// Returns `None` when the id cannot own a namespace: empty, the legacy
    /// shared owner ([`crate::legacy::LEGACY_OWNER`]), or one whose sanitised
    /// form is empty. A caller that fails this check gets `401` rather than a
    /// shared prefix.
    pub fn verified(id: impl Into<String>, display_name: impl Into<String>) -> Option<Self> {
        let id = id.into().trim().to_string();
        let segment = sanitize_owner(&id);
        // An id that sanitises to separators only would build a prefix no
        // verified principal can own, so it is rejected like an empty one.
        if id.is_empty()
            || is_legacy_owner(&id)
            || !segment.chars().any(char::is_alphanumeric)
        {
            return None;
        }
        let display_name = display_name.into().trim().to_string();
        let display_name = if display_name.is_empty() {
            id.clone()
        } else {
            display_name
        };
        Some(Self { id, display_name })
    }

    /// The verified owner id used as the namespace segment.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Human-readable name for authored content (comment authors).
    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    /// Drive prefix owned by this caller.
    pub fn docs_path(&self) -> String {
        get_user_docs_path(&self.id)
    }

    /// Reads the identity the Docs middleware inserted.
    ///
    /// The global auth middleware always inserts an extension — including the
    /// anonymous fallback for public paths — so a missing *or* unverified
    /// extension must be a rejection, never a default identity.
    pub fn from_extensions(extensions: &Extensions) -> Result<Self, DocAuthError> {
        extensions.get::<DocUser>().cloned().ok_or(DocAuthError::Unauthenticated)
    }
}

/// Why a Docs request was rejected.
///
/// A single variant on purpose: the caller learns that it is unauthenticated,
/// never which check failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocAuthError {
    Unauthenticated,
}

impl IntoResponse for DocAuthError {
    fn into_response(self) -> Response {
        let DocAuthError::Unauthenticated = self;
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "authentication required" })),
        )
            .into_response()
    }
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for DocUser
where
    S: Send + Sync,
{
    type Rejection = DocAuthError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        Self::from_extensions(&parts.extensions)
    }
}

#[cfg(test)]
#[path = "auth_tests.rs"]
mod tests;
