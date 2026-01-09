use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::{DateTime, Duration, Utc};
use hmac::{Hmac, Mac};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

const DEFAULT_TIMESTAMP_TOLERANCE_SECONDS: i64 = 300;
const DEFAULT_REPLAY_WINDOW_SECONDS: i64 = 600;
const SIGNATURE_HEADER: &str = "X-Webhook-Signature";
const SIGNATURE_VERSION: &str = "v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub enabled: bool,
    pub timestamp_tolerance_seconds: i64,
    pub replay_window_seconds: i64,
    pub require_https: bool,
    pub max_payload_size: usize,
    pub retry_count: u32,
    pub retry_delay_seconds: u64,
    pub timeout_seconds: u64,
}

impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            timestamp_tolerance_seconds: DEFAULT_TIMESTAMP_TOLERANCE_SECONDS,
            replay_window_seconds: DEFAULT_REPLAY_WINDOW_SECONDS,
            require_https: true,
            max_payload_size: 1024 * 1024,
            retry_count: 3,
            retry_delay_seconds: 60,
            timeout_seconds: 30,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WebhookStatus {
    Active,
    Inactive,
    Failed,
    Suspended,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEndpoint {
    pub id: Uuid,
    pub url: String,
    pub secret: String,
    pub events: Vec<String>,
    pub status: WebhookStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub failure_count: u32,
    pub allowed_ips: Option<Vec<String>>,
    pub metadata: std::collections::HashMap<String, String>,
}

impl WebhookEndpoint {
    pub fn new(url: &str, events: Vec<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            url: url.to_string(),
            secret: generate_webhook_secret(),
            events,
            status: WebhookStatus::Active,
            created_at: now,
            updated_at: now,
            last_triggered_at: None,
            failure_count: 0,
            allowed_ips: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    pub fn is_active(&self) -> bool {
        self.status == WebhookStatus::Active
    }

    pub fn subscribes_to(&self, event: &str) -> bool {
        self.events.iter().any(|e| e == event || e == "*")
    }

    pub fn record_success(&mut self) {
        self.last_triggered_at = Some(Utc::now());
        self.failure_count = 0;
        self.status = WebhookStatus::Active;
        self.updated_at = Utc::now();
    }

    pub fn record_failure(&mut self, max_failures: u32) {
        self.failure_count += 1;
        self.updated_at = Utc::now();

        if self.failure_count >= max_failures {
            self.status = WebhookStatus::Suspended;
            warn!(
                "Webhook {} suspended after {} failures",
                self.id, self.failure_count
            );
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub id: String,
    pub event: String,
    pub timestamp: DateTime<Utc>,
    pub data: serde_json::Value,
    pub webhook_id: Uuid,
}

impl WebhookPayload {
    pub fn new(event: &str, data: serde_json::Value, webhook_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            event: event.to_string(),
            timestamp: Utc::now(),
            data,
            webhook_id,
        }
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| anyhow!("Failed to serialize payload: {e}"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookDelivery {
    pub id: Uuid,
    pub webhook_id: Uuid,
    pub payload_id: String,
    pub event: String,
    pub url: String,
    pub status: DeliveryStatus,
    pub response_code: Option<u16>,
    pub response_body: Option<String>,
    pub attempt: u32,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeliveryStatus {
    Pending,
    Success,
    Failed,
    Retrying,
}

impl WebhookDelivery {
    pub fn new(webhook: &WebhookEndpoint, payload: &WebhookPayload) -> Self {
        Self {
            id: Uuid::new_v4(),
            webhook_id: webhook.id,
            payload_id: payload.id.clone(),
            event: payload.event.clone(),
            url: webhook.url.clone(),
            status: DeliveryStatus::Pending,
            response_code: None,
            response_body: None,
            attempt: 0,
            created_at: Utc::now(),
            completed_at: None,
            next_retry_at: None,
            error: None,
        }
    }

    pub fn mark_success(&mut self, response_code: u16, response_body: Option<String>) {
        self.status = DeliveryStatus::Success;
        self.response_code = Some(response_code);
        self.response_body = response_body;
        self.completed_at = Some(Utc::now());
        self.attempt += 1;
    }

    pub fn mark_failed(&mut self, error: &str, should_retry: bool, retry_delay: Duration) {
        self.attempt += 1;
        self.error = Some(error.to_string());

        if should_retry {
            self.status = DeliveryStatus::Retrying;
            self.next_retry_at = Some(Utc::now() + retry_delay);
        } else {
            self.status = DeliveryStatus::Failed;
            self.completed_at = Some(Utc::now());
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignatureValidation {
    Valid,
    Missing,
    Invalid,
    Expired,
    Replayed,
}

impl SignatureValidation {
    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Valid)
    }

    pub fn error_message(&self) -> &'static str {
        match self {
            Self::Valid => "Valid",
            Self::Missing => "Signature header missing",
            Self::Invalid => "Invalid signature",
            Self::Expired => "Timestamp expired",
            Self::Replayed => "Duplicate request detected",
        }
    }
}

pub struct WebhookSecurityManager {
    config: WebhookConfig,
    used_signatures: Arc<RwLock<HashSet<String>>>,
    signature_timestamps: Arc<RwLock<std::collections::HashMap<String, DateTime<Utc>>>>,
}

impl WebhookSecurityManager {
    pub fn new(config: WebhookConfig) -> Self {
        Self {
            config,
            used_signatures: Arc::new(RwLock::new(HashSet::new())),
            signature_timestamps: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(WebhookConfig::default())
    }

    pub fn sign_payload(&self, payload: &str, secret: &str, timestamp: DateTime<Utc>) -> String {
        let timestamp_str = timestamp.timestamp().to_string();
        let signed_payload = format!("{}.{}", timestamp_str, payload);

        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(signed_payload.as_bytes());
        let result = mac.finalize();

        let signature = hex::encode(result.into_bytes());
        format!("{}={}", SIGNATURE_VERSION, signature)
    }

    pub fn create_signature_header(
        &self,
        payload: &str,
        secret: &str,
    ) -> (String, String, String) {
        let timestamp = Utc::now();
        let timestamp_str = timestamp.timestamp().to_string();
        let signature = self.sign_payload(payload, secret, timestamp);

        (
            SIGNATURE_HEADER.to_string(),
            signature,
            timestamp_str,
        )
    }

    pub async fn verify_signature(
        &self,
        payload: &str,
        signature_header: &str,
        timestamp_header: &str,
        secret: &str,
    ) -> SignatureValidation {
        if signature_header.is_empty() {
            return SignatureValidation::Missing;
        }

        let timestamp: i64 = match timestamp_header.parse() {
            Ok(t) => t,
            Err(_) => return SignatureValidation::Invalid,
        };

        let request_time = match DateTime::from_timestamp(timestamp, 0) {
            Some(t) => t,
            None => return SignatureValidation::Invalid,
        };

        let now = Utc::now();
        let tolerance = Duration::seconds(self.config.timestamp_tolerance_seconds);

        if now - request_time > tolerance || request_time - now > tolerance {
            return SignatureValidation::Expired;
        }

        let expected_signature = self.sign_payload(payload, secret, request_time);

        if !constant_time_compare(signature_header, &expected_signature) {
            return SignatureValidation::Invalid;
        }

        let signature_key = format!("{}:{}", signature_header, timestamp_header);

        {
            let signatures = self.used_signatures.read().await;
            if signatures.contains(&signature_key) {
                return SignatureValidation::Replayed;
            }
        }

        {
            let mut signatures = self.used_signatures.write().await;
            signatures.insert(signature_key.clone());
        }

        {
            let mut timestamps = self.signature_timestamps.write().await;
            timestamps.insert(signature_key, Utc::now());
        }

        SignatureValidation::Valid
    }

    pub async fn cleanup_old_signatures(&self) -> usize {
        let cutoff = Utc::now() - Duration::seconds(self.config.replay_window_seconds);

        let expired_keys: Vec<String> = {
            let timestamps = self.signature_timestamps.read().await;
            timestamps
                .iter()
                .filter(|(_, &time)| time < cutoff)
                .map(|(key, _)| key.clone())
                .collect()
        };

        let count = expired_keys.len();

        if count > 0 {
            let mut signatures = self.used_signatures.write().await;
            let mut timestamps = self.signature_timestamps.write().await;

            for key in expired_keys {
                signatures.remove(&key);
                timestamps.remove(&key);
            }

            info!("Cleaned up {} expired webhook signatures", count);
        }

        count
    }

    pub fn validate_url(&self, url: &str) -> Result<()> {
        if url.is_empty() {
            return Err(anyhow!("Webhook URL cannot be empty"));
        }

        if self.config.require_https && !url.starts_with("https://") {
            return Err(anyhow!("Webhook URL must use HTTPS"));
        }

        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(anyhow!("Invalid webhook URL scheme"));
        }

        let forbidden_hosts = ["localhost", "127.0.0.1", "0.0.0.0", "::1", "[::1]"];
        for host in forbidden_hosts {
            if url.contains(host) {
                return Err(anyhow!("Webhook URL cannot point to localhost"));
            }
        }

        if url.contains("169.254.") || url.contains("10.") || url.contains("192.168.") {
            return Err(anyhow!("Webhook URL cannot point to private IP addresses"));
        }

        Ok(())
    }

    pub fn validate_payload_size(&self, payload: &[u8]) -> Result<()> {
        if payload.len() > self.config.max_payload_size {
            return Err(anyhow!(
                "Payload size {} exceeds maximum {}",
                payload.len(),
                self.config.max_payload_size
            ));
        }
        Ok(())
    }

    pub fn config(&self) -> &WebhookConfig {
        &self.config
    }
}

pub struct WebhookManager {
    config: WebhookConfig,
    security: WebhookSecurityManager,
    endpoints: Arc<RwLock<std::collections::HashMap<Uuid, WebhookEndpoint>>>,
    deliveries: Arc<RwLock<Vec<WebhookDelivery>>>,
}

impl WebhookManager {
    pub fn new(config: WebhookConfig) -> Self {
        let security = WebhookSecurityManager::new(config.clone());
        Self {
            config,
            security,
            endpoints: Arc::new(RwLock::new(std::collections::HashMap::new())),
            deliveries: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(WebhookConfig::default())
    }

    pub async fn register_endpoint(&self, url: &str, events: Vec<String>) -> Result<WebhookEndpoint> {
        self.security.validate_url(url)?;

        let endpoint = WebhookEndpoint::new(url, events);
        let id = endpoint.id;

        let mut endpoints = self.endpoints.write().await;
        endpoints.insert(id, endpoint.clone());

        info!("Registered webhook endpoint {} for URL {}", id, url);
        Ok(endpoint)
    }

    pub async fn get_endpoint(&self, id: Uuid) -> Option<WebhookEndpoint> {
        let endpoints = self.endpoints.read().await;
        endpoints.get(&id).cloned()
    }

    pub async fn update_endpoint(&self, id: Uuid, updates: WebhookEndpointUpdate) -> Result<WebhookEndpoint> {
        let mut endpoints = self.endpoints.write().await;
        let endpoint = endpoints
            .get_mut(&id)
            .ok_or_else(|| anyhow!("Webhook endpoint not found"))?;

        if let Some(url) = updates.url {
            self.security.validate_url(&url)?;
            endpoint.url = url;
        }

        if let Some(events) = updates.events {
            endpoint.events = events;
        }

        if let Some(status) = updates.status {
            endpoint.status = status;
        }

        endpoint.updated_at = Utc::now();

        Ok(endpoint.clone())
    }

    pub async fn delete_endpoint(&self, id: Uuid) -> bool {
        let mut endpoints = self.endpoints.write().await;
        endpoints.remove(&id).is_some()
    }

    pub async fn get_endpoints_for_event(&self, event: &str) -> Vec<WebhookEndpoint> {
        let endpoints = self.endpoints.read().await;
        endpoints
            .values()
            .filter(|e| e.is_active() && e.subscribes_to(event))
            .cloned()
            .collect()
    }

    pub async fn create_delivery(
        &self,
        endpoint: &WebhookEndpoint,
        event: &str,
        data: serde_json::Value,
    ) -> Result<(WebhookDelivery, String, String, String)> {
        let payload = WebhookPayload::new(event, data, endpoint.id);
        let payload_json = payload.to_json()?;

        self.security.validate_payload_size(payload_json.as_bytes())?;

        let (header_name, signature, timestamp) =
            self.security.create_signature_header(&payload_json, &endpoint.secret);

        let delivery = WebhookDelivery::new(endpoint, &payload);

        let mut deliveries = self.deliveries.write().await;
        deliveries.push(delivery.clone());

        Ok((delivery, payload_json, format!("{header_name}: {signature}"), timestamp))
    }

    pub async fn record_delivery_result(
        &self,
        delivery_id: Uuid,
        success: bool,
        response_code: Option<u16>,
        response_body: Option<String>,
        error: Option<&str>,
    ) -> Result<()> {
        let webhook_id = {
            let mut deliveries = self.deliveries.write().await;
            let delivery = deliveries
                .iter_mut()
                .find(|d| d.id == delivery_id)
                .ok_or_else(|| anyhow!("Delivery not found"))?;

            if success {
                delivery.mark_success(response_code.unwrap_or(200), response_body);
            } else {
                let should_retry = delivery.attempt < self.config.retry_count;
                let retry_delay = Duration::seconds(
                    self.config.retry_delay_seconds as i64 * 2i64.pow(delivery.attempt),
                );
                delivery.mark_failed(error.unwrap_or("Unknown error"), should_retry, retry_delay);
            }

            delivery.webhook_id
        };

        let mut endpoints = self.endpoints.write().await;
        if let Some(endpoint) = endpoints.get_mut(&webhook_id) {
            if success {
                endpoint.record_success();
            } else {
                endpoint.record_failure(self.config.retry_count);
            }
        }

        Ok(())
    }

    pub async fn get_pending_retries(&self) -> Vec<WebhookDelivery> {
        let now = Utc::now();
        let deliveries = self.deliveries.read().await;

        deliveries
            .iter()
            .filter(|d| {
                d.status == DeliveryStatus::Retrying
                    && d.next_retry_at.map(|t| t <= now).unwrap_or(false)
            })
            .cloned()
            .collect()
    }

    pub async fn get_delivery_history(&self, webhook_id: Uuid, limit: usize) -> Vec<WebhookDelivery> {
        let deliveries = self.deliveries.read().await;

        let mut history: Vec<WebhookDelivery> = deliveries
            .iter()
            .filter(|d| d.webhook_id == webhook_id)
            .cloned()
            .collect();

        history.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        history.truncate(limit);
        history
    }

    pub fn security(&self) -> &WebhookSecurityManager {
        &self.security
    }

    pub fn config(&self) -> &WebhookConfig {
        &self.config
    }
}

#[derive(Debug, Clone, Default)]
pub struct WebhookEndpointUpdate {
    pub url: Option<String>,
    pub events: Option<Vec<String>>,
    pub status: Option<WebhookStatus>,
}

fn generate_webhook_secret() -> String {
    let mut rng = rand::rng();
    let bytes: Vec<u8> = (0..32).map(|_| rng.random()).collect();
    format!("whsec_{}", BASE64.encode(&bytes))
}

fn constant_time_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        result |= x ^ y;
    }
    result == 0
}

pub fn parse_signature_header(header: &str) -> Option<(&str, &str)> {
    let parts: Vec<&str> = header.split('=').collect();
    if parts.len() == 2 {
        Some((parts[0], parts[1]))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_endpoint_creation() {
        let endpoint = WebhookEndpoint::new(
            "https://example.com/webhook",
            vec!["user.created".into(), "user.updated".into()],
        );

        assert!(endpoint.is_active());
        assert!(endpoint.secret.starts_with("whsec_"));
        assert!(endpoint.subscribes_to("user.created"));
        assert!(!endpoint.subscribes_to("user.deleted"));
    }

    #[test]
    fn test_webhook_wildcard_subscription() {
        let endpoint = WebhookEndpoint::new("https://example.com/webhook", vec!["*".into()]);

        assert!(endpoint.subscribes_to("user.created"));
        assert!(endpoint.subscribes_to("order.completed"));
        assert!(endpoint.subscribes_to("any.event"));
    }

    #[test]
    fn test_signature_generation() {
        let manager = WebhookSecurityManager::with_defaults();
        let payload = r#"{"event":"test"}"#;
        let secret = "test_secret";
        let timestamp = Utc::now();

        let signature = manager.sign_payload(payload, secret, timestamp);

        assert!(signature.starts_with("v1="));
        assert_eq!(signature.len(), 3 + 64);
    }

    #[tokio::test]
    async fn test_signature_verification() {
        let manager = WebhookSecurityManager::with_defaults();
        let payload = r#"{"event":"test"}"#;
        let secret = "test_secret";
        let timestamp = Utc::now();

        let signature = manager.sign_payload(payload, secret, timestamp);
        let timestamp_str = timestamp.timestamp().to_string();

        let result = manager
            .verify_signature(payload, &signature, &timestamp_str, secret)
            .await;

        assert!(result.is_valid());
    }

    #[tokio::test]
    async fn test_replay_protection() {
        let manager = WebhookSecurityManager::with_defaults();
        let payload = r#"{"event":"test"}"#;
        let secret = "test_secret";
        let timestamp = Utc::now();

        let signature = manager.sign_payload(payload, secret, timestamp);
        let timestamp_str = timestamp.timestamp().to_string();

        let result1 = manager
            .verify_signature(payload, &signature, &timestamp_str, secret)
            .await;
        assert!(result1.is_valid());

        let result2 = manager
            .verify_signature(payload, &signature, &timestamp_str, secret)
            .await;
        assert_eq!(result2, SignatureValidation::Replayed);
    }

    #[tokio::test]
    async fn test_expired_timestamp() {
        let mut config = WebhookConfig::default();
        config.timestamp_tolerance_seconds = 60;
        let manager = WebhookSecurityManager::new(config);

        let payload = r#"{"event":"test"}"#;
        let secret = "test_secret";
        let old_timestamp = Utc::now() - Duration::seconds(120);

        let signature = manager.sign_payload(payload, secret, old_timestamp);
        let timestamp_str = old_timestamp.timestamp().to_string();

        let result = manager
            .verify_signature(payload, &signature, &timestamp_str, secret)
            .await;

        assert_eq!(result, SignatureValidation::Expired);
    }

    #[test]
    fn test_url_validation() {
        let manager = WebhookSecurityManager::with_defaults();

        assert!(manager.validate_url("https://example.com/webhook").is_ok());
        assert!(manager.validate_url("http://example.com/webhook").is_err());
        assert!(manager.validate_url("https://localhost/webhook").is_err());
        assert!(manager.validate_url("https://127.0.0.1/webhook").is_err());
        assert!(manager.validate_url("").is_err());
    }

    #[test]
    fn test_url_validation_no_https_required() {
        let mut config = WebhookConfig::default();
        config.require_https = false;
        let manager = WebhookSecurityManager::new(config);

        assert!(manager.validate_url("http://example.com/webhook").is_ok());
        assert!(manager.validate_url("https://example.com/webhook").is_ok());
    }

    #[test]
    fn test_payload_size_validation() {
        let mut config = WebhookConfig::default();
        config.max_payload_size = 100;
        let manager = WebhookSecurityManager::new(config);

        let small_payload = vec![0u8; 50];
        assert!(manager.validate_payload_size(&small_payload).is_ok());

        let large_payload = vec![0u8; 200];
        assert!(manager.validate_payload_size(&large_payload).is_err());
    }

    #[test]
    fn test_webhook_payload_creation() {
        let webhook_id = Uuid::new_v4();
        let payload = WebhookPayload::new(
            "user.created",
            serde_json::json!({"user_id": "123"}),
            webhook_id,
        );

        assert_eq!(payload.event, "user.created");
        assert_eq!(payload.webhook_id, webhook_id);

        let json = payload.to_json().expect("Serialization failed");
        assert!(json.contains("user.created"));
    }

    #[test]
    fn test_delivery_status_transitions() {
        let endpoint = WebhookEndpoint::new("https://example.com", vec!["test".into()]);
        let payload = WebhookPayload::new("test", serde_json::json!({}), endpoint.id);
        let mut delivery = WebhookDelivery::new(&endpoint, &payload);

        assert_eq!(delivery.status, DeliveryStatus::Pending);
        assert_eq!(delivery.attempt, 0);

        delivery.mark_success(200, Some("OK".into()));
        assert_eq!(delivery.status, DeliveryStatus::Success);
        assert_eq!(delivery.attempt, 1);
        assert!(delivery.completed_at.is_some());
    }

    #[test]
    fn test_delivery_retry_logic() {
        let endpoint = WebhookEndpoint::new("https://example.com", vec!["test".into()]);
        let payload = WebhookPayload::new("test", serde_json::json!({}), endpoint.id);
        let mut delivery = WebhookDelivery::new(&endpoint, &payload);

        delivery.mark_failed("Connection error", true, Duration::seconds(60));
        assert_eq!(delivery.status, DeliveryStatus::Retrying);
        assert!(delivery.next_retry_at.is_some());
        assert!(delivery.completed_at.is_none());

        delivery.mark_failed("Connection error", false, Duration::seconds(60));
        assert_eq!(delivery.status, DeliveryStatus::Failed);
        assert!(delivery.completed_at.is_some());
    }

    #[test]
    fn test_endpoint_failure_tracking() {
        let mut endpoint = WebhookEndpoint::new("https://example.com", vec!["test".into()]);

        endpoint.record_failure(3);
        assert_eq!(endpoint.failure_count, 1);
        assert_eq!(endpoint.status, WebhookStatus::Active);

        endpoint.record_failure(3);
        endpoint.record_failure(3);
        assert_eq!(endpoint.failure_count, 3);
        assert_eq!(endpoint.status, WebhookStatus::Suspended);

        endpoint.record_success();
        assert_eq!(endpoint.failure_count, 0);
        assert_eq!(endpoint.status, WebhookStatus::Active);
    }

    #[tokio::test]
    async fn test_webhook_manager_registration() {
        let manager = WebhookManager::with_defaults();

        let endpoint = manager
            .register_endpoint("https://example.com/webhook", vec!["user.created".into()])
            .await
            .expect("Registration failed");

        assert!(endpoint.is_active());

        let retrieved = manager.get_endpoint(endpoint.id).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().url, "https://example.com/webhook");
    }

    #[tokio::test]
    async fn test_webhook_manager_event_filtering() {
        let manager = WebhookManager::with_defaults();

        manager
            .register_endpoint("https://a.com/webhook", vec!["user.created".into()])
            .await
            .expect("Registration failed");

        manager
            .register_endpoint("https://b.com/webhook", vec!["order.created".into()])
            .await
            .expect("Registration failed");

        let user_endpoints = manager.get_endpoints_for_event("user.created").await;
        assert_eq!(user_endpoints.len(), 1);
        assert_eq!(user_endpoints[0].url, "https://a.com/webhook");
    }

    #[test]
    fn test_parse_signature_header() {
        let (version, sig) = parse_signature_header("v1=abc123").expect("Parse failed");
        assert_eq!(version, "v1");
        assert_eq!(sig, "abc123");

        assert!(parse_signature_header("invalid").is_none());
    }

    #[test]
    fn test_constant_time_compare() {
        assert!(constant_time_compare("abc123", "abc123"));
        assert!(!constant_time_compare("abc123", "abc124"));
        assert!(!constant_time_compare("abc", "abcd"));
    }

    #[tokio::test]
    async fn test_signature_cleanup() {
        let mut config = WebhookConfig::default();
        config.replay_window_seconds = 0;
        let manager = WebhookSecurityManager::new(config);

        let payload = r#"{"event":"test"}"#;
        let secret = "test_secret";
        let timestamp = Utc::now();
        let signature = manager.sign_payload(payload, secret, timestamp);
        let timestamp_str = timestamp.timestamp().to_string();

        manager
            .verify_signature(payload, &signature, &timestamp_str, secret)
            .await;

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let cleaned = manager.cleanup_old_signatures().await;
        assert!(cleaned >= 1);
    }
}
