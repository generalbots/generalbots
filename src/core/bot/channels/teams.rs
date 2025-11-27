use async_trait::async_trait;
use log::{error, info};
use serde::{Deserialize, Serialize};
// use std::collections::HashMap; // Unused import

use crate::core::bot::channels::ChannelAdapter;
use crate::shared::models::BotResponse;

pub struct TeamsAdapter {
    app_id: String,
    app_password: String,
    tenant_id: String,
    service_url: String,
    bot_id: String,
}

impl TeamsAdapter {
    pub fn new() -> Self {
        // Load from environment variables (would be from config.csv in production)
        let app_id = std::env::var("TEAMS_APP_ID").unwrap_or_default();
        let app_password = std::env::var("TEAMS_APP_PASSWORD").unwrap_or_default();
        let tenant_id = std::env::var("TEAMS_TENANT_ID").unwrap_or_default();
        let service_url = std::env::var("TEAMS_SERVICE_URL")
            .unwrap_or_else(|_| "https://smba.trafficmanager.net".to_string());
        let bot_id = std::env::var("TEAMS_BOT_ID").unwrap_or_else(|_| app_id.clone());

        Self {
            app_id,
            app_password,
            tenant_id,
            service_url,
            bot_id,
        }
    }

    async fn get_access_token(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();

        let token_url = format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
            if self.tenant_id.is_empty() {
                "common"
            } else {
                &self.tenant_id
            }
        );

        let params = [
            ("client_id", &self.app_id),
            ("client_secret", &self.app_password),
            ("grant_type", &"client_credentials".to_string()),
            (
                "scope",
                &"https://api.botframework.com/.default".to_string(),
            ),
        ];

        let response = client.post(&token_url).form(&params).send().await?;

        if response.status().is_success() {
            let token_response: serde_json::Value = response.json().await?;
            Ok(token_response["access_token"]
                .as_str()
                .unwrap_or("")
                .to_string())
        } else {
            let error_text = response.text().await?;
            Err(format!("Failed to get Teams access token: {}", error_text).into())
        }
    }

    async fn send_teams_message(
        &self,
        conversation_id: &str,
        activity_id: Option<&str>,
        message: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let token = self.get_access_token().await?;
        let client = reqwest::Client::new();

        let url = if let Some(reply_to_id) = activity_id {
            format!(
                "{}/v3/conversations/{}/activities/{}/reply",
                self.service_url, conversation_id, reply_to_id
            )
        } else {
            format!(
                "{}/v3/conversations/{}/activities",
                self.service_url, conversation_id
            )
        };

        let activity = TeamsActivity {
            activity_type: "message".to_string(),
            text: message.to_string(),
            from: TeamsChannelAccount {
                id: self.bot_id.clone(),
                name: Some("Bot".to_string()),
            },
            conversation: TeamsConversationAccount {
                id: conversation_id.to_string(),
                conversation_type: None,
                tenant_id: Some(self.tenant_id.clone()),
            },
            recipient: None,
            attachments: None,
            entities: None,
        };

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&activity)
            .send()
            .await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            Ok(result["id"].as_str().unwrap_or("").to_string())
        } else {
            let error_text = response.text().await?;
            Err(format!("Teams API error: {}", error_text).into())
        }
    }

    pub async fn send_card(
        &self,
        conversation_id: &str,
        card: TeamsAdaptiveCard,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let token = self.get_access_token().await?;
        let client = reqwest::Client::new();

        let url = format!(
            "{}/v3/conversations/{}/activities",
            self.service_url, conversation_id
        );

        let attachment = TeamsAttachment {
            content_type: "application/vnd.microsoft.card.adaptive".to_string(),
            content: serde_json::to_value(card)?,
        };

        let activity = TeamsActivity {
            activity_type: "message".to_string(),
            text: String::new(),
            from: TeamsChannelAccount {
                id: self.bot_id.clone(),
                name: Some("Bot".to_string()),
            },
            conversation: TeamsConversationAccount {
                id: conversation_id.to_string(),
                conversation_type: None,
                tenant_id: Some(self.tenant_id.clone()),
            },
            recipient: None,
            attachments: Some(vec![attachment]),
            entities: None,
        };

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&activity)
            .send()
            .await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            Ok(result["id"].as_str().unwrap_or("").to_string())
        } else {
            let error_text = response.text().await?;
            Err(format!("Teams API error: {}", error_text).into())
        }
    }

    pub async fn update_message(
        &self,
        conversation_id: &str,
        activity_id: &str,
        new_message: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let token = self.get_access_token().await?;
        let client = reqwest::Client::new();

        let url = format!(
            "{}/v3/conversations/{}/activities/{}",
            self.service_url, conversation_id, activity_id
        );

        let activity = TeamsActivity {
            activity_type: "message".to_string(),
            text: new_message.to_string(),
            from: TeamsChannelAccount {
                id: self.bot_id.clone(),
                name: Some("Bot".to_string()),
            },
            conversation: TeamsConversationAccount {
                id: conversation_id.to_string(),
                conversation_type: None,
                tenant_id: Some(self.tenant_id.clone()),
            },
            recipient: None,
            attachments: None,
            entities: None,
        };

        let response = client
            .put(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&activity)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Teams API error: {}", error_text).into());
        }

        Ok(())
    }
}

#[async_trait]
impl ChannelAdapter for TeamsAdapter {
    fn name(&self) -> &str {
        "Teams"
    }

    fn is_configured(&self) -> bool {
        !self.app_id.is_empty() && !self.app_password.is_empty()
    }

    async fn send_message(
        &self,
        response: BotResponse,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.is_configured() {
            error!("Teams adapter not configured. Please set teams-app-id and teams-app-password in config.csv");
            return Err("Teams not configured".into());
        }

        // In Teams, user_id is typically the conversation ID
        let message_id = self
            .send_teams_message(&response.user_id, None, &response.content)
            .await?;

        info!(
            "Teams message sent to conversation {}: {} (message_id: {})",
            response.user_id, response.content, message_id
        );

        Ok(())
    }

    async fn receive_message(
        &self,
        payload: serde_json::Value,
    ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        // Parse Teams activity payload
        if let Some(activity_type) = payload["type"].as_str() {
            match activity_type {
                "message" => {
                    return Ok(payload["text"].as_str().map(|s| s.to_string()));
                }
                "invoke" => {
                    // Handle Teams-specific invokes (like adaptive card actions)
                    if let Some(name) = payload["name"].as_str() {
                        return Ok(Some(format!("Teams invoke: {}", name)));
                    }
                }
                _ => {
                    return Ok(None);
                }
            }
        }

        Ok(None)
    }

    async fn get_user_info(
        &self,
        user_id: &str,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let token = self.get_access_token().await?;
        let client = reqwest::Client::new();

        // In Teams, user_id might be in format "29:1xyz..."
        let url = format!("{}/v3/conversations/{}/members", self.service_url, user_id);

        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        if response.status().is_success() {
            let members: Vec<serde_json::Value> = response.json().await?;
            if let Some(first_member) = members.first() {
                return Ok(first_member.clone());
            }
        }

        Ok(serde_json::json!({
            "id": user_id,
            "platform": "teams"
        }))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TeamsActivity {
    #[serde(rename = "type")]
    pub activity_type: String,
    pub text: String,
    pub from: TeamsChannelAccount,
    pub conversation: TeamsConversationAccount,
    pub recipient: Option<TeamsChannelAccount>,
    pub attachments: Option<Vec<TeamsAttachment>>,
    pub entities: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TeamsChannelAccount {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TeamsConversationAccount {
    pub id: String,
    #[serde(rename = "conversationType")]
    pub conversation_type: Option<String>,
    #[serde(rename = "tenantId")]
    pub tenant_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TeamsAttachment {
    #[serde(rename = "contentType")]
    pub content_type: String,
    pub content: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TeamsAdaptiveCard {
    #[serde(rename = "$schema")]
    pub schema: String,
    #[serde(rename = "type")]
    pub card_type: String,
    pub version: String,
    pub body: Vec<serde_json::Value>,
    pub actions: Option<Vec<serde_json::Value>>,
}

impl Default for TeamsAdaptiveCard {
    fn default() -> Self {
        Self {
            schema: "http://adaptivecards.io/schemas/adaptive-card.json".to_string(),
            card_type: "AdaptiveCard".to_string(),
            version: "1.4".to_string(),
            body: Vec::new(),
            actions: None,
        }
    }
}
