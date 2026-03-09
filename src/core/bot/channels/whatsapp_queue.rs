//! WhatsApp Message Queue
//!
//! Implements a Redis-backed queue for WhatsApp messages to enforce
//! Meta's official rate limits: 1 message per 6 seconds per recipient (0.17 msg/sec).
//!
//! ## Meta WhatsApp Rate Limits (Official)
//! - **Base Rate**: 1 message per 6 seconds per recipient (0.17 msg/sec)
//! - **Burst Limit**: Up to 45 messages in 6 seconds (borrows from future quota)
//! - **Post-Burst**: Must wait equivalent time at normal rate
//! - **Error Code**: 131056 when rate limit exceeded
//! - **Retry Strategy**: 4^X seconds (X starts at 0, increments on each failure)

use log::{error, info, warn};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::sleep;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedWhatsAppMessage {
    pub to: String,
    pub message: String,
    pub api_key: String,
    pub phone_number_id: String,
    pub api_version: String,
}

#[derive(Debug)]
struct RecipientState {
    last_sent: Instant,
    burst_count: u32,
    burst_started: Option<Instant>,
}

#[derive(Debug)]
pub struct WhatsAppMessageQueue {
    redis_client: redis::Client,
    recipient_states: Arc<Mutex<HashMap<String, RecipientState>>>,
}

impl WhatsAppMessageQueue {
    const QUEUE_KEY: &'static str = "whatsapp:message_queue";
    const MIN_DELAY: Duration = Duration::from_secs(6); // 1 msg per 6 seconds
    const BURST_WINDOW: Duration = Duration::from_secs(6);
    const MAX_BURST: u32 = 45;

    pub fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        Ok(Self {
            redis_client: redis::Client::open(redis_url)?,
            recipient_states: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub async fn enqueue(&self, msg: QueuedWhatsAppMessage) -> Result<(), redis::RedisError> {
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        let json = serde_json::to_string(&msg).map_err(|e| {
            redis::RedisError::from((redis::ErrorKind::TypeError, "JSON serialization failed", e.to_string()))
        })?;
        conn.rpush::<_, _, ()>(Self::QUEUE_KEY, json).await?;
        Ok(())
    }

    pub async fn start_worker(self: Arc<Self>) {
        info!("WhatsApp queue worker started (Meta official rate: 1 msg/6s per recipient)");
        loop {
            if let Err(e) = self.process_next().await {
                error!("WhatsApp queue worker error: {}", e);
                sleep(Duration::from_secs(1)).await;
            }
        }
    }

    async fn process_next(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        
        // BLPOP returns (key, value) tuple
        let result: Option<(String, String)> = conn.blpop(Self::QUEUE_KEY, 5.0).await?;
        
        if let Some((_key, json)) = result {
            let msg: QueuedWhatsAppMessage = serde_json::from_str(&json)?;
            
            // Check and enforce rate limit for this recipient
            self.wait_for_rate_limit(&msg.to).await;
            
            // Send with retry logic
            let mut retry_count = 0;
            loop {
                match self.send_message(&msg).await {
                    Ok(_) => {
                        // Update recipient state
                        let mut states = self.recipient_states.lock().await;
                        let state = states.entry(msg.to.clone()).or_insert(RecipientState {
                            last_sent: Instant::now(),
                            burst_count: 0,
                            burst_started: None,
                        });
                        
                        // Track burst
                        if let Some(burst_start) = state.burst_started {
                            if burst_start.elapsed() < Self::BURST_WINDOW {
                                state.burst_count += 1;
                            } else {
                                state.burst_count = 1;
                                state.burst_started = Some(Instant::now());
                            }
                        } else {
                            state.burst_count = 1;
                            state.burst_started = Some(Instant::now());
                        }
                        
                        state.last_sent = Instant::now();
                        break;
                    }
                    Err(e) => {
                        let error_str = e.to_string();
                        if error_str.contains("131056") {
                            // Rate limit error - exponential backoff
                            let wait_secs = 4_u64.pow(retry_count);
                            warn!("Rate limit hit for {}, retrying in {}s (attempt {})", msg.to, wait_secs, retry_count + 1);
                            sleep(Duration::from_secs(wait_secs)).await;
                            retry_count += 1;
                            if retry_count > 5 {
                                error!("Max retries exceeded for {}: {}", msg.to, e);
                                break;
                            }
                        } else {
                            error!("Failed to send WhatsApp message to {}: {}", msg.to, e);
                            break;
                        }
                    }
                }
            }
        }
        
        Ok(())
    }

    async fn wait_for_rate_limit(&self, recipient: &str) {
        let mut states = self.recipient_states.lock().await;
        let state = states.entry(recipient.to_string()).or_insert(RecipientState {
            last_sent: Instant::now() - Self::MIN_DELAY,
            burst_count: 0,
            burst_started: None,
        });
        
        // Reset burst if window expired
        if let Some(burst_start) = state.burst_started {
            if burst_start.elapsed() >= Self::BURST_WINDOW {
                state.burst_count = 0;
                state.burst_started = None;
            }
        }

        // Check if we can send in burst mode (within 6-second window)
        if let Some(burst_start) = state.burst_started {
            if burst_start.elapsed() < Self::BURST_WINDOW && state.burst_count < Self::MAX_BURST {
                // Can send immediately in burst mode
                return;
            }
        }

        // Check if in burst cooldown
        if state.burst_count > 0 {
            let cooldown = Self::MIN_DELAY * state.burst_count;
            let elapsed = state.last_sent.elapsed();
            if elapsed < cooldown {
                let wait_time = cooldown - elapsed;
                warn!("Burst cooldown for {}: waiting {:?} (sent {} msgs)", recipient, wait_time, state.burst_count);
                drop(states);
                sleep(wait_time).await;
                return;
            } else {
                // Cooldown complete, reset burst
                let mut states = self.recipient_states.lock().await;
                if let Some(s) = states.get_mut(recipient) {
                    s.burst_count = 0;
                    s.burst_started = None;
                }
                drop(states);
            }
        }

        // Normal rate limit: 6 seconds between messages (if not in burst)
        let elapsed = state.last_sent.elapsed();
        if elapsed < Self::MIN_DELAY {
            let wait_time = Self::MIN_DELAY - elapsed;
            drop(states);
            sleep(wait_time).await;
        }
    }

    async fn send_message(&self, msg: &QueuedWhatsAppMessage) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();
        let url = format!(
            "https://graph.facebook.com/{}/{}/messages",
            msg.api_version, msg.phone_number_id
        );

        let payload = serde_json::json!({
            "messaging_product": "whatsapp",
            "to": msg.to,
            "type": "text",
            "text": {
                "body": msg.message
            }
        });

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", msg.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            let msg_id = result["messages"][0]["id"].as_str().unwrap_or("");
            info!("WhatsApp message sent to {}: {} (id: {})", msg.to, &msg.message.chars().take(50).collect::<String>(), msg_id);
            Ok(())
        } else {
            let error_text = response.text().await?;
            Err(format!("WhatsApp API error: {}", error_text).into())
        }
    }
}

#[cfg(test)]
#[path = "whatsapp_queue_tests.rs"]
mod whatsapp_queue_tests;
