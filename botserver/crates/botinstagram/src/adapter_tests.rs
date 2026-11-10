//! Offline tests for the Instagram adapter: webhook verification decisions and
//! the inbound payload normaliser (issue #2075). No Graph API call is made.

use super::*;
use crate::channel::ChannelAdapter;
use crate::state::GetConfigFn;
use std::sync::Arc;

fn config_with(entries: Vec<(&'static str, &'static str)>) -> GetConfigFn {
    Arc::new(move |_bot: &str, key: &str, default: Option<&str>| {
        entries
            .iter()
            .find(|(name, _)| *name == key)
            .map(|(_, value)| (*value).to_string())
            .or_else(|| default.map(str::to_string))
            .ok_or_else(|| format!("missing {key}"))
    })
}

fn adapter() -> InstagramAdapter {
    InstagramAdapter::with_config(
        &config_with(vec![
            ("instagram-access-token", "token"),
            ("instagram-page-id", "page-1"),
            ("instagram-account-id", "ig-1"),
            ("instagram-verify-token", "verify-me"),
        ]),
        "bot",
    )
}

fn receive(payload: serde_json::Value) -> Option<String> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime")
        .block_on(adapter().receive_message(payload))
        .expect("receive_message never fails")
}

#[test]
fn webhook_verification_returns_the_challenge_for_the_matching_token() {
    let adapter = adapter();
    assert_eq!(
        adapter.handle_webhook_verification("subscribe", "verify-me", "challenge-42"),
        Some("challenge-42".to_string())
    );
}

#[test]
fn webhook_verification_rejects_a_wrong_token_or_mode() {
    let adapter = adapter();
    assert_eq!(
        adapter.handle_webhook_verification("subscribe", "wrong", "challenge-42"),
        None
    );
    assert_eq!(
        adapter.handle_webhook_verification("unsubscribe", "verify-me", "challenge-42"),
        None
    );
}

#[test]
fn verify_token_defaults_when_the_config_has_none() {
    let adapter = InstagramAdapter::with_config(&config_with(vec![]), "bot");
    assert!(adapter.verify_webhook("webhook_verify"));
    assert!(!adapter.verify_webhook("anything-else"));
}

#[test]
fn adapter_is_configured_only_with_token_and_page_id() {
    let configured = adapter();
    assert!(configured.is_configured());
    assert_eq!(configured.channel_type(), "instagram");
    assert_eq!(configured.display_name(), "Instagram");
    assert_eq!(configured.get_instagram_account_id(), "ig-1");

    let incomplete = InstagramAdapter::with_config(
        &config_with(vec![("instagram-access-token", "token")]),
        "bot",
    );
    assert!(!incomplete.is_configured());
}

#[test]
fn text_messages_are_returned_verbatim() {
    let payload = serde_json::json!({
        "entry": [{ "messaging": [{ "message": { "text": "olá" } }] }]
    });
    assert_eq!(receive(payload).as_deref(), Some("olá"));
}

#[test]
fn attachments_are_reported_by_type() {
    let payload = serde_json::json!({
        "entry": [{ "messaging": [{ "message": {
            "attachments": [{ "type": "image", "payload": { "url": "https://x.test/a.jpg" } }]
        } }] }]
    });
    assert_eq!(receive(payload).as_deref(), Some("Received image attachment"));
}

#[test]
fn attachments_without_a_type_are_reported_as_unknown() {
    let payload = serde_json::json!({
        "entry": [{ "messaging": [{ "message": { "attachments": [{ "payload": {} }] } }] }]
    });
    assert_eq!(
        receive(payload).as_deref(),
        Some("Received unknown attachment")
    );
}

#[test]
fn postbacks_are_reported_with_their_payload() {
    let payload = serde_json::json!({
        "entry": [{ "messaging": [{ "postback": { "payload": "MENU_1" } }] }]
    });
    assert_eq!(receive(payload).as_deref(), Some("Postback: MENU_1"));
}

#[test]
fn a_postback_without_a_payload_is_ignored_instead_of_panicking() {
    let payload = serde_json::json!({
        "entry": [{ "messaging": [{ "postback": {} }] }]
    });
    assert_eq!(receive(payload), None);
}

#[test]
fn comment_changes_are_reported_as_comment_text() {
    let payload = serde_json::json!({
        "entry": [{ "changes": [{ "field": "comments", "value": { "text": "nice shot" } }] }]
    });
    assert_eq!(receive(payload).as_deref(), Some("Comment: nice shot"));
}

#[test]
fn mention_changes_are_reported_with_the_media_id() {
    let payload = serde_json::json!({
        "entry": [{ "changes": [{ "field": "mentions", "value": { "media_id": "m-9" } }] }]
    });
    assert_eq!(
        receive(payload).as_deref(),
        Some("Mentioned in media: m-9")
    );
}

#[test]
fn unknown_change_fields_and_empty_payloads_yield_none() {
    for payload in [
        serde_json::json!({ "entry": [{ "changes": [{ "field": "story_insights", "value": {} }] }] }),
        serde_json::json!({ "entry": [] }),
        serde_json::json!({}),
    ] {
        assert_eq!(receive(payload), None);
    }
}
