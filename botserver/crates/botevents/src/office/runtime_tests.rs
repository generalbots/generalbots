//! Inline tests for the process-wide Office event runtime (#2503).

use serde_json::json;

use super::{DemoTransport, OfficeEventStream, OfficeEventsRuntime};
use crate::office::delivery::DeliveryTransport;
use crate::office::event::{DomainEvent, EventContent};
use crate::office::taxonomy::OfficeEventType;
use crate::office::ReplayError;

#[test]
fn demo_dead_letter_is_seeded_and_privacy_safe() {
    let runtime = OfficeEventsRuntime::new();
    let letters = runtime.dead_letters();
    assert!(!letters.is_empty(), "a demo dead letter must be present");
    let first = &letters[0];
    assert_eq!(first.event_type, "sheets.exported");
    assert_eq!(first.event_id, "evt-demo-001");
    // The admin view has no content field at all.
    let json = serde_json::to_value(first).unwrap_or_else(|_| json!({}));
    assert!(json.get("body").is_none());
    assert!(json.get("payload").is_none());
    assert!(json.get("content").is_none());
}

#[test]
fn replay_is_exactly_once_per_idempotency_key() {
    let runtime = OfficeEventsRuntime::new();
    let key = runtime.dead_letters()[0].idempotency_key.clone();

    let first_deduped = runtime.replay(&key).map_or(true, |result| result.deduped);
    assert!(!first_deduped, "first replay must perform a delivery");
    assert!(
        runtime.dead_letters().is_empty(),
        "DLQ entry removed on success"
    );

    let second_deduped = runtime.replay(&key).map_or(false, |result| result.deduped);
    assert!(second_deduped, "ledger must dedupe a repeated key");

    let first_at = runtime
        .replay(&key)
        .map_or(None, |result| Some(result.receipt.delivered_at));
    assert!(first_at.is_some(), "replay keeps returning the stored receipt");
    assert!(matches!(runtime.replay("missing:key"), Err(ReplayError::NotFound)));
}

#[test]
fn publish_creates_a_dead_letter_then_replay_succeeds() {
    let runtime = OfficeEventsRuntime::new();
    let event = DomainEvent::new(
        "evt-live-test",
        OfficeEventType::DocCommentAdded,
        "ws-1",
        "doc-1",
        "usr-1",
    )
    .with_content(EventContent::new(
        "Secret title",
        "Secret body that must never be delivered",
    ));
    let report = runtime.publish(&event, Some("https://example.test/hook"));
    assert_eq!(report.dead_lettered, 1);
    assert!(
        runtime
            .dead_letters()
            .iter()
            .any(|letter| letter.event_id == "evt-live-test")
    );
    let key = event.idempotency_key("sub-evt-live-test");
    let deduped = runtime.replay(&key).map_or(true, |result| result.deduped);
    assert!(!deduped, "replay after the demo transport recovers succeeds");
    let json = serde_json::to_value(&runtime.dead_letters()).unwrap_or_else(|_| json!([]));
    assert!(!json.to_string().contains("Secret body"));
}

#[test]
fn native_stream_is_push_only_and_private() {
    let runtime = OfficeEventsRuntime::new();
    let cursor = runtime.stream().len();
    let event = DomainEvent::new(
        "evt-stream-test",
        OfficeEventType::SheetsExported,
        "ws-1",
        "sheet-1",
        "usr-1",
    )
    .with_content(EventContent::new(
        "Secret title",
        "Secret body that must not leak",
    ));
    let report = runtime.publish(&event, None);
    assert_eq!(report.native_enqueued, 1);
    assert_eq!(report.webhooks_attempted, 0);
    let fresh = runtime.stream().since(cursor);
    assert_eq!(fresh.len(), 1);
    assert_eq!(fresh[0].event_type, "sheets.exported");
    assert!(fresh[0].payload.get("content").is_none());
    assert!(
        !serde_json::to_string(&fresh[0])
            .unwrap_or_else(|_| String::new())
            .contains("Secret body")
    );
    assert!(runtime.stream().since(cursor + 1).is_empty());
}

#[test]
fn stream_cursors_are_independent() {
    let stream = OfficeEventStream::new();
    assert!(stream.is_empty());
    let event = DomainEvent::new("e1", OfficeEventType::DocCreated, "w", "d", "u");
    let envelope = crate::office::dispatcher::NativeEnvelope {
        id: "native-1".to_string(),
        bot_id: "default".to_string(),
        subscription_id: "sub".to_string(),
        event_id: event.event_id.clone(),
        event_type: event.event_type.as_str().to_string(),
        app: event.app.clone(),
        payload: json!({}),
        enqueued_at: chrono::Utc::now(),
    };
    stream.push(envelope.clone());
    stream.push(envelope);
    assert_eq!(stream.len(), 2);
    assert_eq!(stream.since(0).len(), 2);
    assert_eq!(stream.since(1).len(), 1);
}

#[test]
fn demo_transport_fails_twice_then_succeeds_per_key() {
    let transport = DemoTransport::new(2);
    let headers = vec![("X-GB-Delivery".to_string(), "key-1".to_string())];
    assert_eq!(transport.deliver("u", &headers, b"{}").status_code, 503);
    assert_eq!(transport.deliver("u", &headers, b"{}").status_code, 503);
    assert_eq!(transport.deliver("u", &headers, b"{}").status_code, 200);
    let other = vec![("X-GB-Delivery".to_string(), "key-2".to_string())];
    assert_eq!(transport.deliver("u", &other, b"{}").status_code, 503);
}
