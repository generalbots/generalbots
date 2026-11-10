//! WhatsApp webhook tests: handshake, signature gate, normalization and
//! redelivery dedup (WA-5, WA-6, WA-8 in #2070).

use super::*;
use serde_json::json;

fn params(mode: Option<&str>, token: Option<&str>, challenge: Option<&str>) -> VerifyParams {
    VerifyParams {
        hub_mode: mode.map(str::to_string),
        hub_challenge: challenge.map(str::to_string),
        hub_verify_token: token.map(str::to_string),
    }
}

fn payload(value: serde_json::Value) -> WhatsAppWebhookPayload {
    serde_json::from_value(value).expect("fixture must parse")
}

fn text_message(from: &str, id: &str, body: &str) -> serde_json::Value {
    json!({
        "from": from,
        "id": id,
        "timestamp": "1700000000",
        "type": "text",
        "text": {"body": body}
    })
}

fn envelope(messages: Vec<serde_json::Value>) -> serde_json::Value {
    json!({
        "object": "whatsapp_business_account",
        "entry": [{
            "id": "waba-1",
            "changes": [{
                "field": "messages",
                "value": {
                    "messaging_product": "whatsapp",
                    "metadata": {"display_phone_number": "5511999999999",
                                 "phone_number_id": "pn-1"},
                    "messages": messages
                }
            }]
        }]
    })
}

// ------------------------------------------------------- WA-6 verification

#[test]
fn a_subscribe_handshake_echoes_the_challenge_verbatim() {
    let request = params(Some("subscribe"), Some("test-auth-token"), Some("1158201444"));

    assert_eq!(
        verify_challenge(&request, "test-auth-token"),
        Ok("1158201444".to_string())
    );
}

#[test]
fn an_empty_challenge_is_echoed_as_an_empty_string() {
    let request = params(Some("subscribe"), Some("test-auth-token"), Some(""));

    assert_eq!(verify_challenge(&request, "test-auth-token"), Ok(String::new()));
}

#[test]
fn a_wrong_verify_token_is_forbidden() {
    let request = params(Some("subscribe"), Some("wrong"), Some("42"));

    assert_eq!(
        verify_challenge(&request, "test-auth-token"),
        Err((StatusCode::FORBIDDEN, "Invalid verify token"))
    );
}

#[test]
fn a_missing_verify_token_is_forbidden() {
    let request = params(Some("subscribe"), None, Some("42"));

    assert_eq!(
        verify_challenge(&request, "test-auth-token"),
        Err((StatusCode::FORBIDDEN, "Invalid verify token"))
    );
}

#[test]
fn a_wrong_or_missing_mode_is_forbidden() {
    assert_eq!(
        verify_challenge(&params(Some("unsubscribe"), Some("t"), Some("1")), "t"),
        Err((StatusCode::FORBIDDEN, "Invalid mode"))
    );
    assert_eq!(
        verify_challenge(&params(None, Some("t"), Some("1")), "t"),
        Err((StatusCode::FORBIDDEN, "Invalid mode"))
    );
    // Mode is checked before the token: an unauthenticated caller learns
    // nothing about the token from a malformed handshake.
    assert_eq!(
        verify_challenge(&params(Some("unsubscribe"), Some("wrong"), Some("1")), "t"),
        Err((StatusCode::FORBIDDEN, "Invalid mode"))
    );
}

#[test]
fn a_missing_challenge_is_a_bad_request() {
    let request = params(Some("subscribe"), Some("test-auth-token"), None);

    assert_eq!(
        verify_challenge(&request, "test-auth-token"),
        Err((StatusCode::BAD_REQUEST, "Missing challenge"))
    );
}

// ---------------------------------------------------- WA-5 signature gate

const BODY: &str = r#"{"object":"whatsapp_business_account"}"#;
const BODY_DIGEST: &str = "sha256=b6978b21c4467654c466607663db9b43fae44b71083568df403e0a077089208e";
const SECRET: &str = "test-app-secret";

#[test]
fn an_unconfigured_secret_fails_closed() {
    // Regression (#2113): an empty secret used to accept every delivery, so
    // any caller could post a forged activity and reach the bot.
    assert_eq!(authorize_delivery("", None, BODY.as_bytes()), DeliveryAuth::Unconfigured);
    assert_eq!(authorize_delivery("   ", None, BODY.as_bytes()), DeliveryAuth::Unconfigured);
    assert_eq!(
        authorize_delivery("", Some(BODY_DIGEST), BODY.as_bytes()),
        DeliveryAuth::Unconfigured
    );
}

#[test]
fn a_configured_secret_requires_a_valid_signature() {
    assert_eq!(
        authorize_delivery(SECRET, Some(BODY_DIGEST), BODY.as_bytes()),
        DeliveryAuth::Verified
    );
    assert_eq!(authorize_delivery(SECRET, None, BODY.as_bytes()), DeliveryAuth::Mismatch);
    assert_eq!(
        authorize_delivery(SECRET, Some(""), BODY.as_bytes()),
        DeliveryAuth::Mismatch
    );
    assert_eq!(
        authorize_delivery(SECRET, Some("sha256=deadbeef"), BODY.as_bytes()),
        DeliveryAuth::Mismatch
    );
    assert_eq!(
        authorize_delivery(SECRET, Some(BODY_DIGEST), b"tampered"),
        DeliveryAuth::Mismatch
    );
}

#[test]
fn the_gate_trims_the_configured_secret() {
    assert_eq!(
        authorize_delivery(&format!(" {SECRET} "), Some(BODY_DIGEST), BODY.as_bytes()),
        DeliveryAuth::Verified
    );
}

// ------------------------------------------- WA-8 inbound normalization

#[test]
fn two_entries_with_two_changes_each_produce_one_turn_per_message() {
    let message = text_message("5511987654321", "wamid.1", "olá");
    let change = json!({
        "field": "messages",
        "value": {
            "metadata": {"phone_number_id": "pn-1"},
            "messages": [message]
        }
    });
    let delivery = payload(json!({
        "object": "whatsapp_business_account",
        "entry": [
            {"id": "waba-1", "changes": [change.clone(), change.clone()]},
            {"id": "waba-2", "changes": [change.clone(), change]}
        ]
    }));

    let turns = normalize_payload(&delivery);

    assert_eq!(turns.len(), 4);
    assert!(turns.iter().all(|turn| turn.content == "olá"));
    assert!(turns.iter().all(|turn| turn.phone_number == "5511987654321"));
    assert!(turns
        .iter()
        .all(|turn| turn.phone_number_id.as_deref() == Some("pn-1")));
}

#[test]
fn a_message_without_from_is_skipped() {
    let mut message = text_message("5511987654321", "wamid.1", "olá");
    message.as_object_mut().expect("object").remove("from");

    let turns = normalize_payload(&payload(envelope(vec![message])));

    assert!(turns.is_empty());
}

#[test]
fn a_media_only_message_yields_an_empty_turn_instead_of_being_dropped() {
    let message = json!({
        "from": "5511987654321",
        "id": "wamid.media",
        "timestamp": "1700000000",
        "type": "image",
        "image": {"id": "media-1", "mime_type": "image/jpeg"}
    });

    let turns = normalize_payload(&payload(envelope(vec![message])));

    assert_eq!(turns.len(), 1);
    // Empty, not dropped: media ingestion fills the content afterwards.
    assert_eq!(turns[0].content, "");
    assert_eq!(turns[0].message.message_type.as_deref(), Some("image"));
}

#[test]
fn every_media_kind_is_kept() {
    for kind in ["image", "video", "audio", "document", "sticker"] {
        let mut fields = serde_json::Map::new();
        fields.insert("from".to_string(), json!("5511987654321"));
        fields.insert("id".to_string(), json!(format!("wamid.{kind}")));
        fields.insert("type".to_string(), json!(kind));
        fields.insert(kind.to_string(), json!({"id": "media-1"}));
        let message = serde_json::Value::Object(fields);

        let turns = normalize_payload(&payload(envelope(vec![message])));
        assert_eq!(turns.len(), 1, "{kind} must be kept");
        assert_eq!(turns[0].content, "", "{kind} carries no text");
    }
}

#[test]
fn interactive_and_button_replies_carry_their_label() {
    let interactive = json!({
        "from": "5511987654321",
        "id": "wamid.interactive",
        "type": "interactive",
        "interactive": {
            "type": "button_reply",
            "button_reply": {"id": "btn-1", "title": "Confirmar"}
        }
    });
    let button = json!({
        "from": "5511987654321",
        "id": "wamid.button",
        "type": "button",
        "button": {"payload": "yes", "text": "Sim"}
    });

    let turns = normalize_payload(&payload(envelope(vec![interactive, button])));

    assert_eq!(turns.len(), 2);
    assert_eq!(turns[0].content, "Confirmar");
    assert_eq!(turns[1].content, "Sim");
}

#[test]
fn an_unsupported_type_is_skipped() {
    let message = json!({
        "from": "5511987654321",
        "id": "wamid.location",
        "type": "location",
        "location": {"latitude": -23.5, "longitude": -46.6}
    });

    assert!(normalize_payload(&payload(envelope(vec![message]))).is_empty());
}

#[test]
fn a_change_without_messages_yields_no_turns() {
    let delivery = payload(json!({
        "object": "whatsapp_business_account",
        "entry": [{
            "id": "waba-1",
            "changes": [{
                "field": "messages",
                "value": {
                    "metadata": {"phone_number_id": "pn-1"},
                    "statuses": [{"id": "wamid.1", "status": "delivered"}]
                }
            }]
        }]
    }));

    assert!(normalize_payload(&delivery).is_empty());
}

#[test]
fn a_turn_without_a_phone_number_id_keeps_none() {
    let delivery = payload(json!({
        "object": "whatsapp_business_account",
        "entry": [{
            "id": "waba-1",
            "changes": [{
                "field": "messages",
                "value": {"messages": [text_message("5511987654321", "wamid.1", "oi")]}
            }]
        }]
    }));

    let turns = normalize_payload(&delivery);
    assert_eq!(turns.len(), 1);
    assert!(turns[0].phone_number_id.is_none());
}

// ------------------------------------------------------- dedup state machine

#[test]
fn the_first_delivery_is_processed_and_the_redelivery_dropped() {
    let mut dedup = DedupCache::new(1000);

    assert!(dedup.accept("wamid.1"));
    assert!(!dedup.accept("wamid.1"));
    assert_eq!(dedup.len(), 1);
}

#[test]
fn distinct_ids_are_all_processed() {
    let mut dedup = DedupCache::new(1000);

    assert!(dedup.accept("wamid.1"));
    assert!(dedup.accept("wamid.2"));
    assert!(dedup.accept("wamid.3"));
    assert_eq!(dedup.len(), 3);
}

#[test]
fn the_cache_resets_at_its_limit_and_keeps_the_triggering_id() {
    let mut dedup = DedupCache::new(3);

    assert!(dedup.accept("a"));
    assert!(dedup.accept("b"));
    assert!(dedup.accept("c"));
    // The fourth id fills the cache to its limit, so the reset happens first
    // and "d" is still tracked afterwards.
    assert!(dedup.accept("d"));
    assert_eq!(dedup.len(), 1);
    assert!(!dedup.accept("d"));
    // The evicted ids are accepted again — bounded memory, best-effort dedup.
    assert!(dedup.accept("a"));
}

#[test]
fn the_default_limit_is_the_documented_thousand() {
    assert_eq!(DEDUP_LIMIT, 1000);
}
