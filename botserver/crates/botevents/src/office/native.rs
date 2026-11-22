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
        let mut queue = self.queue.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
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
