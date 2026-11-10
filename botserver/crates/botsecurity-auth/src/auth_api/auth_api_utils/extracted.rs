//! Structured credential fields read from a request.

use crate::auth_api::config::AuthConfig;
use axum::body::Body;
use tracing::{debug, warn};
use uuid::Uuid;

use super::extraction::{extract_bot_id_from_request, extract_session_from_cookies};
use super::tokens::basic_password_as_token;

pub struct ExtractedAuthData {
    pub api_key: Option<String>,
    pub bearer_token: Option<String>,
    pub query_token: Option<String>,
    pub session_id: Option<String>,
    pub bot_id: Option<Uuid>,
}

impl ExtractedAuthData {
    pub fn from_request(request: &axum::http::Request<Body>, config: &AuthConfig) -> Self {
        let api_key = request
            .headers()
            .get(&config.api_key_header)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        // Browser WebSocket clients cannot set headers; allow the bearer token
        // via the `?token=` query param for WS upgrades (/api/terminal/ws,
        // /api/browser/.../ws) and XHR-delivered WS session bootstraps.
        let query_token: Option<String> = request
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
            .map(|v| v.replace("%20", " ").replace("%2B", "+").trim().to_string());

        // Debug: log raw Authorization header
        let raw_auth = request
            .headers()
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok());

        if let Some(auth) = raw_auth {
            // Truncate on a character boundary: slicing a fixed byte offset can
            // split a multi-byte character and panic.
            let preview: String = auth.chars().take(50).collect();
            debug!("Raw Authorization header: {preview}");
        } else {
            // An absent Authorization header is the normal case for anonymous
            // traffic on public endpoints (/health, /ws, /api/bot/public, the
            // pre-login desktop calls). The authorization decision is made
            // below, so this is diagnostic only and must not be a warning.
            debug!(
                "No Authorization header found in request to {}",
                request.uri().path()
            );
        }

        // Native clients (calendar, mail) authenticate with HTTP Basic, which
        // has no field for a bearer token. The password field carries the token
        // so those clients reach the same verified path as `Bearer`. The token
        // is still validated below; only the transport is widened.
        let basic_token: Option<String> = raw_auth.and_then(basic_password_as_token);

        let bearer_token: Option<String> = basic_token.or_else(|| raw_auth.and_then(|s| {
            // Try exact prefix first
            if let Some(token) = s.strip_prefix(&config.bearer_prefix) {
                return Some(token.to_string());
            }
            // Case-insensitive fallback for proxies that normalize header values.
            // `get` keeps the slice on a character boundary, so a header that is
            // not ASCII cannot panic the request path.
            let prefix_len = config.bearer_prefix.len();
            if let Some(head) = s.get(..prefix_len) {
                if head.eq_ignore_ascii_case(&config.bearer_prefix) {
                    return Some(s[prefix_len..].to_string());
                }
            }
            let preview: String = s.chars().take(20).collect();
            warn!(
                "Authorization header present but failed to extract bearer token. Prefix expected: '{}', raw: '{preview}...'",
                config.bearer_prefix,
            );
            None
        }));

        let session_id = extract_session_from_cookies(request, &config.session_cookie_name);

        // `X-User-ID` is caller-supplied and therefore NOT a credential
        // (#2122). It is only ever logged, for correlation with the internal
        // `api.exec` executor; identity comes exclusively from a verified
        // API key, bearer/JWT or session above.
        if let Some(correlation) = request
            .headers()
            .get("X-User-ID")
            .and_then(|v| v.to_str().ok())
        {
            debug!(
                "Ignoring X-User-ID correlation header '{}': not an authentication source",
                botlib::text::truncate_at_char_boundary(correlation, 36)
            );
        }

        let bot_id = extract_bot_id_from_request(request, config);

        Self {
            api_key,
            bearer_token,
            query_token,
            session_id,
            bot_id,
        }
    }
}
