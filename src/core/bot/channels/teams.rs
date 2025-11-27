use async_trait::async_trait;
use log::{error, info};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::bot::channels::ChannelAdapter;
use crate::core::config::ConfigManager;
use crate::shared::models::BotResponse;
use crate::shared::utils::DbPool;

#[derive(Debug)]
pub struct TeamsAdapter {
    app_id: String,
    app_password: String,
    tenant_id: String,
    service_url: String,
    bot_id: String,
}

impl TeamsAdapter {
    pub fn new(pool: DbPool, bot_id: Uuid) -> Self {
        let config_manager = ConfigManager::new(pool);

        // Load from bot_configuration table with fallback to environment variables
        let app_id = config_manager
            .get_config(&bot_id, "teams-app-id", None)
            .unwrap_or_else(|_| std::env::var("TEAMS_APP_ID").unwrap_or_default());

        let app_password = config_manager
            .get_config(&bot_id, "teams-app-password", None)
            .unwrap_or_else(|_| std::env::var("TEAMS_APP_PASSWORD").unwrap_or_default());

        let tenant_id = config_manager
            .get_config(&bot_id, "teams-tenant-id", None)
            .unwrap_or_else(|_| std::env::var("TEAMS_TENANT_ID").unwrap_or_default());

        let service_url = config_manager
            .get_config(
                &bot_id,
                "teams-service-url",
                Some("https://smba.trafficmanager.net"),
            )
            .unwrap_or_else(|_| {
                std::env::var("TEAMS_SERVICE_URL")
                    .unwrap_or_else(|_| "https://smba.trafficmanager.net".to_string())
            });

        let teams_bot_id = config_manager
            .get_config(&bot_id, "teams-bot-id", None)
            .unwrap_or_else(|_| std::env::var("TEAMS_BOT_ID").unwrap_or_else(|_| app_id.clone()));

        Self {
            app_id,
            app_password,
            tenant_id,
            service_url,
            bot_id: teams_bot_id,
        }
    }

    async fn get_access_token(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();

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

        if response.status().is_success() {
            let token_response: serde_json::Value = response.json().await?;
            Ok(token_response["access_token"]
                .as_str()
                .unwrap_or("")
                .to_string())
        } else {
            let error_text = response.text().await?;
            Err(format!("Failed to get access token: {}", error_text).into())
        }
    }

    async fn send_teams_message(
        &self,
        conversation_id: &str,
        activity: TeamsActivity,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();

        let token = self.get_access_token().await?;

        let url = format!(
            "{}/v3/conversations/{}/activities",
            self.service_url, conversation_id
        );

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
        card: serde_json::Value,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let activity = TeamsActivity {
            activity_type: "message".to_string(),
            text: None,
            attachments: Some(vec![TeamsAttachment {
                content_type: "application/vnd.microsoft.card.adaptive".to_string(),
                content: card,
            }]),
            ..Default::default()
        };

        self.send_teams_message(conversation_id, activity).await
    }

    pub async fn send_hero_card(
        &self,
        conversation_id: &str,
        title: &str,
        subtitle: Option<&str>,
        text: Option<&str>,
        images: Vec<String>,
        buttons: Vec<TeamsCardAction>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let hero_card = TeamsHeroCard {
            title: Some(title.to_string()),
            subtitle: subtitle.map(|s| s.to_string()),
            text: text.map(|s| s.to_string()),
            images: images
                .into_iter()
                .map(|url| TeamsCardImage { url, alt: None })
                .collect(),
            buttons: if buttons.is_empty() {
                None
            } else {
                Some(buttons)
            },
        };

        let activity = TeamsActivity {
            activity_type: "message".to_string(),
            text: None,
            attachments: Some(vec![TeamsAttachment {
                content_type: "application/vnd.microsoft.card.hero".to_string(),
                content: serde_json::to_value(hero_card)?,
            }]),
            ..Default::default()
        };

        self.send_teams_message(conversation_id, activity).await
    }

    pub async fn create_conversation(
        &self,
        to: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();

        let token = self.get_access_token().await?;

        let url = format!("{}/v3/conversations", self.service_url);

        let payload = serde_json::json!({
            "bot": {
                "id": self.bot_id,
                "name": "Bot"
            },
            "members": [{
                "id": to
            }],
            "channelData": {
                "tenant": {
                    "id": self.tenant_id
                }
            }
        });

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            Ok(result["id"].as_str().unwrap_or("").to_string())
        } else {
            let error_text = response.text().await?;
            Err(format!("Failed to create conversation: {}", error_text).into())
        }
    }

    pub async fn update_message(
        &self,
        conversation_id: &str,
        activity_id: &str,
        new_text: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();

        let token = self.get_access_token().await?;

        let url = format!(
            "{}/v3/conversations/{}/activities/{}",
            self.service_url, conversation_id, activity_id
        );

        let activity = TeamsActivity {
            activity_type: "message".to_string(),
            text: Some(new_text.to_string()),
            ..Default::default()
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
            return Err(format!("Failed to update message: {}", error_text).into());
        }

        Ok(())
    }

    pub async fn delete_message(
        &self,
        conversation_id: &str,
        activity_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();

        let token = self.get_access_token().await?;

        let url = format!(
            "{}/v3/conversations/{}/activities/{}",
            self.service_url, conversation_id, activity_id
        );

        let response = client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Failed to delete message: {}", error_text).into());
        }

        Ok(())
    }

    pub async fn send_typing_indicator(
        &self,
        conversation_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let activity = TeamsActivity {
            activity_type: "typing".to_string(),
            ..Default::default()
        };

        self.send_teams_message(conversation_id, activity).await?;
        Ok(())
    }

    pub async fn get_conversation_members(
        &self,
        conversation_id: &str,
    ) -> Result<Vec<TeamsMember>, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();

        let token = self.get_access_token().await?;

        let url = format!(
            "{}/v3/conversations/{}/members",
            self.service_url, conversation_id
        );

        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        if response.status().is_success() {
            let members: Vec<TeamsMember> = response.json().await?;
            Ok(members)
        } else {
            let error_text = response.text().await?;
            Err(format!("Failed to get conversation members: {}", error_text).into())
        }
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

        // Try to use existing conversation or create a new one
        let conversation_id = self.create_conversation(&response.user_id).await?;

        let activity = TeamsActivity {
            activity_type: "message".to_string(),
            text: Some(response.content.clone()),
            ..Default::default()
        };

        let message_id = self.send_teams_message(&conversation_id, activity).await?;

        info!(
            "Teams message sent to conversation {}: {} (message_id: {})",
            conversation_id, response.content, message_id
        );

        Ok(())
    }

    async fn receive_message(
        &self,
        payload: serde_json::Value,
    ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        // Parse Teams activity
        let activity_type = payload["type"].as_str().unwrap_or("");

        match activity_type {
            "message" => {
                if let Some(text) = payload["text"].as_str() {
                    // Remove bot mention if present
                    let cleaned_text = text
                        .replace(&format!("<at>{}</at>", self.bot_id), "")
                        .trim()
                        .to_string();
                    Ok(Some(cleaned_text))
                } else if let Some(attachments) = payload["attachments"].as_array() {
                    if let Some(first_attachment) = attachments.first() {
                        let content_type = first_attachment["contentType"]
                            .as_str()
                            .unwrap_or("unknown");
                        Ok(Some(format!("Received attachment: {}", content_type)))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            "messageReaction" => {
                let reaction_type = payload["reactionsAdded"]
                    .as_array()
                    .and_then(|r| r.first())
                    .and_then(|r| r["type"].as_str())
                    .unwrap_or("unknown");
                Ok(Some(format!("Reaction: {}", reaction_type)))
            }
            _ => Ok(None),
        }
    }

    async fn get_user_info(
        &self,
        user_id: &str,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        // Teams user info would require Graph API access
        // For now, return basic info
        Ok(serde_json::json!({
            "id": user_id,
            "platform": "teams"
        }))
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TeamsActivity {
    #[serde(rename = "type")]
    pub activity_type: String,
    pub text: Option<String>,
    pub attachments: Option<Vec<TeamsAttachment>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_actions: Option<TeamsSuggestedActions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TeamsAttachment {
    #[serde(rename = "contentType")]
    pub content_type: String,
    pub content: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TeamsSuggestedActions {
    pub actions: Vec<TeamsCardAction>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TeamsCardAction {
    #[serde(rename = "type")]
    pub action_type: String,
    pub title: String,
    pub value: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TeamsHeroCard {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub text: Option<String>,
    pub images: Vec<TeamsCardImage>,
    pub buttons: Option<Vec<TeamsCardAction>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TeamsCardImage {
    pub url: String,
    pub alt: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TeamsMember {
    pub id: String,
    pub name: Option<String>,
    #[serde(rename = "userPrincipalName")]
    pub user_principal_name: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TeamsChannelAccount {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TeamsConversationAccount {
    pub id: String,
    pub name: Option<String>,
    #[serde(rename = "conversationType")]
    pub conversation_type: Option<String>,
    #[serde(rename = "isGroup")]
    pub is_group: Option<bool>,
}

// Helper functions for Teams-specific features

pub fn create_adaptive_card(
    title: &str,
    body: Vec<serde_json::Value>,
    actions: Vec<serde_json::Value>,
) -> serde_json::Value {
    let mut all_body_items = vec![serde_json::json!({
        "type": "TextBlock",
        "text": title,
        "weight": "Bolder",
        "size": "Medium"
    })];
    all_body_items.extend(body);

    serde_json::json!({
        "type": "AdaptiveCard",
        "$schema": "http://adaptivecards.io/schemas/adaptive-card.json",
        "version": "1.3",
        "body": all_body_items,
        "actions": actions
    })
}

pub fn create_thumbnail_card(
    title: &str,
    subtitle: Option<&str>,
    text: Option<&str>,
    image_url: Option<&str>,
    buttons: Vec<(&str, &str, &str)>,
) -> serde_json::Value {
    let mut card = serde_json::json!({
        "title": title
    });

    if let Some(sub) = subtitle {
        card["subtitle"] = serde_json::Value::String(sub.to_string());
    }
    if let Some(txt) = text {
        card["text"] = serde_json::Value::String(txt.to_string());
    }
    if let Some(img) = image_url {
        card["images"] = serde_json::json!([{
            "url": img
        }]);
    }

    let button_list: Vec<serde_json::Value> = buttons
        .into_iter()
        .map(|(action_type, title, value)| {
            serde_json::json!({
                "type": action_type,
                "title": title,
                "value": value
            })
        })
        .collect();

    if !button_list.is_empty() {
        card["buttons"] = serde_json::Value::Array(button_list);
    }

    card
}

pub fn create_message_with_mentions(
    text: &str,
    mentions: Vec<(&str, &str)>,
) -> (String, Vec<serde_json::Value>) {
    let mut message = text.to_string();
    let mention_entities: Vec<serde_json::Value> = mentions
        .into_iter()
        .map(|(user_id, display_name)| {
            let mention_text = format!("<at>{}</at>", display_name);
            message = message.replace(&format!("@{}", display_name), &mention_text);

            serde_json::json!({
                "type": "mention",
                "mentioned": {
                    "id": user_id,
                    "name": display_name
                },
                "text": mention_text
            })
        })
        .collect();

    (message, mention_entities)
}
