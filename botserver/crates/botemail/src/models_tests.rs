//! Offline tests for the mail identity helpers (issues #2075, #2123): the
//! stable user UUID and the header-based identity resolution, which now only
//! trusts a signature-verified JWT.

use super::*;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use hmac::{Hmac, Mac};
use sha2::Sha256;

/// Secret the fixtures sign with; handed to `extract_user_with_secret` so the
/// tests never depend on the process environment.
const SECRET: &str = "unit-test-secret-for-mail-identity-0123456789";
const OTHER_SECRET: &str = "another-key-that-nobody-should-accept-9876";

fn headers_with(name: &'static str, value: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        name,
        axum::http::HeaderValue::from_str(value).expect("valid header value"),
    );
    headers
}

fn bearer(token: &str) -> HeaderMap {
    headers_with("authorization", &format!("Bearer {token}"))
}

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
fn canonical_uuids_pass_through_unchanged() {
    let raw = "6ba7b810-9dad-11d1-80b4-00c04fd430c8";
    assert_eq!(stable_user_uuid(raw).to_string(), raw);
}

#[test]
fn non_uuid_subjects_map_to_the_published_rfc_4122_v5_vector() {
    // UUIDv5(NAMESPACE_DNS, "zitadel:<id>"), computed with the reference
    // implementation (RFC 4122 appendix).
    assert_eq!(
        stable_user_uuid("123456789").to_string(),
        "08574835-da22-54b4-8680-655f0c1e6546"
    );
    assert_eq!(
        stable_user_uuid("user@example.com").to_string(),
        "ea31c760-5e96-5a57-93ea-d33f8f9b36f0"
    );
}

#[test]
fn the_same_subject_always_yields_the_same_uuid() {
    let first = stable_user_uuid("zitadel-user-7");
    let second = stable_user_uuid("zitadel-user-7");
    assert_eq!(first, second);
    assert_ne!(first, stable_user_uuid("zitadel-user-8"));
}

#[test]
fn an_x_user_id_header_alone_is_not_an_identity() {
    // #2122 — the header is caller-supplied; only a verified token counts.
    let headers = headers_with("x-user-id", "00000000-0000-0000-0000-00000000002a");
    assert_eq!(
        extract_user_with_secret(&headers, SECRET),
        Err("Missing Authorization header".to_string())
    );
}

#[test]
fn an_invalid_x_user_id_header_is_ignored() {
    let headers = headers_with("x-user-id", "not-a-uuid");
    assert_eq!(
        extract_user_with_secret(&headers, SECRET),
        Err("Missing Authorization header".to_string())
    );
}

#[test]
fn a_missing_or_non_bearer_authorization_header_is_rejected() {
    assert_eq!(
        extract_user_with_secret(&HeaderMap::new(), SECRET),
        Err("Missing Authorization header".to_string())
    );

    let headers = headers_with("authorization", "Basic dXNlcjpwYXNz");
    assert_eq!(
        extract_user_with_secret(&headers, SECRET),
        Err("Authorization header is not a Bearer token".to_string())
    );
}

#[test]
fn an_unknown_opaque_session_token_is_rejected() {
    let headers = bearer("gb_unknown_token");
    assert_eq!(
        extract_user_with_secret(&headers, SECRET),
        Err("Unknown or expired session token".to_string())
    );
}

#[test]
fn a_valid_token_resolves_to_the_expected_user() {
    let payload = format!(r#"{{"sub":"123456789","email":"a@b.c","exp":{}}}"#, in_an_hour());
    let headers = bearer(&token(&payload, SECRET));

    assert_eq!(
        extract_user_with_secret(&headers, SECRET),
        Ok(stable_user_uuid("123456789"))
    );
}

#[test]
fn the_legacy_user_id_claim_still_resolves() {
    let payload = format!(r#"{{"user_id":"42","exp":{}}}"#, in_an_hour());
    let headers = bearer(&token(&payload, SECRET));

    assert_eq!(
        extract_user_with_secret(&headers, SECRET),
        Ok(stable_user_uuid("42"))
    );
}

#[test]
fn a_forged_alg_none_token_identifying_an_admin_is_rejected() {
    // #2123 — the payload names an admin but there is no signature.
    let payload = format!(r#"{{"sub":"admin","exp":{}}}"#, in_an_hour());
    let forged = format!(
        "{}.{}.",
        b64(br#"{"alg":"none","typ":"JWT"}"#),
        b64(payload.as_bytes())
    );
    let headers = bearer(&forged);

    let error = extract_user_with_secret(&headers, SECRET).expect_err("forged token");
    assert!(error.starts_with("Invalid JWT token:"), "{error}");
}

#[test]
fn a_token_signed_with_another_key_is_rejected() {
    let payload = format!(r#"{{"sub":"admin","exp":{}}}"#, in_an_hour());
    let headers = bearer(&token(&payload, OTHER_SECRET));

    let error = extract_user_with_secret(&headers, SECRET).expect_err("wrong key");
    assert!(error.contains("signature"), "{error}");
}

#[test]
fn a_tampered_payload_is_rejected() {
    let payload = format!(r#"{{"sub":"user-1","exp":{}}}"#, in_an_hour());
    let valid = token(&payload, SECRET);
    let parts: Vec<&str> = valid.split('.').collect();
    let forged_payload = format!(r#"{{"sub":"admin","exp":{}}}"#, in_an_hour());
    let tampered = format!("{}.{}.{}", parts[0], b64(forged_payload.as_bytes()), parts[2]);
    let headers = bearer(&tampered);

    let error = extract_user_with_secret(&headers, SECRET).expect_err("tampered payload");
    assert!(error.contains("signature"), "{error}");
}

#[test]
fn an_expired_token_is_rejected() {
    let payload = format!(r#"{{"sub":"user-1","exp":{}}}"#, an_hour_ago());
    let headers = bearer(&token(&payload, SECRET));

    let error = extract_user_with_secret(&headers, SECRET).expect_err("expired");
    assert!(error.contains("expired"), "{error}");
}

#[test]
fn a_token_without_an_expiry_is_rejected() {
    let headers = bearer(&token(r#"{"sub":"user-1"}"#, SECRET));

    let error = extract_user_with_secret(&headers, SECRET).expect_err("no exp");
    assert!(error.contains("expiry"), "{error}");
}

#[test]
fn a_token_without_a_subject_is_rejected() {
    let payload = format!(r#"{{"email":"a@b.c","exp":{}}}"#, in_an_hour());
    let headers = bearer(&token(&payload, SECRET));

    let error = extract_user_with_secret(&headers, SECRET).expect_err("no subject");
    assert!(error.contains("identity"), "{error}");
}

#[test]
fn malformed_jwt_shapes_are_rejected() {
    let two_parts = bearer("header.payload");
    let error = extract_user_with_secret(&two_parts, SECRET).expect_err("two segments");
    assert!(error.starts_with("Invalid JWT token:"), "{error}");

    let bad_payload = bearer("header.!!!.signature");
    let error = extract_user_with_secret(&bad_payload, SECRET).expect_err("bad base64");
    assert!(error.starts_with("Invalid JWT token:"), "{error}");
}
