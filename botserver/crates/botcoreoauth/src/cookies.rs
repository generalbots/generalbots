//! `Set-Cookie` builders for the OAuth login/link flow (#2130).
//!
//! Two cookies are involved:
//!
//! * `session` — the authenticated session minted on a successful callback.
//! * `gb_oauth_nonce` — short-lived, `HttpOnly` value binding a pending state
//!   token to the browser that started the flow.
//!
//! Both are `Secure` (a plaintext hop must never carry them), `HttpOnly` (no
//! script reads them) and `SameSite=Lax`. `Lax` — not `Strict` — is required
//! for the nonce cookie: the provider redirects back with a cross-site
//! top-level `GET`, and `Strict` would withhold the cookie on exactly that
//! request, breaking every flow. `SameSite=None` is never needed here because
//! the callback is a top-level navigation, not an embedded cross-site request.
//! `localhost` counts as a trustworthy origin in Chrome/Firefox, so the
//! `Secure` flag does not break local development over plain HTTP.

use axum::http::header::HeaderValue;
use axum::http::HeaderMap;

use crate::state::STATE_TTL_SECS;

/// Name of the authenticated session cookie.
pub const SESSION_COOKIE: &str = "session";
/// Name of the browser-binding nonce cookie.
pub const NONCE_COOKIE: &str = "gb_oauth_nonce";
/// Session lifetime, in seconds.
pub const SESSION_MAX_AGE: i64 = 86_400;
/// Nonce lifetime: the same window as the state token.
pub const NONCE_MAX_AGE: i64 = STATE_TTL_SECS;
/// Cookie path of the login callback (`/auth/oauth/:provider/callback`).
pub const LOGIN_NONCE_PATH: &str = "/auth/oauth";
/// Cookie path of the Settings account-link callback
/// (`/api/oauth/:provider/callback`).
pub const LINK_NONCE_PATH: &str = "/api/oauth";

/// Builds the `session` cookie. Returns `None` when the token cannot be
/// represented as a header value (never the case for our UUID tokens).
pub fn session_cookie(token: &str) -> Option<HeaderValue> {
    header(&format!(
        "{SESSION_COOKIE}={token}; Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age={SESSION_MAX_AGE}"
    ))
}

/// Builds the short-lived nonce cookie for a pending flow.
pub fn nonce_cookie(nonce: &str, path: &str) -> Option<HeaderValue> {
    header(&format!(
        "{NONCE_COOKIE}={nonce}; Path={path}; HttpOnly; Secure; SameSite=Lax; Max-Age={NONCE_MAX_AGE}"
    ))
}

/// Expires the nonce cookie once the flow finished (success or failure).
pub fn clear_nonce_cookie(path: &str) -> Option<HeaderValue> {
    header(&format!(
        "{NONCE_COOKIE}=; Path={path}; HttpOnly; Secure; SameSite=Lax; Max-Age=0"
    ))
}

/// Reads a cookie value from the request headers.
pub fn cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    for header in headers.get_all(axum::http::header::COOKIE) {
        let Ok(raw) = header.to_str() else { continue };
        for pair in raw.split(';') {
            let Some((key, value)) = pair.trim().split_once('=') else {
                continue;
            };
            if key.trim() != name {
                continue;
            }
            let value = value.trim();
            if value.is_empty() || value.chars().any(|c| c.is_control()) {
                continue;
            }
            return Some(value.to_string());
        }
    }
    None
}

fn header(value: &str) -> Option<HeaderValue> {
    match HeaderValue::from_str(value) {
        Ok(value) => Some(value),
        Err(e) => {
            log::warn!("refusing to emit an unusable OAuth cookie: {e}");
            None
        }
    }
}
