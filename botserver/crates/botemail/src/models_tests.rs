//! Offline tests for the mail identity helpers (issue #2075): JWT payload
//! decoding, the stable user UUID and the header-based identity resolution.

use super::*;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};

fn headers_with(name: &'static str, value: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        name,
        axum::http::HeaderValue::from_str(value).expect("valid header value"),
    );
    headers
}

fn jwt_with_payload(claims: &str) -> String {
    let payload = URL_SAFE_NO_PAD.encode(claims.as_bytes());
    format!("header.{payload}.signature")
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
fn jwt_payload_decoding_accepts_base64url_with_and_without_padding() {
    let encoded = URL_SAFE_NO_PAD.encode(b"{\"sub\":\"abc\"}");

    assert_eq!(
        decode_jwt_payload(&encoded),
        Some(b"{\"sub\":\"abc\"}".to_vec())
    );
    assert_eq!(
        decode_jwt_payload(&format!("{encoded}=")),
        Some(b"{\"sub\":\"abc\"}".to_vec())
    );
}

#[test]
fn jwt_payload_decoding_rejects_characters_outside_the_alphabet() {
    assert_eq!(decode_jwt_payload("a+b/c"), None);
    assert_eq!(decode_jwt_payload("!!!"), None);
    assert_eq!(decode_jwt_payload(""), Some(Vec::new()));
}

#[test]
fn the_x_user_id_header_wins_and_needs_no_token() {
    let headers = headers_with("x-user-id", "00000000-0000-0000-0000-00000000002a");
    assert_eq!(
        extract_user_from_session(&headers),
        Ok(Uuid::from_u128(42))
    );
}

#[test]
fn an_invalid_x_user_id_header_is_ignored() {
    let headers = headers_with("x-user-id", "not-a-uuid");
    assert_eq!(
        extract_user_from_session(&headers),
        Err("Missing Authorization header".to_string())
    );
}

#[test]
fn a_missing_or_non_bearer_authorization_header_is_rejected() {
    assert_eq!(
        extract_user_from_session(&HeaderMap::new()),
        Err("Missing Authorization header".to_string())
    );

    let headers = headers_with("authorization", "Basic dXNlcjpwYXNz");
    assert_eq!(
        extract_user_from_session(&headers),
        Err("Authorization header is not a Bearer token".to_string())
    );
}

#[test]
fn an_unknown_opaque_session_token_is_rejected() {
    let headers = headers_with("authorization", "Bearer gb_unknown_token");
    assert_eq!(
        extract_user_from_session(&headers),
        Err("Unknown or expired session token".to_string())
    );
}

#[test]
fn malformed_jwt_shapes_are_rejected() {
    let two_parts = headers_with("authorization", "Bearer header.payload");
    assert_eq!(
        extract_user_from_session(&two_parts),
        Err("Malformed JWT token".to_string())
    );

    let bad_payload = headers_with("authorization", "Bearer header.!!!.signature");
    assert_eq!(
        extract_user_from_session(&bad_payload),
        Err("Invalid JWT payload encoding".to_string())
    );

    let not_json = headers_with("authorization", &jwt_with_payload("not json"));
    assert!(extract_user_from_session(&not_json)
        .expect_err("invalid JSON")
        .starts_with("Invalid JWT payload JSON:"));
}

#[test]
fn the_jwt_subject_claim_resolves_to_the_stable_uuid() {
    let headers = headers_with(
        "authorization",
        &format!("Bearer {}", jwt_with_payload(r#"{"sub":"123456789"}"#)),
    );

    assert_eq!(
        extract_user_from_session(&headers),
        Ok(stable_user_uuid("123456789"))
    );
}

#[test]
fn the_jwt_user_id_claim_is_the_fallback() {
    let headers = headers_with(
        "authorization",
        &format!("Bearer {}", jwt_with_payload(r#"{"user_id":"42"}"#)),
    );

    assert_eq!(
        extract_user_from_session(&headers),
        Ok(stable_user_uuid("42"))
    );
}

#[test]
fn a_jwt_without_identity_or_with_an_empty_one_is_rejected() {
    let anonymous = headers_with(
        "authorization",
        &format!("Bearer {}", jwt_with_payload(r#"{"scope":"mail"}"#)),
    );
    assert_eq!(
        extract_user_from_session(&anonymous),
        Err("JWT carries no identity claim".to_string())
    );

    let empty = headers_with(
        "authorization",
        &format!("Bearer {}", jwt_with_payload(r#"{"sub":""}"#)),
    );
    assert_eq!(
        extract_user_from_session(&empty),
        Err("JWT identity claim is empty".to_string())
    );
}
