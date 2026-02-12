use async_trait::async_trait;
use log::{error, info};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::bot::channels::ChannelAdapter;
use crate::core::config::ConfigManager;
use crate::core::shared::models::BotResponse;
use crate::core::shared::utils::DbPool;

#[derive(Debug)]
pub struct WhatsAppAdapter {
    api_key: String,
    phone_number_id: String,
    webhook_verify_token: String,
    _business_account_id: String,
    api_version: String,
}

impl WhatsAppAdapter {
    pub fn new(pool: DbPool, bot_id: Uuid) -> Self {
        let config_manager = ConfigManager::new(pool);

        let api_key = config_manager
            .get_config(&bot_id, "whatsapp-api-key", None)
            .unwrap_or_default();

        let phone_number_id = config_manager
            .get_config(&bot_id, "whatsapp-phone-number-id", None)
            .unwrap_or_default();

        let verify_token = config_manager
            .get_config(&bot_id, "whatsapp-verify-token", None)
            .unwrap_or_else(|_| "webhook_verify".to_string());

        let business_account_id = config_manager
            .get_config(&bot_id, "whatsapp-business-account-id", None)
            .unwrap_or_default();

        let api_version = config_manager
            .get_config(&bot_id, "whatsapp-api-version", Some("v17.0"))
            .unwrap_or_else(|_| "v17.0".to_string());

        Self {
            api_key,
            phone_number_id,
            webhook_verify_token: verify_token,
            _business_account_id: business_account_id,
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
            "to": to,
            "type": "text",
            "text": {
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
        components: Vec<serde_json::Value>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();

        let url = format!(
            "https://graph.facebook.com/{}/{}/messages",
            self.api_version, self.phone_number_id
        );

        let mut payload = serde_json::json!({
            "messaging_product": "whatsapp",
            "to": to,
            "type": "template",
            "template": {
                "name": template_name,
                "language": {
                    "code": language_code
                }
            }
        });

        if !components.is_empty() {
            payload["template"]["components"] = serde_json::Value::Array(components);
        }

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
        media_url: &str,
        media_type: &str,
        caption: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();

        let url = format!(
            "https://graph.facebook.com/{}/{}/messages",
            self.api_version, self.phone_number_id
        );

        let media_object = match media_type {
            "image" | "video" => {
                let mut obj = serde_json::json!({
                    "link": media_url
                });
                if let Some(cap) = caption {
                    obj["caption"] = serde_json::Value::String(cap.to_string());
                }
                obj
            }
            "document" => {
                let mut obj = serde_json::json!({
                    "link": media_url
                });
                if let Some(cap) = caption {
                    obj["filename"] = serde_json::Value::String(cap.to_string());
                }
                obj
            }
            // audio and any other type
            _ => serde_json::json!({
                "link": media_url
            }),
        };

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

    pub async fn send_location_message(
        &self,
        to: &str,
        latitude: f64,
        longitude: f64,
        name: Option<&str>,
        address: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();

        let url = format!(
            "https://graph.facebook.com/{}/{}/messages",
            self.api_version, self.phone_number_id
        );

        let mut location = serde_json::json!({
            "latitude": latitude,
            "longitude": longitude
        });

        if let Some(n) = name {
            location["name"] = serde_json::Value::String(n.to_string());
        }
        if let Some(a) = address {
            location["address"] = serde_json::Value::String(a.to_string());
        }

        let payload = serde_json::json!({
            "messaging_product": "whatsapp",
            "to": to,
            "type": "location",
            "location": location
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

    pub async fn mark_message_as_read(
        &self,
        message_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();

        let url = format!(
            "https://graph.facebook.com/{}/{}/messages",
            self.api_version, self.phone_number_id
        );

        let payload = serde_json::json!({
            "messaging_product": "whatsapp",
            "status": "read",
            "message_id": message_id
        });

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Failed to mark message as read: {}", error_text).into());
        }

        Ok(())
    }

    pub async fn get_business_profile(
        &self,
    ) -> Result<WhatsAppBusinessProfile, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();

        let url = format!(
            "https://graph.facebook.com/{}/{}/whatsapp_business_profile",
            self.api_version, self.phone_number_id
        );

        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .query(&[(
                "fields",
                "about,address,description,email,profile_picture_url,websites,vertical",
            )])
            .send()
            .await?;

        if response.status().is_success() {
            let profiles: serde_json::Value = response.json().await?;
            if let Some(data) = profiles["data"].as_array() {
                if let Some(first_profile) = data.first() {
                    let profile: WhatsAppBusinessProfile =
                        serde_json::from_value(first_profile.clone())?;
                    return Ok(profile);
                }
            }
            Err("No business profile found".into())
        } else {
            let error_text = response.text().await?;
            Err(format!("Failed to get business profile: {}", error_text).into())
        }
    }

    pub async fn upload_media(
        &self,
        file_path: &str,
        mime_type: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();

        let url = format!(
            "https://graph.facebook.com/{}/{}/media",
            self.api_version, self.phone_number_id
        );

        let file_data = tokio::fs::read(file_path).await?;

        let part = reqwest::multipart::Part::bytes(file_data)
            .mime_str(mime_type)?
            .file_name(file_path.to_string());

        let form = reqwest::multipart::Form::new()
            .part("file", part)
            .text("messaging_product", "whatsapp");

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .multipart(form)
            .send()
            .await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            Ok(result["id"].as_str().unwrap_or("").to_string())
        } else {
            let error_text = response.text().await?;
            Err(format!("Failed to upload media: {}", error_text).into())
        }
    }

    pub fn verify_webhook(&self, token: &str) -> bool {
        token == self.webhook_verify_token
    }

    pub fn handle_webhook_verification(
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
impl ChannelAdapter for WhatsAppAdapter {
    fn name(&self) -> &'static str {
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
        if let Some(entry) = payload["entry"].as_array() {
            if let Some(first_entry) = entry.first() {
                if let Some(changes) = first_entry["changes"].as_array() {
                    if let Some(first_change) = changes.first() {
                        if let Some(messages) = first_change["value"]["messages"].as_array() {
                            if let Some(first_message) = messages.first() {
                                if let Some(message_id) = first_message["id"].as_str() {
                                    let _ = self.mark_message_as_read(message_id).await;
                                }

                                let message_type =
                                    first_message["type"].as_str().unwrap_or("unknown");

                                return match message_type {
                                    "text" => Ok(first_message["text"]["body"]
                                        .as_str()
                                        .map(|s| s.to_string())),
                                    "image" | "video" | "audio" | "document" => {
                                        let caption = first_message[message_type]["caption"]
                                            .as_str()
                                            .unwrap_or("");
                                        Ok(Some(format!(
                                            "Received {} with caption: {}",
                                            message_type, caption
                                        )))
                                    }
                                    "location" => {
                                        let lat = first_message["location"]["latitude"]
                                            .as_f64()
                                            .unwrap_or(0.0);
                                        let lon = first_message["location"]["longitude"]
                                            .as_f64()
                                            .unwrap_or(0.0);
                                        Ok(Some(format!("Location: {}, {}", lat, lon)))
                                    }
                                    "button" => Ok(first_message["button"]["text"]
                                        .as_str()
                                        .map(|s| s.to_string())),
                                    "interactive" => {
                                        if let Some(button_reply) =
                                            first_message["interactive"]["button_reply"].as_object()
                                        {
                                            Ok(button_reply["id"].as_str().map(|s| s.to_string()))
                                        } else if let Some(list_reply) =
                                            first_message["interactive"]["list_reply"].as_object()
                                        {
                                            Ok(list_reply["id"].as_str().map(|s| s.to_string()))
                                        } else {
                                            Ok(None)
                                        }
                                    }
                                    _ => Ok(None),
                                };
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
        Ok(serde_json::json!({
            "id": user_id,
            "platform": "whatsapp"
        }))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhatsAppBusinessProfile {
    pub about: Option<String>,
    pub address: Option<String>,
    pub description: Option<String>,
    pub email: Option<String>,
    pub profile_picture_url: Option<String>,
    pub websites: Option<Vec<String>>,
    pub vertical: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhatsAppWebhookPayload {
    pub object: String,
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
    pub contacts: Option<Vec<WhatsAppContact>>,
    pub messages: Option<Vec<WhatsAppMessage>>,
    pub statuses: Option<Vec<WhatsAppStatus>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhatsAppMetadata {
    pub display_phone_number: String,
    pub phone_number_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhatsAppContact {
    pub profile: WhatsAppProfile,
    pub wa_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhatsAppProfile {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhatsAppMessage {
    pub from: String,
    pub id: String,
    pub timestamp: String,
    #[serde(rename = "type")]
    pub message_type: String,
    pub text: Option<WhatsAppText>,
    pub image: Option<WhatsAppMedia>,
    pub video: Option<WhatsAppMedia>,
    pub audio: Option<WhatsAppMedia>,
    pub document: Option<WhatsAppMedia>,
    pub location: Option<WhatsAppLocation>,
    pub button: Option<WhatsAppButton>,
    pub interactive: Option<WhatsAppInteractive>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhatsAppText {
    pub body: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhatsAppMedia {
    pub id: String,
    pub mime_type: Option<String>,
    pub sha256: Option<String>,
    pub caption: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhatsAppLocation {
    pub latitude: f64,
    pub longitude: f64,
    pub name: Option<String>,
    pub address: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhatsAppButton {
    pub text: String,
    pub payload: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhatsAppInteractive {
    #[serde(rename = "type")]
    pub interactive_type: String,
    pub button_reply: Option<WhatsAppButtonReply>,
    pub list_reply: Option<WhatsAppListReply>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhatsAppButtonReply {
    pub id: String,
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhatsAppListReply {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhatsAppStatus {
    pub id: String,
    pub status: String,
    pub timestamp: String,
    pub recipient_id: String,
}

pub fn create_interactive_buttons(text: &str, buttons: Vec<(&str, &str)>) -> serde_json::Value {
    let button_list: Vec<serde_json::Value> = buttons
        .into_iter()
        .take(3)
        .map(|(id, title)| {
            serde_json::json!({
                "type": "reply",
                "reply": {
                    "id": id,
                    "title": title
                }
            })
        })
        .collect();

    serde_json::json!({
        "type": "button",
        "body": {
            "text": text
        },
        "action": {
            "buttons": button_list
        }
    })
}

pub type InteractiveListSections = Vec<(String, Vec<(String, String, Option<String>)>)>;

pub fn create_interactive_list(
    text: &str,
    button_text: &str,
    sections: InteractiveListSections,
) -> serde_json::Value {
    let section_list: Vec<serde_json::Value> = sections
        .into_iter()
        .map(|(title, rows)| {
            let row_list: Vec<serde_json::Value> = rows
                .into_iter()
                .take(10)
                .map(|(id, title, description)| {
                    let mut row = serde_json::json!({
                        "id": id,
                        "title": title
                    });
                    if let Some(desc) = description {
                        row["description"] = serde_json::Value::String(desc);
                    }
                    row
                })
                .collect();

            serde_json::json!({
                "title": title,
                "rows": row_list
            })
        })
        .collect();

    serde_json::json!({
        "type": "list",
        "body": {
            "text": text
        },
        "action": {
            "button": button_text,
            "sections": section_list
        }
    })
}
