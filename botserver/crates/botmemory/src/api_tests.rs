//! Memory HTTP surface helpers: verified caller extraction (#2123).
//!
//! The identity of a memory caller now comes from
//! `botsecurity_core::verified_jwt::verify_saas_jwt`, so these tests mint real
//! HS256 tokens with the secret the verifier resolves in this process and pin
//! the accept/reject boundary: a valid token yields the caller, and a tampered,
//! unsigned (`alg: none`), expired or foreign-key token yields nothing at all.

use axum::http::{HeaderMap, HeaderValue, StatusCode};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use uuid::Uuid;

use botsecurity_core::verified_jwt::resolve_saas_jwt_secret;

use super::{bad_request, caller_from_headers, labeled_memory, require_caller};

const USER: Uuid = Uuid::from_u128(0xaa);
const BRANCH: Uuid = Uuid::from_u128(0xbb);
const ORG: Uuid = Uuid::from_u128(0xcc);

fn b64url(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Signs `claims` as HS256 with `secret`, the way the login path mints tokens.
fn sign_with(claims: serde_json::Value, secret: &str) -> String {
    let header = b64url(br#"{"alg":"HS256","typ":"JWT"}"#);
    let payload = b64url(claims.to_string().as_bytes());
    let signing_input = format!("{header}.{payload}");
    let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(secret.as_bytes()) else {
        return String::new();
    };
    mac.update(signing_input.as_bytes());
    format!(
        "{signing_input}.{}",
        b64url(&mac.finalize().into_bytes())
    )
}

/// A token signed with the secret the process verifies against.
fn token(claims: serde_json::Value) -> String {
    sign_with(claims, &resolve_saas_jwt_secret())
}

fn live_claims() -> serde_json::Value {
    serde_json::json!({
        "sub": USER.to_string(),
        "org_id": ORG.to_string(),
        "branch_id": BRANCH.to_string(),
        "exp": chrono::Utc::now().timestamp() + 3600,
    })
}

fn headers_with(token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let value = HeaderValue::from_str(&format!("Bearer {token}"))
        .unwrap_or_else(|_| HeaderValue::from_static("Bearer "));
    headers.insert("authorization", value);
    headers
}

#[test]
fn a_valid_token_yields_the_caller() {
    let caller = caller_from_headers(&headers_with(&token(live_claims()))).expect("caller");
    assert_eq!(caller.user_id, USER);
    assert_eq!(caller.org_id, Some(ORG));
    assert_eq!(caller.branch_id, Some(BRANCH));
}

#[test]
fn a_verified_token_without_tenant_claims_has_no_scope() {
    let claims = serde_json::json!({
        "sub": USER.to_string(),
        "exp": chrono::Utc::now().timestamp() + 3600,
    });
    let caller = caller_from_headers(&headers_with(&token(claims))).expect("caller");
    assert_eq!(caller.user_id, USER);
    assert_eq!(caller.org_id, None);
    assert_eq!(caller.branch_id, None);
}

#[test]
fn a_tampered_payload_is_refused() {
    // Re-encode the payload with another user, keeping the original signature.
    let valid = token(live_claims());
    let mut parts: Vec<String> = valid.split('.').map(str::to_string).collect();
    let forged_claims = serde_json::json!({
        "sub": Uuid::from_u128(0xdead).to_string(),
        "org_id": ORG.to_string(),
        "branch_id": BRANCH.to_string(),
        "exp": chrono::Utc::now().timestamp() + 3600,
    });
    parts[1] = b64url(forged_claims.to_string().as_bytes());
    let tampered = parts.join(".");

    assert_eq!(caller_from_headers(&headers_with(&tampered)), None);
}

#[test]
fn an_unsigned_alg_none_token_is_refused() {
    let header = b64url(br#"{"alg":"none","typ":"JWT"}"#);
    let payload = b64url(live_claims().to_string().as_bytes());
    let unsigned = format!("{header}.{payload}.");

    assert_eq!(caller_from_headers(&headers_with(&unsigned)), None);
}

#[test]
fn an_expired_token_is_refused() {
    let claims = serde_json::json!({
        "sub": USER.to_string(),
        "branch_id": BRANCH.to_string(),
        "exp": 1,
    });
    assert_eq!(caller_from_headers(&headers_with(&token(claims))), None);
}

#[test]
fn a_token_without_expiry_is_refused() {
    let claims = serde_json::json!({ "sub": USER.to_string() });
    assert_eq!(caller_from_headers(&headers_with(&token(claims))), None);
}

#[test]
fn a_token_signed_with_another_key_is_refused() {
    let forged = sign_with(live_claims(), "attacker-controlled-secret");
    assert_eq!(caller_from_headers(&headers_with(&forged)), None);
}

#[test]
fn a_verified_token_whose_subject_is_not_a_uuid_is_refused() {
    let claims = serde_json::json!({
        "sub": "zitadel-user-42",
        "exp": chrono::Utc::now().timestamp() + 3600,
    });
    assert_eq!(caller_from_headers(&headers_with(&token(claims))), None);
}

#[test]
fn a_missing_or_malformed_authorization_header_yields_no_caller() {
    assert!(caller_from_headers(&HeaderMap::new()).is_none());

    let mut basic = HeaderMap::new();
    basic.insert("authorization", HeaderValue::from_static("Basic abc"));
    assert!(caller_from_headers(&basic).is_none());

    for token in ["", "onlyonepart", "a.b", ".."] {
        assert_eq!(caller_from_headers(&headers_with(token)), None, "{token:?}");
    }
}

#[test]
fn an_anonymous_request_is_unauthorized() {
    let error = require_caller(&HeaderMap::new()).expect_err("must refuse");
    assert_eq!(error.0, StatusCode::UNAUTHORIZED);
    assert_eq!(error.1, "Authentication required");
}

#[test]
fn an_authenticated_request_resolves_the_caller() {
    let caller = require_caller(&headers_with(&token(live_claims()))).expect("caller");
    assert_eq!(caller.user_id, USER);
}

#[test]
fn the_response_helpers_shape_the_payload() {
    let memory = crate::models::UserMemory {
        id: Uuid::from_u128(1),
        org_id: None,
        branch_id: None,
        owner_user_id: USER,
        scope: "user".to_string(),
        kind: "note".to_string(),
        content: "hello".to_string(),
        source: "manual".to_string(),
        confidence: 0.9,
        pinned: false,
        superseded_by: None,
        embedding_ref: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    let value = labeled_memory("created", memory);
    assert_eq!(value["status"], "created");
    assert_eq!(value["memory"]["content"], "hello");
    assert_eq!(
        bad_request("bad".to_string()),
        (StatusCode::BAD_REQUEST, "bad".to_string())
    );
}
