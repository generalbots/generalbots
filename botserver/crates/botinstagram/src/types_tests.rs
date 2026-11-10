//! Offline tests for the Instagram message builders (issue #2075).

use super::*;

#[test]
fn quick_reply_builds_one_text_button_per_reply() {
    let payload = create_quick_reply(
        "Choose a size",
        vec![("Small", "SIZE_S"), ("Large", "SIZE_L")],
    );

    assert_eq!(payload["text"], "Choose a size");
    let replies = payload["quick_replies"].as_array().expect("array");
    assert_eq!(replies.len(), 2);
    assert_eq!(replies[0]["content_type"], "text");
    assert_eq!(replies[0]["title"], "Small");
    assert_eq!(replies[0]["payload"], "SIZE_S");
    assert_eq!(replies[1]["title"], "Large");
    assert_eq!(replies[1]["payload"], "SIZE_L");
}

#[test]
fn quick_reply_without_replies_still_emits_an_empty_array() {
    let payload = create_quick_reply("hi", Vec::new());
    assert_eq!(payload["quick_replies"].as_array().map(Vec::len), Some(0));
}

#[test]
fn generic_template_wraps_the_elements_under_a_generic_payload() {
    let element = serde_json::json!({ "title": "Order #1" });
    let payload = create_generic_template(vec![element.clone()]);

    assert_eq!(payload["attachment"]["type"], "template");
    assert_eq!(payload["attachment"]["payload"]["template_type"], "generic");
    assert_eq!(
        payload["attachment"]["payload"]["elements"][0],
        element
    );
}

#[test]
fn media_template_carries_type_and_attachment_id() {
    let payload = create_media_template("image", "123456");

    assert_eq!(payload["attachment"]["payload"]["template_type"], "media");
    assert_eq!(
        payload["attachment"]["payload"]["elements"][0]["media_type"],
        "image"
    );
    assert_eq!(
        payload["attachment"]["payload"]["elements"][0]["attachment_id"],
        "123456"
    );
}

#[test]
fn webhook_payload_deserialises_messaging_and_changes_entries() {
    let raw = r#"{
      "object": "instagram",
      "entry": [{
        "id": "17841400000",
        "time": 1700000000,
        "messaging": [{
          "sender": { "id": "user-1" },
          "recipient": { "id": "page-1" },
          "timestamp": 1700000001,
          "message": { "mid": "mid.1", "text": "hello" }
        }],
        "changes": [{ "field": "comments", "value": { "text": "nice" } }]
      }]
    }"#;
    let payload: InstagramWebhookPayload = serde_json::from_str(raw).expect("payload parses");

    assert_eq!(payload.object, "instagram");
    assert_eq!(payload.entry.len(), 1);
    let entry = &payload.entry[0];
    assert_eq!(entry.time, 1_700_000_000);
    let messaging = entry.messaging.as_ref().expect("messaging");
    assert_eq!(messaging[0].sender.id, "user-1");
    assert_eq!(
        messaging[0].message.as_ref().and_then(|m| m.text.as_deref()),
        Some("hello")
    );
    let changes = entry.changes.as_ref().expect("changes");
    assert_eq!(changes[0].field, "comments");
    assert_eq!(changes[0].value["text"], "nice");
}

#[test]
fn webhook_payload_tolerates_entries_without_messaging_or_changes() {
    let payload: InstagramWebhookPayload =
        serde_json::from_str(r#"{"object":"instagram","entry":[{"id":"1","time":1}]}"#)
            .expect("minimal payload parses");

    assert!(payload.entry[0].messaging.is_none());
    assert!(payload.entry[0].changes.is_none());
}
