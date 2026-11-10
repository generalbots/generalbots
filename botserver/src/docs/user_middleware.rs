//! Bridges the platform auth extension into the Docs handler scope (#2135).
//!
//! Docs handlers take the `botdocs::auth::DocUser` extractor, so the only job
//! here is to publish the verified owner id for the request. Owner resolution is
//! shared with Slides and Paper (`crate::security::office_owner`), so an install
//! has a single owner identity across the office apps.
//!
//! A request whose identity cannot be verified leaves the extension absent and
//! the handler answers `401` — there is no shared fallback owner. Before #2135
//! every request, anonymous included, resolved to the constant `default-user`.

use axum::{body::Body, http::Request, middleware::Next, response::Response};
use botdocs::auth::DocUser;

/// Converts `AuthenticatedUser` into `botdocs::auth::DocUser`.
#[cfg(feature = "security")]
pub async fn docs_user_middleware(mut request: Request<Body>, next: Next) -> Response {
    let owner = request
        .extensions()
        .get::<crate::security::auth::AuthenticatedUser>()
        .and_then(doc_user_for);
    if let Some(owner) = owner {
        request.extensions_mut().insert(owner);
    }
    next.run(request).await
}

/// The Docs identity of a verified principal, or `None` when it has none.
#[cfg(feature = "security")]
fn doc_user_for(user: &crate::security::auth::AuthenticatedUser) -> Option<DocUser> {
    let id = crate::security::office_owner::verified_owner_id(user)?;
    DocUser::verified(id, crate::security::office_owner::owner_display_name(user))
}

/// Builds without the security feature have no auth middleware, hence no
/// verified identity: every Docs handler answers `401`.
#[cfg(not(feature = "security"))]
pub async fn docs_user_middleware(request: Request<Body>, next: Next) -> Response {
    next.run(request).await
}

#[cfg(all(test, feature = "security"))]
#[path = "user_middleware_tests.rs"]
mod tests;
