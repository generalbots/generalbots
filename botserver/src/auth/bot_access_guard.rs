use axum::{extract::Request, middleware::Next, response::Response};

/// Middleware that restricts bot URL access based on organization
/// membership and bot visibility settings.
///
/// Flow:
/// 1. Extract org / user from the session (via cookie or token).
/// 2. Look up the bot's visibility setting (`private` / `org` / `public`).
/// 3. If the bot is `private` and the requester is not a member of the
///    owning org, return 403 Forbidden.
pub async fn bot_access_middleware(req: Request, next: Next) -> Response {
    let path = req.uri().path().to_string();

    // TODO(#499): Extract org from user session, check bot visibility
    // If bot is private and user not in org, return 403

    let _ = path;

    next.run(req).await
}
