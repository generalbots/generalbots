//! Wire-contract tests for the Bot API request bodies (TG-5 in #2070).
//!
//! The shapes here are what `api.telegram.org` validates: a wrong `parse_mode`
//! or a present-but-empty `inline_keyboard` makes Telegram reject the whole
//! message, and the failure is invisible offline unless the body is asserted.

use super::*;
use serde_json::{json, Value};

fn to_json<T: Serialize>(payload: &T) -> Value {
    serde_json::to_value(payload).expect("payload must serialize")
}

#[test]
fn text_payload_carries_chat_id_and_omits_absent_fields() {
    let body = to_json(&text_payload("42", "olá", None));

    assert_eq!(body["chat_id"], json!("42"));
    assert_eq!(body["text"], json!("olá"));
    // `skip_serializing_if` keeps the wire body minimal: Telegram treats an
    // explicit null `parse_mode` as invalid.
    assert!(body.get("parse_mode").is_none(), "body: {body}");
    assert!(body.get("reply_markup").is_none(), "body: {body}");
}

#[test]
fn text_payload_forwards_the_parse_mode() {
    let body = to_json(&text_payload("42", "<b>hi</b>", Some("HTML")));
    assert_eq!(body["parse_mode"], json!("HTML"));
}

#[test]
fn buttons_payload_emits_one_row_per_button_with_callback_data() {
    let body = to_json(&buttons_payload(
        "42",
        "pick one",
        vec![
            ("Yes".to_string(), "cb:yes".to_string()),
            ("No".to_string(), "cb:no".to_string()),
        ],
    ));

    assert_eq!(body["chat_id"], json!("42"));
    assert_eq!(body["text"], json!("pick one"));
    // Buttons are HTML-formatted by contract; the adapter never sends plain.
    assert_eq!(body["parse_mode"], json!("HTML"));

    let keyboard = &body["reply_markup"]["inline_keyboard"];
    let rows = keyboard.as_array().expect("inline_keyboard must be an array");
    assert_eq!(rows.len(), 2, "one row per button: {body}");
    assert_eq!(rows[0][0]["text"], json!("Yes"));
    assert_eq!(rows[0][0]["callback_data"], json!("cb:yes"));
    assert_eq!(rows[1][0]["text"], json!("No"));
    assert_eq!(rows[1][0]["callback_data"], json!("cb:no"));
    // `url` is only used by the keyboard variant; inline buttons here never
    // carry both, which Telegram rejects.
    assert!(rows[0][0].get("url").is_none());
    assert!(body["reply_markup"].get("keyboard").is_none());
}

#[test]
fn buttons_payload_without_buttons_omits_the_reply_markup() {
    let body = to_json(&buttons_payload("42", "no options", Vec::new()));

    assert!(body.get("reply_markup").is_none(), "body: {body}");
    assert_eq!(body["parse_mode"], json!("HTML"));
}

#[test]
fn buttons_payload_preserves_a_200_char_callback_and_emoji_labels() {
    let callback = "c".repeat(200);
    let body = to_json(&buttons_payload(
        "42",
        "menu",
        vec![("✅ Confirmar".to_string(), callback.clone())],
    ));

    let button = &body["reply_markup"]["inline_keyboard"][0][0];
    assert_eq!(button["text"], json!("✅ Confirmar"));
    assert_eq!(button["callback_data"], json!(callback));
}

#[test]
fn photo_payload_keeps_the_caption_optional() {
    let with_caption = to_json(&photo_payload("42", "https://cdn/a.jpg", Some("foto")));
    assert_eq!(with_caption["chat_id"], json!("42"));
    assert_eq!(with_caption["photo"], json!("https://cdn/a.jpg"));
    assert_eq!(with_caption["caption"], json!("foto"));

    let without = to_json(&photo_payload("42", "https://cdn/a.jpg", None));
    assert!(without.get("caption").is_none(), "body: {without}");
}

#[test]
fn document_payload_uses_the_document_field() {
    let body = to_json(&document_payload("42", "https://cdn/a.pdf", Some("contrato")));

    assert_eq!(body["document"], json!("https://cdn/a.pdf"));
    assert_eq!(body["caption"], json!("contrato"));
    assert!(body.get("photo").is_none());
}

#[test]
fn location_payload_serializes_numeric_coordinates() {
    let body = to_json(&location_payload("42", -23.5505, -46.6333));

    assert_eq!(body["chat_id"], json!("42"));
    assert_eq!(body["latitude"], json!(-23.5505));
    assert_eq!(body["longitude"], json!(-46.6333));
}
