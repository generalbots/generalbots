//! Authenticity checks for inbound Microsoft Teams activities (#2113).
//!
//! `/api/msteams/messages` is an anonymous route — Teams posts from its own
//! infrastructure, without a user session — so the only proof a delivery is
//! genuine is the credential the caller presents. The handler used to accept
//! any JSON with `type: "message"` and route it into a bot turn; this module
//! is the gate that runs before the body is even parsed.
//!
//! Two credentials are accepted, matching the two ways Teams reaches a bot:
//!
//! | Header | Used by | Check |
//! |---|---|---|
//! | `Authorization: Bearer <jwt>` | Bot Framework channel | RS256 signature against the Bot Framework OpenID key set, plus `iss`, `aud`, `exp`, `nbf` |
//! | `Authorization: HMAC <sig>` | outgoing webhook | `base64(HMAC-SHA256(security_token, raw_body))` |
//!
//! Both are compared in constant time and neither secret ever reaches a log
//! line. When neither credential is configured the gate fails closed: an
//! unauthenticated route that answers is worse than one that does not.

mod hmac;
mod jwks;
mod jwt;

#[cfg(test)]
#[path = "fixtures_tests.rs"]
mod fixtures;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

#[cfg(test)]
#[path = "jwt_tests.rs"]
mod jwt_tests;

use axum::http::{header::AUTHORIZATION, HeaderMap, StatusCode};

/// Bot config key holding the Microsoft App ID — the `aud` of every Bot
/// Framework token addressed to this bot.
pub const APP_ID_KEY: &str = "teams-app-id";

/// Bot config key holding the base64 security token of an outgoing webhook.
pub const SECURITY_TOKEN_KEY: &str = "teams-security-token";

/// Credentials the gate can check against. Either field may be empty, which
/// disables that scheme; both empty rejects every delivery.
#[derive(Debug, Clone, Default)]
pub struct TeamsCredentials {
    pub app_id: String,
    pub security_token: String,
}

impl TeamsCredentials {
    pub fn new(app_id: &str, security_token: &str) -> Self {
        Self {
            app_id: app_id.trim().to_string(),
            security_token: security_token.trim().to_string(),
        }
    }

    /// Whether at least one scheme can be verified at all.
    pub fn is_configured(&self) -> bool {
        !self.app_id.is_empty() || !self.security_token.is_empty()
    }
}

/// Why a delivery was refused. Every variant answers `401`: the caller is not
/// told which credential is missing, and the detail goes to the log only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthError {
    /// Neither `teams-app-id` nor `teams-security-token` is configured.
    Unconfigured,
    /// No `Authorization` header at all.
    MissingHeader,
    /// The header is neither `Bearer` nor `HMAC`.
    UnsupportedScheme,
    /// The HMAC does not cover this body with the configured token.
    InvalidSignature,
    /// The JWT is absent, malformed, expired or signed by another key.
    InvalidToken,
}

impl AuthError {
    pub fn status(self) -> StatusCode {
        StatusCode::UNAUTHORIZED
    }

    /// Log-safe description: no header value and no secret.
    pub fn reason(self) -> &'static str {
        match self {
            Self::Unconfigured => {
                "no Teams credential configured (teams-app-id / teams-security-token)"
            }
            Self::MissingHeader => "missing Authorization header",
            Self::UnsupportedScheme => "unsupported Authorization scheme",
            Self::InvalidSignature => "HMAC signature mismatch",
            Self::InvalidToken => "Bot Framework token rejected",
        }
    }
}

/// Gate for one inbound activity. `raw_body` must be the bytes exactly as
/// received: the HMAC covers them verbatim and a re-serialized JSON value
/// would never match.
pub async fn authorize(
    headers: &HeaderMap,
    raw_body: &[u8],
    credentials: &TeamsCredentials,
) -> Result<(), AuthError> {
    if !credentials.is_configured() {
        return Err(AuthError::Unconfigured);
    }

    let header = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or(AuthError::MissingHeader)?;

    if let Some(token) = strip_scheme(header, "Bearer") {
        if credentials.app_id.is_empty() {
            return Err(AuthError::Unconfigured);
        }
        return jwt::verify(token, &credentials.app_id).await;
    }

    if let Some(signature) = strip_scheme(header, "HMAC") {
        if credentials.security_token.is_empty() {
            return Err(AuthError::Unconfigured);
        }
        return hmac::verify(raw_body, signature, &credentials.security_token);
    }

    Err(AuthError::UnsupportedScheme)
}

/// `Bearer <value>` → `Some(value)`. The scheme name is case-insensitive in
/// HTTP, and several Bot Framework SDKs send `bearer`.
fn strip_scheme<'a>(header: &'a str, scheme: &str) -> Option<&'a str> {
    let header = header.trim();
    let (name, value) = header.split_once(' ')?;
    if !name.eq_ignore_ascii_case(scheme) {
        return None;
    }
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}
