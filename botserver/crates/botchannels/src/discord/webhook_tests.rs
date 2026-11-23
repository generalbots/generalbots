//! Adversarial tests for the custom Discord webhook target (#2641).
//! Offline only: these exercise the URL policy and the recorded request body,
//! never a live request to Discord.

use super::send_message::{parse_webhook_url, provider_api_error, sanitized_body_preview};
use super::{ChannelError, WebhookContent};

/// Shape of a webhook URL as issued by Discord (recorded fixture, token
/// replaced with a non-secret stand-in).
const RECORDED_WEBHOOK: &str =
    "https://discord.com/api/webhooks/123456789012345678/abcdefghijklmnopqrstuvwxyz";

#[test]
fn a_recorded_discord_webhook_is_accepted_unchanged() {
    let parsed = parse_webhook_url(RECORDED_WEBHOOK).expect("canonical webhook must be accepted");
    assert_eq!(parsed.scheme(), "https");
    assert_eq!(parsed.host_str(), Some("discord.com"));
    assert!(parsed.path().starts_with("/api/webhooks/"));
}

#[test]
fn discordapp_dot_com_is_also_accepted() {
    let url = "https://discordapp.com/api/webhooks/123456789012345678/token";
    let parsed = parse_webhook_url(url).expect("legacy Discord host must be accepted");
    assert_eq!(parsed.host_str(), Some("discordapp.com"));
}

#[test]
fn loopback_and_metadata_targets_are_refused() {
    for target in [
        "http://127.0.0.1:8080/api/webhooks/1/token",
        "http://localhost/api/webhooks/1/token",
        "https://[::1]/api/webhooks/1/token",
        "http://169.254.169.254/latest/meta-data/iam/security-credentials/",
        "http://2130706433/api/webhooks/1/token",
        "http://0x7f000001/api/webhooks/1/token",
    ] {
        assert!(
            parse_webhook_url(target).is_err(),
            "{target} must be refused"
        );
    }
}

#[test]
fn arbitrary_and_lookalike_hosts_are_refused() {
    for target in [
        "https://evil.example/api/webhooks/1/token",
        "https://discord.com.evil.example/api/webhooks/1/token",
        "https://discordapp.com.evil.example/api/webhooks/1/token",
        "https://canary.discord.com/api/webhooks/1/token",
        "https://notdiscord.com/api/webhooks/1/token",
    ] {
        assert!(
            parse_webhook_url(target).is_err(),
            "{target} must be refused"
        );
    }
}

#[test]
fn userinfo_plain_http_and_other_schemes_are_refused() {
    for target in [
        "https://user:pass@discord.com/api/webhooks/1/token",
        "http://discord.com/api/webhooks/1/token",
        "ftp://discord.com/api/webhooks/1/token",
        "file:///etc/passwd",
    ] {
        assert!(
            parse_webhook_url(target).is_err(),
            "{target} must be refused"
        );
    }
}

#[test]
fn recorded_webhook_payload_is_unchanged() {
    let content = WebhookContent {
        content: Some("hello".to_string()),
        username: None,
        avatar_url: None,
        embeds: None,
        tts: Some(false),
    };
    let json = serde_json::to_value(&content).expect("serialize webhook content");
    assert_eq!(
        json,
        serde_json::json!({ "content": "hello", "tts": false })
    );
}

#[test]
fn provider_response_bodies_are_never_returned_to_the_caller() {
    const SECRET: &str = "INTERNAL-PROVIDER-BODY";
    let body = format!("{SECRET}\nwith \u{1b}[31mcontrol codes");

    let error = provider_api_error(reqwest::StatusCode::FORBIDDEN, &body, "webhook delivery");
    let rendered = error.to_string();
    assert!(
        !rendered.contains(SECRET),
        "user-facing error leaked the body"
    );

    match error {
        ChannelError::ApiError { code, message } => {
            assert_eq!(code.as_deref(), Some("403 Forbidden"));
            assert!(!message.contains(SECRET));
        }
        other => panic!("unexpected error variant: {other:?}"),
    }

    let preview = sanitized_body_preview(&body);
    assert!(
        !preview.contains('\n'),
        "log preview kept a control character"
    );
    assert!(
        !preview.contains('\u{1b}'),
        "log preview kept an escape code"
    );
}

#[test]
fn oversized_log_previews_are_bounded() {
    let preview = sanitized_body_preview(&"x".repeat(1024));
    assert!(preview.len() <= 256 + "...[truncated]".len());
    assert!(preview.ends_with("...[truncated]"));
}
