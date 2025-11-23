//! Microsoft Teams Channel Integration
//!
//! This module provides webhook handling and message processing for Microsoft Teams.
//! Currently under development for bot integration with Teams channels and direct messages.
//!
//! Key features:
//! - Bot Framework webhook handling
//! - Teams message and conversation support
//! - Adaptive cards for rich responses
//! - Session management per Teams user
//! - Integration with Microsoft Bot Framework

use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use axum::{http::StatusCode, response::Json, Router};
use log::error;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize)]
pub struct TeamsMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub id: Option<String>,
    pub timestamp: Option<String>,
    pub from: TeamsUser,
    pub conversation: TeamsConversation,
    pub recipient: TeamsUser,
    pub text: Option<String>,
    pub attachments: Option<Vec<TeamsAttachment>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TeamsUser {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TeamsConversation {
    pub id: String,
    #[serde(rename = "conversationType")]
    pub conversation_type: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TeamsAttachment {
    #[serde(rename = "contentType")]
    pub content_type: String,
    pub content: serde_json::Value,
}

#[derive(Debug)]
pub struct TeamsAdapter {
    pub state: Arc<AppState>,
    pub app_id: String,
    pub app_password: String,
    pub service_url: String,
    pub tenant_id: String,
}

impl TeamsAdapter {
    pub fn new(state: Arc<AppState>) -> Self {
        // Load configuration from environment variables
        let app_id = std::env::var("TEAMS_APP_ID").unwrap_or_default();

        let app_password = std::env::var("TEAMS_APP_PASSWORD").unwrap_or_default();

        let service_url = std::env::var("TEAMS_SERVICE_URL")
            .unwrap_or_else(|_| "https://smba.trafficmanager.net/br/".to_string());

        let tenant_id = std::env::var("TEAMS_TENANT_ID").unwrap_or_default();

        Self {
            state,
            app_id,
            app_password,
            service_url,
            tenant_id,
        }
    }

    pub async fn handle_incoming_message(
        &self,
        Json(payload): Json<TeamsMessage>,
    ) -> Result<StatusCode, StatusCode> {
        if payload.msg_type != "message" {
            return Ok(StatusCode::OK);
        }

        if let Some(text) = payload.text {
            if let Err(e) = self
                .process_message(payload.from, payload.conversation, text)
                .await
            {
                error!("Error processing Teams message: {}", e);
            }
        }

        Ok(StatusCode::ACCEPTED)
    }

    async fn process_message(
        &self,
        from: TeamsUser,
        conversation: TeamsConversation,
        text: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Process with bot
        self.process_with_bot(&from.id, &conversation.id, &text)
            .await?;

        Ok(())
    }

    async fn process_with_bot(
        &self,
        user_id: &str,
        conversation_id: &str,
        message: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let _session = self.get_or_create_session(user_id).await?;

        // Process message through bot processor (simplified for now)
        let response = format!("Received on Teams: {}", message);
        self.send_message(conversation_id, user_id, &response)
            .await?;

        Ok(())
    }

    async fn get_or_create_session(
        &self,
        user_id: &str,
    ) -> Result<UserSession, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(redis_client) = &self.state.cache {
            let mut conn = redis_client.get_multiplexed_async_connection().await?;
            let session_key = format!("teams_session:{}", user_id);

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
                title: "Teams Session".to_string(),
                context_data: serde_json::json!({"channel": "teams"}),
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
                title: "Teams Session".to_string(),
                context_data: serde_json::json!({"channel": "teams"}),
                current_tool: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            })
        }
    }

    pub async fn get_access_token(
        &self,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let client = Client::new();
        let token_url = format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
            if self.tenant_id.is_empty() {
                "botframework.com"
            } else {
                &self.tenant_id
            }
        );

        let params = [
            ("grant_type", "client_credentials"),
            ("client_id", &self.app_id),
            ("client_secret", &self.app_password),
            ("scope", "https://api.botframework.com/.default"),
        ];

        let response = client.post(&token_url).form(&params).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Failed to get Teams access token: {}", error_text).into());
        }

        #[derive(Deserialize)]
        struct TokenResponse {
            access_token: String,
        }

        let token_response: TokenResponse = response.json().await?;
        Ok(token_response.access_token)
    }

    pub async fn send_message(
        &self,
        conversation_id: &str,
        user_id: &str,
        message: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let access_token = self.get_access_token().await?;
        let url = format!(
            "{}/v3/conversations/{}/activities",
            self.service_url.trim_end_matches('/'),
            conversation_id
        );

        let activity = json!({
            "type": "message",
            "text": message,
            "from": {
                "id": self.app_id,
                "name": "Bot"
            },
            "conversation": {
                "id": conversation_id
            },
            "recipient": {
                "id": user_id
            }
        });

        let client = Client::new();
        let response = client
            .post(&url)
            .bearer_auth(&access_token)
            .json(&activity)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            error!("Teams API error: {}", error_text);
            return Err(format!("Teams API error: {}", error_text).into());
        }

        Ok(())
    }

    pub async fn send_card(
        &self,
        conversation_id: &str,
        user_id: &str,
        title: &str,
        options: Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let access_token = self.get_access_token().await?;
        let url = format!(
            "{}/v3/conversations/{}/activities",
            self.service_url.trim_end_matches('/'),
            conversation_id
        );

        let actions: Vec<_> = options
            .iter()
            .map(|option| {
                json!({
                    "type": "Action.Submit",
                    "title": option,
                    "data": {
                        "action": option
                    }
                })
            })
            .collect();

        let card = json!({
            "type": "AdaptiveCard",
            "version": "1.3",
            "body": [
                {
                    "type": "TextBlock",
                    "text": title,
                    "size": "Medium",
                    "weight": "Bolder"
                }
            ],
            "actions": actions
        });

        let activity = json!({
            "type": "message",
            "from": {
                "id": self.app_id,
                "name": "Bot"
            },
            "conversation": {
                "id": conversation_id
            },
            "recipient": {
                "id": user_id
            },
            "attachments": [
                {
                    "contentType": "application/vnd.microsoft.card.adaptive",
                    "content": card
                }
            ]
        });

        let client = Client::new();
        let response = client
            .post(&url)
            .bearer_auth(&access_token)
            .json(&activity)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            error!("Teams API error: {}", error_text);
        }

        Ok(())
    }
}

pub fn router(state: Arc<AppState>) -> Router<Arc<AppState>> {
    let adapter = Arc::new(TeamsAdapter::new(state.clone()));

    Router::new()
        .route(
            "/messages",
            axum::routing::post({
                move |payload| async move { adapter.handle_incoming_message(payload).await }
            }),
        )
        .with_state(state)
}
