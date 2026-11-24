use botsecurity_core::validation::{resolve_checked, validate_url_ssrf_strict};
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone)]
pub struct WebhookEvent {
    pub id: Uuid,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub signature: String,
    pub timestamp: DateTime<Utc>,
    pub delivery_status: DeliveryStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeliveryStatus {
    Pending,
    Delivered,
    Failed(String),
}

impl WebhookEvent {
    pub fn new(event: &str, payload: serde_json::Value, secret: &str) -> Self {
        let timestamp = Utc::now();
        let signature = Self::compute_signature(&payload, secret, &timestamp);

        Self {
            id: Uuid::new_v4(),
            event_type: event.to_string(),
            payload,
            signature,
            timestamp,
            delivery_status: DeliveryStatus::Pending,
        }
    }

    pub fn verify_signature(&self, secret: &str) -> bool {
        let expected = Self::compute_signature(&self.payload, secret, &self.timestamp);
        botsecurity_core::constant_time::eq_str(&expected, &self.signature)
    }

    fn compute_signature(
        payload: &serde_json::Value,
        secret: &str,
        timestamp: &DateTime<Utc>,
    ) -> String {
        let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
            Ok(m) => m,
            Err(_) => return String::new(),
        };

        let data = format!("{}{}", timestamp.timestamp(), payload);
        mac.update(data.as_bytes());
        let result = mac.finalize();
        hex::encode(result.into_bytes())
    }

    /// Applies the shared outbound-request policy before an outbound webhook.
    ///
    /// `validate_url_ssrf_strict` rejects non-http(s) schemes, userinfo,
    /// local/metadata hosts, obfuscated IPv4 and every non-global IP literal
    /// (loopback, private, link-local, IPv6 ULA and IPv4-mapped loopback).
    /// `resolve_checked` then refuses a hostname that resolves to any
    /// non-global address, which also stops a mixed public/private DNS answer
    /// set. This replaces the old hand-rolled URL guard that missed those
    /// ranges (issue #2643).
    async fn check_webhook_target(url: &str) -> Result<(), String> {
        let parsed = validate_url_ssrf_strict(url)
            .map_err(|_| "Blocked webhook to unsafe URL".to_string())?;
        resolve_checked(&parsed)
            .await
            .map_err(|_| "Blocked webhook to unsafe URL".to_string())?;
        Ok(())
    }

    pub async fn dispatch(&self, url: &str) -> Result<(), String> {
        Self::check_webhook_target(url).await?;
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let body = serde_json::json!({
            "event": self.event_type,
            "id": self.id.to_string(),
            "timestamp": self.timestamp.to_rfc3339(),
            "payload": self.payload,
            "signature": self.signature,
        });

        let resp = client
            .post(url)
            .header("Content-Type", "application/json")
            .header("X-Webhook-Signature", &self.signature)
            .header("X-Webhook-Event", &self.event_type)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if resp.status().is_success() {
            Ok(())
        } else {
            let status = resp.status();
            let text = resp.text().await.map_or_else(|_| { log::warn!("defaulting after error at {}:{}", file!(), line!()); Default::default() }, |v| v);
            Err(format!("Webhook delivery failed ({}): {}", status, text))
        }
    }
}

pub struct WebhookDispatcher {
    subscribers: Vec<WebhookSubscriber>,
}

#[derive(Debug, Clone)]
pub struct WebhookSubscriber {
    pub id: Uuid,
    pub url: String,
    pub events: Vec<String>,
    pub secret: String,
    pub retry_count: u32,
}

impl WebhookDispatcher {
    pub fn new() -> Self {
        Self {
            subscribers: Vec::new(),
        }
    }

    pub fn subscribe(&mut self, url: &str, events: &[&str], secret: &str) -> Uuid {
        let sub = WebhookSubscriber {
            id: Uuid::new_v4(),
            url: url.to_string(),
            events: events.iter().map(|e| e.to_string()).collect(),
            secret: secret.to_string(),
            retry_count: 3,
        };
        let id = sub.id;
        self.subscribers.push(sub);
        id
    }

    pub fn unsubscribe(&mut self, id: &Uuid) -> bool {
        let before = self.subscribers.len();
        self.subscribers.retain(|s| &s.id != id);
        self.subscribers.len() < before
    }

    pub async fn dispatch_event(&self, event: &WebhookEvent) -> Vec<(String, Result<(), String>)> {
        let mut results = Vec::new();
        for sub in &self.subscribers {
            if !sub.events.contains(&event.event_type) {
                continue;
            }

            let result = event.dispatch(&sub.url).await;

            if result.is_err() && sub.retry_count > 0 {
                for attempt in 1..=sub.retry_count {
                    tokio::time::sleep(std::time::Duration::from_secs(2u64.pow(attempt))).await;
                    let retry_result = event.dispatch(&sub.url).await;
                    if retry_result.is_ok() {
                        results.push((sub.url.clone(), Ok(())));
                        break;
                    }
                    if attempt == sub.retry_count {
                        results.push((sub.url.clone(), retry_result));
                    }
                }
            } else {
                results.push((sub.url.clone(), result));
            }
        }
        results
    }

    pub fn subscribers_for_event(&self, event_type: &str) -> Vec<&WebhookSubscriber> {
        self.subscribers
            .iter()
            .filter(|s| s.events.contains(&event_type.to_string()))
            .collect()
    }
}

impl Default for WebhookDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_event_new() {
        let payload = serde_json::json!({"contact_id": "123"});
        let event = WebhookEvent::new("contact.created", payload.clone(), "secret123");
        assert_eq!(event.event_type, "contact.created");
        assert_eq!(event.payload, payload);
        assert!(!event.signature.is_empty());
    }

    #[test]
    fn test_signature_verification() {
        let payload = serde_json::json!({"test": true});
        let event = WebhookEvent::new("test.event", payload, "secret123");
        assert!(event.verify_signature("secret123"));
        assert!(!event.verify_signature("wrong_secret"));
    }

    #[test]
    fn test_subscribe_unsubscribe() {
        let mut disp = WebhookDispatcher::new();
        let id = disp.subscribe("https://ex.com/hook", &["contact.created"], "sec");
        assert_eq!(disp.subscribers.len(), 1);
        assert!(disp.unsubscribe(&id));
        assert_eq!(disp.subscribers.len(), 0);
    }

    #[test]
    fn test_subscribe_filter() {
        let mut disp = WebhookDispatcher::new();
        disp.subscribe("https://ex.com/hook", &["contact.created"], "sec");
        let subs = disp.subscribers_for_event("contact.created");
        assert_eq!(subs.len(), 1);
        let subs = disp.subscribers_for_event("ticket.updated");
        assert_eq!(subs.len(), 0);
    }

    #[test]
    fn test_public_url_passes_static_policy() {
        assert!(validate_url_ssrf_strict("https://example.com/hook").is_ok());
    }

    #[tokio::test]
    async fn test_adversarial_webhook_urls_rejected() {
        let event = WebhookEvent::new("test.event", serde_json::json!({"k": 1}), "s");
        // The old ad-hoc guard accepted these non-global ranges. The shared
        // policy must refuse them before any request leaves the process.
        let blocked = [
            "http://[fd00::1]/hook",                   // IPv6 unique-local (fc00::/7)
            "http://[fe80::1]/hook",                   // IPv6 link-local (fe80::/10)
            "http://[::ffff:127.0.0.1]/hook",          // IPv4-mapped loopback
            "http://169.254.169.254/latest/meta-data", // IPv4 link-local / metadata
            "http://127.0.0.1/hook",                   // IPv4 loopback
            "http://10.0.0.1/hook",                    // IPv4 private
            "http://localhost/hook",                   // local hostname
            "http://service.internal/hook",            // blocked hostname suffix
        ];
        for url in blocked {
            let result = event.dispatch(url).await;
            assert!(result.is_err(), "expected {url} to be rejected");
        }
    }
}
