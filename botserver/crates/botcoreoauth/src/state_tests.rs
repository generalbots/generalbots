//! Offline tests for the signed, single-use OAuth state (#2130).
//!
//! Everything here is pure: a [`StateSigner`] built from a fixed test key, an
//! explicit clock, and no HTTP/DB/Vault. The properties under test are the
//! ones the issue calls out — forged state, tampered state, another browser,
//! replay and expiry.

use crate::binding::{consume_bound_nonce, consume_bound_user_nonce, BindingError};
use crate::nonce_store;
use crate::state::{generate_nonce, now_secs, OAuthState, StateError, StateSigner, STATE_TTL_SECS};
use crate::OAuthProvider;
use axum::http::{HeaderMap, HeaderValue};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};

const KEY: &[u8] = b"state-signing-key-for-tests";

fn signer() -> StateSigner {
    StateSigner::from_secret(KEY).expect("test signer")
}

fn state() -> OAuthState {
    OAuthState::new(OAuthProvider::Google, Some("/settings".to_string()))
}

fn headers_with_nonce(nonce: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let value = format!("other=1; gb_oauth_nonce={nonce}; trailing=2");
    headers.insert(
        axum::http::header::COOKIE,
        HeaderValue::from_str(&value).expect("cookie header"),
    );
    headers
}

#[test]
fn a_signed_state_round_trips() {
    let state = state();
    let token = signer().sign(&state).expect("sign");
    let verified = signer().verify(&token).expect("verify");
    assert_eq!(verified.token, state.token);
    assert_eq!(verified.provider, state.provider);
    assert_eq!(verified.redirect_after.as_deref(), Some("/settings"));
    assert_eq!(verified.nonce, state.nonce);
    assert_eq!(verified.created_at, state.created_at);
}

#[test]
fn the_token_carries_a_version_and_two_dots() {
    let token = signer().sign(&state()).expect("sign");
    let parts: Vec<&str> = token.split('.').collect();
    assert_eq!(parts.len(), 3, "token shape is v1.<payload>.<mac>: {token}");
    assert_eq!(parts[0], "v1");
    assert!(!parts[1].is_empty());
    assert!(!parts[2].is_empty());
}

#[test]
fn a_forged_state_is_rejected_as_a_bad_signature() {
    // Someone else's key: a perfectly valid payload, wrong MAC.
    let forged = StateSigner::from_secret(b"attacker-key")
        .expect("signer")
        .sign(&state())
        .expect("sign");
    assert_eq!(signer().verify(&forged).err(), Some(StateError::BadSignature));
}

#[test]
fn a_tampered_payload_is_rejected() {
    let token = signer().sign(&state()).expect("sign");
    let mut parts: Vec<String> = token.split('.').map(|p| p.to_string()).collect();

    // Re-encode the payload with an attacker-chosen user id, keeping the MAC.
    let mut payload: serde_json::Value =
        serde_json::from_slice(&URL_SAFE_NO_PAD.decode(&parts[1]).expect("payload base64"))
            .expect("payload json");
    payload["user_id"] = serde_json::Value::String("victim".to_string());
    payload["redirect_after"] = serde_json::Value::String("https://evil.example".to_string());
    parts[1] = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&payload).expect("json"));

    let tampered = parts.join(".");
    assert_eq!(signer().verify(&tampered).err(), Some(StateError::BadSignature));
}

#[test]
fn the_mac_is_checked_before_any_field_is_trusted() {
    // A payload that is not even JSON, with a garbage MAC, is a signature
    // failure (not a parse failure): the MAC check runs first.
    let token = format!("v1.{}.{}", "AAAA", "BBBB");
    assert_eq!(signer().verify(&token).err(), Some(StateError::BadSignature));
}

#[test]
fn malformed_tokens_are_rejected() {
    let signer = signer();
    for token in [
        "",
        "v1",
        "v1.payload",
        "v1..mac",
        "v2.payload.mac",
        "not-a-token",
        "v1.payload.mac.extra",
    ] {
        let error = signer.verify(token).err();
        assert!(
            matches!(
                error,
                Some(StateError::Malformed) | Some(StateError::BadSignature)
            ),
            "token {token:?} must be refused, got {error:?}"
        );
    }
}

#[test]
fn an_expired_state_is_rejected() {
    let mut state = state();
    let now = now_secs();
    state.created_at = now - STATE_TTL_SECS - 1;
    let token = signer().sign(&state).expect("sign");
    assert_eq!(signer().verify(&token).err(), Some(StateError::Expired));
    assert_eq!(
        signer().verify_at(&token, now).err(),
        Some(StateError::Expired)
    );
}

#[test]
fn a_state_from_the_future_is_rejected() {
    // Beyond the tolerated clock skew: a token minted "in the future" must not
    // extend its own lifetime.
    let mut state = state();
    state.created_at = now_secs() + 3600;
    let token = signer().sign(&state).expect("sign");
    assert_eq!(signer().verify(&token).err(), Some(StateError::Expired));
}

#[test]
fn a_fresh_state_is_accepted_at_the_boundary() {
    let mut state = state();
    let now = 1_700_000_000;
    state.created_at = now - STATE_TTL_SECS;
    let token = signer().sign(&state).expect("sign");
    assert!(signer().verify_at(&token, now).is_ok());
}

#[test]
fn an_empty_key_is_refused_so_the_mac_is_never_unkeyed() {
    assert!(StateSigner::from_secret(b"").is_err());
    assert!(StateSigner::from_secret(b" ").is_ok());
}

#[test]
fn every_state_gets_its_own_nonce() {
    let first = state();
    let second = state();
    assert_ne!(first.nonce, second.nonce);
    assert_ne!(generate_nonce(), generate_nonce());
    assert!(!first.nonce.is_empty());
    // base64url of 32 bytes: 43 characters, no padding.
    assert_eq!(first.nonce.len(), 43);
    assert!(!first.nonce.contains('='));
}

#[test]
fn a_state_without_a_nonce_is_malformed() {
    let mut state = state();
    state.nonce = String::new();
    let token = signer().sign(&state).expect("sign");
    assert_eq!(signer().verify(&token).err(), Some(StateError::Malformed));
}

#[test]
fn a_nonce_issued_to_one_browser_is_rejected_in_another() {
    let state = state();
    nonce_store::issue(&state.nonce, now_secs());

    // No cookie at all: the callback was opened by a different browser.
    assert_eq!(
        consume_bound_nonce(&HeaderMap::new(), &state.nonce),
        Err(BindingError::MissingCookie)
    );

    // Someone else's cookie value.
    let other = generate_nonce();
    assert_eq!(
        consume_bound_nonce(&headers_with_nonce(&other), &state.nonce),
        Err(BindingError::CookieMismatch)
    );

    // The real browser still works afterwards: a failed binding attempt must
    // not burn the pending nonce.
    assert_eq!(
        consume_bound_nonce(&headers_with_nonce(&state.nonce), &state.nonce),
        Ok(())
    );
}

#[test]
fn a_nonce_is_single_use() {
    let state = state();
    nonce_store::issue(&state.nonce, now_secs());
    let headers = headers_with_nonce(&state.nonce);

    assert_eq!(consume_bound_nonce(&headers, &state.nonce), Ok(()));
    // Replaying the exact same callback (state + cookie) is refused.
    assert_eq!(
        consume_bound_nonce(&headers, &state.nonce),
        Err(BindingError::Replay)
    );
}

#[test]
fn a_nonce_that_was_never_issued_is_refused() {
    let nonce = generate_nonce();
    assert_eq!(
        consume_bound_nonce(&headers_with_nonce(&nonce), &nonce),
        Err(BindingError::Replay)
    );
}

#[test]
fn an_expired_nonce_is_refused() {
    let nonce = generate_nonce();
    let now = now_secs();
    nonce_store::issue(&nonce, now - STATE_TTL_SECS - 1);
    assert!(nonce_store::consume(&nonce, now).is_none());
    assert!(!nonce_store::is_pending(&nonce, now));
}

#[test]
fn an_empty_nonce_never_matches() {
    assert!(nonce_store::consume("", now_secs()).is_none());
    assert!(!nonce_store::is_pending("", now_secs()));
    nonce_store::issue("", now_secs());
    assert!(nonce_store::consume("", now_secs()).is_none());
}

#[test]
fn issuing_prunes_expired_entries() {
    let stale = generate_nonce();
    let fresh = generate_nonce();
    let now = now_secs();
    nonce_store::issue(&stale, now - STATE_TTL_SECS - 1);
    nonce_store::issue(&fresh, now);
    // The insert pruned the expired entry, so it is gone even before `now`
    // reaches its own expiry.
    assert!(!nonce_store::is_pending(&stale, now));
    assert!(nonce_store::is_pending(&fresh, now));
}

#[test]
fn a_link_nonce_remembers_the_authenticated_user() {
    let state = state();
    let user_id = "7f3d9d1c-0000-4000-8000-000000000001";
    nonce_store::issue_for_user(&state.nonce, Some(user_id), now_secs());

    let bound = consume_bound_user_nonce(&headers_with_nonce(&state.nonce), &state.nonce)
        .expect("bound nonce must be accepted");
    assert_eq!(bound.as_deref(), Some(user_id));

    // Single use applies to the user-bound variant as well.
    assert_eq!(
        consume_bound_user_nonce(&headers_with_nonce(&state.nonce), &state.nonce),
        Err(BindingError::Replay)
    );
}

#[test]
fn a_login_nonce_carries_no_user() {
    let state = state();
    nonce_store::issue(&state.nonce, now_secs());
    let bound = consume_bound_user_nonce(&headers_with_nonce(&state.nonce), &state.nonce)
        .expect("login nonce must be accepted");
    assert_eq!(bound, None);
}
