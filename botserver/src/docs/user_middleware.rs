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

use axum::{
    body::Body,
    extract::State,
    http::Request,
    middleware::Next,
    response::Response,
};
use botdocs::auth::DocUser;
use std::sync::Arc;

/// Injects the verified [`DocUser`] for the request.
///
/// `local_owner` is the per-install id resolved once at boot (see
/// [`super::local_owner`]); it is only used for the single-user desktop profile.
pub async fn docs_user_middleware(
    State(local_owner): State<Arc<str>>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    if let Some(doc_user) = resolve_doc_user(&request, &local_owner) {
        request.extensions_mut().insert(doc_user);
    }
    next.run(request).await
}

/// Resolves the docs identity from the platform principal.
///
/// Two principals can own documents:
///
/// * an authenticated directory user — the namespace is the user id;
/// * the single-user desktop principal — the namespace is the per-install id,
///   never the global `default-user`.
///
/// Everything else (anonymous, service, bot) gets no Docs namespace.
#[cfg(feature = "security")]
fn resolve_doc_user(request: &Request<Body>, local_owner: &str) -> Option<DocUser> {
    let user = request
        .extensions()
        .get::<crate::security::auth::AuthenticatedUser>()?;
    if user.is_authenticated() {
        return DocUser::verified(user.user_id.to_string(), user.username.clone());
    }
    // Inserted only when the product file sets `no_security` (require_auth=false).
    if user.metadata.contains_key("desktop_local") {
        return DocUser::verified(local_owner.to_string(), user.username.clone());
    }
    None
}

/// Without the security feature there are no accounts, so the process is the
/// single-user desktop profile and owns its per-install namespace.
#[cfg(not(feature = "security"))]
fn resolve_doc_user(_request: &Request<Body>, local_owner: &str) -> Option<DocUser> {
    DocUser::verified(local_owner.to_string(), "local".to_string())
}

#[cfg(all(test, feature = "security"))]
#[path = "user_middleware_tests.rs"]
mod tests;
