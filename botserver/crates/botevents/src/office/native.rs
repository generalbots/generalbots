//! Native, in-platform event delivery (#2503).
//!
//! `NativeDispatcher` routes an event to bot automations through a process-local
//! queue — no HTTP request and no polling. `publish_and_collect` additionally
//! returns the envelopes it enqueued so a transport adapter can mirror them
//! into a browser stream without a second fan-out.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::event::{DeliveryPayloadBuilder, DenyAllContent, DomainEvent};
use super::subscription::NativeSubscription;

/// Envelope handed to an in-platform automation. No HTTP is involved.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NativeEnvelope {
    pub id: String,
    pub bot_id: String,
    pub subscription_id: String,
    pub event_id: String,
    pub event_type: String,
    pub app: String,
    pub payload: Value,
    pub enqueued_at: DateTime<Utc>,
}

/// Routes events to bot automations through an in-memory queue.
#[derive(Debug, Default)]
pub struct NativeDispatcher {
    subscriptions: Mutex<Vec<NativeSubscription>>,
    queue: Mutex<Vec<NativeEnvelope>>,
    counter: AtomicU64,
}

impl NativeDispatcher {
    #[must_use]
    pub fn new() -> Self {
        Self {
            subscriptions: Mutex::new(Vec::new()),
            queue: Mutex::new(Vec::new()),
            counter: AtomicU64::new(1),
        }
    }

    pub fn subscribe(&self, subscription: NativeSubscription) {
        let mut subscriptions = self
            .subscriptions
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        subscriptions.push(subscription);
    }

    #[must_use]
    pub fn subscribers(&self) -> Vec<NativeSubscription> {
        self.subscriptions
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    /// Enqueues one envelope per matching subscriber; returns the count.
    #[must_use]
    pub fn publish(&self, event: &DomainEvent) -> usize {
        self.publish_and_collect(event).len()
    }

    /// Enqueues one envelope per matching subscriber and returns those
    /// envelopes so a transport can mirror them into a browser stream.
    #[must_use]
    pub fn publish_and_collect(&self, event: &DomainEvent) -> Vec<NativeEnvelope> {
        let subscriptions = self.subscribers();
        let mut queue = self
            .queue
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut enqueued = Vec::new();
        for subscription in subscriptions {
            if !subscription.matches(event) {
                continue;
            }
            let payload = match DeliveryPayloadBuilder::new(false, &DenyAllContent).build(event) {
                Ok(payload) => payload,
                Err(_) => continue,
            };
            let sequence = self.counter.fetch_add(1, Ordering::SeqCst);
            let envelope = NativeEnvelope {
                id: format!("native-{sequence}"),
                bot_id: subscription.bot_id.clone(),
                subscription_id: subscription.id.clone(),
                event_id: event.event_id.clone(),
                event_type: event.event_type.as_str().to_string(),
                app: event.app.clone(),
                payload,
                enqueued_at: Utc::now(),
            };
            queue.push(envelope.clone());
            enqueued.push(envelope);
        }
        enqueued
    }

    #[must_use]
    pub fn drain(&self, bot_id: &str) -> Vec<NativeEnvelope> {
        let mut queue = self
            .queue
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut delivered = Vec::new();
        let mut index = 0;
        while index < queue.len() {
            if queue[index].bot_id == bot_id {
                delivered.push(queue.remove(index));
            } else {
                index += 1;
            }
        }
        delivered
    }

    #[must_use]
    pub fn pending(&self) -> usize {
        self.queue
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{NativeDispatcher, NativeEnvelope};
    use crate::office::event::{ContentAcl, DeliveryPayloadBuilder};
    use crate::office::taxonomy::OfficeEventType;
    use crate::office::{
        DeadLetter, DeadLetterView, DenyAllContent, DomainEvent, EventContent, NativeSubscription,
        OfficeEventStream,
    };

    fn confidential_event(event_type: OfficeEventType, event_id: &str) -> DomainEvent {
        DomainEvent::new(event_id, event_type, "ws-1", "doc-1", "usr-1").with_content(
            EventContent::new(
                "Confidential quarterly report",
                "The secret body text must not leak",
            )
            .with_excerpt("a secret excerpt"),
        )
    }

    fn typed_subscription(
        id: &str,
        bot_id: &str,
        event_types: Vec<OfficeEventType>,
    ) -> NativeSubscription {
        NativeSubscription::new(id, bot_id).with_event_types(event_types)
    }

    fn envelope(id: &str, event_id: &str) -> NativeEnvelope {
        NativeEnvelope {
            id: id.to_string(),
            bot_id: "bot-1".to_string(),
            subscription_id: "sub-native".to_string(),
            event_id: event_id.to_string(),
            event_type: "sheets.exported".to_string(),
            app: "sheets".to_string(),
            payload: json!({}),
            enqueued_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn native_envelope_carries_ids_but_no_document_body_or_content() {
        let dispatcher = NativeDispatcher::new();
        dispatcher.subscribe(typed_subscription(
            "sub-native",
            "bot-1",
            vec![OfficeEventType::DocCommentAdded],
        ));

        let enqueued = dispatcher.publish_and_collect(&confidential_event(
            OfficeEventType::DocCommentAdded,
            "evt-private",
        ));
        assert_eq!(enqueued.len(), 1);
        let envelope = &enqueued[0];

        // Public IDs and metadata remain available to the automation.
        assert_eq!(envelope.event_id, "evt-private");
        assert_eq!(envelope.event_type, "doc.comment.added");
        assert_eq!(envelope.payload["doc_id"], "doc-1");
        assert_eq!(envelope.payload["actor_id"], "usr-1");

        // The payload is metadata-only: no body/content/title/excerpt key.
        assert!(envelope.payload.get("content").is_none());
        assert!(envelope.payload.get("body").is_none());
        assert!(envelope.payload.get("title").is_none());
        assert!(envelope.payload.get("excerpt").is_none());

        // The serialization surface never leaks the document text, even though
        // the source DomainEvent carried a full EventContent.
        let text = serde_json::to_string(envelope).unwrap_or_default();
        assert!(!text.contains("secret body text"));
        assert!(!text.contains("Confidential quarterly report"));
        assert!(!text.contains("a secret excerpt"));
    }

    #[test]
    fn deny_all_content_guard_blocks_content_even_on_opt_in() {
        let event = confidential_event(OfficeEventType::DocPublished, "evt-guard");

        // The native dispatcher hardcodes include_content = false; this test
        // proves the guard itself is the second, independent barrier.
        let payload = DeliveryPayloadBuilder::new(true, &DenyAllContent)
            .build(&event)
            .unwrap_or_else(|_| json!({}));
        assert!(payload.get("content").is_none());
        assert!(!payload.to_string().contains("secret body text"));

        // DenyAllContent rejects every (actor, workspace, document) scope.
        assert!(!DenyAllContent.can_read_content("usr-1", "ws-1", "doc-1"));
        assert!(!DenyAllContent.can_read_content("admin", "other", "other"));
    }

    #[test]
    fn dead_letter_view_excludes_body_payload_and_content_fields() {
        let letter = DeadLetter {
            idempotency_key: "evt-dlq:sub-1".to_string(),
            event_id: "evt-dlq".to_string(),
            subscription_id: "sub-1".to_string(),
            url: "https://hooks.example.test/hook".to_string(),
            attempts: 2,
            last_error: Some("upstream returned 503".to_string()),
            dead_lettered_at: chrono::Utc::now(),
            // Sensitive data that exists only inside the process.
            body: b"secret body text in the signed bytes".to_vec(),
            payload: json!({
                "type": "doc.comment.added",
                "doc_id": "doc-1",
                "content": {
                    "title": "Confidential quarterly report",
                    "body": "secret body text"
                }
            }),
        };

        let view = DeadLetterView::from(&letter);
        assert_eq!(view.event_id, "evt-dlq");
        assert_eq!(view.idempotency_key, "evt-dlq:sub-1");
        assert_eq!(view.event_type, "doc.comment.added");

        // Enumerate the exact serialized field set: the admin view must not
        // declare body/payload/content at all, not merely blank them.
        let value = serde_json::to_value(&view).unwrap_or_else(|_| json!({}));
        let mut keys: Vec<&str> = value
            .as_object()
            .map(|map| map.keys().map(String::as_str).collect())
            .unwrap_or_default();
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
            ]
        );

        let text = serde_json::to_string(&view).unwrap_or_default();
        assert!(!text.contains("body"));
        assert!(!text.contains("payload"));
        assert!(!text.contains("content"));
        assert!(!text.contains("secret body text"));
        assert!(!text.contains("Confidential quarterly report"));
    }

    #[test]
    fn native_subscriptions_filter_envelopes_by_event_type() {
        let dispatcher = NativeDispatcher::new();
        dispatcher.subscribe(typed_subscription(
            "sub-export",
            "bot-export",
            vec![OfficeEventType::SheetsExported],
        ));
        dispatcher.subscribe(typed_subscription(
            "sub-comment",
            "bot-comment",
            vec![OfficeEventType::DocCommentAdded],
        ));
        let inactive = NativeSubscription {
            active: false,
            ..typed_subscription(
                "sub-inactive",
                "bot-inactive",
                vec![OfficeEventType::SheetsExported],
            )
        };
        dispatcher.subscribe(inactive);

        let exported = dispatcher.publish_and_collect(&confidential_event(
            OfficeEventType::SheetsExported,
            "evt-export",
        ));
        assert_eq!(
            exported.len(),
            1,
            "only the active export subscriber matches"
        );
        assert_eq!(exported[0].subscription_id, "sub-export");
        assert_eq!(exported[0].bot_id, "bot-export");

        let commented = dispatcher.publish_and_collect(&confidential_event(
            OfficeEventType::DocCommentAdded,
            "evt-comment",
        ));
        assert_eq!(commented.len(), 1);
        assert_eq!(commented[0].subscription_id, "sub-comment");

        let unmatched = dispatcher.publish_and_collect(&confidential_event(
            OfficeEventType::SlidesPresented,
            "evt-unmatched",
        ));
        assert!(
            unmatched.is_empty(),
            "a type with no subscriber enqueues nothing"
        );

        // An empty event-type filter means "no restriction".
        let all = NativeDispatcher::new();
        all.subscribe(NativeSubscription::new("sub-all", "bot-all"));
        assert_eq!(
            all.publish(&confidential_event(
                OfficeEventType::SlidesPresented,
                "evt-any"
            )),
            1
        );
    }

    #[test]
    fn cursor_stream_is_append_only_with_owned_snapshots() {
        let stream = OfficeEventStream::new();
        assert!(stream.is_empty());
        assert!(stream.since(0).is_empty());

        stream.push(envelope("native-1", "e1"));
        stream.push(envelope("native-2", "e2"));

        let snapshot = stream.since(0);
        assert_eq!(snapshot.len(), 2);
        assert_eq!(snapshot[0].id, "native-1");
        assert_eq!(snapshot[1].id, "native-2");

        // A later append is visible to an old cursor but not to a snapshot
        // already handed to a subscriber.
        stream.push(envelope("native-3", "e3"));
        assert_eq!(stream.len(), 3);
        assert_eq!(stream.since(0).len(), 3);
        assert_eq!(stream.since(2).len(), 1);
        assert_eq!(snapshot.len(), 2, "returned slices are owned snapshots");

        // Reading never consumes, and an overrun cursor is safe.
        assert_eq!(stream.since(0).len(), 3);
        assert!(stream.since(3).is_empty());
        assert!(stream.since(99).is_empty());
    }
}
