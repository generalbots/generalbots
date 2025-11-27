use async_trait::async_trait;
use log::{error, info};
use serde::{Deserialize, Serialize};
// use std::collections::HashMap; // Unused import

use crate::core::bot::channels::ChannelAdapter;
use crate::shared::models::BotResponse;

pub struct WhatsAppAdapter {
    api_key: String,
    phone_number_id: String,
    webhook_verify_token: String,
    business_account_id: String,
    api_version: String,
}

impl WhatsAppAdapter {
    pub fn new() -> Self {
        // Load from environment variables (would be from config.csv in production)
        let api_key = std::env::var("WHATSAPP_API_KEY").unwrap_or_default();
        let phone_number_id = std::env::var("WHATSAPP_PHONE_NUMBER_ID").unwrap_or_default();
        let webhook_verify_token =
            std::env::var("WHATSAPP_VERIFY_TOKEN").unwrap_or_else(|_| "webhook_verify".to_string());
        let business_account_id = std::env::var("WHATSAPP_BUSINESS_ACCOUNT_ID").unwrap_or_default();
        let api_version = "v17.0".to_string();

        Self {
            api_key,
            phone_number_id,
            webhook_verify_token,
            business_account_id,
            api_version,
        }
    }

    async fn send_whatsapp_message(
        &self,
        to: &str,
        message: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();

        let url = format!(
            "https://graph.facebook.com/{}/{}/messages",
            self.api_version, self.phone_number_id
        );

        let payload = serde_json::json!({
            "messaging_product": "whatsapp",
            "recipient_type": "individual",
            "to": to,
            "type": "text",
            "text": {
                "preview_url": false,
                "body": message
            }
        });

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            Ok(result["messages"][0]["id"]
                .as_str()
                .unwrap_or("")
                .to_string())
        } else {
            let error_text = response.text().await?;
            Err(format!("WhatsApp API error: {}", error_text).into())
        }
    }

    pub async fn send_template_message(
        &self,
        to: &str,
        template_name: &str,
        language_code: &str,
        parameters: Vec<String>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();

        let url = format!(
            "https://graph.facebook.com/{}/{}/messages",
            self.api_version, self.phone_number_id
        );

        let components = if !parameters.is_empty() {
            vec![serde_json::json!({
                "type": "body",
                "parameters": parameters.iter().map(|p| {
                    serde_json::json!({
                        "type": "text",
                        "text": p
                    })
                }).collect::<Vec<_>>()
            })]
        } else {
            vec![]
        };

        let payload = serde_json::json!({
            "messaging_product": "whatsapp",
            "to": to,
            "type": "template",
            "template": {
                "name": template_name,
                "language": {
                    "code": language_code
                },
                "components": components
            }
        });

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            Ok(result["messages"][0]["id"]
                .as_str()
                .unwrap_or("")
                .to_string())
        } else {
            let error_text = response.text().await?;
            Err(format!("WhatsApp API error: {}", error_text).into())
        }
    }

    pub async fn send_media_message(
        &self,
        to: &str,
        media_type: &str,
        media_url: &str,
        caption: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();

        let url = format!(
            "https://graph.facebook.com/{}/{}/messages",
            self.api_version, self.phone_number_id
        );

        let mut media_object = serde_json::json!({
            "link": media_url
        });

        if let Some(caption_text) = caption {
            media_object["caption"] = serde_json::json!(caption_text);
        }

        let payload = serde_json::json!({
            "messaging_product": "whatsapp",
            "to": to,
            "type": media_type,
            media_type: media_object
        });

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            Ok(result["messages"][0]["id"]
                .as_str()
                .unwrap_or("")
                .to_string())
        } else {
            let error_text = response.text().await?;
            Err(format!("WhatsApp API error: {}", error_text).into())
        }
    }

    pub fn verify_webhook(&self, token: &str) -> bool {
        token == self.webhook_verify_token
    }
}

#[async_trait]
impl ChannelAdapter for WhatsAppAdapter {
    fn name(&self) -> &str {
        "WhatsApp"
    }

    fn is_configured(&self) -> bool {
        !self.api_key.is_empty() && !self.phone_number_id.is_empty()
    }

    async fn send_message(
        &self,
        response: BotResponse,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.is_configured() {
            error!("WhatsApp adapter not configured. Please set whatsapp-api-key and whatsapp-phone-number-id in config.csv");
            return Err("WhatsApp not configured".into());
        }

        let message_id = self
            .send_whatsapp_message(&response.user_id, &response.content)
            .await?;

        info!(
            "WhatsApp message sent to {}: {} (message_id: {})",
            response.user_id, response.content, message_id
        );

        Ok(())
    }

    async fn receive_message(
        &self,
        payload: serde_json::Value,
    ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        // Parse WhatsApp webhook payload
        if let Some(entry) = payload["entry"].as_array() {
            if let Some(first_entry) = entry.first() {
                if let Some(changes) = first_entry["changes"].as_array() {
                    if let Some(first_change) = changes.first() {
                        if let Some(messages) = first_change["value"]["messages"].as_array() {
                            if let Some(first_message) = messages.first() {
                                let message_type = first_message["type"].as_str().unwrap_or("");

                                match message_type {
                                    "text" => {
                                        return Ok(first_message["text"]["body"]
                                            .as_str()
                                            .map(|s| s.to_string()));
                                    }
                                    "image" | "document" | "audio" | "video" => {
                                        return Ok(Some(format!(
                                            "Received {} message",
                                            message_type
                                        )));
                                    }
                                    _ => {
                                        return Ok(Some(format!(
                                            "Received unsupported message type: {}",
                                            message_type
                                        )));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    async fn get_user_info(
        &self,
        user_id: &str,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();

        let url = format!(
            "https://graph.facebook.com/{}/{}",
            self.api_version, user_id
        );

        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Ok(serde_json::json!({
                "id": user_id,
                "platform": "whatsapp"
            }))
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhatsAppWebhookPayload {
    pub entry: Vec<WhatsAppEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhatsAppEntry {
    pub id: String,
    pub changes: Vec<WhatsAppChange>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhatsAppChange {
    pub field: String,
    pub value: WhatsAppValue,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhatsAppValue {
    pub messaging_product: String,
    pub metadata: WhatsAppMetadata,
    pub messages: Option<Vec<WhatsAppMessage>>,
    pub statuses: Option<Vec<WhatsAppStatus>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhatsAppMetadata {
    pub display_phone_number: String,
    pub phone_number_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhatsAppMessage {
    pub from: String,
    pub id: String,
    pub timestamp: String,
    #[serde(rename = "type")]
    pub message_type: String,
    pub text: Option<WhatsAppText>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhatsAppText {
    pub body: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhatsAppStatus {
    pub id: String,
    pub status: String,
    pub timestamp: String,
    pub recipient_id: String,
}
