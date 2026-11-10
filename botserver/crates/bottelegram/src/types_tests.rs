//! Fixture tests for the Telegram Bot API payload types (TG-4 in #2070).
//!
//! The webhook deserializes straight into these structs, so a missing
//! `#[serde(default)]` turns a routine delivery (a text message with no
//! `photo`, an update with no `callback_query`) into a 400 that Telegram
//! retries forever.

use crate::types::TelegramUpdate;

const TEXT_UPDATE: &str = r#"{
  "update_id": 100000001,
  "message": {
    "message_id": 55,
    "date": 1700000000,
    "from": {"id": 42, "is_bot": false, "first_name": "Ana"},
    "chat": {"id": 42, "type": "private", "first_name": "Ana"},
    "text": "bom dia"
  }
}"#;

const PHOTO_UPDATE: &str = r#"{
  "update_id": 100000002,
  "message": {
    "message_id": 56,
    "date": 1700000001,
    "chat": {"id": -100200, "type": "group", "title": "Equipe"},
    "caption": "segue a foto",
    "photo": [
      {"file_id": "small", "file_unique_id": "u-small", "width": 90, "height": 90},
      {"file_id": "large", "file_unique_id": "u-large", "width": 1280, "height": 720,
       "file_size": 20480}
    ]
  }
}"#;

const CALLBACK_UPDATE: &str = r#"{
  "update_id": 100000003,
  "callback_query": {
    "id": "cb-1",
    "from": {"id": 42, "is_bot": false, "first_name": "Ana"},
    "data": "menu:yes",
    "message": {
      "message_id": 57,
      "date": 1700000002,
      "chat": {"id": 42, "type": "private"},
      "text": "pick one"
    }
  }
}"#;

#[test]
fn a_text_update_deserializes_with_the_chat_type_renamed() {
    let update: TelegramUpdate = serde_json::from_str(TEXT_UPDATE).expect("fixture must parse");

    assert_eq!(update.update_id, 100_000_001);
    let message = update.message.expect("message");
    assert_eq!(message.message_id, 55);
    assert_eq!(message.text.as_deref(), Some("bom dia"));
    // `type` is a Rust keyword, so it lands in `chat_type`.
    assert_eq!(message.chat.chat_type, "private");
    assert_eq!(message.chat.id, 42);
    assert!(update.callback_query.is_none());
    assert!(update.edited_message.is_none());
}

#[test]
fn a_photo_update_keeps_every_size_and_the_caption() {
    let update: TelegramUpdate = serde_json::from_str(PHOTO_UPDATE).expect("fixture must parse");
    let message = update.message.expect("message");

    let photo = message.photo.expect("photo");
    assert_eq!(photo.len(), 2);
    assert_eq!(photo[0].file_id, "small");
    assert_eq!(photo[1].file_size, Some(20_480));
    assert_eq!(message.caption.as_deref(), Some("segue a foto"));
    assert_eq!(message.chat.chat_type, "group");
    // No `from` on this delivery: the field is Option, never a parse failure.
    assert!(message.from.is_none());
    // `text` is absent and defaults to None.
    assert!(message.text.is_none());
}

#[test]
fn a_callback_update_deserializes_its_data_and_message() {
    let update: TelegramUpdate = serde_json::from_str(CALLBACK_UPDATE).expect("fixture must parse");

    assert!(update.message.is_none());
    let callback = update.callback_query.expect("callback_query");
    assert_eq!(callback.id, "cb-1");
    assert_eq!(callback.data.as_deref(), Some("menu:yes"));
    assert_eq!(callback.from.first_name, "Ana");
    let message = callback.message.expect("callback message");
    assert_eq!(message.message_id, 57);
    assert_eq!(message.chat.chat_type, "private");
}

#[test]
fn a_minimal_message_parses_with_every_optional_field_absent() {
    let json = r#"{
      "update_id": 1,
      "message": {"message_id": 2, "date": 3, "chat": {"id": 4, "type": "private"}}
    }"#;

    let update: TelegramUpdate = serde_json::from_str(json).expect("minimal message must parse");
    let message = update.message.expect("message");

    assert!(message.from.is_none());
    assert!(message.text.is_none());
    assert!(message.photo.is_none());
    assert!(message.document.is_none());
    assert!(message.voice.is_none());
    assert!(message.audio.is_none());
    assert!(message.video.is_none());
    assert!(message.location.is_none());
    assert!(message.contact.is_none());
    assert!(message.caption.is_none());
    assert!(message.chat.title.is_none());
}

#[test]
fn an_update_without_update_id_is_rejected() {
    let json = r#"{"message": {"message_id": 2, "date": 3, "chat": {"id": 4, "type": "private"}}}"#;

    assert!(serde_json::from_str::<TelegramUpdate>(json).is_err());
}

#[test]
fn a_message_without_chat_type_is_rejected() {
    // `chat.type` is not defaulted: a delivery without it cannot be routed, so
    // failing the parse is the intended behaviour.
    let json = r#"{"update_id": 1, "message": {"message_id": 2, "date": 3, "chat": {"id": 4}}}"#;

    assert!(serde_json::from_str::<TelegramUpdate>(json).is_err());
}

#[test]
fn an_empty_update_parses_with_no_payload() {
    let update: TelegramUpdate = serde_json::from_str(r#"{"update_id": 9}"#).expect("must parse");

    assert_eq!(update.update_id, 9);
    assert!(update.message.is_none());
    assert!(update.edited_message.is_none());
    assert!(update.callback_query.is_none());
}
