//! Credential extraction from an HTTP request.

use crate::auth_api::{config::AuthConfig, error::AuthError, types::AuthenticatedUser};
use axum::body::Body;
use uuid::Uuid;

use super::tokens::{validate_api_key_sync, validate_bearer_token_sync, validate_session_sync};

pub fn extract_user_from_request(
    request: &axum::http::Request<Body>,
    config: &AuthConfig,
) -> Result<AuthenticatedUser, AuthError> {
    if let Some(api_key) = request
        .headers()
        .get(&config.api_key_header)
        .and_then(|v| v.to_str().ok())
    {
        let mut user = validate_api_key_sync(api_key)?;

        if let Some(bot_id) = extract_bot_id_from_request(request, config) {
            user = user.with_current_bot(bot_id);
        }

        return Ok(user);
    }

    if let Some(auth_header) = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
    {
        // `Basic user:token` is how native clients (calendar, mail) present the
        // credential, since they have no field for a bearer token.
        let token = auth_header
            .strip_prefix(&config.bearer_prefix)
            .map(str::to_string)
            .or_else(|| super::tokens::basic_password_as_token(auth_header));
        if let Some(token) = token {
            let mut user = validate_bearer_token_sync(&token, config)?;

            if let Some(bot_id) = extract_bot_id_from_request(request, config) {
                user = user.with_current_bot(bot_id);
            }

            return Ok(user);
        }
    }

    // Browser WebSocket clients cannot set headers; allow the bearer
    // token via `?token=` query param for WS upgrades (/api/terminal/ws)
    // and XHR-delivered WebSocket session bootstraps.
    if let Some(tok) = request
        .uri()
        .query()
        .and_then(|q| {
            q.split('&').find_map(|kv| {
                let (k, v) = kv.split_once('=')?;
                if k == "token" && !v.is_empty() {
                    Some(v)
                } else {
                    None
                }
            })
        })
        .map(|v| v.replace("%20", " ").replace("%2B", "+").trim().to_string())
    {
        if let Some(token) = tok
            .strip_prefix(&config.bearer_prefix)
            .map(|s| s.to_string())
            .or_else(|| {
                if tok.starts_with("Bearer ") {
                    None
                } else {
                    Some(tok.clone())
                }
            })
        {
            let mut user = validate_bearer_token_sync(&token, config)?;

            if let Some(bot_id) = extract_bot_id_from_request(request, config) {
                user = user.with_current_bot(bot_id);
            }

            return Ok(user);
        }
    }

    if let Some(session_id) = extract_session_from_cookies(request, &config.session_cookie_name) {
        let mut user = validate_session_sync(&session_id)?;

        if let Some(bot_id) = extract_bot_id_from_request(request, config) {
            user = user.with_current_bot(bot_id);
        }

        return Ok(user);
    }

    Err(AuthError::MissingToken)
}

pub fn extract_bot_id_from_request(
    request: &axum::http::Request<Body>,
    config: &AuthConfig,
) -> Option<Uuid> {
    request
        .headers()
        .get(&config.bot_id_header)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
}

pub fn extract_session_from_cookies(
    request: &axum::http::Request<Body>,
    cookie_name: &str,
) -> Option<String> {
    request
        .headers()
        .get(axum::http::header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';').find_map(|cookie| {
                let (name, value) = cookie.trim().split_once('=')?;

                if name == cookie_name {
                    Some(value.to_string())
                } else {
                    None
                }
            })
        })
}
