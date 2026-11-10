//! Instagram inbound authenticity (#2142).
//!
//! The POST gate and the GET handshake are the only things between an anonymous
//! request and a bot session, so the tests pin both accept/reject boundaries and
//! the fail-closed default: no app secret or no verify token accepts nothing.

use super::*;

/// Raw bytes exactly as Meta would deliver them.
const BODY: &[u8] = br#"{"object":"instagram","entry":[{"id":"1","time":1}]}"#;
const SECRET: &str = "offline-suite-instagram-app-secret";
const OTHER_SECRET: &str = "another-app-secret-that-must-not-verify";

/// `X-Hub-Signature-256: sha256=<lowercase hex>`.
fn signature(body: &[u8], secret: &str) -> String {
    match hmac_sha256_hex(secret.as_bytes(), body) {
        Some(digest) => format!("sha256={digest}"),
        None => String::new(),
    }
}

// ───────────────────────────────────────────────────────────── POST delivery

#[test]
fn a_signature_over_the_raw_body_is_accepted() {
    let signature = signature(BODY, SECRET);
    assert!(signature.starts_with("sha256="), "Meta prefixes the algorithm");
    assert_eq!(
        authorize_delivery(SECRET, Some(&signature), BODY),
        DeliveryAuth::Verified
    );
}

#[test]
fn a_signature_over_different_bytes_is_refused() {
    let signature = signature(BODY, SECRET);

    // Same JSON, different bytes: a re-serialized value must not pass.
    let reserialized = br#"{"object": "instagram", "entry": [{"id": "1", "time": 1}]}"#;
    assert_eq!(
        authorize_delivery(SECRET, Some(&signature), reserialized),
        DeliveryAuth::Mismatch,
        "the digest covers the raw bytes, not the parsed document"
    );

    let extended = [BODY, &b" "[..]].concat();
    assert_eq!(
        authorize_delivery(SECRET, Some(&signature), &extended),
        DeliveryAuth::Mismatch
    );
}

#[test]
fn a_missing_or_malformed_signature_is_refused() {
    let valid = signature(BODY, SECRET);

    for provided in [
        "",
        "sha256=",
        "deadbeef",
        "sha1=0123456789abcdef",
        "sha256=not-hex-at-all",
    ] {
        assert_eq!(
            authorize_delivery(SECRET, Some(provided), BODY),
            DeliveryAuth::Mismatch,
            "{provided:?} must not authenticate a delivery"
        );
    }

    assert_eq!(authorize_delivery(SECRET, None, BODY), DeliveryAuth::Mismatch);

    // Meta sends lowercase hex; an uppercase digest is a different string.
    let uppercase = format!("sha256={}", &valid["sha256=".len()..].to_uppercase());
    assert_eq!(
        authorize_delivery(SECRET, Some(&uppercase), BODY),
        DeliveryAuth::Mismatch
    );

    // A truncated digest has the wrong length.
    let truncated = &valid[..valid.len() - 2];
    assert_eq!(
        authorize_delivery(SECRET, Some(truncated), BODY),
        DeliveryAuth::Mismatch
    );
}

#[test]
fn a_signature_from_another_app_secret_is_refused() {
    let foreign = signature(BODY, OTHER_SECRET);
    assert_eq!(
        authorize_delivery(SECRET, Some(&foreign), BODY),
        DeliveryAuth::Mismatch,
        "only the configured app secret may sign deliveries"
    );
}

#[test]
fn an_unconfigured_app_secret_fails_closed() {
    // Even a digest computed with the empty key must not be accepted: the gate
    // refuses before comparing, so a bot without `instagram-app-secret` cannot
    // be driven by any caller.
    let signed_with_empty_key = signature(BODY, "");
    assert_eq!(
        authorize_delivery("", Some(&signed_with_empty_key), BODY),
        DeliveryAuth::Unconfigured
    );
    assert_eq!(
        authorize_delivery("   ", Some(&signature(BODY, SECRET)), BODY),
        DeliveryAuth::Unconfigured
    );
    assert_eq!(
        authorize_delivery("", None, BODY),
        DeliveryAuth::Unconfigured
    );
}

#[test]
fn the_unconfigured_error_is_logged_once() {
    // The static is process-wide: whatever call reached it first reported the
    // configuration error, and every later delivery is silent.
    warn_unconfigured_once();
    assert!(!warn_unconfigured_once(), "one line per process, not per POST");
}

// ────────────────────────────────────────────────────────── GET handshake

#[test]
fn a_matching_verify_token_echoes_the_challenge() {
    assert_eq!(
        authorize_handshake(
            "verify-me",
            Some("subscribe"),
            Some("verify-me"),
            Some("challenge-42")
        ),
        Ok("challenge-42".to_string())
    );
}

#[test]
fn a_wrong_token_or_mode_is_refused() {
    assert_eq!(
        authorize_handshake("verify-me", Some("subscribe"), Some("wrong"), Some("c")),
        Err(HandshakeError::InvalidToken)
    );
    assert_eq!(
        authorize_handshake("verify-me", Some("unsubscribe"), Some("verify-me"), Some("c")),
        Err(HandshakeError::InvalidMode)
    );
    assert_eq!(
        authorize_handshake("verify-me", Some("subscribe"), None, Some("c")),
        Err(HandshakeError::InvalidToken)
    );
    assert_eq!(
        authorize_handshake("verify-me", Some("subscribe"), Some("verify-me"), None),
        Err(HandshakeError::MissingChallenge)
    );
}

#[test]
fn an_unconfigured_verify_token_refuses_the_handshake() {
    // The old default was the literal `webhook_verify`, so anyone could finish
    // the subscription for any bot.
    for expected in ["", "   "] {
        assert_eq!(
            authorize_handshake(expected, Some("subscribe"), Some("webhook_verify"), Some("c")),
            Err(HandshakeError::Unconfigured)
        );
    }
}

#[test]
fn the_refusal_statuses_name_the_operator_action() {
    assert_eq!(HandshakeError::Unconfigured.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(HandshakeError::InvalidToken.status(), StatusCode::FORBIDDEN);
    assert_eq!(HandshakeError::InvalidMode.status(), StatusCode::FORBIDDEN);
    assert_eq!(HandshakeError::MissingChallenge.status(), StatusCode::BAD_REQUEST);
    assert_eq!(DeliveryAuth::Unconfigured.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(DeliveryAuth::Mismatch.status(), StatusCode::FORBIDDEN);
    assert_eq!(DeliveryAuth::Verified.status(), StatusCode::OK);
}

#[test]
fn the_reason_never_contains_the_token() {
    for error in [
        HandshakeError::Unconfigured,
        HandshakeError::InvalidMode,
        HandshakeError::InvalidToken,
        HandshakeError::MissingChallenge,
    ] {
        assert!(!error.reason().contains("verify-me"));
        assert!(!error.reason().is_empty());
    }
}
