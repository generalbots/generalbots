//! HTTP-surface identity tests (#2112).
//!
//! `resolve_user_id` / `resolve_user_id_sse` are the only places the browser
//! policy API turns a request into a user id. These tests pin the rule: the id
//! comes from a signature-verified token, and any failure is a 401 — never a
//! fallback to decoded-but-unverified claims.

use super::*;
use crate::jwt::test_support::{
    an_hour_ago, b64, claims, hs256, in_an_hour, process_secret, valid_token, OTHER_SECRET,
};

const USER_A: &str = "00000000-0000-0000-0000-00000000000a";
const USER_B: &str = "00000000-0000-0000-0000-00000000000b";

fn headers_with(token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    if let Ok(value) = axum::http::HeaderValue::from_str(&format!("Bearer {token}")) {
        headers.insert("authorization", value);
    }
    headers
}

fn uuid_of(raw: &str) -> Uuid {
    Uuid::parse_str(raw).unwrap_or_default()
}

/// A forged `alg:none` token naming `sub` — the pre-#2112 bypass.
fn forged_alg_none(sub: &str) -> String {
    format!(
        "{}.{}.",
        b64(br#"{"alg":"none","typ":"JWT"}"#),
        b64(claims(sub, in_an_hour()).as_bytes())
    )
}

#[test]
fn a_verified_token_resolves_to_its_subject() {
    let resolved = resolve_user_id(&headers_with(&valid_token(USER_A))).expect("verified user");
    assert_eq!(resolved, uuid_of(USER_A));
}

#[test]
fn a_verified_subject_that_is_not_a_uuid_is_refused() {
    // Zitadel-style numeric subjects have no local user row in this API.
    let numeric = hs256(&claims("123456789", in_an_hour()), &process_secret());
    assert!(resolve_user_id(&headers_with(&numeric)).is_err());
}

#[test]
fn missing_forged_tampered_and_expired_tokens_are_refused() {
    assert!(resolve_user_id(&HeaderMap::new()).is_err());

    // A token without the `Bearer ` scheme is not an Authorization header.
    let mut bare = HeaderMap::new();
    if let Ok(value) = axum::http::HeaderValue::from_str(&valid_token(USER_A)) {
        bare.insert("authorization", value);
    }
    assert!(resolve_user_id(&bare).is_err());

    assert!(resolve_user_id(&headers_with(&forged_alg_none(USER_A))).is_err());

    // Signed with another key.
    let foreign = hs256(&claims(USER_A, in_an_hour()), OTHER_SECRET);
    assert!(resolve_user_id(&headers_with(&foreign)).is_err());

    // Tampered payload carrying a different subject.
    let valid = valid_token(USER_B);
    let parts: Vec<&str> = valid.split('.').collect();
    let tampered = format!(
        "{}.{}.{}",
        parts[0],
        b64(claims(USER_A, in_an_hour()).as_bytes()),
        parts[2]
    );
    assert!(resolve_user_id(&headers_with(&tampered)).is_err());

    // Expired.
    let expired = hs256(&claims(USER_A, an_hour_ago()), &process_secret());
    assert!(resolve_user_id(&headers_with(&expired)).is_err());

    // A `uid` claim is not an identity.
    let legacy_uid = hs256(
        &format!(r#"{{"uid":"{USER_A}","exp":{}}}"#, in_an_hour()),
        &process_secret(),
    );
    assert!(resolve_user_id(&headers_with(&legacy_uid)).is_err());
}

#[test]
fn sse_resolution_prefers_the_header_over_the_query_token() {
    let resolved = resolve_user_id_sse(&headers_with(&valid_token(USER_A)), &valid_token(USER_B))
        .expect("header wins");
    assert_eq!(resolved, uuid_of(USER_A));
}

#[test]
fn sse_resolution_accepts_a_verified_query_token() {
    let resolved =
        resolve_user_id_sse(&HeaderMap::new(), &valid_token(USER_B)).expect("query token");
    assert_eq!(resolved, uuid_of(USER_B));
}

#[test]
fn sse_resolution_never_falls_back_when_the_header_token_fails() {
    let forged = forged_alg_none(USER_A);
    assert!(resolve_user_id_sse(&headers_with(&forged), &valid_token(USER_B)).is_err());
}

#[test]
fn sse_resolution_requires_some_token() {
    assert!(resolve_user_id_sse(&HeaderMap::new(), "").is_err());
    assert!(resolve_user_id_sse(&HeaderMap::new(), "   ").is_err());
    assert!(resolve_user_id_sse(&HeaderMap::new(), "not-a-token").is_err());
}

fn token_with_email(email: &str) -> String {
    let payload = format!(
        r#"{{"sub":"{USER_A}","email":"{email}","exp":{}}}"#,
        in_an_hour()
    );
    hs256(&payload, &process_secret())
}

#[test]
fn the_admin_gate_reads_only_a_verified_email_claim() {
    let admin_email = token_with_email("admin@localhost");
    assert_eq!(
        verified_email(&headers_with(&admin_email)).expect("verified"),
        Some("admin@localhost".to_string())
    );

    // Verified token without an email claim is not an admin, but is not an
    // authentication failure either.
    assert_eq!(
        verified_email(&headers_with(&valid_token(USER_A))).expect("verified"),
        None
    );
}

#[test]
fn the_admin_gate_fails_closed_on_an_unverified_email() {
    // Forged `alg:none` token claiming the dev-admin email — the pre-#2112
    // bypass into `PUT /api/browser/policy`.
    let forged = format!(
        "{}.{}.",
        b64(br#"{"alg":"none","typ":"JWT"}"#),
        b64(format!(r#"{{"email":"admin@localhost","exp":{}}}"#, in_an_hour()).as_bytes())
    );
    assert!(verified_email(&headers_with(&forged)).is_err());

    // A token signed with another key must not promote itself either.
    let foreign = hs256(
        &format!(r#"{{"sub":"{USER_A}","email":"admin@localhost","exp":{}}}"#, in_an_hour()),
        OTHER_SECRET,
    );
    assert!(verified_email(&headers_with(&foreign)).is_err());

    assert!(verified_email(&HeaderMap::new()).is_err());
}
