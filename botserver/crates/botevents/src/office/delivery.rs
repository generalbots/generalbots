//! Delivery records, dead-letter queue and idempotency ledger (#2503).
//!
//! These structures are deliberately store-agnostic. The server adapter can
//! persist them to Postgres/Valkey; the tests run them in memory. Nothing here
//! performs I/O except mutex locking.

use std::collections::HashMap;
use std::sync::Mutex;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryStatus {
    Delivered,
    DeadLettered,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeliveryOutcome {
    /// 0 means a transport/network error with no HTTP response.
    pub status_code: u16,
    pub latency_ms: u64,
    pub error: Option<String>,
}

impl DeliveryOutcome {
    #[must_use]
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status_code)
    }
}

/// Transport seam. The server adapts this to an HTTP client; tests inject a
/// recording implementation.
pub trait DeliveryTransport: Send + Sync {
    fn deliver(&self, url: &str, headers: &[(String, String)], body: &[u8]) -> DeliveryOutcome;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeliveryAttempt {
    pub attempt: u32,
    pub status_code: u16,
    pub latency_ms: u64,
    pub error: Option<String>,
    pub delay_before_ms: u64,
    pub at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeliveryRecord {
    pub idempotency_key: String,
    pub event_id: String,
    pub subscription_id: String,
    pub status: DeliveryStatus,
    pub attempts: Vec<DeliveryAttempt>,
    pub recorded_at: DateTime<Utc>,
}

impl DeliveryRecord {
    #[must_use]
    pub fn last_error(&self) -> Option<&str> {
        self.attempts
            .last()
            .and_then(|attempt| attempt.error.as_deref())
    }

    #[must_use]
    pub fn total_latency_ms(&self) -> u64 {
        self.attempts.iter().map(|attempt| attempt.latency_ms).sum()
    }

    #[must_use]
    pub fn attempt_count(&self) -> usize {
        self.attempts.len()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeliveryReceipt {
    pub idempotency_key: String,
    pub event_id: String,
    pub subscription_id: String,
    pub status_code: u16,
    pub attempt_count: u32,
    pub delivered_at: DateTime<Utc>,
}

/// Result of one delivery loop, including all attempts for the audit log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeliveryResult {
    Delivered(DeliveryReceipt),
    Failed {
        attempts: Vec<DeliveryAttempt>,
        last_error: String,
    },
}

#[derive(Debug, Default)]
pub struct DeliveryLog {
    records: Mutex<Vec<DeliveryRecord>>,
}

impl DeliveryLog {
    #[must_use]
    pub fn new() -> Self {
        Self {
            records: Mutex::new(Vec::new()),
        }
    }

    pub fn append(&self, record: DeliveryRecord) {
        let mut records = self.records.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        records.push(record);
    }

    #[must_use]
    pub fn records(&self) -> Vec<DeliveryRecord> {
        self.records
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    #[must_use]
    pub fn find(&self, idempotency_key: &str) -> Option<DeliveryRecord> {
        self.records()
            .into_iter()
            .find(|record| record.idempotency_key == idempotency_key)
    }

    /// Number of consecutive dead-lettered deliveries at the tail of the log.
    #[must_use]
    pub fn consecutive_failures(&self) -> u32 {
        self.records()
            .iter()
            .rev()
            .take_while(|record| record.status == DeliveryStatus::DeadLettered)
            .count() as u32
    }

    /// Alerting hook: sustained failure across distinct deliveries.
    #[must_use]
    pub fn should_alert(&self, threshold: u32) -> bool {
        self.consecutive_failures() >= threshold
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeadLetter {
    pub idempotency_key: String,
    pub event_id: String,
    pub subscription_id: String,
    pub url: String,
    pub attempts: u32,
    pub last_error: Option<String>,
    pub dead_lettered_at: DateTime<Utc>,
    /// Exact signed bytes replayed on request.
    pub body: Vec<u8>,
    pub payload: Value,
}

#[derive(Debug, Default)]
pub struct DeadLetterQueue {
    entries: Mutex<HashMap<String, DeadLetter>>,
}

impl DeadLetterQueue {
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
        }
    }

    pub fn enqueue(&self, letter: DeadLetter) {
        let mut entries = self.entries.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        entries.insert(letter.idempotency_key.clone(), letter);
    }

    #[must_use]
    pub fn get(&self, idempotency_key: &str) -> Option<DeadLetter> {
        self.entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(idempotency_key)
            .cloned()
    }

    pub fn update_error(&self, idempotency_key: &str, error: String) {
        let mut entries = self.entries.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(letter) = entries.get_mut(idempotency_key) {
            letter.last_error = Some(error);
            letter.attempts += 1;
        }
    }

    pub fn remove(&self, idempotency_key: &str) -> Option<DeadLetter> {
        self.entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(idempotency_key)
    }

    #[must_use]
    pub fn list(&self) -> Vec<DeadLetter> {
        let mut letters: Vec<DeadLetter> = self
            .entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .values()
            .cloned()
            .collect();
        letters.sort_by(|left, right| left.idempotency_key.cmp(&right.idempotency_key));
        letters
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Records successful deliveries by idempotency key. A duplicate `record`
/// keeps the first receipt, which is what makes replay exactly once.
#[derive(Debug, Default)]
pub struct IdempotencyLedger {
    receipts: Mutex<HashMap<String, DeliveryReceipt>>,
}

impl IdempotencyLedger {
    #[must_use]
    pub fn new() -> Self {
        Self {
            receipts: Mutex::new(HashMap::new()),
        }
    }

    #[must_use]
    pub fn contains(&self, idempotency_key: &str) -> bool {
        self.receipts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .contains_key(idempotency_key)
    }

    #[must_use]
    pub fn get(&self, idempotency_key: &str) -> Option<DeliveryReceipt> {
        self.receipts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(idempotency_key)
            .cloned()
    }

    /// Returns the stored receipt: the first one wins for a repeated key.
    pub fn record(&self, receipt: DeliveryReceipt) -> DeliveryReceipt {
        let mut receipts = self.receipts.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        receipts
            .entry(receipt.idempotency_key.clone())
            .or_insert(receipt)
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use serde_json::json;

    use super::{
        DeadLetter, DeadLetterQueue, DeliveryAttempt, DeliveryLog, DeliveryOutcome, DeliveryRecord,
        DeliveryReceipt, DeliveryStatus, IdempotencyLedger,
    };

    fn record(status: DeliveryStatus) -> DeliveryRecord {
        DeliveryRecord {
            idempotency_key: "evt:sub".to_string(),
            event_id: "evt".to_string(),
            subscription_id: "sub".to_string(),
            status,
            attempts: vec![DeliveryAttempt {
                attempt: 1,
                status_code: 503,
                latency_ms: 12,
                error: Some("down".to_string()),
                delay_before_ms: 0,
                at: Utc::now(),
            }],
            recorded_at: Utc::now(),
        }
    }

    #[test]
    fn delivery_log_tracks_failures_for_alerting() {
        let log = DeliveryLog::new();
        assert!(!log.should_alert(2));
        log.append(record(DeliveryStatus::DeadLettered));
        log.append(record(DeliveryStatus::DeadLettered));
        assert_eq!(log.consecutive_failures(), 2);
        assert!(log.should_alert(2));
        log.append(record(DeliveryStatus::Delivered));
        assert_eq!(log.consecutive_failures(), 0);
        assert_eq!(log.records().len(), 3);
    }

    #[test]
    fn outcome_classifies_success() {
        assert!(DeliveryOutcome {
            status_code: 204,
            latency_ms: 1,
            error: None
        }
        .is_success());
        assert!(!DeliveryOutcome {
            status_code: 503,
            latency_ms: 1,
            error: None
        }
        .is_success());
    }

    #[test]
    fn dead_letter_queue_round_trip() {
        let queue = DeadLetterQueue::new();
        let letter = DeadLetter {
            idempotency_key: "evt:sub".to_string(),
            event_id: "evt".to_string(),
            subscription_id: "sub".to_string(),
            url: "https://x.test".to_string(),
            attempts: 5,
            last_error: Some("HTTP 503".to_string()),
            dead_lettered_at: Utc::now(),
            body: b"{}".to_vec(),
            payload: json!({}),
        };
        queue.enqueue(letter);
        assert_eq!(queue.len(), 1);
        assert!(queue.get("evt:sub").is_some());
        queue.update_error("evt:sub", "still down".to_string());
        assert_eq!(queue.get("evt:sub").map(|l| l.attempts), Some(6));
        assert!(queue.remove("evt:sub").is_some());
        assert!(queue.is_empty());
    }

    #[test]
    fn idempotency_ledger_keeps_the_first_receipt() {
        let ledger = IdempotencyLedger::new();
        let first = DeliveryReceipt {
            idempotency_key: "evt:sub".to_string(),
            event_id: "evt".to_string(),
            subscription_id: "sub".to_string(),
            status_code: 200,
            attempt_count: 1,
            delivered_at: Utc::now(),
        };
        let mut duplicate = first.clone();
        duplicate.status_code = 500;
        assert_eq!(ledger.record(first.clone()), first);
        assert_eq!(ledger.record(duplicate), first);
        assert!(ledger.contains("evt:sub"));
    }
}
