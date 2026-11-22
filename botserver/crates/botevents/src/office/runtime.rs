//! Process-wide runtime for the Office event API (#2503).
//!
//! The rest of [`crate::office`] is transport-independent and store-agnostic.
//! This module is the small adapter the HTTP/WebSocket surface mounts: it owns
//! one `WebhookDispatcher` (retries, dead letters, idempotency ledger and
//! native subscriptions), an append-only stream of native envelopes for
//! browsers, and a deterministic in-process transport. Nothing here performs
//! network I/O — the transport models a 5xx endpoint so the replay flow is
//! reproducible without an external service.
//!
//! A single runtime is shared by every request through [`runtime()`]. The
//! event/streaming handlers in `botserver` only read from it.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::{json, Value};

use super::delivery::{DeadLetter, DeliveryOutcome, DeliveryReceipt, DeliveryTransport};
use super::dispatcher::{DispatchReport, NativeEnvelope, ReplayError, WebhookDispatcher};
use super::event::{DenyAllContent, DomainEvent};
use super::retry::RetryPolicy;
use super::subscription::{NativeSubscription, WebhookSubscription};
use super::taxonomy::OfficeEventType;

/// Number of failed deliveries before the demo transport succeeds. With the
/// runtime's two-attempt retry policy the first publish dead-letters and the
/// replay (a later delivery for the same key) succeeds.
const DEMO_FAILURES_BEFORE_SUCCESS: u32 = 2;

/// Deterministic transport for the admin/event surface. Failures are counted
/// per idempotency key (the `X-GB-Delivery` header), so a publish exhausts its
/// retry budget into the DLQ and a subsequent replay of the same key succeeds
/// without any external service or mutable global toggle.
pub struct DemoTransport {
    failures_before_success: u32,
    deliveries: Mutex<HashMap<String, u32>>,
}

impl DemoTransport {
    #[must_use]
    pub fn new(failures_before_success: u32) -> Self {
        Self {
            failures_before_success,
            deliveries: Mutex::new(HashMap::new()),
        }
    }
}

impl DeliveryTransport for DemoTransport {
    fn deliver(&self, _url: &str, headers: &[(String, String)], _body: &[u8]) -> DeliveryOutcome {
        let key = headers
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(super::signing::DELIVERY_HEADER))
            .map_or_else(|| "unknown".to_string(), |(_, value)| value.clone());
        let mut deliveries = self
            .deliveries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let count = deliveries.entry(key).or_insert(0);
        *count += 1;
        if *count <= self.failures_before_success {
            DeliveryOutcome {
                status_code: 503,
                latency_ms: 1,
                error: Some("demo endpoint returned 5xx".to_string()),
            }
        } else {
            DeliveryOutcome {
                status_code: 200,
                latency_ms: 1,
                error: None,
            }
        }
    }
}

/// Append-only log of native envelopes. Browsers subscribe with a cursor
/// (`?cursor=`); the SSE/WebSocket handlers only ever read a slice, so several
/// subscribers can consume the same events independently.
#[derive(Debug, Default)]
pub struct OfficeEventStream {
    envelopes: Mutex<Vec<NativeEnvelope>>,
}

impl OfficeEventStream {
    #[must_use]
    pub fn new() -> Self {
        Self {
            envelopes: Mutex::new(Vec::new()),
        }
    }

    pub fn push(&self, envelope: NativeEnvelope) {
        self.envelopes
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(envelope);
    }

    #[must_use]
    pub fn since(&self, cursor: usize) -> Vec<NativeEnvelope> {
        let envelopes = self
            .envelopes
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if cursor >= envelopes.len() {
            Vec::new()
        } else {
            envelopes[cursor..].to_vec()
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.envelopes
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// A dead letter as exposed to the admin API. Deliberately excludes `body`
/// and `payload` from the internal `DeadLetter`: only IDs and metadata ever
/// leave the process through this surface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeadLetterView {
    pub idempotency_key: String,
    pub event_id: String,
    pub event_type: String,
    pub subscription_id: String,
    pub url: String,
    pub attempts: u32,
    pub last_error: Option<String>,
    pub dead_lettered_at: DateTime<Utc>,
}

impl From<&DeadLetter> for DeadLetterView {
    fn from(letter: &DeadLetter) -> Self {
        Self {
            idempotency_key: letter.idempotency_key.clone(),
            event_id: letter.event_id.clone(),
            event_type: letter
                .payload
                .get("type")
                .and_then(Value::as_str)
                .map_or_else(String::new, str::to_string),
            subscription_id: letter.subscription_id.clone(),
            url: letter.url.clone(),
            attempts: letter.attempts,
            last_error: letter.last_error.clone(),
            dead_lettered_at: letter.dead_lettered_at,
        }
    }
}

/// Result of one admin replay. `deduped` is true when the idempotency ledger
/// already held a receipt, i.e. no second delivery was made.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReplayResult {
    pub receipt: DeliveryReceipt,
    pub deduped: bool,
}

/// The process-wide event runtime shared by every HTTP/WS handler.
pub struct OfficeEventsRuntime {
    dispatcher: WebhookDispatcher,
    stream: OfficeEventStream,
    sequence: AtomicU64,
}

impl Default for OfficeEventsRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl OfficeEventsRuntime {
    #[must_use]
    pub fn new() -> Self {
        let transport = Arc::new(DemoTransport::new(DEMO_FAILURES_BEFORE_SUCCESS));
        let dispatcher = WebhookDispatcher::new(transport, Arc::new(DenyAllContent))
            .with_retry(RetryPolicy::new(2, 1, 1, 1));
        // The suite automation consumes this in-platform queue; no HTTP hook.
        dispatcher.native().subscribe(NativeSubscription::new(
            "sub-suite-automation",
            "default",
        ));
        let runtime = Self {
            dispatcher,
            stream: OfficeEventStream::new(),
            sequence: AtomicU64::new(1),
        };
        runtime.seed_demo_dead_letter();
        runtime
    }

    #[must_use]
    pub fn stream(&self) -> &OfficeEventStream {
        &self.stream
    }

    /// Generates a unique, server-controlled event id. Clients never supply
    /// their own id (or content) through the publish endpoint.
    #[must_use]
    pub fn next_event_id(&self) -> String {
        let sequence = self.sequence.fetch_add(1, Ordering::SeqCst);
        format!("evt-live-{}-{}", Utc::now().timestamp_millis(), sequence)
    }

    /// Publishes one event to webhook subscribers (when a URL is supplied)
    /// and to the native queue, mirroring envelopes into the browser stream.
    pub fn publish(&self, event: &DomainEvent, webhook_url: Option<&str>) -> DispatchReport {
        if let Some(url) = webhook_url {
            let id = format!("sub-{}", event.event_id);
            if !self.dispatcher.subscriptions().iter().any(|s| s.id == id) {
                // Per-publish secret, derived at runtime — no credential is
                // embedded in source or shared across deliveries.
                let secret = format!("demo-signing-{}", Utc::now().timestamp_millis());
                self.dispatcher.add_subscription(
                    WebhookSubscription::new(id, url, "runtime-key", secret)
                        .with_event_types(vec![event.event_type]),
                );
            }
        }
        let mut report = self.dispatcher.publish_webhooks(event);
        let envelopes = self.dispatcher.native().publish_and_collect(event);
        for envelope in &envelopes {
            self.stream.push(envelope.clone());
        }
        report.native_enqueued = envelopes.len();
        report
    }

    #[must_use]
    pub fn dead_letters(&self) -> Vec<DeadLetterView> {
        self.dispatcher
            .dlq()
            .list()
            .iter()
            .map(DeadLetterView::from)
            .collect()
    }

    /// Replays a dead letter. A key already recorded in the ledger returns the
    /// stored receipt with `deduped: true` and performs no delivery.
    pub fn replay(&self, idempotency_key: &str) -> Result<ReplayResult, ReplayError> {
        let deduped = self.dispatcher.ledger().contains(idempotency_key);
        let receipt = self.dispatcher.replay(idempotency_key)?;
        Ok(ReplayResult { receipt, deduped })
    }

    fn seed_demo_dead_letter(&self) {
        let event = DomainEvent::new(
            "evt-demo-001",
            OfficeEventType::SheetsExported,
            "ws_123",
            "sheet_456",
            "usr_789",
        )
        .with_tag("finance")
        .with_metadata("title", json!("Quarterly report"));
        self.publish(&event, Some("https://events.example.test/hook"));
    }
}

static RUNTIME: OnceLock<OfficeEventsRuntime> = OnceLock::new();

/// The single runtime mounted by `botserver`.
#[must_use]
pub fn runtime() -> &'static OfficeEventsRuntime {
    RUNTIME.get_or_init(OfficeEventsRuntime::new)
}

#[cfg(test)]
#[path = "runtime_tests.rs"]
mod runtime_tests;
