//! WhatsApp Business Channel Integration
//!
//! This module provides webhook handling and message processing for WhatsApp Business API.
//! Currently under development for bot integration with WhatsApp Business accounts.
//!
//! Key features:
//! - Webhook verification and message handling
//! - WhatsApp text, media, and location messages
//! - Session management per WhatsApp user
//! - Media attachments support
//! - Integration with Meta's WhatsApp Business API

use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use axum::{extract::Query, http::StatusCode, response::Json, Router};
use log::{error, info};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct WhatsAppWebhook {
    #[serde(rename = "hub.mode")]
    pub hub_mode: Option<String>,
    #[serde(rename = "hub.verify_token")]
    pub hub_verify_token: Option<String>,
    #[serde(rename = "hub.challenge")]
    pub hub_challenge: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WhatsAppMessage {
    pub entry: Vec<WhatsAppEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WhatsAppEntry {
    pub id: String,
    pub changes: Vec<WhatsAppChange>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WhatsAppChange {
    pub value: WhatsAppValue,
    pub field: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WhatsAppValue {
    pub messaging_product: String,
    pub metadata: WhatsAppMetadata,
    pub contacts: Option<Vec<WhatsAppContact>>,
    pub messages: Option<Vec<WhatsAppIncomingMessage>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WhatsAppMetadata {
    pub display_phone_number: String,
    pub phone_number_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WhatsAppContact {
    pub profile: WhatsAppProfile,
    pub wa_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WhatsAppProfile {
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WhatsAppIncomingMessage {
    pub from: String,
    pub id: String,
    pub timestamp: String,
    #[serde(rename = "type")]
    pub msg_type: String,
    pub text: Option<WhatsAppText>,
    pub image: Option<WhatsAppMedia>,
    pub document: Option<WhatsAppMedia>,
    pub audio: Option<WhatsAppMedia>,
    pub video: Option<WhatsAppMedia>,
    pub location: Option<WhatsAppLocation>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WhatsAppText {
    pub body: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WhatsAppMedia {
    pub id: String,
    pub mime_type: Option<String>,
    pub sha256: Option<String>,
    pub caption: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WhatsAppLocation {
    pub latitude: f64,
    pub longitude: f64,
    pub name: Option<String>,
    pub address: Option<String>,
}

pub struct WhatsAppAdapter {
    pub state: Arc<AppState>,
    pub access_token: String,
    pub phone_number_id: String,
    pub verify_token: String,
}

impl WhatsAppAdapter {
    pub fn new(state: Arc<AppState>) -> Self {
        // Load configuration from environment variables
        let access_token = std::env::var("WHATSAPP_ACCESS_TOKEN").unwrap_or_default();

        let phone_number_id = std::env::var("WHATSAPP_PHONE_ID").unwrap_or_default();

        let verify_token =
            std::env::var("WHATSAPP_VERIFY_TOKEN").unwrap_or_else(|_| "webhook_verify".to_string());

        Self {
            state,
            access_token,
            phone_number_id,
            verify_token,
        }
    }

    pub async fn handle_webhook_verification(
        &self,
        params: Query<WhatsAppWebhook>,
    ) -> Result<String, StatusCode> {
        if let (Some(mode), Some(token), Some(challenge)) = (
            &params.hub_mode,
            &params.hub_verify_token,
            &params.hub_challenge,
        ) {
            if mode == "subscribe" && token == &self.verify_token {
                info!("WhatsApp webhook verified successfully");
                return Ok(challenge.clone());
            }
        }

        error!("WhatsApp webhook verification failed");
        Err(StatusCode::FORBIDDEN)
    }

    pub async fn handle_incoming_message(
        &self,
        Json(payload): Json<WhatsAppMessage>,
    ) -> Result<StatusCode, StatusCode> {
        for entry in payload.entry {
            for change in entry.changes {
                if change.field == "messages" {
                    if let Some(messages) = change.value.messages {
                        for message in messages {
                            if let Err(e) = self.process_message(message).await {
                                error!("Error processing WhatsApp message: {}", e);
                            }
                        }
                    }
                }
            }
        }

        Ok(StatusCode::OK)
    }

    async fn process_message(
        &self,
        message: WhatsAppIncomingMessage,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let user_phone = message.from.clone();
        let message_id = message.id.clone();

        // Mark message as read
        self.mark_as_read(&message_id).await?;

        // Extract message content based on type
        let content = match message.msg_type.as_str() {
            "text" => message.text.map(|t| t.body).unwrap_or_default(),
            "image" => {
                if let Some(image) = message.image {
                    format!("[Image: {}]", image.caption.unwrap_or_default())
                } else {
                    String::new()
                }
            }
            "audio" => "[Audio message]".to_string(),
            "video" => "[Video message]".to_string(),
            "document" => "[Document]".to_string(),
            "location" => {
                if let Some(loc) = message.location {
                    format!("[Location: {}, {}]", loc.latitude, loc.longitude)
                } else {
                    String::new()
                }
            }
            _ => String::new(),
        };

        if content.is_empty() {
            return Ok(());
        }

        // Process with bot
        self.process_with_bot(&user_phone, &content).await?;

        Ok(())
    }

    async fn process_with_bot(
        &self,
        from_number: &str,
        message: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Create or get user session
        let session = self.get_or_create_session(from_number).await?;

        // Process message through bot processor (simplified for now)
        // In real implementation, this would call the bot processor

        // Send response back to WhatsApp
        let response = format!("Received (session {}): {}", session.id, message);
        self.send_message(from_number, &response).await?;

        Ok(())
    }

    async fn get_or_create_session(
        &self,
        phone_number: &str,
    ) -> Result<UserSession, Box<dyn std::error::Error + Send + Sync>> {
        // Check Redis for existing session
        if let Some(redis_client) = &self.state.cache {
            let mut conn = redis_client.get_multiplexed_async_connection().await?;
            let session_key = format!("whatsapp_session:{}", phone_number);

            if let Ok(session_data) = redis::cmd("GET")
                .arg(&session_key)
                .query_async::<String>(&mut conn)
                .await
            {
                if let Ok(session) = serde_json::from_str::<UserSession>(&session_data) {
                    return Ok(session);
                }
            }

            // Create new session
            let user_uuid =
                uuid::Uuid::parse_str(phone_number).unwrap_or_else(|_| uuid::Uuid::new_v4());
            let session = UserSession {
                id: uuid::Uuid::new_v4(),
                user_id: user_uuid,
                bot_id: uuid::Uuid::default(), // Default bot
                title: "WhatsApp Session".to_string(),
                context_data: serde_json::json!({"channel": "whatsapp"}),
                current_tool: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };

            // Store in Redis
            let session_data = serde_json::to_string(&session)?;
            redis::cmd("SET")
                .arg(&session_key)
                .arg(&session_data)
                .arg("EX")
                .arg(86400) // 24 hours
                .query_async::<()>(&mut conn)
                .await?;

            Ok(session)
        } else {
            // Create ephemeral session
            let user_uuid =
                uuid::Uuid::parse_str(phone_number).unwrap_or_else(|_| uuid::Uuid::new_v4());
            Ok(UserSession {
                id: uuid::Uuid::new_v4(),
                user_id: user_uuid,
                bot_id: uuid::Uuid::default(),
                title: "WhatsApp Session".to_string(),
                context_data: serde_json::json!({"channel": "whatsapp"}),
                current_tool: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            })
        }
    }

    pub async fn send_message(
        &self,
        to_number: &str,
        message: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "https://graph.facebook.com/v17.0/{}/messages",
            self.phone_number_id
        );

        let payload = json!({
            "messaging_product": "whatsapp",
            "to": to_number,
            "type": "text",
            "text": {
                "body": message
            }
        });

        let client = Client::new();
        let response = client
            .post(&url)
            .bearer_auth(&self.access_token)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            error!("WhatsApp API error: {}", error_text);
            return Err(format!("WhatsApp API error: {}", error_text).into());
        }

        Ok(())
    }

    pub async fn send_interactive_buttons(
        &self,
        to_number: &str,
        header: &str,
        buttons: Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!(
            "https://graph.facebook.com/v17.0/{}/messages",
            self.phone_number_id
        );

        let button_list: Vec<_> = buttons
            .iter()
            .take(3) // WhatsApp limits to 3 buttons
            .enumerate()
            .map(|(i, text)| {
                json!({
                    "type": "reply",
                    "reply": {
                        "id": format!("button_{}", i),
                        "title": text
                    }
                })
            })
            .collect();

        let payload = json!({
            "messaging_product": "whatsapp",
            "to": to_number,
            "type": "interactive",
            "interactive": {
                "type": "button",
                "header": {
                    "type": "text",
                    "text": header
                },
                "body": {
                    "text": "Escolha uma opção:"
                },
                "action": {
                    "buttons": button_list
                }
            }
        });

        let client = Client::new();
        let response = client
            .post(&url)
            .bearer_auth(&self.access_token)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            error!("WhatsApp API error: {}", error_text);
        }

        Ok(())
    }

    async fn mark_as_read(
        &self,
        message_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "https://graph.facebook.com/v17.0/{}/messages",
            self.phone_number_id
        );

        let payload = json!({
            "messaging_product": "whatsapp",
            "status": "read",
            "message_id": message_id
        });

        let client = Client::new();
        client
            .post(&url)
            .bearer_auth(&self.access_token)
            .json(&payload)
            .send()
            .await?;

        Ok(())
    }

    pub async fn get_access_token(&self) -> &str {
        &self.access_token
    }
}

pub fn router(state: Arc<AppState>) -> Router<Arc<AppState>> {
    let adapter = Arc::new(WhatsAppAdapter::new(state.clone()));

    Router::new()
        .route(
            "/webhook",
            axum::routing::get({
                let adapter = adapter.clone();
                move |params| async move { adapter.handle_webhook_verification(params).await }
            }),
        )
        .route(
            "/webhook",
            axum::routing::post({
                move |payload| async move { adapter.handle_incoming_message(payload).await }
            }),
        )
        .with_state(state)
}
