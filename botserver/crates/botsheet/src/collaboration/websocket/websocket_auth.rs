//! Sheet WebSocket authentication (split from [`super::websocket`]).
//!
//! #2136 — the token used to be base64-decoded and trusted as-is, so anyone
//! could join a sheet as another user, or as a writable "guest", by handing
//! over a hand-made `{"alg":"none"}` payload. Identity now comes only from
//! [`botsecurity_core::verified_jwt::verify_saas_jwt`]: HS256 pinned, HMAC
//! compared in constant time, `exp`/`nbf` enforced. A refusal carries a stable
//! machine-readable code so the client sees why the upgrade was rejected.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use botsecurity_core::verified_jwt::{verify_saas_jwt, JwtVerifyError};
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct WsAuthQuery {
    #[serde(default)]
    pub token: String,
}

/// Resolves `(user_id, display_name)` from a **verified** sheet token.
///
/// The display name prefers the verified `email` claim and falls back to the
/// subject, so a collaborator never gets to choose their own label.
pub fn extract_user_from_token(token: &str) -> Result<(String, String), JwtVerifyError> {
    let verified = verify_saas_jwt(token)?;
    let name = verified.email.clone().unwrap_or_else(|| verified.sub.clone());
    Ok((verified.sub, name))
}

/// Stable machine-readable code for a refusal, safe to log and to return.
pub fn reason_code(reason: &JwtVerifyError) -> &'static str {
    match reason {
        JwtVerifyError::Malformed => "malformed_token",
        JwtVerifyError::InvalidHeader => "invalid_header",
        JwtVerifyError::UnsupportedAlgorithm => "unsupported_algorithm",
        JwtVerifyError::InvalidSignature => "invalid_signature",
        JwtVerifyError::MissingSecret => "verifier_unavailable",
        JwtVerifyError::InvalidPayload => "invalid_payload",
        JwtVerifyError::MissingSubject => "missing_subject",
        JwtVerifyError::MissingExpiry => "missing_expiry",
        JwtVerifyError::Expired => "token_expired",
        JwtVerifyError::NotYetValid => "token_not_yet_valid",
    }
}

/// Refuses the WebSocket upgrade with a machine-readable 401.
///
/// The connection never upgrades, so no presence row and no write-capable
/// socket exists for a rejected token.
pub fn unauthorized(reason: JwtVerifyError) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        [
            ("x-auth-error", "invalid_token"),
            ("x-auth-reason", reason_code(&reason)),
        ],
        Json(serde_json::json!({
            "error": "invalid_token",
            "code": "sheet_ws_unauthorized",
            "reason": reason_code(&reason),
        })),
    )
        .into_response()
}

#[cfg(test)]
#[path = "websocket_auth_tests.rs"]
mod websocket_auth_tests;
