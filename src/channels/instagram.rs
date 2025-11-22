//! Instagram Messaging Channel Integration
//!
//! This module provides webhook handling and message processing for Instagram Direct Messages.
//! Currently under development for bot integration with Instagram Business accounts.
//!
//! Key features:
//! - Webhook verification and message handling
//! - Instagram Direct Message support
//! - Media attachments (images, videos)
//! - Quick replies
//! - Session management per Instagram user

use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use axum::{extract::Query, http::StatusCode, response::Json, Router};
use log::{error, info};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct InstagramWebhook {
    #[serde(rename = "hub.mode")]
    pub hub_mode: Option<String>,
    #[serde(rename = "hub.verify_token")]
    pub hub_verify_token: Option<String>,
    #[serde(rename = "hub.challenge")]
    pub hub_challenge: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InstagramMessage {
    pub entry: Vec<InstagramEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InstagramEntry {
    pub id: String,
    pub time: i64,
    pub messaging: Vec<InstagramMessaging>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InstagramMessaging {
    pub sender: InstagramUser,
    pub recipient: InstagramUser,
    pub timestamp: i64,
    pub message: Option<InstagramMessageContent>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InstagramUser {
    pub id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InstagramMessageContent {
    pub mid: String,
    pub text: Option<String>,
    pub attachments: Option<Vec<InstagramAttachment>>,
    pub quick_reply: Option<InstagramQuickReply>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InstagramAttachment {
    #[serde(rename = "type")]
    pub attachment_type: String,
    pub payload: InstagramAttachmentPayload,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InstagramAttachmentPayload {
    pub url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InstagramQuickReply {
    pub payload: String,
}

pub struct InstagramAdapter {
    pub state: Arc<AppState>,
    pub access_token: String,
    pub verify_token: String,
    pub page_id: String,
}

impl InstagramAdapter {
    pub fn new(state: Arc<AppState>) -> Self {
        // TODO: Load from config file or environment variables
        let access_token = std::env::var("INSTAGRAM_ACCESS_TOKEN").unwrap_or_default();
        let verify_token = std::env::var("INSTAGRAM_VERIFY_TOKEN")
            .unwrap_or_else(|_| "webhook_verify".to_string());
        let page_id = std::env::var("INSTAGRAM_PAGE_ID").unwrap_or_default();

        Self {
            state,
            access_token,
            verify_token,
            page_id,
        }
    }

    pub async fn handle_webhook_verification(
        &self,
        params: Query<InstagramWebhook>,
    ) -> Result<String, StatusCode> {
        if let (Some(mode), Some(token), Some(challenge)) = (
            &params.hub_mode,
            &params.hub_verify_token,
            &params.hub_challenge,
        ) {
            if mode == "subscribe" && token == &self.verify_token {
                info!("Instagram webhook verified successfully");
                return Ok(challenge.clone());
            }
        }

        error!("Instagram webhook verification failed");
        Err(StatusCode::FORBIDDEN)
    }

    pub async fn handle_incoming_message(
        &self,
        Json(payload): Json<InstagramMessage>,
    ) -> Result<StatusCode, StatusCode> {
        for entry in payload.entry {
            for messaging in entry.messaging {
                if let Some(message) = messaging.message {
                    if let Err(e) = self.process_message(messaging.sender.id, message).await {
                        error!("Error processing Instagram message: {}", e);
                    }
                }
            }
        }

        Ok(StatusCode::OK)
    }

    async fn process_message(
        &self,
        sender_id: String,
        message: InstagramMessageContent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Extract message content
        let content = if let Some(text) = message.text {
            text
        } else if let Some(attachments) = message.attachments {
            if !attachments.is_empty() {
                format!("[Attachment: {}]", attachments[0].attachment_type)
            } else {
                return Ok(());
            }
        } else {
            return Ok(());
        };

        // Process with bot
        self.process_with_bot(&sender_id, &content).await?;

        Ok(())
    }

    async fn process_with_bot(
        &self,
        sender_id: &str,
        message: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let session = self.get_or_create_session(sender_id).await?;

        // Process message through bot processor (simplified for now)
        let response = format!(
            "Received on Instagram (session {}): {}",
            session.id, message
        );
        self.send_message(sender_id, &response).await?;

        Ok(())
    }

    async fn get_or_create_session(
        &self,
        user_id: &str,
    ) -> Result<UserSession, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(redis_client) = &self.state.cache {
            let mut conn = redis_client.get_multiplexed_async_connection().await?;
            let session_key = format!("instagram_session:{}", user_id);

            if let Ok(session_data) = redis::cmd("GET")
                .arg(&session_key)
                .query_async::<String>(&mut conn)
                .await
            {
                if let Ok(session) = serde_json::from_str::<UserSession>(&session_data) {
                    return Ok(session);
                }
            }

            let user_uuid = uuid::Uuid::parse_str(user_id).unwrap_or_else(|_| uuid::Uuid::new_v4());
            let session = UserSession {
                id: uuid::Uuid::new_v4(),
                user_id: user_uuid,
                bot_id: uuid::Uuid::default(),
                title: "Instagram Session".to_string(),
                context_data: serde_json::json!({"channel": "instagram"}),
                current_tool: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };

            let session_data = serde_json::to_string(&session)?;
            redis::cmd("SET")
                .arg(&session_key)
                .arg(&session_data)
                .arg("EX")
                .arg(86400)
                .query_async::<()>(&mut conn)
                .await?;

            Ok(session)
        } else {
            let user_uuid = uuid::Uuid::parse_str(user_id).unwrap_or_else(|_| uuid::Uuid::new_v4());
            Ok(UserSession {
                id: uuid::Uuid::new_v4(),
                user_id: user_uuid,
                bot_id: uuid::Uuid::default(),
                title: "Instagram Session".to_string(),
                context_data: serde_json::json!({"channel": "instagram"}),
                current_tool: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            })
        }
    }

    pub async fn send_message(
        &self,
        recipient_id: &str,
        message: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("https://graph.facebook.com/v17.0/{}/messages", self.page_id);

        let payload = json!({
            "recipient": {
                "id": recipient_id
            },
            "message": {
                "text": message
            }
        });

        let client = Client::new();
        let response = client
            .post(&url)
            .query(&[("access_token", &self.access_token)])
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            error!("Instagram API error: {}", error_text);
            return Err(format!("Instagram API error: {}", error_text).into());
        }

        Ok(())
    }

    pub async fn send_quick_replies(
        &self,
        recipient_id: &str,
        title: &str,
        options: Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("https://graph.facebook.com/v17.0/{}/messages", self.page_id);

        let quick_replies: Vec<_> = options
            .iter()
            .take(13) // Instagram limits to 13 quick replies
            .map(|text| {
                json!({
                    "content_type": "text",
                    "title": text,
                    "payload": text
                })
            })
            .collect();

        let payload = json!({
            "recipient": {
                "id": recipient_id
            },
            "message": {
                "text": title,
                "quick_replies": quick_replies
            }
        });

        let client = Client::new();
        let response = client
            .post(&url)
            .query(&[("access_token", &self.access_token)])
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            error!("Instagram API error: {}", error_text);
        }

        Ok(())
    }
}

pub fn router(state: Arc<AppState>) -> Router<Arc<AppState>> {
    let adapter = Arc::new(InstagramAdapter::new(state.clone()));

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
