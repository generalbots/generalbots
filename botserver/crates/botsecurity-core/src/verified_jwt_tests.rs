//! Tests for the verified JWT helper (issue #2123).

use super::*;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use hmac::{Hmac, Mac};
use sha2::Sha256;

const SECRET: &str = "unit-test-secret-for-verified-jwt-0123456789";
const OTHER_SECRET: &str = "another-key-that-nobody-should-accept-9876";

fn b64(input: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(input)
}

fn sign(signing_input: &str, secret: &str) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("hmac key");
    mac.update(signing_input.as_bytes());
    b64(&mac.finalize().into_bytes())
}

fn token_with_header(header: &str, payload: &str, secret: &str) -> String {
    let signing_input = format!("{}.{}", b64(header.as_bytes()), b64(payload.as_bytes()));
    let signature = sign(&signing_input, secret);
    format!("{signing_input}.{signature}")
}

fn token(payload: &str, secret: &str) -> String {
    token_with_header(r#"{"alg":"HS256","typ":"JWT"}"#, payload, secret)
}

fn in_an_hour() -> i64 {
    chrono::Utc::now().timestamp() + 3600
}

fn an_hour_ago() -> i64 {
    chrono::Utc::now().timestamp() - 3600
}

#[test]
fn a_valid_token_yields_the_expected_identity() {
    let branch = Uuid::new_v4();
    let org = Uuid::new_v4();
    let payload = format!(
        r#"{{"sub":"123456789","email":"admin@example.com","org_id":"{org}","branch_id":"{branch}","exp":{}}}"#,
        in_an_hour()
    );
    let claims = verify_hs256_jwt(&token(&payload, SECRET), SECRET).expect("valid token");

    assert_eq!(claims.sub, "123456789");
    assert_eq!(claims.email.as_deref(), Some("admin@example.com"));
    assert_eq!(claims.org_id, Some(org));
    assert_eq!(claims.branch_id, Some(branch));
}

#[test]
fn an_unsigned_alg_none_token_is_rejected() {
    // Forged header, no signature at all — the classic bypass.
    let payload = format!(r#"{{"sub":"admin","exp":{}}}"#, in_an_hour());
    let forged = format!(
        "{}.{}.",
        b64(br#"{"alg":"none","typ":"JWT"}"#),
        b64(payload.as_bytes())
    );
    assert_eq!(
        verify_hs256_jwt(&forged, SECRET),
        Err(JwtVerifyError::UnsupportedAlgorithm)
    );
}

#[test]
fn an_unsigned_alg_none_token_with_a_fake_signature_is_rejected() {
    let payload = format!(r#"{{"sub":"admin","exp":{}}}"#, in_an_hour());
    let forged = token_with_header(r#"{"alg":"none","typ":"JWT"}"#, &payload, SECRET);
    assert_eq!(
        verify_hs256_jwt(&forged, SECRET),
        Err(JwtVerifyError::UnsupportedAlgorithm)
    );
}

#[test]
fn a_token_signed_with_another_key_is_rejected() {
    let payload = format!(r#"{{"sub":"admin","exp":{}}}"#, in_an_hour());
    let wrong_key = token(&payload, OTHER_SECRET);
    assert_eq!(
        verify_hs256_jwt(&wrong_key, SECRET),
        Err(JwtVerifyError::InvalidSignature)
    );
}

#[test]
fn a_tampered_payload_is_rejected() {
    let payload = format!(r#"{{"sub":"user-1","exp":{}}}"#, in_an_hour());
    let valid = token(&payload, SECRET);
    let parts: Vec<&str> = valid.split('.').collect();
    let forged_payload = format!(r#"{{"sub":"admin","exp":{}}}"#, in_an_hour());
    let tampered = format!("{}.{}.{}", parts[0], b64(forged_payload.as_bytes()), parts[2]);
    assert_eq!(
        verify_hs256_jwt(&tampered, SECRET),
        Err(JwtVerifyError::InvalidSignature)
    );
}

#[test]
fn an_expired_token_is_rejected() {
    let payload = format!(r#"{{"sub":"user-1","exp":{}}}"#, an_hour_ago());
    assert_eq!(
        verify_hs256_jwt(&token(&payload, SECRET), SECRET),
        Err(JwtVerifyError::Expired)
    );
}

#[test]
fn a_token_without_an_expiry_is_rejected() {
    let payload = r#"{"sub":"user-1"}"#;
    assert_eq!(
        verify_hs256_jwt(&token(payload, SECRET), SECRET),
        Err(JwtVerifyError::MissingExpiry)
    );
}

#[test]
fn a_token_without_a_subject_is_rejected() {
    let payload = format!(r#"{{"email":"user@example.com","exp":{}}}"#, in_an_hour());
    assert_eq!(
        verify_hs256_jwt(&token(&payload, SECRET), SECRET),
        Err(JwtVerifyError::MissingSubject)
    );

    let empty = format!(r#"{{"sub":"  ","exp":{}}}"#, in_an_hour());
    assert_eq!(
        verify_hs256_jwt(&token(&empty, SECRET), SECRET),
        Err(JwtVerifyError::MissingSubject)
    );
}

#[test]
fn the_legacy_user_id_claim_is_accepted() {
    let payload = format!(r#"{{"user_id":"42","exp":{}}}"#, in_an_hour());
    let claims = verify_hs256_jwt(&token(&payload, SECRET), SECRET).expect("valid token");
    assert_eq!(claims.sub, "42");
}

#[test]
fn asymmetric_algorithms_are_rejected() {
    let payload = format!(r#"{{"sub":"user-1","exp":{}}}"#, in_an_hour());
    let token = token_with_header(r#"{"alg":"RS256","typ":"JWT"}"#, &payload, SECRET);
    assert_eq!(
        verify_hs256_jwt(&token, SECRET),
        Err(JwtVerifyError::UnsupportedAlgorithm)
    );
}

#[test]
fn malformed_tokens_never_panic() {
    for value in ["", "a", "a.b", "..", "!!!.payload.sig", "a.@@@.c"] {
        assert!(verify_hs256_jwt(value, SECRET).is_err(), "{value:?}");
    }
    let header_only = format!("{}.payload.sig", b64(br#"{"alg":"HS256"}"#));
    assert!(verify_hs256_jwt(&header_only, SECRET).is_err());
}

#[test]
fn an_empty_secret_can_never_verify() {
    let payload = format!(r#"{{"sub":"user-1","exp":{}}}"#, in_an_hour());
    let token = token(&payload, "");
    assert_eq!(
        verify_hs256_jwt(&token, ""),
        Err(JwtVerifyError::MissingSecret)
    );
}

#[test]
fn the_bearer_token_ignores_a_trailing_credential() {
    let mut headers = HeaderMap::new();
    headers.insert(
        "authorization",
        "Bearer abc.def.ghi,extra-credential"
            .parse()
            .expect("header value"),
    );
    assert_eq!(bearer_token(&headers), Some("abc.def.ghi"));

    let mut basic = HeaderMap::new();
    basic.insert("authorization", "Basic dXNlcjpwYXNz".parse().expect("header"));
    assert_eq!(bearer_token(&basic), None);
}
