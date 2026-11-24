//! Inline tests for the process-wide Office event runtime (#2503).
//!
//! Every assertion here goes through the public `OfficeEventsRuntime`,
//! `DemoTransport` and `OfficeEventStream` surface. The four guarantees covered
//! are exactly-once replay, body/payload-free admin dead-letter views,
//! append-only native streams with independent cursors, and the full 5xx
//! publish -> dead letter -> replay -> dedup flow.

use serde_json::json;

use super::{DemoTransport, OfficeEventStream, OfficeEventsRuntime, ReplayResult};
use crate::office::delivery::{DeliveryReceipt, DeliveryTransport};
use crate::office::event::{DomainEvent, EventContent};
use crate::office::taxonomy::OfficeEventType;
use crate::office::{NativeEnvelope, ReplayError};

fn envelope(id: &str, event_id: &str) -> NativeEnvelope {
    NativeEnvelope {
        id: id.to_string(),
        bot_id: "default".to_string(),
        subscription_id: "sub-suite-automation".to_string(),
        event_id: event_id.to_string(),
        event_type: "sheets.exported".to_string(),
        app: "sheets".to_string(),
        payload: json!({}),
        enqueued_at: chrono::Utc::now(),
    }
}

#[test]
fn demo_transport_fails_twice_then_succeeds_per_key() {
    let transport = DemoTransport::new(2);
    let first_headers = vec![("X-GB-Delivery".to_string(), "key-1".to_string())];

    let first = transport.deliver("u", &first_headers, b"{}");
    assert_eq!(first.status_code, 503, "first call is a 5xx");
    assert_eq!(first.latency_ms, 1);
    assert!(first.error.as_deref().map_or(false, |e| e.contains("5xx")));
    let second = transport.deliver("u", &first_headers, b"{}");
    assert_eq!(second.status_code, 503, "second call is a 5xx");
    let third = transport.deliver("u", &first_headers, b"{}");
    assert_eq!(third.status_code, 200, "third call recovers");
    assert!(third.error.is_none());

    // The failure budget is counted per delivery key, not globally.
    let other = vec![("X-GB-Delivery".to_string(), "key-2".to_string())];
    assert_eq!(transport.deliver("u", &other, b"{}").status_code, 503);
    assert_eq!(transport.deliver("u", &other, b"{}").status_code, 503);
    assert_eq!(transport.deliver("u", &other, b"{}").status_code, 200);

    // A transport configured with no failures succeeds on the first attempt.
    let always_ok = DemoTransport::new(0);
    assert_eq!(
        always_ok.deliver("u", &first_headers, b"{}").status_code,
        200
    );
}

#[test]
fn demo_dead_letter_is_seeded_and_privacy_safe() {
    let runtime = OfficeEventsRuntime::new();
    let letters = runtime.dead_letters();
    assert!(!letters.is_empty(), "a demo dead letter must be present");

    let first = &letters[0];
    assert_eq!(first.event_type, "sheets.exported");
    assert_eq!(first.event_id, "evt-demo-001");
    assert_eq!(first.subscription_id, "sub-evt-demo-001");
    assert_eq!(first.attempts, 2, "the runtime retry budget was exhausted");
    assert!(first
        .last_error
        .as_deref()
        .map_or(false, |error| error.contains("5xx")));

    // Enumerate the exact field set: the admin view must not merely blank the
    // sensitive fields, it must not declare them at all.
    for letter in &letters {
        let value = serde_json::to_value(letter).unwrap_or_else(|_| json!({}));
        let mut keys: Vec<&str> = value
            .as_object()
            .map(|map| map.keys().map(String::as_str).collect())
            .unwrap_or_else(Vec::new);
        keys.sort_unstable();
        assert_eq!(
            keys,
            vec![
                "attempts",
                "dead_lettered_at",
                "event_id",
                "event_type",
                "idempotency_key",
                "last_error",
                "subscription_id",
                "url",
            ],
            "no body/payload/content field exists on a dead-letter view"
        );
        let text = serde_json::to_string(letter).unwrap_or_else(|e| {
            eprintln!("dead letter serialization failed: {e}");
            String::new()
        });
        assert!(
            !text.contains("Quarterly report"),
            "event metadata must not leak through the DLQ view"
        );
    }
}

#[test]
fn replay_is_exactly_once_per_idempotency_key() {
    let runtime = OfficeEventsRuntime::new();
    let key = runtime.dead_letters()[0].idempotency_key.clone();

    let replays: Vec<Option<ReplayResult>> = (0..4).map(|_| runtime.replay(&key).ok()).collect();
    assert!(
        replays.iter().all(Option::is_some),
        "every replay returns a stored receipt"
    );
    let deduped: Vec<bool> = replays
        .iter()
        .map(|result| result.as_ref().map_or(true, |value| value.deduped))
        .collect();
    assert_eq!(
        deduped,
        vec![false, true, true, true],
        "only the first replay performs a delivery"
    );

    let receipts: Vec<&DeliveryReceipt> = replays
        .iter()
        .filter_map(|result| result.as_ref().map(|value| &value.receipt))
        .collect();
    if let Some(first_receipt) = receipts.first() {
        assert!(
            receipts.iter().all(|receipt| receipt == first_receipt),
            "every repeated replay returns the identical stored receipt"
        );
        assert_eq!(first_receipt.status_code, 200);
        assert_eq!(first_receipt.idempotency_key, key);
    }
    assert!(
        runtime.dead_letters().is_empty(),
        "the DLQ entry is drained on the first successful replay"
    );
    assert!(matches!(
        runtime.replay("missing:key"),
        Err(ReplayError::NotFound)
    ));
}

#[test]
fn a_5xx_publish_dead_letters_then_replay_delivers_once_and_dedups() {
    let runtime = OfficeEventsRuntime::new();
    let event = DomainEvent::new(
        "evt-5xx-replay",
        OfficeEventType::DocPublished,
        "ws-1",
        "doc-1",
        "usr-1",
    )
    .with_content(EventContent::new(
        "Secret title",
        "Secret body that must never be delivered",
    ));
    let subscription_id = format!("sub-{}", event.event_id);
    let key = event.idempotency_key(&subscription_id);

    let report = runtime.publish(&event, Some("https://events.example.test/hook"));
    assert_eq!(
        report.webhooks_attempted, 1,
        "one matching webhook subscription"
    );
    assert_eq!(report.delivered, 0, "both 5xx attempts fail");
    assert_eq!(
        report.dead_lettered, 1,
        "an exhausted retry budget is dead lettered"
    );

    let letter = runtime
        .dead_letters()
        .into_iter()
        .find(|entry| entry.idempotency_key == key);
    assert!(
        letter.is_some(),
        "the failed delivery is present in the DLQ"
    );
    if let Some(letter) = letter {
        assert_eq!(letter.event_id, "evt-5xx-replay");
        assert_eq!(letter.subscription_id, subscription_id);
        assert_eq!(letter.attempts, 2, "the two 5xx attempts were recorded");
        assert!(letter
            .last_error
            .as_deref()
            .map_or(false, |error| error.contains("5xx")));
    }
    let dlq_text = serde_json::to_string(&runtime.dead_letters()).unwrap_or_else(|e| {
        eprintln!("DLQ serialization failed: {e}");
        String::new()
    });
    assert!(
        !dlq_text.contains("Secret body") && !dlq_text.contains("Secret title"),
        "the admin DLQ surface never exposes document content"
    );

    let first = runtime.replay(&key).ok();
    assert!(
        first.is_some(),
        "replay succeeds once the transport recovers"
    );
    if let Some(result) = &first {
        assert!(!result.deduped, "the first replay performs the delivery");
        assert_eq!(result.receipt.status_code, 200);
        assert_eq!(
            result.receipt.attempt_count, 1,
            "the third transport call succeeds on its first attempt"
        );
        assert_eq!(result.receipt.event_id, "evt-5xx-replay");
    }
    assert!(
        runtime
            .dead_letters()
            .iter()
            .all(|entry| entry.idempotency_key != key),
        "a successful replay drains the DLQ entry"
    );

    let second = runtime.replay(&key).ok();
    assert!(
        second.is_some(),
        "a repeated replay succeeds from the ledger"
    );
    if let Some(result) = &second {
        assert!(result.deduped, "a repeated replay is deduped");
        assert_eq!(
            first.as_ref().map(|value| &value.receipt),
            Some(&result.receipt),
            "the deduped replay returns the original receipt"
        );
    }
}

#[test]
fn native_stream_is_push_only_and_private() {
    let runtime = OfficeEventsRuntime::new();
    let cursor = runtime.stream().len();
    assert_eq!(
        cursor, 1,
        "the seeded demo event was mirrored into the stream"
    );
    let before = runtime.stream().since(0);

    // Use a type the seeded webhook subscription does not filter for, so this
    // publish exercises the native path with zero HTTP fan-out.
    let event = DomainEvent::new(
        "evt-stream-test",
        OfficeEventType::DocExported,
        "ws-1",
        "doc-1",
        "usr-1",
    )
    .with_content(EventContent::new(
        "Secret title",
        "Secret body that must not leak",
    ));
    let report = runtime.publish(&event, None);
    assert_eq!(report.native_enqueued, 1);
    assert_eq!(
        report.webhooks_attempted, 0,
        "native delivery makes no HTTP call"
    );
    assert_eq!(
        runtime.stream().len(),
        cursor + 1,
        "publish appends exactly one envelope"
    );

    let fresh = runtime.stream().since(cursor);
    assert_eq!(fresh.len(), 1);
    assert_eq!(fresh[0].event_type, "doc.exported");
    assert_eq!(fresh[0].event_id, "evt-stream-test");
    assert_eq!(
        fresh[0].payload["doc_id"], "doc-1",
        "public IDs remain available"
    );
    assert!(
        fresh[0].payload.get("content").is_none(),
        "content is excluded from the native payload"
    );
    let text = serde_json::to_string(&fresh[0]).unwrap_or_else(|e| {
        eprintln!("stream envelope serialization failed: {e}");
        String::new()
    });
    assert!(!text.contains("Secret body") && !text.contains("Secret title"));

    let after = runtime.stream().since(0);
    assert_eq!(
        &after[..cursor],
        &before[..],
        "append-only: previously enqueued envelopes never mutate"
    );
    assert!(runtime.stream().since(cursor + 1).is_empty());
    assert!(
        runtime.stream().since(cursor + 10).is_empty(),
        "an overrun cursor is safe"
    );
}

#[test]
fn stream_cursors_are_independent_and_append_only() {
    let stream = OfficeEventStream::new();
    assert!(stream.is_empty());
    assert!(stream.since(0).is_empty());
    assert!(stream.since(5).is_empty());

    stream.push(envelope("native-1", "e1"));
    stream.push(envelope("native-2", "e2"));
    assert_eq!(stream.len(), 2);

    let slow = stream.since(0);
    let fast = stream.since(1);
    assert_eq!(slow.len(), 2);
    assert_eq!(fast.len(), 1);
    assert_eq!(fast[0].event_id, "e2");

    // A later append is visible to an old cursor but never to a consumed one.
    stream.push(envelope("native-3", "e3"));
    assert_eq!(stream.len(), 3);
    assert_eq!(
        stream.since(0).len(),
        3,
        "cursors are read-only views, not consumers"
    );
    assert_eq!(stream.since(1).len(), 2);
    assert_eq!(stream.since(2).len(), 1);
    assert!(stream.since(3).is_empty());

    assert_eq!(slow.len(), 2, "a previously returned snapshot stays stable");
    assert_eq!(
        fast[0].id, "native-2",
        "a returned slice is an owned snapshot"
    );
}
