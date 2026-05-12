use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub mod bluesky;
pub mod discord;
pub mod facebook;
pub mod instagram_channel;
pub mod linkedin;
pub mod media_upload;
pub mod oauth;
pub mod pinterest;
pub mod reddit;
pub mod snapchat;
pub mod telegram_channel;
pub mod threads;
pub mod tiktok;
pub mod twilio_sms;
pub mod twitter;
pub mod wechat;
pub mod youtube;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ChannelType {
    Bluesky,
    Discord,
    Facebook,
    Instagram,
    LinkedIn,
    Pinterest,
    Reddit,
    Snapchat,
    Telegram,
    Threads,
    TikTok,
    TwilioSms,
    Twitter,
    WeChat,
    WhatsApp,
    YouTube,
}

impl std::fmt::Display for ChannelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bluesky => write!(f, "bluesky"),
            Self::Discord => write!(f, "discord"),
            Self::Facebook => write!(f, "facebook"),
            Self::Instagram => write!(f, "instagram"),
            Self::LinkedIn => write!(f, "linkedin"),
            Self::Pinterest => write!(f, "pinterest"),
            Self::Reddit => write!(f, "reddit"),
            Self::Snapchat => write!(f, "snapchat"),
            Self::Telegram => write!(f, "telegram"),
            Self::Threads => write!(f, "threads"),
            Self::TikTok => write!(f, "tiktok"),
            Self::TwilioSms => write!(f, "twilio_sms"),
            Self::Twitter => write!(f, "twitter"),
            Self::WeChat => write!(f, "wechat"),
            Self::WhatsApp => write!(f, "whatsapp"),
            Self::YouTube => write!(f, "youtube"),
        }
    }
}

impl std::str::FromStr for ChannelType {
    type Err = ChannelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bluesky" => Ok(Self::Bluesky),
            "discord" => Ok(Self::Discord),
            "facebook" | "fb" => Ok(Self::Facebook),
            "instagram" | "ig" => Ok(Self::Instagram),
            "linkedin" => Ok(Self::LinkedIn),
            "pinterest" => Ok(Self::Pinterest),
            "reddit" => Ok(Self::Reddit),
            "snapchat" => Ok(Self::Snapchat),
            "telegram" | "tg" => Ok(Self::Telegram),
            "threads" => Ok(Self::Threads),
            "tiktok" => Ok(Self::TikTok),
            "twilio" | "twilio_sms" | "sms" => Ok(Self::TwilioSms),
            "twitter" | "x" => Ok(Self::Twitter),
            "wechat" => Ok(Self::WeChat),
            "whatsapp" | "wa" => Ok(Self::WhatsApp),
            "youtube" | "yt" => Ok(Self::YouTube),
            _ => Err(ChannelError::UnknownChannel(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelAccount {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub channel_type: ChannelType,
    pub credentials: ChannelCredentials,
    pub settings: ChannelSettings,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChannelCredentials {
    OAuth {
        access_token: String,
        refresh_token: Option<String>,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
        scope: Option<String>,
    },
    ApiKey {
        api_key: String,
        api_secret: Option<String>,
    },
    UsernamePassword {
        username: String,
        password: String,
        app_password: Option<String>,
    },
    Twilio {
        account_sid: String,
        auth_token: String,
        from_number: String,
    },
    Custom {
        data: HashMap<String, String>,
    },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChannelSettings {
    pub default_hashtags: Vec<String>,
    pub auto_shorten_links: bool,
    pub schedule_enabled: bool,
    pub timezone: Option<String>,
    pub rate_limit_per_hour: Option<u32>,
    pub custom: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostContent {
    pub text: Option<String>,
    pub image_urls: Vec<String>,
    pub video_url: Option<String>,
    pub link: Option<String>,
    pub hashtags: Vec<String>,
    pub mentions: Vec<String>,
    pub scheduled_at: Option<chrono::DateTime<chrono::Utc>>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl PostContent {
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            text: Some(text.into()),
            image_urls: vec![],
            video_url: None,
            link: None,
            hashtags: vec![],
            mentions: vec![],
            scheduled_at: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_image(mut self, url: impl Into<String>) -> Self {
        self.image_urls.push(url.into());
        self
    }

    pub fn with_images(mut self, urls: Vec<String>) -> Self {
        self.image_urls.extend(urls);
        self
    }

    pub fn with_video(mut self, url: impl Into<String>) -> Self {
        self.video_url = Some(url.into());
        self
    }

    pub fn with_link(mut self, url: impl Into<String>) -> Self {
        self.link = Some(url.into());
        self
    }

    pub fn with_hashtags(mut self, tags: Vec<String>) -> Self {
        self.hashtags.extend(tags);
        self
    }

    pub fn with_mentions(mut self, mentions: Vec<String>) -> Self {
        self.mentions.extend(mentions);
        self
    }

    pub fn scheduled(mut self, at: chrono::DateTime<chrono::Utc>) -> Self {
        self.scheduled_at = Some(at);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostResult {
    pub success: bool,
    pub channel_type: ChannelType,
    pub post_id: Option<String>,
    pub url: Option<String>,
    pub error: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl PostResult {
    pub fn success(channel_type: ChannelType, post_id: String, url: Option<String>) -> Self {
        Self {
            success: true,
            channel_type,
            post_id: Some(post_id),
            url,
            error: None,
            metadata: HashMap::new(),
        }
    }

    pub fn error(channel_type: ChannelType, error: impl Into<String>) -> Self {
        Self {
            success: false,
            channel_type,
            post_id: None,
            url: None,
            error: Some(error.into()),
            metadata: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ChannelError {
    UnknownChannel(String),
    AccountNotFound(String),
    AuthenticationFailed(String),
    RateLimited { retry_after: Option<u64> },
    ContentTooLong { max_length: usize, actual_length: usize },
    UnsupportedMediaType(String),
    NetworkError(String),
    ApiError { code: Option<String>, message: String },
    NotConfigured,
}

impl std::fmt::Display for ChannelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownChannel(name) => write!(f, "Unknown channel: {name}"),
            Self::AccountNotFound(name) => write!(f, "Account not found: {name}"),
            Self::AuthenticationFailed(msg) => write!(f, "Authentication failed: {msg}"),
            Self::RateLimited { retry_after } => {
                if let Some(secs) = retry_after {
                    write!(f, "Rate limited, retry after {secs} seconds")
                } else {
                    write!(f, "Rate limited")
                }
            }
            Self::ContentTooLong { max_length, actual_length } => {
                write!(f, "Content too long: {actual_length} characters (max: {max_length})")
            }
            Self::UnsupportedMediaType(media_type) => {
                write!(f, "Unsupported media type: {media_type}")
            }
            Self::NetworkError(msg) => write!(f, "Network error: {msg}"),
            Self::ApiError { code, message } => {
                if let Some(c) = code {
                    write!(f, "API error [{c}]: {message}")
                } else {
                    write!(f, "API error: {message}")
                }
            }
            Self::NotConfigured => write!(f, "Channel not configured"),
        }
    }
}

impl std::error::Error for ChannelError {}

#[async_trait::async_trait]
pub trait ChannelProvider: Send + Sync {
    fn channel_type(&self) -> ChannelType;
    fn max_text_length(&self) -> usize;
    fn supports_images(&self) -> bool;
    fn supports_video(&self) -> bool;
    fn supports_links(&self) -> bool;

    async fn post(&self, account: &ChannelAccount, content: &PostContent) -> Result<PostResult, ChannelError>;
    async fn validate_credentials(&self, credentials: &ChannelCredentials) -> Result<bool, ChannelError>;
    async fn refresh_token(&self, account: &mut ChannelAccount) -> Result<(), ChannelError>;
}

pub struct ChannelManager {
    accounts: Arc<RwLock<HashMap<String, ChannelAccount>>>,
    providers: HashMap<ChannelType, Arc<dyn ChannelProvider>>,
}

impl ChannelManager {
    pub fn new() -> Self {
        Self {
            accounts: Arc::new(RwLock::new(HashMap::new())),
            providers: HashMap::new(),
        }
    }

    pub fn register_provider(&mut self, provider: Arc<dyn ChannelProvider>) {
        self.providers.insert(provider.channel_type(), provider);
    }

    pub async fn add_account(&self, account: ChannelAccount) {
        let mut accounts = self.accounts.write().await;
        accounts.insert(account.name.clone(), account);
    }

    pub async fn get_account(&self, name: &str) -> Option<ChannelAccount> {
        let accounts = self.accounts.read().await;
        accounts.get(name).cloned()
    }

    pub async fn remove_account(&self, name: &str) -> Option<ChannelAccount> {
        let mut accounts = self.accounts.write().await;
        accounts.remove(name)
    }

    pub async fn list_accounts(&self) -> Vec<ChannelAccount> {
        let accounts = self.accounts.read().await;
        accounts.values().cloned().collect()
    }

    pub async fn post_to(
        &self,
        account_name: &str,
        content: &PostContent,
    ) -> Result<PostResult, ChannelError> {
        let account = self
            .get_account(account_name)
            .await
            .ok_or_else(|| ChannelError::AccountNotFound(account_name.to_string()))?;

        let provider = self
            .providers
            .get(&account.channel_type)
            .ok_or(ChannelError::NotConfigured)?;

        provider.post(&account, content).await
    }

    pub async fn post_to_multiple(
        &self,
        account_names: &[String],
        content: &PostContent,
    ) -> Vec<Result<PostResult, ChannelError>> {
        let mut results = Vec::with_capacity(account_names.len());

        for name in account_names {
            let result = self.post_to(name, content).await;
            results.push(result);
        }

        results
    }

    pub async fn post_to_channels(
        &self,
        channels: &[ChannelType],
        content: &PostContent,
    ) -> Vec<Result<PostResult, ChannelError>> {
        let accounts = self.accounts.read().await;
        let mut results = Vec::new();

        for channel_type in channels {
            let matching_accounts: Vec<_> = accounts
                .values()
                .filter(|a| &a.channel_type == channel_type && a.is_active)
                .collect();

            for account in matching_accounts {
                if let Some(provider) = self.providers.get(channel_type) {
                    let result = provider.post(account, content).await;
                    results.push(result);
                } else {
                    results.push(Err(ChannelError::NotConfigured));
                }
            }
        }

        results
    }

    pub fn get_channel_limits(&self, channel_type: &ChannelType) -> Option<ChannelLimits> {
        self.providers.get(channel_type).map(|p| ChannelLimits {
            max_text_length: p.max_text_length(),
            supports_images: p.supports_images(),
            supports_video: p.supports_video(),
            supports_links: p.supports_links(),
        })
    }
}

impl Default for ChannelManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelLimits {
    pub max_text_length: usize,
    pub supports_images: bool,
    pub supports_video: bool,
    pub supports_links: bool,
}

pub struct MultiPostRequest {
    pub content: PostContent,
    pub targets: Vec<PostTarget>,
}

#[derive(Debug, Clone)]
pub enum PostTarget {
    Account(String),
    Channel(ChannelType),
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiPostResult {
    pub total: usize,
    pub successful: usize,
    pub failed: usize,
    pub results: Vec<PostResult>,
}

impl MultiPostResult {
    pub fn from_results(results: Vec<Result<PostResult, ChannelError>>) -> Self {
        let total = results.len();
        let mut successful = 0;
        let mut failed = 0;
        let mut post_results = Vec::with_capacity(total);

        for result in results {
            match result {
                Ok(r) => {
                    if r.success {
                        successful += 1;
                    } else {
                        failed += 1;
                    }
                    post_results.push(r);
                }
                Err(e) => {
                    failed += 1;
                    post_results.push(PostResult {
                        success: false,
                        channel_type: ChannelType::Twitter,
                        post_id: None,
                        url: None,
                        error: Some(e.to_string()),
                        metadata: HashMap::new(),
                    });
                }
            }
        }

        Self {
            total,
            successful,
            failed,
            results: post_results,
        }
    }
}
