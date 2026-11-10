//! Offline tests for the Teams payload builders and activity parsing
//! (issue #2075, channels wave). Every helper is pure: no HTTP, no DB.

use super::*;
use crate::webhook::TeamsActivity;

fn text_block(text: &str) -> serde_json::Value {
    serde_json::json!({ "type": "TextBlock", "text": text })
}

#[test]
fn adaptive_card_puts_the_title_first_and_keeps_the_schema() {
    let card = create_adaptive_card(
        "Deploy finished",
        vec![text_block("build 42")],
        vec![serde_json::json!({ "type": "Action.Submit", "title": "Retry" })],
    );

    assert_eq!(card["type"], "AdaptiveCard");
    assert_eq!(card["version"], "1.3");
    assert_eq!(
        card["$schema"],
        "http://adaptivecards.io/schemas/adaptive-card.json"
    );
    assert_eq!(card["body"].as_array().map(Vec::len), Some(2));
    assert_eq!(card["body"][0]["text"], "Deploy finished");
    assert_eq!(card["body"][0]["weight"], "Bolder");
    assert_eq!(card["body"][0]["size"], "Medium");
    assert_eq!(card["body"][1]["text"], "build 42");
    assert_eq!(card["actions"][0]["title"], "Retry");
}

#[test]
fn adaptive_card_without_body_or_actions_still_emits_both_arrays() {
    let card = create_adaptive_card("Only title", Vec::new(), Vec::new());

    assert_eq!(card["body"].as_array().map(Vec::len), Some(1));
    assert_eq!(card["actions"].as_array().map(Vec::len), Some(0));
}

#[test]
fn thumbnail_card_only_serialises_the_fields_it_receives() {
    let minimal = create_thumbnail_card("Release", None, None, None, Vec::new());
    assert_eq!(minimal, serde_json::json!({ "title": "Release" }));

    let full = create_thumbnail_card(
        "Release",
        Some("v1.2.3"),
        Some("shipped"),
        Some("https://example.test/logo.png"),
        vec![("openUrl", "Details", "https://example.test/release")],
    );
    assert_eq!(full["subtitle"], "v1.2.3");
    assert_eq!(full["text"], "shipped");
    assert_eq!(full["images"][0]["url"], "https://example.test/logo.png");
    assert_eq!(full["buttons"][0]["type"], "openUrl");
    assert_eq!(full["buttons"][0]["title"], "Details");
    assert_eq!(full["buttons"][0]["value"], "https://example.test/release");
}

#[test]
fn thumbnail_card_omits_the_buttons_key_when_no_button_is_given() {
    let card = create_thumbnail_card("T", Some("s"), None, None, Vec::new());
    assert!(card.get("buttons").is_none());
    assert!(card.get("images").is_none());
}

#[test]
fn mentions_are_rewritten_in_place_and_emitted_as_entities() {
    let (text, entities) = create_message_with_mentions(
        "Hi @Ann, please review @Bob",
        vec![("id-ann", "Ann"), ("id-bob", "Bob")],
    );

    assert_eq!(text, "Hi <at>Ann</at>, please review <at>Bob</at>");
    assert_eq!(entities.len(), 2);
    assert_eq!(entities[0]["type"], "mention");
    assert_eq!(entities[0]["mentioned"]["id"], "id-ann");
    assert_eq!(entities[0]["mentioned"]["name"], "Ann");
    assert_eq!(entities[0]["text"], "<at>Ann</at>");
    assert_eq!(entities[1]["mentioned"]["id"], "id-bob");
}

#[test]
fn mention_of_a_longer_name_is_not_truncated_by_a_shorter_prefix() {
    // "@Ann" must not rewrite the first four characters of "@Anna": the
    // mention ends only at a non-word boundary.
    let (text, entities) = create_message_with_mentions(
        "@Anna owns this, @Ann reviews",
        vec![("id-anna", "Anna"), ("id-ann", "Ann")],
    );

    assert_eq!(
        text,
        "<at>Anna</at> owns this, <at>Ann</at> reviews",
        "mention replacement must respect word boundaries"
    );
    assert_eq!(entities.len(), 2);
    assert_eq!(entities[0]["text"], "<at>Anna</at>");
    assert_eq!(entities[1]["text"], "<at>Ann</at>");
}

#[test]
fn mention_absent_from_the_text_produces_no_entity() {
    let (text, entities) = create_message_with_mentions("no mentions here", vec![("id", "Ann")]);
    assert_eq!(text, "no mentions here");
    assert!(entities.is_empty());
}

const ACTIVITY_JSON: &str = r#"{
  "type": "message",
  "id": "1:abc",
  "serviceUrl": "https://smba.trafficmanager.net/br/",
  "channelId": "msteams",
  "from": { "id": "29:user", "name": "Alice", "aadObjectId": "aad-1" },
  "conversation": { "id": "19:conv", "conversationType": "personal", "tenantId": "tenant-1" },
  "recipient": { "id": "28:bot" },
  "text": "<at>Bot</at> hello"
}"#;

#[test]
fn activity_payload_is_parsed_with_the_bot_framework_field_names() {
    let activity: TeamsActivity = serde_json::from_str(ACTIVITY_JSON).expect("activity parses");

    assert_eq!(activity.activity_type, "message");
    assert_eq!(activity.service_url.as_deref(), Some("https://smba.trafficmanager.net/br/"));
    assert_eq!(activity.channel_id.as_deref(), Some("msteams"));
    assert_eq!(activity.from.id, "29:user");
    assert_eq!(activity.from.aad_object_id.as_deref(), Some("aad-1"));
    assert_eq!(activity.conversation.id, "19:conv");
    assert_eq!(activity.conversation.tenant_id.as_deref(), Some("tenant-1"));
    assert_eq!(activity.text.as_deref(), Some("<at>Bot</at> hello"));
    assert!(activity.value.is_none());
    assert!(activity.timestamp.is_none());
}

#[test]
fn activity_payload_tolerates_missing_optional_fields() {
    let activity: TeamsActivity = serde_json::from_str(
        r#"{"type":"conversationUpdate","id":"2","from":{"id":"u"},"conversation":{"id":"c"}}"#,
    )
    .expect("minimal activity parses");

    assert_eq!(activity.activity_type, "conversationUpdate");
    assert!(activity.from.name.is_none());
    assert!(activity.conversation.conversation_type.is_none());
    assert!(activity.recipient.is_none());
}

#[test]
fn outbound_activity_skips_none_fields_on_serialisation() {
    let activity = TeamsOutboundActivity {
        activity_type: "message".to_string(),
        text: Some("hi".to_string()),
        attachments: None,
        suggested_actions: None,
        channel_data: None,
    };
    let json = serde_json::to_value(&activity).expect("serialises");

    assert_eq!(json["type"], "message");
    assert_eq!(json["text"], "hi");
    assert!(json.get("attachments").is_none());
    assert!(json.get("suggested_actions").is_none());
    assert!(json.get("channel_data").is_none());
}
