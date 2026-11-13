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

pub fn extract_user_from_token(token: &str) -> Result<(String, String), JwtVerifyError> {
    if token.trim().is_empty() {
        return Ok(("anon".to_string(), "Guest".to_string()));
    }
    let verified = verify_saas_jwt(token)?;
    let name = verified
        .email
        .clone()
        .unwrap_or_else(|| verified.sub.clone());
    Ok((verified.sub, name))
}

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

pub fn unauthorized(reason: JwtVerifyError) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        [
            ("x-auth-error", "invalid_token"),
            ("x-auth-reason", reason_code(&reason)),
        ],
        Json(serde_json::json!({
            "error": "invalid_token",
            "code": "canvas_ws_unauthorized",
            "reason": reason_code(&reason),
        })),
    )
        .into_response()
}
