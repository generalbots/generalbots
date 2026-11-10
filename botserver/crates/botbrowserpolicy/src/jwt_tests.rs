//! Tests for verified JWT claims in the browser control plane (#2112).
//!
//! Every fixture is a real HS256 token: the point of the fix is that a token
//! whose signature does not match — including the hand-built `alg:none` and
//! placeholder-signature tokens the old decoder accepted — yields no claims.

use super::test_support::{
    an_hour_ago, b64, claims, hs256, in_an_hour, process_secret, token_with_header, valid_token,
    OTHER_SECRET,
};
use super::verified_claims;

const SUBJECT: &str = "00000000-0000-0000-0000-000000000001";

#[test]
fn a_valid_token_yields_its_claims() {
    let token = valid_token(SUBJECT);
    let claims = verified_claims(&token).expect("verified claims");
    assert_eq!(claims.sub, SUBJECT);
    assert!(claims.exp > chrono::Utc::now().timestamp());
}

#[test]
fn surrounding_whitespace_does_not_hide_a_valid_token() {
    let token = format!("  {}\n", valid_token(SUBJECT));
    assert_eq!(
        verified_claims(&token).map(|c| c.sub),
        Some(SUBJECT.to_string())
    );
}

#[test]
fn a_tampered_payload_is_rejected() {
    let valid = valid_token("00000000-0000-0000-0000-000000000002");
    let parts: Vec<&str> = valid.split('.').collect();
    let forged = format!(
        "{}.{}.{}",
        parts[0],
        b64(claims(SUBJECT, in_an_hour()).as_bytes()),
        parts[2]
    );
    assert!(verified_claims(&forged).is_none());
}

#[test]
fn an_alg_none_token_is_rejected() {
    // Classic bypass: unsigned header naming an arbitrary identity.
    let payload = claims(SUBJECT, in_an_hour());
    let forged = format!(
        "{}.{}.",
        b64(br#"{"alg":"none","typ":"JWT"}"#),
        b64(payload.as_bytes())
    );
    assert!(verified_claims(&forged).is_none());

    // Same header but with a signature attached — still not HS256.
    let signed_none = token_with_header(r#"{"alg":"none","typ":"JWT"}"#, &payload, &process_secret());
    assert!(verified_claims(&signed_none).is_none());
}

#[test]
fn an_expired_token_is_rejected() {
    let token = hs256(&claims(SUBJECT, an_hour_ago()), &process_secret());
    assert!(verified_claims(&token).is_none());
}

#[test]
fn a_token_without_a_signature_is_rejected() {
    let payload = claims(SUBJECT, in_an_hour());
    let unsigned = format!(
        "{}.{}.",
        b64(br#"{"alg":"HS256","typ":"JWT"}"#),
        b64(payload.as_bytes())
    );
    assert!(verified_claims(&unsigned).is_none());

    // Two segments only — never a token.
    let truncated = format!(
        "{}.{}",
        b64(br#"{"alg":"HS256","typ":"JWT"}"#),
        b64(payload.as_bytes())
    );
    assert!(verified_claims(&truncated).is_none());
}

#[test]
fn a_token_signed_with_another_key_is_rejected() {
    let token = hs256(&claims(SUBJECT, in_an_hour()), OTHER_SECRET);
    assert!(verified_claims(&token).is_none());
}

#[test]
fn a_token_without_a_subject_or_expiry_is_rejected() {
    let no_subject = hs256(&format!(r#"{{"exp":{}}}"#, in_an_hour()), &process_secret());
    assert!(verified_claims(&no_subject).is_none());

    let no_expiry = hs256(&format!(r#"{{"sub":"{SUBJECT}"}}"#), &process_secret());
    assert!(verified_claims(&no_expiry).is_none());

    // A `uid` claim is not an identity: it is not part of the verified shape.
    let legacy_uid = hs256(&format!(r#"{{"uid":"{SUBJECT}","exp":{}}}"#, in_an_hour()), &process_secret());
    assert!(verified_claims(&legacy_uid).is_none());
}

#[test]
fn malformed_and_empty_tokens_yield_no_claims() {
    for value in ["", "   ", "a", "a.b", "a.b.c.d", "..", "not-a-token", "!!!.@@@.###"] {
        assert!(verified_claims(value).is_none(), "{value:?}");
    }
}
