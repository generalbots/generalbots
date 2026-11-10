//! Bridges the platform auth extension into the Slides handler scope (#2137).
//!
//! Slides handlers take the `botslides::auth::SlidesUser` extractor, so the
//! only job here is to publish the verified owner id for the request. A
//! request whose identity cannot be verified leaves the extension absent and
//! the handler answers `401` — there is no shared fallback owner.

use axum::{body::Body, http::Request, middleware::Next, response::Response};

/// Converts `AuthenticatedUser` into `botslides::auth::SlidesUser`.
#[cfg(feature = "security")]
pub async fn slides_user_middleware(mut request: Request<Body>, next: Next) -> Response {
    let owner = request
        .extensions()
        .get::<crate::security::auth::AuthenticatedUser>()
        .and_then(|user| {
            crate::security::office_owner::verified_owner_id(user).and_then(|id| {
                botslides::auth::SlidesUser::verified(
                    id,
                    crate::security::office_owner::owner_display_name(user),
                )
            })
        });
    if let Some(owner) = owner {
        request.extensions_mut().insert(owner);
    }
    next.run(request).await
}

/// Builds without the security feature have no auth middleware, hence no
/// verified identity: every Slides handler answers `401`.
#[cfg(not(feature = "security"))]
pub async fn slides_user_middleware(request: Request<Body>, next: Next) -> Response {
    next.run(request).await
}
