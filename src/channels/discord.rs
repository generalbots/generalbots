use crate::channels::{
    ChannelAccount, ChannelCredentials, ChannelError, ChannelProvider, ChannelType,
    PostContent, PostResult,
};
use serde::{Deserialize, Serialize};

pub struct DiscordProvider {
    client: reqwest::Client,
    base_url: String,
}

impl DiscordProvider {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "https://discord.com/api/v10".to_string(),
        }
    }

    async fn send_message(
        &self,
        bot_token: &str,
        channel_id: &str,
        content: &MessageContent,
    ) -> Result<DiscordMessage, ChannelError> {
        let response = self
            .client
            .post(format!("{}/channels/{}/messages", self.base_url, channel_id))
            .header("Authorization", format!("Bot {}", bot_token))
            .header("Content-Type", "application/json")
            .json(content)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        let status = response.status();

        if status.as_u16() == 429 {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok());
            return Err(ChannelError::RateLimited { retry_after });
        }

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ChannelError::ApiError {
                code: Some(status.to_string()),
                message: error_text,
            });
        }

        response
            .json::<DiscordMessage>()
            .await
            .map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })
    }

    async fn send_webhook_message(
        &self,
        webhook_url: &str,
        content: &WebhookContent,
    ) -> Result<DiscordMessage, ChannelError> {
        let url = format!("{}?wait=true", webhook_url);

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(content)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        let status = response.status();

        if status.as_u16() == 429 {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok());
            return Err(ChannelError::RateLimited { retry_after });
        }

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ChannelError::ApiError {
                code: Some(status.to_string()),
                message: error_text,
            });
        }

        response
            .json::<DiscordMessage>()
            .await
            .map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })
    }

    async fn get_current_user(&self, bot_token: &str) -> Result<DiscordUser, ChannelError> {
        let response = self
            .client
            .get(format!("{}/users/@me", self.base_url))
            .header("Authorization", format!("Bot {}", bot_token))
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ChannelError::AuthenticationFailed(error_text));
        }

        response
            .json::<DiscordUser>()
            .await
            .map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })
    }

    fn build_message_url(guild_id: &str, channel_id: &str, message_id: &str) -> String {
        format!(
            "https://discord.com/channels/{}/{}/{}",
            guild_id, channel_id, message_id
        )
    }

    fn create_embeds(content: &PostContent) -> Vec<DiscordEmbed> {
        let mut embeds = Vec::new();

        if content.link.is_some() || !content.image_urls.is_empty() {
            let mut embed = DiscordEmbed {
                title: None,
                description: None,
                url: content.link.clone(),
                color: Some(0x5865F2),
                image: None,
                thumbnail: None,
                fields: vec![],
                footer: None,
                timestamp: None,
            };

            if let Some(first_image) = content.image_urls.first() {
                embed.image = Some(EmbedImage {
                    url: first_image.clone(),
                });
            }

            embeds.push(embed);

            for image_url in content.image_urls.iter().skip(1).take(3) {
                embeds.push(DiscordEmbed {
                    title: None,
                    description: None,
                    url: None,
                    color: None,
                    image: Some(EmbedImage {
                        url: image_url.clone(),
                    }),
                    thumbnail: None,
                    fields: vec![],
                    footer: None,
                    timestamp: None,
                });
            }
        }

        embeds
    }
}

impl Default for DiscordProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl ChannelProvider for DiscordProvider {
    fn channel_type(&self) -> ChannelType {
        ChannelType::Discord
    }

    fn max_text_length(&self) -> usize {
        2000
    }

    fn supports_images(&self) -> bool {
        true
    }

    fn supports_video(&self) -> bool {
        true
    }

    fn supports_links(&self) -> bool {
        true
    }

    async fn post(
        &self,
        account: &ChannelAccount,
        content: &PostContent,
    ) -> Result<PostResult, ChannelError> {
        let text = content.text.as_deref().unwrap_or("");

        if text.len() > self.max_text_length() {
            return Err(ChannelError::ContentTooLong {
                max_length: self.max_text_length(),
                actual_length: text.len(),
            });
        }

        let mut full_text = text.to_string();

        if !content.hashtags.is_empty() {
            let tags = content
                .hashtags
                .iter()
                .map(|t| {
                    if t.starts_with('#') {
                        t.clone()
                    } else {
                        format!("#{}", t)
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
            full_text = format!("{}\n\n{}", full_text, tags);
        }

        match &account.credentials {
            ChannelCredentials::ApiKey { api_key, api_secret } => {
                let channel_id = api_secret
                    .as_ref()
                    .ok_or_else(|| ChannelError::AuthenticationFailed(
                        "Channel ID required in api_secret".to_string(),
                    ))?;

                let embeds = Self::create_embeds(content);

                let message_content = MessageContent {
                    content: if full_text.is_empty() { None } else { Some(full_text) },
                    embeds: if embeds.is_empty() { None } else { Some(embeds) },
                    tts: Some(false),
                };

                let message = self.send_message(api_key, channel_id, &message_content).await?;

                let guild_id = account
                    .settings
                    .custom
                    .get("guild_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("@me");

                let url = Self::build_message_url(guild_id, channel_id, &message.id);

                Ok(PostResult::success(
                    ChannelType::Discord,
                    message.id,
                    Some(url),
                ))
            }
            ChannelCredentials::Custom { data } => {
                let webhook_url = data
                    .get("webhook_url")
                    .ok_or_else(|| ChannelError::AuthenticationFailed(
                        "Webhook URL required".to_string(),
                    ))?;

                let username = data.get("username").cloned();
                let avatar_url = data.get("avatar_url").cloned();

                let embeds = Self::create_embeds(content);

                let webhook_content = WebhookContent {
                    content: if full_text.is_empty() { None } else { Some(full_text) },
                    username,
                    avatar_url,
                    embeds: if embeds.is_empty() { None } else { Some(embeds) },
                    tts: Some(false),
                };

                let message = self.send_webhook_message(webhook_url, &webhook_content).await?;

                Ok(PostResult::success(
                    ChannelType::Discord,
                    message.id,
                    None,
                ))
            }
            _ => Err(ChannelError::AuthenticationFailed(
                "Invalid credentials type for Discord".to_string(),
            )),
        }
    }

    async fn validate_credentials(
        &self,
        credentials: &ChannelCredentials,
    ) -> Result<bool, ChannelError> {
        match credentials {
            ChannelCredentials::ApiKey { api_key, .. } => {
                match self.get_current_user(api_key).await {
                    Ok(_) => Ok(true),
                    Err(ChannelError::AuthenticationFailed(_)) => Ok(false),
                    Err(e) => Err(e),
                }
            }
            ChannelCredentials::Custom { data } => {
                let webhook_url = data.get("webhook_url");
                Ok(webhook_url.is_some())
            }
            _ => Ok(false),
        }
    }

    async fn refresh_token(&self, _account: &mut ChannelAccount) -> Result<(), ChannelError> {
        Ok(())
    }
}

#[derive(Debug, Serialize)]
struct MessageContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    embeds: Option<Vec<DiscordEmbed>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tts: Option<bool>,
}

#[derive(Debug, Serialize)]
struct WebhookContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    embeds: Option<Vec<DiscordEmbed>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tts: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DiscordEmbed {
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<EmbedImage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    thumbnail: Option<EmbedImage>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    fields: Vec<EmbedField>,
    #[serde(skip_serializing_if = "Option::is_none")]
    footer: Option<EmbedFooter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct EmbedImage {
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct EmbedField {
    name: String,
    value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    inline: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct EmbedFooter {
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DiscordMessage {
    id: String,
    channel_id: String,
    #[serde(default)]
    content: String,
    timestamp: String,
    author: Option<DiscordUser>,
}

#[derive(Debug, Deserialize)]
struct DiscordUser {
    id: String,
    username: String,
    discriminator: String,
    #[serde(default)]
    bot: bool,
}

#[derive(Debug, Serialize)]
struct CreateDMChannel {
    recipient_id: String,
}

pub struct DiscordBotConfig {
    pub token: String,
    pub application_id: String,
    pub guild_ids: Vec<String>,
}

impl DiscordProvider {
    pub async fn send_dm(
        &self,
        bot_token: &str,
        user_id: &str,
        message: &str,
    ) -> Result<DiscordMessage, ChannelError> {
        let dm_channel = self
            .client
            .post(format!("{}/users/@me/channels", self.base_url))
            .header("Authorization", format!("Bot {}", bot_token))
            .json(&CreateDMChannel {
                recipient_id: user_id.to_string(),
            })
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !dm_channel.status().is_success() {
            let error_text = dm_channel.text().await.unwrap_or_default();
            return Err(ChannelError::ApiError {
                code: None,
                message: format!("Failed to create DM channel: {}", error_text),
            });
        }

        #[derive(Deserialize)]
        struct DMChannel {
            id: String,
        }

        let channel: DMChannel = dm_channel
            .json()
            .await
            .map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })?;

        let content = MessageContent {
            content: Some(message.to_string()),
            embeds: None,
            tts: Some(false),
        };

        self.send_message(bot_token, &channel.id, &content).await
    }

    pub async fn get_guild_channels(
        &self,
        bot_token: &str,
        guild_id: &str,
    ) -> Result<Vec<GuildChannel>, ChannelError> {
        let response = self
            .client
            .get(format!("{}/guilds/{}/channels", self.base_url, guild_id))
            .header("Authorization", format!("Bot {}", bot_token))
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ChannelError::ApiError {
                code: None,
                message: error_text,
            });
        }

        response
            .json::<Vec<GuildChannel>>()
            .await
            .map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })
    }
}

#[derive(Debug, Deserialize)]
pub struct GuildChannel {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub channel_type: u8,
    pub guild_id: Option<String>,
    pub position: Option<i32>,
    pub topic: Option<String>,
}

impl GuildChannel {
    pub fn is_text_channel(&self) -> bool {
        self.channel_type == 0
    }

    pub fn is_voice_channel(&self) -> bool {
        self.channel_type == 2
    }

    pub fn is_category(&self) -> bool {
        self.channel_type == 4
    }

    pub fn is_announcement_channel(&self) -> bool {
        self.channel_type == 5
    }
}
