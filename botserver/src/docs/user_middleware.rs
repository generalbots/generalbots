//! Bridges the platform auth extension into the Docs handler scope (#2135).
//!
//! The platform auth middleware inserts [`crate::security::auth::AuthenticatedUser`]
//! into request extensions. This middleware converts it into the docs-scoped
//! [`botdocs::auth::DocUser`], so the `botdocs` crate never depends on the
//! security crate.
//!
//! Nothing is inserted when the caller is not verified: the Docs handlers then
//! reject the request with `401` instead of writing to a namespace shared with
//! other accounts. Before #2135 every request — anonymous included — resolved to
//! the constant `default-user`.

use axum::{body::Body, extract::State, http::Request, middleware::Next, response::Response};
use botdocs::auth::DocUser;
use std::sync::Arc;

/// Injects the verified [`DocUser`] for the request.
///
/// `local_owner` is the per-install id resolved once at boot (see
/// [`super::install_owner`]); it is only used for the single-user desktop
/// profile.
#[cfg(feature = "security")]
pub async fn docs_user_middleware(
    State(local_owner): State<Arc<str>>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    let doc_user = {
        let principal = request
            .extensions()
            .get::<crate::security::auth::AuthenticatedUser>();
        resolve_doc_user(principal, &local_owner)
    };
    if let Some(doc_user) = doc_user {
        request.extensions_mut().insert(doc_user);
    }
    next.run(request).await
}

/// Without the security feature there are no accounts, so the process is the
/// single-user desktop profile and owns its per-install namespace.
#[cfg(not(feature = "security"))]
pub async fn docs_user_middleware(
    State(local_owner): State<Arc<str>>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    if let Some(doc_user) = DocUser::verified(local_owner.to_string(), "local".to_string()) {
        request.extensions_mut().insert(doc_user);
    }
    next.run(request).await
}

/// Resolves the docs identity from the platform principal.
///
/// Two principals can own documents:
///
/// * an authenticated directory user — the namespace is the verified user id;
/// * the single-user desktop principal — the namespace is the per-install id,
///   never the global `default-user`.
///
/// Everything else (anonymous, service, bot) gets no Docs namespace.
#[cfg(feature = "security")]
fn resolve_doc_user(
    principal: Option<&crate::security::auth::AuthenticatedUser>,
    local_owner: &str,
) -> Option<DocUser> {
    let principal = principal?;
    if principal.is_authenticated() {
        return DocUser::verified(principal.user_id.to_string(), principal.username.clone());
    }
    // Inserted only when the product file sets `no_security` (require_auth=false).
    if principal.metadata.contains_key("desktop_local") {
        return DocUser::verified(local_owner.to_string(), principal.username.clone());
    }
    None
}

#[cfg(all(test, feature = "security"))]
#[path = "user_middleware_tests.rs"]
mod tests;
