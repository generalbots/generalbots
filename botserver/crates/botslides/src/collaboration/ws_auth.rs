//! Deck WebSocket authentication (split from [`super::ws`]).
//!
//! #2075 — the token used to be base64-decoded and trusted as-is, so anyone
//! could join a deck as another user, or as a writable "guest", by handing over
//! a hand-made `{"alg":"none"}` payload. Identity now comes only from
//! [`botsecurity_core::verified_jwt::verify_saas_jwt`]: HS256 pinned, HMAC
//! compared in constant time, `exp`/`nbf` enforced. A refusal carries a stable
//! machine-readable code so the client sees why the upgrade was rejected.

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use botsecurity_core::verified_jwt::{self, JwtVerifyError};
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct WsAuthQuery {
    #[serde(default)]
    pub token: String,
}

/// Deterministic signing secret used only by this module's unit tests.
///
/// Production builds never read it: [`verify_token`] resolves the persisted
/// SaaS secret through `verify_saas_jwt`. Tests inject this value into
/// `verify_hs256_jwt` so they are independent of the process working directory
/// and of a provisioned `botserver-stack`, while exercising the very same
/// header / algorithm / HMAC / expiry checks production relies on.
#[cfg(test)]
pub(crate) const TEST_JWT_SECRET: &str = "botslides-ws-auth-test-only-secret";

/// Verifies `token` into claims.
///
/// The `#[cfg(test)]` overload is the seam that keeps the unit tests hermetic;
/// the production overload is unchanged and keeps failing closed when no secret
/// has been persisted.
#[cfg(test)]
fn verify_token(token: &str) -> Result<verified_jwt::VerifiedJwt, JwtVerifyError> {
    verified_jwt::verify_hs256_jwt(token, TEST_JWT_SECRET)
}

#[cfg(not(test))]
fn verify_token(token: &str) -> Result<verified_jwt::VerifiedJwt, JwtVerifyError> {
    verified_jwt::verify_saas_jwt(token)
}

/// Resolves `(user_id, display_name)` from a **verified** deck token.
///
/// The display name prefers the verified `email` claim and falls back to the
/// subject, so a collaborator never gets to choose their own label.
pub fn extract_user_from_token(token: &str) -> Result<(String, String), JwtVerifyError> {
    let verified = verify_token(token)?;
    let name = verified
        .email
        .clone()
        .unwrap_or_else(|| verified.sub.clone());
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
            "code": "slides_ws_unauthorized",
            "reason": reason_code(&reason),
        })),
    )
        .into_response()
}

#[cfg(test)]
#[path = "ws_auth_tests.rs"]
mod ws_auth_tests;
