//! Offline unit tests for the plan WebSocket token decoder (#2068).
//!
//! The collaboration socket authenticates by reading the JWT payload segment
//! sent as a query parameter. `extract_user_from_token` is a *display* decoder:
//! it never verifies a signature (the server already did) and it must return
//! `None` for anything it cannot read, so the caller can fall back to a guest
//! identity instead of panicking or trusting a half-decoded claim.

use super::extract_user_from_token;

/// Builds a three-segment token around an already base64url-encoded payload.
fn token(payload: &str) -> String {
    format!("header.{payload}.signature")
}

#[test]
fn sub_and_name_are_returned_when_both_are_present() {
    // {"sub":"u-1","name":"Ada"}
    let got = extract_user_from_token(&token("eyJzdWIiOiJ1LTEiLCJuYW1lIjoiQWRhIn0"));
    assert_eq!(got, Some(("u-1".to_string(), "Ada".to_string())));
}

#[test]
fn a_missing_name_falls_back_to_the_subject() {
    // {"user_id":"u-2"}
    let got = extract_user_from_token(&token("eyJ1c2VyX2lkIjoidS0yIn0"));
    assert_eq!(got, Some(("u-2".to_string(), "u-2".to_string())));
}

#[test]
fn email_serves_as_both_subject_and_name() {
    // {"email":"a@b.test"}
    let got = extract_user_from_token(&token("eyJlbWFpbCI6ImFAYi50ZXN0In0"));
    assert_eq!(
        got,
        Some(("a@b.test".to_string(), "a@b.test".to_string()))
    );
}

#[test]
fn preferred_username_and_display_name_are_honoured() {
    // {"sub":"u-3","preferred_username":"ada"}
    let got = extract_user_from_token(&token("eyJzdWIiOiJ1LTMiLCJwcmVmZXJyZWRfdXNlcm5hbWUiOiJhZGEifQ"));
    assert_eq!(got, Some(("u-3".to_string(), "ada".to_string())));

    // {"sub":"u-4","display_name":"Grace"}
    let got = extract_user_from_token(&token("eyJzdWIiOiJ1LTQiLCJkaXNwbGF5X25hbWUiOiJHcmFjZSJ9"));
    assert_eq!(got, Some(("u-4".to_string(), "Grace".to_string())));
}

#[test]
fn a_payload_without_any_subject_claim_is_rejected() {
    // {"name":"only-name"} — a name alone cannot identify the collaborator.
    assert_eq!(extract_user_from_token(&token("eyJuYW1lIjoib25seS1uYW1lIn0")), None);
}

#[test]
fn malformed_tokens_are_rejected() {
    assert_eq!(extract_user_from_token(""), None);
    assert_eq!(extract_user_from_token("only-one-segment"), None);
    assert_eq!(extract_user_from_token("a.eyJzdWIiOiJ1LTEifQ"), None);
    assert_eq!(
        extract_user_from_token("a.eyJzdWIiOiJ1LTEifQ.c.d"),
        None,
        "four segments is not a JWT"
    );
}

#[test]
fn a_non_json_payload_is_rejected() {
    // base64url("not json")
    assert_eq!(extract_user_from_token(&token("bm90IGpzb24")), None);
}

#[test]
fn illegal_base64_characters_are_rejected() {
    assert_eq!(extract_user_from_token(&token("not*base64!")), None);
}

#[test]
fn base64url_alphabet_is_translated_back_to_standard() {
    // {"sub":"?"} encodes to `eyJzdWIiOiI/In0` in standard base64 and
    // `eyJzdWIiOiI_In0` in base64url; the decoder must accept the URL form.
    let got = extract_user_from_token(&token("eyJzdWIiOiI_In0"));
    assert_eq!(got, Some(("?".to_string(), "?".to_string())));
}

#[test]
fn padding_characters_are_tolerated() {
    // The same payload with explicit padding must decode identically.
    let padded = extract_user_from_token(&token("eyJzdWIiOiJ1LTEiLCJuYW1lIjoiQWRhIn0="));
    assert_eq!(padded, Some(("u-1".to_string(), "Ada".to_string())));
}
