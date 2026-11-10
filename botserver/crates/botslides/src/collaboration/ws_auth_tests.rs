//! Offline unit tests for the collaboration token verifier (#2075).
//!
//! The slides WebSocket identifies a collaborator from a JWT that must now be
//! *verified*, not merely decoded: `extract_user_from_token` delegates to
//! `botsecurity_core::verified_jwt::verify_saas_jwt`, which pins HS256,
//! compares the HMAC in constant time and enforces `exp`/`nbf`. These tests
//! mint real tokens with the secret the verifier resolves in this process and
//! pin the accept/reject boundary: a forged, unsigned (`alg: none`), expired or
//! foreign-key token yields an error instead of an identity, so the upgrade is
//! refused and no writable guest session is created.

use super::extract_user_from_token;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use botsecurity_core::verified_jwt::{resolve_saas_jwt_secret, JwtVerifyError};
use hmac::{Hmac, Mac};
use sha2::Sha256;

fn b64url(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Signs `claims` as HS256 with `secret`, the way the login path mints tokens.
fn sign_with(claims: serde_json::Value, secret: &str) -> String {
    let header = b64url(br#"{"alg":"HS256","typ":"JWT"}"#);
    let payload = b64url(claims.to_string().as_bytes());
    let signing_input = format!("{header}.{payload}");
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("hmac key");
    mac.update(signing_input.as_bytes());
    format!("{signing_input}.{}", b64url(&mac.finalize().into_bytes()))
}

fn token(claims: serde_json::Value) -> String {
    sign_with(claims, &resolve_saas_jwt_secret().expect("saas jwt secret"))
}

fn claims() -> serde_json::Value {
    serde_json::json!({
        "sub": "u1",
        "email": "ana@example.com",
        "exp": chrono::Utc::now().timestamp() + 3600,
    })
}

#[test]
fn a_valid_token_yields_the_collaborator() {
    assert_eq!(
        extract_user_from_token(&token(claims())),
        Ok(("u1".to_string(), "ana@example.com".to_string()))
    );
}

#[test]
fn the_subject_is_the_display_name_when_no_email_is_present() {
    let token = token(serde_json::json!({
        "sub": "u2",
        "exp": chrono::Utc::now().timestamp() + 3600,
    }));
    assert_eq!(
        extract_user_from_token(&token),
        Ok(("u2".to_string(), "u2".to_string()))
    );
}

#[test]
fn a_tampered_payload_is_refused() {
    // Same signature, another subject: the identity claim must not be trusted.
    let valid = token(claims());
    let mut parts: Vec<String> = valid.split('.').map(str::to_string).collect();
    let forged = serde_json::json!({
        "sub": "someone-else",
        "email": "victim@example.com",
        "exp": chrono::Utc::now().timestamp() + 3600,
    });
    parts[1] = b64url(forged.to_string().as_bytes());
    let tampered = parts.join(".");

    assert_eq!(
        extract_user_from_token(&tampered),
        Err(JwtVerifyError::InvalidSignature)
    );
}

#[test]
fn an_unsigned_alg_none_token_is_refused() {
    let header = b64url(br#"{"alg":"none","typ":"JWT"}"#);
    let payload = b64url(claims().to_string().as_bytes());
    let unsigned = format!("{header}.{payload}.");

    assert_eq!(
        extract_user_from_token(&unsigned),
        Err(JwtVerifyError::UnsupportedAlgorithm)
    );
}

#[test]
fn an_expired_token_is_refused() {
    let token = token(serde_json::json!({
        "sub": "u1",
        "email": "ana@example.com",
        "exp": 1,
    }));
    assert_eq!(
        extract_user_from_token(&token),
        Err(JwtVerifyError::Expired)
    );
}

#[test]
fn a_token_without_expiry_is_refused() {
    let token = token(serde_json::json!({ "sub": "u1" }));
    assert_eq!(
        extract_user_from_token(&token),
        Err(JwtVerifyError::MissingExpiry)
    );
}

#[test]
fn a_token_signed_with_another_key_is_refused() {
    let forged = sign_with(claims(), "attacker-controlled-secret");
    assert_eq!(
        extract_user_from_token(&forged),
        Err(JwtVerifyError::InvalidSignature)
    );
}

#[test]
fn a_token_without_subject_is_refused() {
    let token = token(serde_json::json!({
        "email": "ana@example.com",
        "exp": chrono::Utc::now().timestamp() + 3600,
    }));
    assert_eq!(
        extract_user_from_token(&token),
        Err(JwtVerifyError::MissingSubject)
    );
}

#[test]
fn malformed_tokens_are_rejected() {
    for token in ["", "no-dots", "only.two", "a.b.c.d"] {
        assert_eq!(
            extract_user_from_token(token),
            Err(JwtVerifyError::Malformed),
            "{token:?}"
        );
    }
}

#[test]
fn a_header_that_is_not_json_is_refused() {
    // Three segments, but the header is not JSON.
    assert_eq!(
        extract_user_from_token("h.bm90LWpzb24.s"),
        Err(JwtVerifyError::InvalidHeader)
    );
}
