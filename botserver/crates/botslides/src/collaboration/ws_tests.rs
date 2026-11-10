//! Offline unit tests for the collaboration token decoder (#2075).
//!
//! The slides WebSocket identifies a collaborator from the JWT payload segment.
//! `extract_user_from_token` only *decodes*; it does not verify the signature
//! (the token is minted by the platform auth service). The tests therefore pin
//! the decoding contract — accepted claim names, padding and base64url — and
//! the rejection paths that must fall back to a guest identity.

use super::extract_user_from_token;

// base64url({"sub":"u1","name":"Ana"})
const FULL: &str = "eyJzdWIiOiJ1MSIsIm5hbWUiOiJBbmEifQ";
// base64url({"sub":"u2"})
const SUB_ONLY: &str = "eyJzdWIiOiJ1MiJ9";
// base64url({"email":"a@b.test"})
const EMAIL_ONLY: &str = "eyJlbWFpbCI6ImFAYi50ZXN0In0";
// base64url({"user_id":"u3","preferred_username":"ana"})
const USER_ID: &str = "eyJ1c2VyX2lkIjoidTMiLCJwcmVmZXJyZWRfdXNlcm5hbWUiOiJhbmEifQ";
// base64url({"name":"Ana"})
const NO_IDENTITY: &str = "eyJuYW1lIjoiQW5hIn0";

fn token(payload: &str) -> String {
    format!("header.{payload}.signature")
}

#[test]
fn sub_and_name_are_both_returned() {
    assert_eq!(
        extract_user_from_token(&token(FULL)),
        Some(("u1".to_string(), "Ana".to_string()))
    );
}

#[test]
fn a_payload_with_only_sub_uses_it_as_the_display_name() {
    assert_eq!(
        extract_user_from_token(&token(SUB_ONLY)),
        Some(("u2".to_string(), "u2".to_string()))
    );
}

#[test]
fn email_is_the_last_resort_identity() {
    assert_eq!(
        extract_user_from_token(&token(EMAIL_ONLY)),
        Some(("a@b.test".to_string(), "a@b.test".to_string()))
    );
}

#[test]
fn user_id_and_preferred_username_are_accepted() {
    assert_eq!(
        extract_user_from_token(&token(USER_ID)),
        Some(("u3".to_string(), "ana".to_string()))
    );
}

#[test]
fn malformed_tokens_are_rejected() {
    assert_eq!(extract_user_from_token(""), None);
    assert_eq!(extract_user_from_token("no-dots"), None);
    assert_eq!(extract_user_from_token("only.two"), None);
    assert_eq!(extract_user_from_token("a.b.c.d"), None);
    // A payload that is not JSON, and one with a character outside base64url.
    assert_eq!(extract_user_from_token("h.bm90LWpzb24.s"), None);
    assert_eq!(extract_user_from_token("h.eyJzdWIiOiJ1NCJ9!.s"), None);
    // Valid base64 but no identity claim.
    assert_eq!(extract_user_from_token(&token(NO_IDENTITY)), None);
}

#[test]
fn padding_is_restored_for_payloads_of_any_length() {
    // The three payloads above have 0, 1 and 2 missing padding characters.
    for payload in [FULL, SUB_ONLY, EMAIL_ONLY] {
        assert!(
            extract_user_from_token(&token(payload)).is_some(),
            "payload {payload} must decode"
        );
    }
}
