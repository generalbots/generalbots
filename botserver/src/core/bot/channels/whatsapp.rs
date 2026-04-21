use async_trait::async_trait;
use log::{error, info};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::bot::channels::ChannelAdapter;
use crate::core::bot::channels::whatsapp_queue::{QueuedWhatsAppMessage, WhatsAppMessageQueue};
use crate::core::config::ConfigManager;
use crate::core::shared::models::BotResponse;
use crate::core::shared::state::AppState;
use std::sync::Arc;

/// Global WhatsApp message queue (shared across all adapters)
static WHATSAPP_QUEUE: std::sync::OnceLock<Option<Arc<WhatsAppMessageQueue>>> = std::sync::OnceLock::new();

#[derive(Debug, Clone)]
pub struct WhatsAppAdapter {
    api_key: String,
    phone_number_id: String,
    webhook_verify_token: String,
    _business_account_id: String,
    api_version: String,
    _voice_response: bool,
    queue: Option<&'static Arc<WhatsAppMessageQueue>>,
}

impl WhatsAppAdapter {
    pub fn new(state: &Arc<AppState>, bot_id: Uuid) -> Self {
        let config_manager = ConfigManager::new(state.conn.clone());

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

        let voice_response = config_manager
            .get_config(&bot_id, "whatsapp-voice-response", Some("false"))
            .unwrap_or_else(|_| "false".to_string())
            .to_lowercase()
            == "true";

        Self {
            api_key,
            phone_number_id,
            webhook_verify_token: verify_token,
            _business_account_id: business_account_id,
            api_version,
            _voice_response: voice_response,
            queue: WHATSAPP_QUEUE.get_or_init(|| {
                state.cache.as_ref().map(|client| {
                    let q = WhatsAppMessageQueue::new(client.clone());
                    let q_arc = Arc::new(q);
                    let worker_queue = Arc::clone(&q_arc);
                    tokio::spawn(async move {
                        worker_queue.start_worker().await;
                    });
                    q_arc
                })
            }).as_ref(),
        }
    }

    /// Sanitize Markdown text for WhatsApp compatibility
    /// WhatsApp only supports: *bold*, _italic_, ~strikethrough~, ```monospace```
    /// Does NOT support: headers (###), links [text](url), checkboxes, etc.
    pub fn sanitize_for_whatsapp(text: &str) -> String {
        let mut result = text.to_string();

        // Remove Markdown headers (### ## # at start of lines)
        result = regex::Regex::new(r"(?m)^#{1,6}\s*")
            .map(|re| re.replace_all(&result, "").to_string())
            .unwrap_or(result);

        // Convert Markdown links [text](url) to "text: url"
        result = regex::Regex::new(r"\[([^\]]+)\]\(([^)]+)\)")
            .map(|re| re.replace_all(&result, "$1: $2").to_string())
            .unwrap_or(result);

        // Remove image syntax ![alt](url) - just keep alt text
        result = regex::Regex::new(r"!\[([^\]]*)\]\([^)]+\)")
            .map(|re| re.replace_all(&result, "$1").to_string())
            .unwrap_or(result);

        // Remove checkbox syntax [ ] and [x]
        result = regex::Regex::new(r"\[[ x]\]")
            .map(|re| re.replace_all(&result, "•").to_string())
            .unwrap_or(result);

        // Remove horizontal rules (--- or ***)
        result = regex::Regex::new(r"(?m)^[-*]{3,}\s*$")
            .map(|re| re.replace_all(&result, "").to_string())
            .unwrap_or(result);

        // Remove code blocks with triple backticks ```code```
        result = regex::Regex::new(r"```[\s\S]*?```")
            .map(|re| re.replace_all(&result, "").to_string())
            .unwrap_or(result);

        // Remove inline code with single backticks `code`
        result = regex::Regex::new(r"`[^`]+`")
            .map(|re| re.replace_all(&result, "").to_string())
            .unwrap_or(result);

        // Remove HTML tags if any
        result = regex::Regex::new(r"<[^>]+>")
            .map(|re| re.replace_all(&result, "").to_string())
            .unwrap_or(result);

        // Clean up multiple consecutive blank lines
        result = regex::Regex::new(r"\n{3,}")
            .map(|re| re.replace_all(&result, "\n\n").to_string())
            .unwrap_or(result);

        // Clean up trailing whitespace on lines
        result = regex::Regex::new(r"[ \t]+$")
            .map(|re| re.replace_all(&result, "").to_string())
            .unwrap_or(result);

        result.trim().to_string()
    }

    async fn send_whatsapp_message(
        &self,
        to: &str,
        message: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Enqueue message instead of sending directly
        let queued_msg = QueuedWhatsAppMessage {
            to: to.to_string(),
            message: message.to_string(),
            api_key: self.api_key.clone(),
            phone_number_id: self.phone_number_id.clone(),
            api_version: self.api_version.clone(),
        };

        let queue = self.queue.ok_or_else(|| {
            error!("WhatsApp queue not available (was initialization failed?)");
            "WhatsApp queue not available"
        })?;

        queue.enqueue(queued_msg).await
            .map_err(|e| format!("Failed to enqueue WhatsApp message: {}", e))?;

        info!("WhatsApp message enqueued for {}: {}", to, &message.chars().take(50).collect::<String>());
        Ok("queued".to_string())
    }

    pub async fn send_template_message(
        &self,
        to: &str,
        template_name: &str,
        language_code: &str,
        components: Vec<serde_json::Value>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Enqueue template message
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

    pub async fn download_media(
        &self,
        media_id: &str,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();

        // 1. Get media URL
        let url = format!(
            "https://graph.facebook.com/{}/{}",
            self.api_version, media_id
        );

        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Failed to get media URL: {}", error_text).into());
        }

        let media_info: serde_json::Value = response.json().await?;
        let download_url = media_info["url"]
            .as_str()
            .ok_or("Media URL not found in response")?;

        // 2. Download the binary
        let download_response = client
            .get(download_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("User-Agent", "Mozilla/5.0") // Meta requires a User-Agent sometimes
            .send()
            .await?;

        if !download_response.status().is_success() {
            let error_text = download_response.text().await?;
            return Err(format!("Failed to download media binary: {}", error_text).into());
        }

        let binary_data = download_response.bytes().await?;
        Ok(binary_data.to_vec())
    }

    pub async fn send_voice_message(
        &self,
        to: &str,
        audio_url: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();

        info!("Sending voice message to {} from URL: {}", to, audio_url);

        let audio_data = if audio_url.starts_with("http") {
            let response = client.get(audio_url).send().await?;
            if !response.status().is_success() {
                return Err(format!("Failed to download audio from {}: {:?}", audio_url, response.status()).into());
            }
            response.bytes().await?.to_vec()
        } else {
            tokio::fs::read(audio_url).await?
        };

        let temp_path = format!("/tmp/whatsapp_voice_{}.mp3", uuid::Uuid::new_v4());
        tokio::fs::write(&temp_path, &audio_data).await?;

        let media_id = match self.upload_media(&temp_path, "audio/mpeg").await {
            Ok(id) => id,
            Err(e) => {
                let _ = tokio::fs::remove_file(&temp_path).await;
                return Err(format!("Failed to upload voice: {}", e).into());
            }
        };

        let _ = tokio::fs::remove_file(&temp_path).await;

        let url = format!(
            "https://graph.facebook.com/{}/{}/messages",
            self.api_version, self.phone_number_id
        );

        let payload = serde_json::json!({
            "messaging_product": "whatsapp",
            "to": to,
            "type": "audio",
            "audio": {
                "id": media_id
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
            info!("Voice message sent successfully: {:?}", result);
            Ok(result["messages"][0]["id"].as_str().unwrap_or("").to_string())
        } else {
            let error_text = response.text().await?;
            Err(format!("WhatsApp voice API error: {}", error_text).into())
        }
    }

    /// Smart message splitting for WhatsApp's character limit.
    /// Splits at paragraph boundaries, keeping lists together.
    /// Groups up to 3 paragraphs per message when possible.
    pub fn split_message_smart(&self, content: &str, max_length: usize) -> Vec<String> {
        let mut parts = Vec::new();
        let mut current_part = String::new();
        let mut paragraph_count = 0;

        // Split content into blocks (paragraphs or list items)
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i];
            let is_list_item = line.trim().starts_with("- ")
                || line.trim().starts_with("* ")
                || line.trim().starts_with("• ")
                || line.trim().starts_with(|c: char| c.is_numeric());

            // Check if this is the start of a list
            if is_list_item {
                // Flush current part if it has content and adding list would exceed limit
                if !current_part.is_empty() {
                    // If we have 3+ paragraphs, flush
                    if paragraph_count >= 3 || current_part.len() + line.len() > max_length {
                        parts.push(current_part.trim().to_string());
                        current_part = String::new();
                        paragraph_count = 0;
                    }
                }

                // Collect entire list as one block
                let mut list_block = String::new();
                while i < lines.len() {
                    let list_line = lines[i];
                    let is_still_list = list_line.trim().starts_with("- ")
                        || list_line.trim().starts_with("* ")
                        || list_line.trim().starts_with("• ")
                        || list_line.trim().starts_with(|c: char| c.is_numeric())
                        || (list_line.trim().is_empty() && i + 1 < lines.len() && {
                            let next = lines[i + 1];
                            next.trim().starts_with("- ")
                                || next.trim().starts_with("* ")
                                || next.trim().starts_with("• ")
                        });

                    if is_still_list || (list_line.trim().is_empty() && !list_block.is_empty()) {
                        if list_block.len() + list_line.len() + 1 > max_length {
                            // List is too long, split it
                            if !list_block.is_empty() {
                                if !current_part.is_empty() {
                                    parts.push(current_part.trim().to_string());
                                    current_part = String::new();
                                }
                                parts.push(list_block.trim().to_string());
                                list_block = String::new();
                            }
                        }
                        if !list_line.trim().is_empty() {
                            if !list_block.is_empty() {
                                list_block.push('\n');
                            }
                            list_block.push_str(list_line);
                        }
                        i += 1;
                    } else {
                        break;
                    }
                }

                if !list_block.is_empty() {
                    if !current_part.is_empty() && current_part.len() + list_block.len() < max_length {
                        current_part.push('\n');
                        current_part.push_str(&list_block);
                    } else {
                        if !current_part.is_empty() {
                            parts.push(current_part.trim().to_string());
                        }
                        parts.push(list_block.trim().to_string());
                        current_part = String::new();
                        paragraph_count = 0;
                    }
                }
                continue;
            }

            // Regular paragraph
            if !line.trim().is_empty() {
                if !current_part.is_empty() {
                    current_part.push('\n');
                }
                current_part.push_str(line);
                paragraph_count += 1;

                // Flush if we have 3 paragraphs or exceeded max length
                if paragraph_count >= 3 || current_part.len() > max_length {
                    parts.push(current_part.trim().to_string());
                    current_part = String::new();
                    paragraph_count = 0;
                }
            } else if !current_part.is_empty() {
                // Empty line marks paragraph end
                paragraph_count += 1;
                if paragraph_count >= 3 {
                    parts.push(current_part.trim().to_string());
                    current_part = String::new();
                    paragraph_count = 0;
                }
            }

            i += 1;
        }

        // Don't forget the last part
        if !current_part.trim().is_empty() {
            parts.push(current_part.trim().to_string());
        }

        // Handle edge case: if a single part exceeds max_length, force split
        let mut final_parts = Vec::new();
        for part in parts {
            if part.len() <= max_length {
                final_parts.push(part);
            } else {
                // Hard split at max_length, trying to break at word boundary
                let mut remaining = part.as_str();
                while !remaining.is_empty() {
                    if remaining.len() <= max_length {
                        final_parts.push(remaining.to_string());
                        break;
                    }
                    // Find last space before max_length
                    let split_pos = remaining[..max_length]
                        .rfind(' ')
                        .unwrap_or(max_length);
                    final_parts.push(remaining[..split_pos].to_string());
                    remaining = remaining[split_pos..].trim();
                }
            }
        }

        if final_parts.is_empty() {
            final_parts.push(content.to_string());
        }

        final_parts
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

        // WhatsApp has a 4096 character limit per message
        // Split message at paragraph/list boundaries
        const MAX_WHATSAPP_LENGTH: usize = 4000; // Leave some buffer

        // Sanitize Markdown for WhatsApp compatibility
        let sanitized_content = Self::sanitize_for_whatsapp(&response.content);

        if sanitized_content.len() <= MAX_WHATSAPP_LENGTH {
            // Message fits in one part
            let message_id = self
                .send_whatsapp_message(&response.user_id, &sanitized_content)
                .await?;

            info!(
                "WhatsApp message sent to {}: {} (message_id: {})",
                response.user_id, &sanitized_content.chars().take(100).collect::<String>(), message_id
            );
        } else {
            // Split message at appropriate boundaries
            let parts = self.split_message_smart(&sanitized_content, MAX_WHATSAPP_LENGTH);

            for (i, part) in parts.iter().enumerate() {
                let message_id = self
                    .send_whatsapp_message(&response.user_id, part)
                    .await?;

                info!(
                    "WhatsApp message part {}/{} sent to {}: {} (message_id: {})",
                    i + 1, parts.len(), response.user_id, &part.chars().take(50).collect::<String>(), message_id
                );
                // Rate limiting is now handled inside send_whatsapp_message (per-recipient)
            }
        }

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
