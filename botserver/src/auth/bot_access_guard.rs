use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use uuid::Uuid;
use std::sync::Arc;
use botcore::shared::state::AppState;
use crate::core::bot::check_bot_access;

/// Middleware that restricts bot URL access based on organization
/// membership and bot visibility settings.
///
/// Flow:
/// 1. Extract org / user from the session (via cookie or token).
/// 2. Look up the bot's visibility setting (`private` / `org` / `public`).
/// 3. If the bot is `private` and the requester is not a member of the
///    owning org, return 403 Forbidden.
pub async fn bot_access_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = req.uri().path();

    // Extract bot name from path: /bot/{bot_name}
    if let Some(bot_name) = extract_bot_name(path) {
        // Get user session from request extensions
        let user_id = req
            .extensions()
            .get::<Uuid>()
            .copied()
            .ok_or(StatusCode::UNAUTHORIZED)?;

        let state = req
            .extensions()
            .get::<Arc<AppState>>()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        // Check bot access permissions
        check_bot_access(&state, &bot_name, user_id)
            .await
            .map_err(|_| StatusCode::FORBIDDEN)?;
    }

    Ok(next.run(req).await)
}

fn extract_bot_name(path: &str) -> Option<String> {
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() >= 2 && parts[1] == "bot" {
        parts.get(2).map(|s| s.to_string())
    } else {
        None
    }
}
