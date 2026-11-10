//! Outgoing-webhook signature check.
//!
//! A Teams outgoing webhook is registered with a base64 security token that
//! Microsoft shows once. Every delivery carries
//! `Authorization: HMAC base64(HMAC-SHA256(base64decode(token), body))`.
//!
//! `Mac::verify_slice` performs the comparison in constant time, so a caller
//! cannot learn the expected digest byte by byte from the response time.

use base64::Engine as _;
use hmac::{Hmac, Mac};
use sha2::Sha256;

use super::AuthError;

/// Verifies `signature` (the header value, base64) against the raw body.
pub fn verify(raw_body: &[u8], signature: &str, security_token: &str) -> Result<(), AuthError> {
    let key = decode_security_token(security_token);
    if key.is_empty() {
        return Err(AuthError::Unconfigured);
    }

    let provided = base64::engine::general_purpose::STANDARD
        .decode(signature.trim())
        .map_err(|_| AuthError::InvalidSignature)?;

    let mut mac = Hmac::<Sha256>::new_from_slice(&key).map_err(|e| {
        log::error!("Teams webhook HMAC key error: {e}");
        AuthError::InvalidSignature
    })?;
    mac.update(raw_body);
    mac.verify_slice(&provided)
        .map_err(|_| AuthError::InvalidSignature)
}

/// The security token is base64, and the HMAC key is its decoded bytes. An
/// operator who stored a plain random string instead gets the raw bytes as the
/// key rather than a permanently rejected webhook — the token is still secret,
/// only the derivation differs.
fn decode_security_token(security_token: &str) -> Vec<u8> {
    let token = security_token.trim();
    match base64::engine::general_purpose::STANDARD.decode(token) {
        Ok(decoded) => decoded,
        Err(_) => token.as_bytes().to_vec(),
    }
}
