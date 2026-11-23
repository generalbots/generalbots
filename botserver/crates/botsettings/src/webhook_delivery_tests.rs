//! Webhook guards: the SSRF allow-list, the HMAC signature, event parsing and
//! the display helpers.

use super::{
    create_signature_headers, escape_html, parse_events, sign_payload, sign_payload_at, truncate,
    validate_webhook_url, webhook_vault_path, WebhookSignatureVerifier,
    CLOCK_SKEW_TOLERANCE_SECONDS,
};
use botsecurity::webhook::SignatureValidation;
use serde_json::json;
use uuid::Uuid;

#[test]
fn the_vault_path_is_namespaced_by_id() {
    let id = Uuid::from_u128(0xabc);
    assert_eq!(
        webhook_vault_path(id),
        format!("secret/gbo/settings/webhooks/{id}")
    );
    assert!(webhook_vault_path(id).starts_with("secret/gbo/settings/webhooks/"));
}

#[test]
fn an_empty_url_is_rejected() {
    assert_eq!(validate_webhook_url(""), Err("Webhook URL is required"));
    assert_eq!(validate_webhook_url("   "), Err("Webhook URL is required"));
}

#[test]
fn a_non_http_scheme_is_rejected() {
    // In production the HTTPS rule fires first, so only the error is asserted.
    for url in ["ftp://example.com/hook", "file:///etc/passwd", "example.com/hook"] {
        let error = validate_webhook_url(url).expect_err("must reject");
        assert!(
            error == "Webhook URL cannot point to localhost or private networks"
                || error == "Webhook URL must use HTTPS in production",
            "{url} produced {error}"
        );
    }
}

#[test]
fn localhost_and_private_networks_are_rejected() {
    for url in [
        "https://localhost/hook",
        "https://127.0.0.1:8080/hook",
        "https://0.0.0.0/hook",
        "https://[::1]/hook",
        "https://169.254.169.254/latest/meta-data",
        "https://10.0.0.5/hook",
        "https://192.168.1.10/hook",
        "https://172.16.0.5/hook",
        "https://2130706433/hook",
        "https://127.0.0.2/hook",
    ] {
        assert_eq!(
            validate_webhook_url(url),
            Err("Webhook URL cannot point to localhost or private networks"),
            "{url}"
        );
    }
}

#[test]
fn a_public_https_url_is_accepted() {
    assert_eq!(validate_webhook_url("https://hooks.example.com/services/1"), Ok(()));
}

#[test]
fn events_default_to_the_wildcard() {
    assert_eq!(parse_events(None), json!(["*"]));
    assert_eq!(parse_events(Some("")), json!(["*"]));
    assert_eq!(parse_events(Some(" , , ")), json!(["*"]));
}

#[test]
fn events_are_split_trimmed_and_deduplicated_by_position() {
    assert_eq!(
        parse_events(Some("message.created, ticket.closed ,,message.created")),
        json!(["message.created", "ticket.closed", "message.created"])
    );
}

#[test]
fn the_signature_matches_the_published_hmac_vector() {
    // RFC 4231, test case 2.
    assert_eq!(
        sign_payload("what do ya want for nothing?", "Jefe"),
        "v1=5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
    );
}

#[test]
fn the_signature_is_key_and_payload_sensitive() {
    let base = sign_payload("payload", "secret");
    assert!(base.starts_with("v1="));
    assert_eq!(base.len(), 3 + 64);
    assert_ne!(base, sign_payload("payload", "other-secret"));
    assert_ne!(base, sign_payload("Payload", "secret"));
}

#[test]
fn truncate_counts_characters_not_bytes() {
    assert_eq!(truncate("short", 10), "short");
    assert_eq!(truncate("exact", 5), "exact");
    assert_eq!(truncate("longer", 3), "lon…");
    // Multi-byte characters must not be split.
    assert_eq!(truncate("ação", 2), "aç…");
    assert_eq!(truncate("", 0), "");
}

#[test]
fn escaping_covers_the_markup_breakers() {
    assert_eq!(
        escape_html("<script>alert(\"x\")</script> & more"),
        "&lt;script&gt;alert(&quot;x&quot;)&lt;/script&gt; &amp; more"
    );
    assert_eq!(escape_html("plain text"), "plain text");
    assert_eq!(escape_html(""), "");
}

#[test]
fn an_existing_entity_gains_one_more_escaping_level() {
    // `&` is escaped first, so an already-escaped entity gains one more level.
    assert_eq!(escape_html("&amp;"), "&amp;amp;");
}

// ── #2645: timestamped + nonced delivery signatures ──────────────────────

#[test]
fn the_v2_signature_covers_timestamp_nonce_and_payload() {
    let a = sign_payload_at("payload", "secret", 5000, "nonce-a");
    let b = sign_payload_at("payload", "secret", 5001, "nonce-a");
    let c = sign_payload_at("payload", "secret", 5000, "nonce-b");
    let d = sign_payload_at("payload!", "secret", 5000, "nonce-a");
    let e = sign_payload_at("payload", "other-secret", 5000, "nonce-a");

    assert_eq!(a, sign_payload_at("payload", "secret", 5000, "nonce-a"));
    assert_ne!(a, b, "the timestamp must be signed");
    assert_ne!(a, c, "the nonce must be signed");
    assert_ne!(a, d, "the payload must be signed");
    assert_ne!(a, e, "the secret must be part of the key");
    assert!(a.starts_with("v2="));
    assert_eq!(a.len(), 3 + 64);
    // A v2 signature must not be mistaken for a legacy v1 one.
    assert_ne!(a, sign_payload("payload", "secret"));
}

#[test]
fn producer_and_receiver_agree_without_clock_skew() {
    let (signature, timestamp, nonce) = create_signature_headers("{\"a\":1}", "secret");
    let mut verifier = WebhookSignatureVerifier::new();
    let result = verifier.verify("{\"a\":1}", &signature, &timestamp, &nonce, "secret");
    assert_eq!(result, SignatureValidation::Valid);
    assert_eq!(verifier.cached_nonce_count(), 1);
}

#[test]
fn a_fresh_delivery_is_valid() {
    let mut verifier = WebhookSignatureVerifier::new();
    let now = 5_000;
    let signature = sign_payload_at("body", "secret", now, "n-fresh");
    let result = verifier.verify_at("body", &signature, &now.to_string(), "n-fresh", "secret", now);
    assert_eq!(result, SignatureValidation::Valid);
}

#[test]
fn a_delivery_just_inside_the_tolerance_window_is_valid() {
    let now = 5_000;
    let signed_at = now - CLOCK_SKEW_TOLERANCE_SECONDS;
    let signature = sign_payload_at("body", "secret", signed_at, "n-boundary");
    let mut verifier = WebhookSignatureVerifier::new();
    let result = verifier.verify_at(
        "body",
        &signature,
        &signed_at.to_string(),
        "n-boundary",
        "secret",
        now,
    );
    assert_eq!(result, SignatureValidation::Valid);
}

#[test]
fn a_stale_delivery_is_refused_in_both_directions() {
    let now = 10_000;
    let mut verifier = WebhookSignatureVerifier::new();

    for signed_at in [
        now - CLOCK_SKEW_TOLERANCE_SECONDS - 1,
        now + CLOCK_SKEW_TOLERANCE_SECONDS + 1,
    ] {
        let signature = sign_payload_at("body", "secret", signed_at, "n-stale");
        let result = verifier.verify_at(
            "body",
            &signature,
            &signed_at.to_string(),
            "n-stale",
            "secret",
            now,
        );
        assert_eq!(result, SignatureValidation::Expired, "signed at {signed_at}");
    }
}

#[test]
fn a_replayed_delivery_is_refused() {
    let now = 5_000;
    let signature = sign_payload_at("body", "secret", now, "n-replay");
    let timestamp = now.to_string();
    let mut verifier = WebhookSignatureVerifier::new();

    assert_eq!(
        verifier.verify_at("body", &signature, &timestamp, "n-replay", "secret", now),
        SignatureValidation::Valid
    );
    assert_eq!(
        verifier.verify_at("body", &signature, &timestamp, "n-replay", "secret", now),
        SignatureValidation::Replayed
    );
}

#[test]
fn a_tampered_payload_or_signature_is_refused() {
    let now = 5_000;
    let signature = sign_payload_at("{\"amount\":10}", "secret", now, "n-tamper");
    let timestamp = now.to_string();
    let mut verifier = WebhookSignatureVerifier::new();

    assert_eq!(
        verifier.verify_at("{\"amount\":1000}", &signature, &timestamp, "n-tamper", "secret", now),
        SignatureValidation::Invalid
    );
    assert_eq!(
        verifier.verify_at("{\"amount\":10}", &signature, &timestamp, "n-tamper", "wrong", now),
        SignatureValidation::Invalid
    );

    let mut flipped = signature.clone().into_bytes();
    let last = flipped.len() - 1;
    flipped[last] = if flipped[last] == b'0' { b'1' } else { b'0' };
    let flipped = String::from_utf8(flipped).expect("utf8");
    assert_eq!(
        verifier.verify_at("{\"amount\":10}", &flipped, &timestamp, "n-tamper", "secret", now),
        SignatureValidation::Invalid
    );
}

#[test]
fn a_signature_reused_under_a_different_timestamp_is_invalid() {
    let now = 5_000;
    let signature = sign_payload_at("body", "secret", now, "n-shift");
    let mut verifier = WebhookSignatureVerifier::new();
    let result = verifier.verify_at(
        "body",
        &signature,
        &(now - 5).to_string(),
        "n-shift",
        "secret",
        now,
    );
    assert_eq!(result, SignatureValidation::Invalid);
}

#[test]
fn missing_and_malformed_headers_are_refused() {
    let now = 5_000;
    let signature = sign_payload_at("body", "secret", now, "n-malformed");
    let mut verifier = WebhookSignatureVerifier::new();

    assert_eq!(
        verifier.verify_at("body", "", &now.to_string(), "n-malformed", "secret", now),
        SignatureValidation::Missing
    );
    assert_eq!(
        verifier.verify_at("body", &signature, "not-a-number", "n-malformed", "secret", now),
        SignatureValidation::Invalid
    );
    assert_eq!(
        verifier.verify_at("body", &signature, &now.to_string(), "", "secret", now),
        SignatureValidation::Missing
    );
    // Legacy v1 signatures have no freshness data and are rejected by the v2 verifier.
    let legacy = sign_payload("body", "secret");
    assert_eq!(
        verifier.verify_at("body", &legacy, &now.to_string(), "n-legacy", "secret", now),
        SignatureValidation::Invalid
    );
}

#[test]
fn extreme_timestamps_do_not_overflow_the_freshness_check() {
    let now = 5_000;
    let mut verifier = WebhookSignatureVerifier::new();
    for timestamp in [i64::MIN.to_string(), i64::MAX.to_string()] {
        let signature = sign_payload_at("body", "secret", 5000, "n-extreme");
        let result = verifier.verify_at("body", &signature, &timestamp, "n-extreme", "secret", now);
        assert_eq!(result, SignatureValidation::Expired, "timestamp {timestamp}");
    }
}
