use axum::{
    extract::{Path, Request},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::core::bot::manager::BotManager;
use crate::core::session::SessionManager;

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

        // Check bot access permissions
        check_bot_access(&bot_name, user_id)
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

async fn check_bot_access(bot_name: &str, user_id: Uuid) -> Result<(), String> {
    // TODO: Implement actual bot access check
    // 1. Load bot config from drive
    // 2. Check if bot is public
    // 3. If private, verify user is in allowed org/users
    Ok(())
}
