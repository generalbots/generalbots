use async_trait::async_trait;
use log::{error, info};
use serde::{Deserialize, Serialize};

use crate::core::bot::channels::ChannelAdapter;
use crate::shared::models::BotResponse;

#[derive(Debug)]
pub struct InstagramAdapter {
    access_token: String,
    verify_token: String,
    page_id: String,
    api_version: String,
    instagram_account_id: String,
}

impl InstagramAdapter {
    pub fn new() -> Self {
        // Load from config.csv in production
        let access_token = String::new(); // Configure via config.csv: instagram-access-token
        let verify_token = "webhook_verify".to_string(); // Configure via config.csv: instagram-verify-token
        let page_id = String::new(); // Configure via config.csv: instagram-page-id
        let api_version = "v17.0".to_string();
        let instagram_account_id = String::new(); // Configure via config.csv: instagram-account-id

        Self {
            access_token,
            verify_token,
            page_id,
            api_version,
            instagram_account_id,
        }
    }

    pub fn get_instagram_account_id(&self) -> &str {
        &self.instagram_account_id
    }

    pub async fn get_instagram_business_account(
        &self,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();

        let url = format!(
            "https://graph.facebook.com/{}/{}/instagram_business_account",
            self.api_version, self.page_id
        );

        let response = client
            .get(&url)
            .query(&[("access_token", &self.access_token)])
            .send()
            .await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            Ok(result["id"]
                .as_str()
                .unwrap_or(&self.instagram_account_id)
                .to_string())
        } else {
            Ok(self.instagram_account_id.clone())
        }
    }

    pub async fn post_to_instagram(
        &self,
        image_url: &str,
        caption: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();
        let account_id = if self.instagram_account_id.is_empty() {
            self.get_instagram_business_account().await?
        } else {
            self.instagram_account_id.clone()
        };

        // Step 1: Create media container
        let container_url = format!(
            "https://graph.facebook.com/{}/{}/media",
            self.api_version, account_id
        );

        let container_response = client
            .post(&container_url)
            .query(&[
                ("access_token", &self.access_token),
                ("image_url", &image_url.to_string()),
                ("caption", &caption.to_string()),
            ])
            .send()
            .await?;

        if !container_response.status().is_success() {
            let error_text = container_response.text().await?;
            return Err(format!("Failed to create media container: {}", error_text).into());
        }

        let container_result: serde_json::Value = container_response.json().await?;
        let creation_id = container_result["id"]
            .as_str()
            .ok_or("No creation_id in response")?;

        // Step 2: Publish the media
        let publish_url = format!(
            "https://graph.facebook.com/{}/{}/media_publish",
            self.api_version, account_id
        );

        let publish_response = client
            .post(&publish_url)
            .query(&[
                ("access_token", &self.access_token),
                ("creation_id", &creation_id.to_string()),
            ])
            .send()
            .await?;

        if publish_response.status().is_success() {
            let publish_result: serde_json::Value = publish_response.json().await?;
            Ok(publish_result["id"].as_str().unwrap_or("").to_string())
        } else {
            let error_text = publish_response.text().await?;
            Err(format!("Failed to publish media: {}", error_text).into())
        }
    }

    pub async fn send_instagram_message(
        &self,
        recipient_id: &str,
        message: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();

        let url = format!(
            "https://graph.facebook.com/{}/{}/messages",
            self.api_version, self.page_id
        );

        let payload = serde_json::json!({
            "recipient": {
                "id": recipient_id
            },
            "message": {
                "text": message
            },
            "messaging_type": "RESPONSE"
        });

        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .query(&[("access_token", &self.access_token)])
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            Ok(result["message_id"].as_str().unwrap_or("").to_string())
        } else {
            let error_text = response.text().await?;
            Err(format!("Instagram API error: {}", error_text).into())
        }
    }

    pub async fn send_media_message(
        &self,
        recipient_id: &str,
        media_url: &str,
        media_type: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();

        let url = format!(
            "https://graph.facebook.com/{}/{}/messages",
            self.api_version, self.page_id
        );

        let attachment_type = match media_type {
            "image" => "image",
            "video" => "video",
            "audio" => "audio",
            _ => "file",
        };

        let payload = serde_json::json!({
            "recipient": {
                "id": recipient_id
            },
            "message": {
                "attachment": {
                    "type": attachment_type,
                    "payload": {
                        "url": media_url,
                        "is_reusable": true
                    }
                }
            }
        });

        let response = client
            .post(&url)
            .query(&[("access_token", &self.access_token)])
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            Ok(result["message_id"].as_str().unwrap_or("").to_string())
        } else {
            let error_text = response.text().await?;
            Err(format!("Instagram API error: {}", error_text).into())
        }
    }

    pub async fn send_story_reply(
        &self,
        recipient_id: &str,
        message: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Story replies use the same messaging API
        self.send_instagram_message(recipient_id, message).await
    }

    pub async fn get_user_profile(
        &self,
        user_id: &str,
    ) -> Result<InstagramProfile, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();

        let url = format!(
            "https://graph.facebook.com/{}/{}",
            self.api_version, user_id
        );

        let response = client
            .get(&url)
            .query(&[
                ("access_token", &self.access_token),
                ("fields", &"name,profile_pic".to_string()),
            ])
            .send()
            .await?;

        if response.status().is_success() {
            let profile: InstagramProfile = response.json().await?;
            Ok(profile)
        } else {
            Err("Failed to get Instagram profile".into())
        }
    }

    pub fn verify_webhook(&self, token: &str) -> bool {
        token == self.verify_token
    }

    pub async fn handle_webhook_verification(
        &self,
        mode: &str,
        token: &str,
        challenge: &str,
    ) -> Option<String> {
        if mode == "subscribe" && self.verify_webhook(token) {
            Some(challenge.to_string())
        } else {
            None
        }
    }
}

#[async_trait]
impl ChannelAdapter for InstagramAdapter {
    fn name(&self) -> &str {
        "Instagram"
    }

    fn is_configured(&self) -> bool {
        !self.access_token.is_empty() && !self.page_id.is_empty()
    }

    async fn send_message(
        &self,
        response: BotResponse,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.is_configured() {
            error!("Instagram adapter not configured. Please set instagram-access-token and instagram-page-id in config.csv");
            return Err("Instagram not configured".into());
        }

        let message_id = self
            .send_instagram_message(&response.user_id, &response.content)
            .await?;

        info!(
            "Instagram message sent to {}: {} (message_id: {})",
            response.user_id, response.content, message_id
        );

        Ok(())
    }

    async fn receive_message(
        &self,
        payload: serde_json::Value,
    ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        // Parse Instagram webhook payload
        if let Some(entry) = payload["entry"].as_array() {
            if let Some(first_entry) = entry.first() {
                if let Some(messaging) = first_entry["messaging"].as_array() {
                    if let Some(first_message) = messaging.first() {
                        // Check for different message types
                        if let Some(message) = first_message["message"].as_object() {
                            if let Some(text) = message["text"].as_str() {
                                return Ok(Some(text.to_string()));
                            } else if let Some(attachments) = message["attachments"].as_array() {
                                if let Some(first_attachment) = attachments.first() {
                                    let attachment_type =
                                        first_attachment["type"].as_str().unwrap_or("unknown");
                                    return Ok(Some(format!(
                                        "Received {} attachment",
                                        attachment_type
                                    )));
                                }
                            }
                        } else if let Some(postback) = first_message["postback"].as_object() {
                            if let Some(payload_str) = postback["payload"].as_str() {
                                return Ok(Some(format!("Postback: {}", payload_str)));
                            }
                        }
                    }
                } else if let Some(changes) = first_entry["changes"].as_array() {
                    // Handle Instagram mentions and comments
                    if let Some(first_change) = changes.first() {
                        let field = first_change["field"].as_str().unwrap_or("");
                        match field {
                            "comments" => {
                                if let Some(text) = first_change["value"]["text"].as_str() {
                                    return Ok(Some(format!("Comment: {}", text)));
                                }
                            }
                            "mentions" => {
                                if let Some(media_id) = first_change["value"]["media_id"].as_str() {
                                    return Ok(Some(format!("Mentioned in media: {}", media_id)));
                                }
                            }
                            _ => {}
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
        match self.get_user_profile(user_id).await {
            Ok(profile) => Ok(serde_json::to_value(profile)?),
            Err(_) => Ok(serde_json::json!({
                "id": user_id,
                "platform": "instagram"
            })),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstagramProfile {
    pub id: String,
    pub name: Option<String>,
    pub profile_pic: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstagramWebhookPayload {
    pub object: String,
    pub entry: Vec<InstagramEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstagramEntry {
    pub id: String,
    pub time: i64,
    pub messaging: Option<Vec<InstagramMessaging>>,
    pub changes: Option<Vec<InstagramChange>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstagramMessaging {
    pub sender: InstagramUser,
    pub recipient: InstagramUser,
    pub timestamp: i64,
    pub message: Option<InstagramMessage>,
    pub postback: Option<InstagramPostback>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstagramUser {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstagramMessage {
    pub mid: String,
    pub text: Option<String>,
    pub attachments: Option<Vec<InstagramAttachment>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstagramAttachment {
    #[serde(rename = "type")]
    pub attachment_type: String,
    pub payload: InstagramAttachmentPayload,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstagramAttachmentPayload {
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstagramPostback {
    pub payload: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstagramChange {
    pub field: String,
    pub value: serde_json::Value,
}

// Helper functions for Instagram-specific features

pub fn create_quick_reply(text: &str, replies: Vec<(&str, &str)>) -> serde_json::Value {
    let quick_replies: Vec<serde_json::Value> = replies
        .into_iter()
        .map(|(title, payload)| {
            serde_json::json!({
                "content_type": "text",
                "title": title,
                "payload": payload
            })
        })
        .collect();

    serde_json::json!({
        "text": text,
        "quick_replies": quick_replies
    })
}

pub fn create_generic_template(elements: Vec<serde_json::Value>) -> serde_json::Value {
    serde_json::json!({
        "attachment": {
            "type": "template",
            "payload": {
                "template_type": "generic",
                "elements": elements
            }
        }
    })
}

pub fn create_media_template(media_type: &str, attachment_id: &str) -> serde_json::Value {
    serde_json::json!({
        "attachment": {
            "type": "template",
            "payload": {
                "template_type": "media",
                "elements": [{
                    "media_type": media_type,
                    "attachment_id": attachment_id
                }]
            }
        }
    })
}
