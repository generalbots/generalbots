//! HMAC-SHA256 webhook signature matrix (#2066).
//!
//! Signatures use the RFC 2104 construction over `"{timestamp}.{payload}"`.
//! The expected digests below are the RFC 4231 test vectors applied to that
//! exact message, so a consistently-broken HMAC cannot pass as a round trip.

use super::generate_webhook_secret::constant_time_compare;
use super::*;

fn manager() -> WebhookSecurityManager {
    WebhookSecurityManager::new(WebhookConfig {
        timestamp_tolerance_seconds: 60,
        ..Default::default()
    })
}

fn at(seconds: i64) -> DateTime<Utc> {
    DateTime::from_timestamp(seconds, 0).expect("timestamp")
}

#[test]
fn the_signature_matches_the_rfc_4231_vectors() {
    // RFC 4231 case 1: key = 20 bytes of 0x0b, data = "Hi There".
    let key = "\u{000b}".repeat(20);
    assert_eq!(
        manager().sign_payload("Hi There", &key, at(1_700_000_000)),
        "v1=931b0eed568d243adecea23bdfaea1fa89ca9581f28abf24320ee9d0944d98bb"
    );

    // RFC 4231 case 2: key = "Jefe", data = "what do ya want for nothing?".
    assert_eq!(
        manager().sign_payload(
            "what do ya want for nothing?",
            "Jefe",
            at(1_700_000_000)
        ),
        "v1=1cdd0650c8be1cb0974b1788d458b1e781206cfef59b85faafc582d2e182c57e"
    );
}

#[test]
fn the_signature_covers_the_timestamp_and_the_payload() {
    let m = manager();
    let a = m.sign_payload("payload", "secret", at(1_700_000_000));
    let b = m.sign_payload("payload", "secret", at(1_700_000_001));
    let c = m.sign_payload("payload!", "secret", at(1_700_000_000));
    let d = m.sign_payload("payload", "other-secret", at(1_700_000_000));

    assert_ne!(a, b, "the timestamp must be signed");
    assert_ne!(a, c, "the payload must be signed");
    assert_ne!(a, d, "the secret must be part of the key");
    assert!(a.starts_with("v1="));
    assert_eq!(a.len(), "v1=".len() + 64);
}

#[test]
fn constant_time_compare_requires_equal_bytes() {
    assert!(constant_time_compare("abc", "abc"));
    assert!(constant_time_compare("", ""));
    assert!(!constant_time_compare("abc", "abd"));
    assert!(!constant_time_compare("abc", "abcd"));
    assert!(!constant_time_compare("abc", ""));
    assert!(!constant_time_compare("v1=aa", "v1=ab"));
}

#[test]
fn the_signature_header_parser_accepts_only_one_separator() {
    assert_eq!(parse_signature_header("v1=abc"), Some(("v1", "abc")));
    assert_eq!(parse_signature_header("abc"), None);
    assert_eq!(parse_signature_header("v1=a=b"), None);
    assert_eq!(parse_signature_header(""), None);
    assert_eq!(parse_signature_header("v1="), Some(("v1", "")));
}

#[tokio::test]
async fn a_fresh_signature_is_valid_exactly_once() {
    let m = manager();
    let timestamp = Utc::now();
    let header = m.sign_payload("{\"a\":1}", "secret", timestamp);
    let ts = timestamp.timestamp().to_string();

    assert!(matches!(
        m.verify_signature("{\"a\":1}", &header, &ts, "secret").await,
        SignatureValidation::Valid
    ));
    assert!(matches!(
        m.verify_signature("{\"a\":1}", &header, &ts, "secret").await,
        SignatureValidation::Replayed
    ));
}

#[tokio::test]
async fn a_missing_signature_is_reported_as_missing() {
    let m = manager();
    assert!(matches!(
        m.verify_signature("body", "", "0", "secret").await,
        SignatureValidation::Missing
    ));
}

#[tokio::test]
async fn malformed_and_out_of_range_timestamps_are_invalid() {
    let m = manager();
    let header = m.sign_payload("body", "secret", Utc::now());
    let huge = i64::MAX.to_string();
    for ts in ["", "abc", "1.5", "99999999999999999999", huge.as_str()] {
        assert!(
            matches!(
                m.verify_signature("body", &header, ts, "secret").await,
                SignatureValidation::Invalid
            ),
            "{ts:?} must be invalid"
        );
    }
}

#[tokio::test]
async fn signatures_outside_the_tolerance_window_expire_in_both_directions() {
    let m = manager();
    let now = Utc::now().timestamp();

    for skew in [-61i64, 61, -3600, 3600] {
        let timestamp = at(now + skew);
        let header = m.sign_payload("body", "secret", timestamp);
        let ts = timestamp.timestamp().to_string();
        assert!(
            matches!(
                m.verify_signature("body", &header, &ts, "secret").await,
                SignatureValidation::Expired
            ),
            "skew {skew} must expire"
        );
    }
}

#[tokio::test]
async fn the_boundary_of_the_tolerance_window_is_accepted() {
    let m = manager();
    let now = Utc::now().timestamp();
    let timestamp = at(now - 30);
    let header = m.sign_payload("body", "secret", timestamp);
    let ts = timestamp.timestamp().to_string();
    assert!(matches!(
        m.verify_signature("body", &header, &ts, "secret").await,
        SignatureValidation::Valid
    ));
}

#[tokio::test]
async fn a_tampered_payload_or_secret_is_invalid() {
    let m = manager();
    let timestamp = Utc::now();
    let ts = timestamp.timestamp().to_string();
    let header = m.sign_payload("{\"amount\":10}", "secret", timestamp);

    for (payload, secret) in [
        ("{\"amount\":1000}", "secret"),
        ("{\"amount\":10}", "wrong-secret"),
        ("{\"amount\":10} ", "secret"),
        ("", "secret"),
    ] {
        assert!(
            matches!(
                m.verify_signature(payload, &header, &ts, secret).await,
                SignatureValidation::Invalid
            ),
            "payload {payload:?} with secret {secret:?} must be invalid"
        );
    }
}

#[tokio::test]
async fn a_single_flipped_signature_character_is_invalid() {
    let m = manager();
    let timestamp = Utc::now();
    let ts = timestamp.timestamp().to_string();
    let header = m.sign_payload("body", "secret", timestamp);

    let mut tampered = header.clone().into_bytes();
    let index = tampered.len() - 1;
    tampered[index] = if tampered[index] == b'0' { b'1' } else { b'0' };
    let tampered = String::from_utf8(tampered).expect("utf8");

    assert!(matches!(
        m.verify_signature("body", &tampered, &ts, "secret").await,
        SignatureValidation::Invalid
    ));
}

#[tokio::test]
async fn a_signature_replayed_under_a_different_timestamp_is_invalid() {
    let m = manager();
    let timestamp = Utc::now();
    let header = m.sign_payload("body", "secret", timestamp);
    let other_ts = (timestamp.timestamp() - 5).to_string();

    assert!(matches!(
        m.verify_signature("body", &header, &other_ts, "secret").await,
        SignatureValidation::Invalid
    ));
}

#[test]
fn the_created_header_carries_the_version_and_matching_timestamp() {
    let m = manager();
    let (name, signature, ts) = m.create_signature_header("payload", "secret");
    assert_eq!(name, "X-Webhook-Signature");
    assert!(signature.starts_with("v1="));

    let parsed: i64 = ts.parse().expect("timestamp");
    assert_eq!(
        signature,
        m.sign_payload("payload", "secret", at(parsed))
    );
}

#[test]
fn url_validation_blocks_internal_targets_and_requires_https() {
    let m = manager();
    for url in [
        "",
        "http://example.com/hook",
        "https://localhost/hook",
        "https://127.0.0.1/hook",
        "https://169.254.169.254/latest",
        "https://10.1.2.3/hook",
        "https://192.168.1.1/hook",
        "ftp://example.com/hook",
    ] {
        assert!(m.validate_url(url).is_err(), "{url} must be refused");
    }
    assert!(m.validate_url("https://example.com/hook").is_ok());

    let permissive = WebhookSecurityManager::new(WebhookConfig {
        require_https: false,
        ..Default::default()
    });
    assert!(permissive.validate_url("http://example.com/hook").is_ok());
}

#[test]
fn the_payload_size_boundary_is_inclusive() {
    let m = WebhookSecurityManager::new(WebhookConfig {
        max_payload_size: 16,
        ..Default::default()
    });
    assert!(m.validate_payload_size(&[0u8; 16]).is_ok());
    assert!(m.validate_payload_size(&[0u8; 17]).is_err());
    assert!(m.validate_payload_size(&[]).is_ok());
}
