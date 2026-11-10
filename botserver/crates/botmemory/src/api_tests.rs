//! Memory HTTP surface helpers: JWT claim extraction and the lenient base64url
//! decoder that feeds it.

use axum::http::{HeaderMap, HeaderValue, StatusCode};
use serde_json::json;
use uuid::Uuid;

use super::{
    b64_decode_flexible, bad_request, claim_uuid, jwt_claims, jwt_scope, jwt_user_id,
    labeled_memory, require_user,
};

/// Minimal base64url encoder so the tests build tokens without a new dependency.
fn b64url(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        out.push(ALPHABET[(n >> 18) as usize & 63] as char);
        out.push(ALPHABET[(n >> 12) as usize & 63] as char);
        if chunk.len() > 1 {
            out.push(ALPHABET[(n >> 6) as usize & 63] as char);
        }
        if chunk.len() > 2 {
            out.push(ALPHABET[n as usize & 63] as char);
        }
    }
    out
}

fn token_with(claims: serde_json::Value) -> String {
    let payload = b64url(claims.to_string().as_bytes());
    format!("header.{payload}.signature")
}

fn headers_with(token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        "authorization",
        HeaderValue::from_str(&format!("Bearer {token}")).expect("header"),
    );
    headers
}

const USER: Uuid = Uuid::from_u128(0xaa);
const BRANCH: Uuid = Uuid::from_u128(0xbb);

#[test]
fn the_decoder_accepts_standard_and_url_safe_alphabets() {
    assert_eq!(b64_decode_flexible("aGVsbG8="), Some(b"hello".to_vec()));
    assert_eq!(b64_decode_flexible("aGVsbG8"), Some(b"hello".to_vec()));

    let bytes = [0xfb, 0xff, 0xbf];
    let url = b64url(&bytes);
    let standard = url.replace('-', "+").replace('_', "/");
    assert_ne!(url, standard, "the sample must exercise both alphabets");
    assert_eq!(b64_decode_flexible(&standard), Some(bytes.to_vec()));
    assert_eq!(b64_decode_flexible(&url), Some(bytes.to_vec()));
}

#[test]
fn the_decoder_ignores_whitespace_and_stops_at_padding() {
    assert_eq!(b64_decode_flexible(" aGVs bG8= "), Some(b"hello".to_vec()));
    assert_eq!(b64_decode_flexible("aGVsbG8=trailing"), Some(b"hello".to_vec()));
    assert_eq!(b64_decode_flexible(""), Some(Vec::new()));
}

#[test]
fn the_decoder_rejects_invalid_input() {
    assert_eq!(b64_decode_flexible("aGVsbG8*"), None);
    assert_eq!(b64_decode_flexible("a"), None);
}

#[test]
fn a_missing_or_malformed_authorization_header_yields_no_claims() {
    assert!(jwt_claims(&HeaderMap::new()).is_none());

    let mut basic = HeaderMap::new();
    basic.insert("authorization", HeaderValue::from_static("Basic abc"));
    assert!(jwt_claims(&basic).is_none());

    let mut single = HeaderMap::new();
    single.insert("authorization", HeaderValue::from_static("Bearer onlyonepart"));
    assert!(jwt_claims(&single).is_none());

    let mut garbage = HeaderMap::new();
    garbage.insert("authorization", HeaderValue::from_static("Bearer a.*.c"));
    assert!(jwt_claims(&garbage).is_none());
}

#[test]
fn claims_are_decoded_from_the_bearer_token() {
    let headers = headers_with(&token_with(json!({ "sub": USER.to_string() })));
    let claims = jwt_claims(&headers).expect("claims");
    assert_eq!(claims["sub"], USER.to_string());
}

#[test]
fn the_first_known_claim_key_wins() {
    let claims = json!({
        "user_id": USER.to_string(),
        "sub": BRANCH.to_string(),
    });
    assert_eq!(claim_uuid(&claims, &["user_id", "sub"]), Some(USER));
    assert_eq!(claim_uuid(&claims, &["sub", "user_id"]), Some(BRANCH));
}

#[test]
fn an_unparsable_claim_value_is_skipped() {
    let claims = json!({ "user_id": "not-a-uuid", "sub": USER.to_string() });
    assert_eq!(claim_uuid(&claims, &["user_id", "sub"]), Some(USER));
    assert_eq!(claim_uuid(&json!({ "user_id": 7 }), &["user_id"]), None);
    assert_eq!(claim_uuid(&json!({}), &["user_id"]), None);
}

#[test]
fn the_user_and_scope_come_from_the_documented_claim_names() {
    let headers = headers_with(&token_with(json!({
        "sub": USER.to_string(),
        "branch_id": BRANCH.to_string(),
        "org_id": Uuid::nil().to_string(),
    })));
    assert_eq!(jwt_user_id(&headers), Some(USER));
    assert_eq!(jwt_scope(&headers), (Some(Uuid::nil()), Some(BRANCH)));

    let aliases = headers_with(&token_with(json!({
        "uid": USER.to_string(),
        "branch": BRANCH.to_string(),
        "org": Uuid::nil().to_string(),
    })));
    assert_eq!(jwt_user_id(&aliases), Some(USER));
    assert_eq!(jwt_scope(&aliases), (Some(Uuid::nil()), Some(BRANCH)));
}

#[test]
fn an_anonymous_request_has_no_scope_and_is_refused() {
    let anonymous = HeaderMap::new();
    assert_eq!(jwt_scope(&anonymous), (None, None));
    let error = require_user(&anonymous).expect_err("must refuse");
    assert_eq!(error.0, StatusCode::UNAUTHORIZED);
    assert_eq!(error.1, "Authentication required");
}

#[test]
fn an_authenticated_request_resolves_the_owner() {
    let headers = headers_with(&token_with(json!({ "sub": USER.to_string() })));
    assert_eq!(require_user(&headers), Ok(USER));
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
    assert_eq!(bad_request("bad".to_string()), (StatusCode::BAD_REQUEST, "bad".to_string()));
}
