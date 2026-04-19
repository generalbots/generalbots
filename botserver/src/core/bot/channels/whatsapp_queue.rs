//! WhatsApp Message Queue
//!
//! Enforces Meta's bursting rules (up to 45 msgs in a burst window) and 
//! handles cooling off (steady state 1 msg/6s) and rate limit errors (131056)
//! with exponential backoff 4^X.

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
    redis_client: Arc<redis::Client>,
}

impl WhatsAppMessageQueue {
    const QUEUE_KEY: &'static str = "whatsapp:message_queue";
    const TFT_PREFIX: &'static str = "whatsapp:tft:"; // Theoretical Finish Time
    const BURST_CAPACITY: i64 = 45;
    const RATE_SECS: i64 = 6;

    pub fn new(redis_client: Arc<redis::Client>) -> Self {
        Self {
            redis_client,
        }
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
        info!("WhatsApp queue worker started (Burst: up to 45 msgs in 6s per recipient)");
        loop {
            if let Err(e) = self.process_next().await {
                error!("WhatsApp queue worker error: {}", e);
                sleep(Duration::from_secs(1)).await;
            }
        }
    }

    async fn process_next(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        
        let result: Option<(String, String)> = conn.blpop::<&str, Option<(String, String)>>(Self::QUEUE_KEY, 5.0).await?;
        
        if let Some((_key, json)) = result {
            let msg: QueuedWhatsAppMessage = serde_json::from_str(&json)?;
            
            // 1. Proactive Rate Limiting (Burst 45, steady state 1/6s)
            self.wait_for_rate_limit(&msg.to, &mut conn).await?;
            
            // 2. Reactive Retry Logic (4^X for error 131056)
            let mut x = 0;
            loop {
                match self.send_message(&msg).await {
                    Ok(_) => break,
                    Err(e) => {
                        let error_str = e.to_string();
                        if error_str.contains("131056") {
                            let wait_secs = 4_u64.pow(x as u32);
                            warn!("WhatsApp 131056 rate limit for {}: retrying in 4^{} = {}s", msg.to, x, wait_secs);
                            sleep(Duration::from_secs(wait_secs)).await;
                            x += 1;
                            if x > 5 {
                                error!("Max retries (4^5) exceeded for {}: {}", msg.to, e);
                                break;
                            }
                        } else {
                            error!("WhatsApp send failure for {}: {}", msg.to, e);
                            break;
                        }
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Implements Virtual Clock / Leaky Bucket for Meta's Bursting Rules.
    /// Capacity: 45 messages (represented as 45 * 6s = 270s of "debt").
    /// Steady rate: 1 message per 6 seconds.
    async fn wait_for_rate_limit(&self, recipient: &str, conn: &mut redis::aio::MultiplexedConnection) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let tft_key = format!("{}{}", Self::TFT_PREFIX, recipient);
        
        let now = chrono::Utc::now().timestamp();
        let tft: i64 = conn.get::<_, Option<i64>>(&tft_key).await?.unwrap_or(0);
        
        // Max "borrowing" is 45 messages * 6s = 270s
        let max_debt_secs = Self::BURST_CAPACITY * Self::RATE_SECS;
        
        let mut wait_secs = 0;
        let mut new_tft = if tft > now {
            // Recipient is in debt
            let debt = tft - now;
            if debt + Self::RATE_SECS > max_debt_secs {
                // Next message would exceed burst capacity
                wait_secs = (debt + Self::RATE_SECS) - max_debt_secs;
                tft + Self::RATE_SECS
            } else {
                tft + Self::RATE_SECS
            }
        } else {
            // Recipient has no active debt
            now + Self::RATE_SECS
        };

        if wait_secs > 0 {
            warn!("Burst capacity exhausted for {}: waiting {}s cooling off", recipient, wait_secs);
            sleep(Duration::from_secs(wait_secs as u64)).await;
            // Advance TFT if we waited (now has changed)
            new_tft = chrono::Utc::now().timestamp() + (new_tft - (now + wait_secs));
        }
        
        // Store the new Theoretical Finish Time with TTL to clean up Redis
        let _: () = conn.set(&tft_key, new_tft).await?;
        let _: () = conn.expire(&tft_key, max_debt_secs + 3600).await?;
        
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
