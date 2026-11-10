//! Webhook guards: the SSRF allow-list, the HMAC signature, event parsing and
//! the display helpers.

use super::{escape_html, parse_events, sign_payload, truncate, validate_webhook_url, webhook_vault_path};
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
            error == "Webhook URL must start with http:// or https://"
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
