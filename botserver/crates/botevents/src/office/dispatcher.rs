//! Webhook and native dispatch for Office events (#2503).
//!
//! `publish` fans one event out to matching webhook subscriptions and native
//! bot subscriptions. Webhook delivery is at-least-once: every attempt is
//! logged, a 5xx is retried with capped exponential backoff, and a delivery
//! that exhausts the policy lands in the dead-letter queue. Successful
//! deliveries are recorded by idempotency key so `replay` never delivers the
//! same key twice.

use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::delivery::{
    DeadLetter, DeadLetterQueue, DeliveryAttempt, DeliveryLog, DeliveryReceipt, DeliveryRecord,
    DeliveryResult, DeliveryStatus, DeliveryTransport, IdempotencyLedger,
};
use super::event::{ContentAcl, DeliveryPayloadBuilder, DenyAllContent, DomainEvent};
use super::retry::RetryPolicy;
use super::signing::{self, Signature};
use super::subscription::{NativeSubscription, WebhookSubscription};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DispatchReport {
    pub webhooks_attempted: usize,
    pub delivered: usize,
    pub dead_lettered: usize,
    pub native_enqueued: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayError {
    NotFound,
    SubscriptionGone,
    DeliveryFailed(String),
}

impl fmt::Display for ReplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => f.write_str("no dead letter for that idempotency key"),
            Self::SubscriptionGone => f.write_str("the subscription no longer exists"),
            Self::DeliveryFailed(error) => write!(f, "replay delivery failed: {error}"),
        }
    }
}

impl std::error::Error for ReplayError {}

pub struct WebhookDispatcher {
    subscriptions: Mutex<Vec<WebhookSubscription>>,
    transport: Arc<dyn DeliveryTransport>,
    retry: RetryPolicy,
    log: DeliveryLog,
    dlq: DeadLetterQueue,
    ledger: IdempotencyLedger,
    native: NativeDispatcher,
    acl: Arc<dyn ContentAcl>,
}

impl WebhookDispatcher {
    #[must_use]
    pub fn new(transport: Arc<dyn DeliveryTransport>, acl: Arc<dyn ContentAcl>) -> Self {
        Self {
            subscriptions: Mutex::new(Vec::new()),
            transport,
            retry: RetryPolicy::default(),
            log: DeliveryLog::new(),
            dlq: DeadLetterQueue::new(),
            ledger: IdempotencyLedger::new(),
            native: NativeDispatcher::new(),
            acl,
        }
    }

    #[must_use]
    pub fn with_retry(mut self, retry: RetryPolicy) -> Self {
        self.retry = retry;
        self
    }

    pub fn add_subscription(&self, subscription: WebhookSubscription) {
        let mut subscriptions = self
            .subscriptions
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        subscriptions.push(subscription);
    }

    #[must_use]
    pub fn subscriptions(&self) -> Vec<WebhookSubscription> {
        self.subscriptions
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    #[must_use]
    pub fn log(&self) -> &DeliveryLog {
        &self.log
    }

    #[must_use]
    pub fn dlq(&self) -> &DeadLetterQueue {
        &self.dlq
    }

    #[must_use]
    pub fn native(&self) -> &NativeDispatcher {
        &self.native
    }

    /// Fans an event out to all matching subscriptions.
    #[must_use]
    pub fn publish(&self, event: &DomainEvent) -> DispatchReport {
        let mut report = DispatchReport::default();
        for subscription in self.subscriptions() {
            if !subscription.matches(event) {
                continue;
            }
            let key = event.idempotency_key(&subscription.id);
            if self.ledger.contains(&key) {
                continue;
            }
            report.webhooks_attempted += 1;

            let payload = match DeliveryPayloadBuilder::new(
                subscription.include_content,
                self.acl.as_ref(),
            )
            .with_fields(subscription.fields.clone())
            .build(event)
            {
                Ok(payload) => payload,
                Err(errors) => {
                    self.record_payload_failure(&subscription, &key, event, &errors.join("; "));
                    report.dead_lettered += 1;
                    continue;
                }
            };
            let body = match serde_json::to_vec(&payload) {
                Ok(body) => body,
                Err(error) => {
                    self.record_payload_failure(
                        &subscription,
                        &key,
                        event,
                        &format!("payload serialization failed: {error}"),
                    );
                    report.dead_lettered += 1;
                    continue;
                }
            };

            match self.attempt_delivery(&subscription, &key, &event.event_id, &body) {
                DeliveryResult::Delivered(receipt) => {
                    self.ledger.record(receipt);
                    report.delivered += 1;
                }
                DeliveryResult::Failed {
                    attempts,
                    last_error,
                } => {
                    self.dlq.enqueue(DeadLetter {
                        idempotency_key: key,
                        event_id: event.event_id.clone(),
                        subscription_id: subscription.id.clone(),
                        url: subscription.url.clone(),
                        attempts: attempts.len() as u32,
                        last_error: Some(last_error),
                        dead_lettered_at: Utc::now(),
                        body,
                        payload,
                    });
                    report.dead_lettered += 1;
                }
            }
        }
        report.native_enqueued = self.native.publish(event);
        report
    }

    /// Replays a dead lettered delivery. If the idempotency key was already
    /// delivered, the recorded receipt is returned and no request is made.
    pub fn replay(&self, idempotency_key: &str) -> Result<DeliveryReceipt, ReplayError> {
        if let Some(receipt) = self.ledger.get(idempotency_key) {
            return Ok(receipt);
        }
        let letter = self
            .dlq
            .get(idempotency_key)
            .ok_or(ReplayError::NotFound)?;
        let subscription = self
            .subscriptions()
            .into_iter()
            .find(|subscription| subscription.id == letter.subscription_id)
            .ok_or(ReplayError::SubscriptionGone)?;

        match self.attempt_delivery(
            &subscription,
            idempotency_key,
            &letter.event_id,
            &letter.body,
        ) {
            DeliveryResult::Delivered(receipt) => {
                self.ledger.record(receipt.clone());
                self.dlq.remove(idempotency_key);
                Ok(receipt)
            }
            DeliveryResult::Failed { last_error, .. } => {
                self.dlq.update_error(idempotency_key, last_error.clone());
                Err(ReplayError::DeliveryFailed(last_error))
            }
        }
    }

    fn record_payload_failure(
        &self,
        subscription: &WebhookSubscription,
        key: &str,
        event: &DomainEvent,
        error: &str,
    ) {
        self.log.append(DeliveryRecord {
            idempotency_key: key.to_string(),
            event_id: event.event_id.clone(),
            subscription_id: subscription.id.clone(),
            status: DeliveryStatus::DeadLettered,
            attempts: Vec::new(),
            recorded_at: Utc::now(),
        });
        self.dlq.enqueue(DeadLetter {
            idempotency_key: key.to_string(),
            event_id: event.event_id.clone(),
            subscription_id: subscription.id.clone(),
            url: subscription.url.clone(),
            attempts: 0,
            last_error: Some(error.to_string()),
            dead_lettered_at: Utc::now(),
            body: Vec::new(),
            payload: Value::Null,
        });
    }

    fn attempt_delivery(
        &self,
        subscription: &WebhookSubscription,
        key: &str,
        event_id: &str,
        body: &[u8],
    ) -> DeliveryResult {
        let timestamp = Utc::now().timestamp();
        let headers = match subscription.active_secret() {
            Some(secret) => signed_headers(
                &signing::sign(secret.value.as_bytes(), timestamp, body),
                key,
                event_id,
            ),
            None => Vec::new(),
        };

        let mut attempts = Vec::new();
        for attempt in 1..=self.retry.max_attempts {
            let delay = if attempt == 1 {
                0
            } else {
                self.retry.delay_for(attempt)
            };
            let outcome = self.transport.deliver(&subscription.url, &headers, body);
            attempts.push(DeliveryAttempt {
                attempt,
                status_code: outcome.status_code,
                latency_ms: outcome.latency_ms,
                error: outcome.error.clone(),
                delay_before_ms: delay,
                at: Utc::now(),
            });
            if outcome.is_success() {
                let receipt = DeliveryReceipt {
                    idempotency_key: key.to_string(),
                    event_id: event_id.to_string(),
                    subscription_id: subscription.id.clone(),
                    status_code: outcome.status_code,
                    attempt_count: attempts.len() as u32,
                    delivered_at: Utc::now(),
                };
                self.log.append(DeliveryRecord {
                    idempotency_key: key.to_string(),
                    event_id: event_id.to_string(),
                    subscription_id: subscription.id.clone(),
                    status: DeliveryStatus::Delivered,
                    attempts,
                    recorded_at: Utc::now(),
                });
                return DeliveryResult::Delivered(receipt);
            }
            if !self.retry.should_retry(attempt, outcome.status_code) {
                break;
            }
        }

        let last_error = attempts
            .last()
            .and_then(|attempt| attempt.error.clone())
            .or_else(|| {
                attempts
                    .last()
                    .map(|attempt| format!("HTTP {}", attempt.status_code))
            })
            .unwrap_or_else(|| "delivery failed".to_string());
        let failed_attempts = attempts.clone();
        self.log.append(DeliveryRecord {
            idempotency_key: key.to_string(),
            event_id: event_id.to_string(),
            subscription_id: subscription.id.clone(),
            status: DeliveryStatus::DeadLettered,
            attempts,
            recorded_at: Utc::now(),
        });
        DeliveryResult::Failed {
            attempts: failed_attempts,
            last_error,
        }
    }
}

fn signed_headers(signature: &Signature, key: &str, event_id: &str) -> Vec<(String, String)> {
    vec![
        (
            signing::SIGNATURE_HEADER.to_string(),
            signature.header_value(),
        ),
        (signing::DELIVERY_HEADER.to_string(), key.to_string()),
        (signing::EVENT_HEADER.to_string(), event_id.to_string()),
        ("content-type".to_string(), "application/json".to_string()),
    ]
}

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
        let subscriptions = self.subscribers();
        let mut queue = self.queue.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut enqueued = 0;
        for subscription in subscriptions {
            if !subscription.matches(event) {
                continue;
            }
            let payload = match DeliveryPayloadBuilder::new(false, &DenyAllContent).build(event) {
                Ok(payload) => payload,
                Err(_) => continue,
            };
            let sequence = self.counter.fetch_add(1, Ordering::SeqCst);
            queue.push(NativeEnvelope {
                id: format!("native-{sequence}"),
                bot_id: subscription.bot_id.clone(),
                subscription_id: subscription.id.clone(),
                event_id: event.event_id.clone(),
                event_type: event.event_type.as_str().to_string(),
                app: event.app.clone(),
                payload,
                enqueued_at: Utc::now(),
            });
            enqueued += 1;
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

#[cfg(test)]
#[path = "dispatcher_tests.rs"]
mod dispatcher_tests;
