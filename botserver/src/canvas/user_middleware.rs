//! Bridges the platform auth extension into the Canvas handler scope (#2735).
//!
//! The global auth middleware inserts `AuthenticatedUser` into request
//! extensions. This middleware converts a verified principal into
//! `botcanvas::CanvasUser`; collaboration handlers take that extractor, so a
//! request with no verified identity is answered `401` and no collaborator,
//! comment or version is read or written. Body-supplied identities are never
//! consulted.

use axum::{body::Body, http::Request, middleware::Next, response::Response};
#[cfg(feature = "security")]
use botcanvas::CanvasUser;

#[cfg(feature = "security")]
pub async fn canvas_user_middleware(mut request: Request<Body>, next: Next) -> Response {
    let caller = request
        .extensions()
        .get::<crate::security::auth::AuthenticatedUser>()
        .filter(|user| user.is_authenticated() && !user.user_id.is_nil())
        .map(|user| CanvasUser::new(user.user_id));
    if let Some(caller) = caller {
        request.extensions_mut().insert(caller);
    }
    next.run(request).await
}

/// Builds without the security feature have no platform auth layer, hence no
/// verified identity: every Canvas REST handler answers `401`.
#[cfg(not(feature = "security"))]
pub async fn canvas_user_middleware(request: Request<Body>, next: Next) -> Response {
    next.run(request).await
}
