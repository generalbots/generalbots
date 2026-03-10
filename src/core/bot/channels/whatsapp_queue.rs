//! WhatsApp Message Queue
//!
//! Implements a Redis-backed queue for WhatsApp messages to enforce
//! Meta's official rate limits: 1 message per 6 seconds per recipient.

use log::{error, info, warn};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
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
pub struct WhatsAppMessageQueue {
    redis_client: redis::Client,
}

impl WhatsAppMessageQueue {
    const QUEUE_KEY: &'static str = "whatsapp:message_queue";
    const LAST_SENT_PREFIX: &'static str = "whatsapp:last_sent:";
    const MIN_DELAY_SECS: i64 = 6;

    pub fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        Ok(Self {
            redis_client: redis::Client::open(redis_url)?,
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
        info!("WhatsApp queue worker started (Meta rate: 1 msg/6s per recipient)");
        loop {
            if let Err(e) = self.process_next().await {
                error!("WhatsApp queue worker error: {}", e);
                sleep(Duration::from_secs(1)).await;
            }
        }
    }

    async fn process_next(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        
        let result: Option<(String, String)> = conn.blpop(Self::QUEUE_KEY, 5.0).await?;
        
        if let Some((_key, json)) = result {
            let msg: QueuedWhatsAppMessage = serde_json::from_str(&json)?;
            
            // Wait for rate limit (stored in Redis)
            self.wait_for_rate_limit(&msg.to, &mut conn).await?;
            
            // Send with retry logic
            let mut retry_count = 0;
            loop {
                match self.send_message(&msg).await {
                    Ok(_) => {
                        // Update last sent time in Redis
                        let last_sent_key = format!("{}{}", Self::LAST_SENT_PREFIX, msg.to);
                        let now = chrono::Utc::now().timestamp();
                        let _: () = conn.set(last_sent_key, now).await?;
                        break;
                    }
                    Err(e) => {
                        let error_str = e.to_string();
                        if error_str.contains("131056") {
                            let wait_secs = 4_i64.pow(retry_count);
                            warn!("Rate limit hit for {}, retrying in {}s", msg.to, wait_secs);
                            sleep(Duration::from_secs(wait_secs as u64)).await;
                            retry_count += 1;
                            if retry_count > 5 {
                                error!("Max retries for {}: {}", msg.to, e);
                                break;
                            }
                        } else {
                            error!("Failed to send to {}: {}", msg.to, e);
                            break;
                        }
                    }
                }
            }
        }
        
        Ok(())
    }

    async fn wait_for_rate_limit(&self, recipient: &str, conn: &mut redis::aio::MultiplexedConnection) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let last_sent_key = format!("{}{}", Self::LAST_SENT_PREFIX, recipient);
        
        // Get last sent time from Redis
        let last_sent: Option<i64> = conn.get(&last_sent_key).await?;
        
        if let Some(last_ts) = last_sent {
            let now = chrono::Utc::now().timestamp();
            let since_last = now - last_ts;
            
            if since_last < Self::MIN_DELAY_SECS {
                let wait_time = Self::MIN_DELAY_SECS - since_last;
                warn!("Rate limiting {}: waiting {}s", recipient, wait_time);
                sleep(Duration::from_secs(wait_time as u64)).await;
            }
        }
        
        Ok(())
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
            info!("WhatsApp sent to {}: {} (id: {})", msg.to, &msg.message.chars().take(50).collect::<String>(), msg_id);
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
