use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SocialPlatform {
    Instagram,
    Facebook,
    Twitter,
    LinkedIn,
    Bluesky,
    Threads,
    TikTok,
    YouTube,
    Pinterest,
    Reddit,
    Snapchat,
    Discord,
    WhatsApp,
    Telegram,
    WeChat,
    Twilio,
}

impl SocialPlatform {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Instagram => "Instagram",
            Self::Facebook => "Facebook",
            Self::Twitter => "Twitter / X",
            Self::LinkedIn => "LinkedIn",
            Self::Bluesky => "Bluesky",
            Self::Threads => "Threads",
            Self::TikTok => "TikTok",
            Self::YouTube => "YouTube",
            Self::Pinterest => "Pinterest",
            Self::Reddit => "Reddit",
            Self::Snapchat => "Snapchat",
            Self::Discord => "Discord",
            Self::WhatsApp => "WhatsApp",
            Self::Telegram => "Telegram",
            Self::WeChat => "WeChat",
            Self::Twilio => "Twilio SMS",
        }
    }

    pub fn requires_oauth(&self) -> bool {
        !matches!(self, Self::Bluesky | Self::Telegram | Self::Twilio)
    }

    pub fn authorization_url(&self) -> Option<&'static str> {
        match self {
            Self::Instagram | Self::Facebook | Self::Threads => {
                Some("https://www.facebook.com/v18.0/dialog/oauth")
            }
            Self::Twitter => Some("https://twitter.com/i/oauth2/authorize"),
            Self::LinkedIn => Some("https://www.linkedin.com/oauth/v2/authorization"),
            Self::TikTok => Some("https://www.tiktok.com/v2/auth/authorize/"),
            Self::YouTube => Some("https://accounts.google.com/o/oauth2/v2/auth"),
            Self::Pinterest => Some("https://www.pinterest.com/oauth/"),
            Self::Reddit => Some("https://www.reddit.com/api/v1/authorize"),
            Self::Snapchat => Some("https://accounts.snapchat.com/login/oauth2/authorize"),
            Self::Discord => Some("https://discord.com/api/oauth2/authorize"),
            Self::WeChat => Some("https://open.weixin.qq.com/connect/oauth2/authorize"),
            _ => None,
        }
    }

    pub fn token_url(&self) -> Option<&'static str> {
        match self {
            Self::Instagram | Self::Facebook | Self::Threads => {
                Some("https://graph.facebook.com/v18.0/oauth/access_token")
            }
            Self::Twitter => Some("https://api.twitter.com/2/oauth2/token"),
            Self::LinkedIn => Some("https://www.linkedin.com/oauth/v2/accessToken"),
            Self::TikTok => Some("https://open.tiktokapis.com/v2/oauth/token/"),
            Self::YouTube => Some("https://oauth2.googleapis.com/token"),
            Self::Pinterest => Some("https://api.pinterest.com/v5/oauth/token"),
            Self::Reddit => Some("https://www.reddit.com/api/v1/access_token"),
            Self::Snapchat => Some("https://accounts.snapchat.com/login/oauth2/access_token"),
            Self::Discord => Some("https://discord.com/api/oauth2/token"),
            Self::WeChat => Some("https://api.weixin.qq.com/sns/oauth2/access_token"),
            _ => None,
        }
    }

    pub fn default_scopes(&self) -> Vec<&'static str> {
        match self {
            Self::Instagram => vec![
                "instagram_basic",
                "instagram_content_publish",
                "instagram_manage_comments",
            ],
            Self::Facebook => vec![
                "pages_manage_posts",
                "pages_read_engagement",
                "pages_show_list",
            ],
            Self::Threads => vec!["threads_basic", "threads_content_publish"],
            Self::Twitter => vec!["tweet.read", "tweet.write", "users.read", "offline.access"],
            Self::LinkedIn => vec!["w_organization_social", "r_organization_social"],
            Self::TikTok => vec!["video.upload", "video.publish"],
            Self::YouTube => vec![
                "https://www.googleapis.com/auth/youtube.upload",
                "https://www.googleapis.com/auth/youtube",
            ],
            Self::Pinterest => vec!["boards:read", "pins:read", "pins:write"],
            Self::Reddit => vec!["submit", "read", "identity"],
            Self::Snapchat => vec!["snapchat-marketing-api"],
            Self::Discord => vec!["bot", "applications.commands"],
            Self::WeChat => vec!["snsapi_userinfo"],
            _ => vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthCredentials {
    pub platform: SocialPlatform,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthToken {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub platform: SocialPlatform,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub scopes: Vec<String>,
    pub account_id: Option<String>,
    pub account_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl OAuthToken {
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(expires) => Utc::now() >= expires,
            None => false,
        }
    }

    pub fn needs_refresh(&self) -> bool {
        match self.expires_at {
            Some(expires) => {
                let buffer = chrono::Duration::minutes(5);
                Utc::now() >= expires - buffer
            }
            None => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthState {
    pub id: Uuid,
    pub platform: SocialPlatform,
    pub organization_id: Uuid,
    pub user_id: Uuid,
    pub redirect_after: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl OAuthState {
    pub fn new(
        platform: SocialPlatform,
        organization_id: Uuid,
        user_id: Uuid,
        redirect_after: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            platform,
            organization_id,
            user_id,
            redirect_after,
            created_at: now,
            expires_at: now + chrono::Duration::minutes(10),
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at
    }

    pub fn encode(&self) -> String {
        use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
        let json = serde_json::to_string(self).unwrap_or_default();
        URL_SAFE_NO_PAD.encode(json.as_bytes())
    }

    pub fn decode(encoded: &str) -> Option<Self> {
        use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
        let bytes = URL_SAFE_NO_PAD.decode(encoded).ok()?;
        let json = String::from_utf8(bytes).ok()?;
        serde_json::from_str(&json).ok()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationUrl {
    pub url: String,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenExchangeRequest {
    pub code: String,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    #[serde(default)]
    pub token_type: String,
    pub expires_in: Option<i64>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectedAccount {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub platform: SocialPlatform,
    pub account_id: String,
    pub account_name: String,
    pub account_url: Option<String>,
    pub avatar_url: Option<String>,
    pub connected_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    pub status: AccountStatus,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AccountStatus {
    Active,
    Expired,
    Revoked,
    Error,
}

pub struct SocialOAuthService {
    credentials: Arc<RwLock<HashMap<SocialPlatform, OAuthCredentials>>>,
    tokens: Arc<RwLock<HashMap<Uuid, OAuthToken>>>,
    states: Arc<RwLock<HashMap<String, OAuthState>>>,
    accounts: Arc<RwLock<HashMap<Uuid, ConnectedAccount>>>,
}

impl SocialOAuthService {
    pub fn new() -> Self {
        Self {
            credentials: Arc::new(RwLock::new(HashMap::new())),
            tokens: Arc::new(RwLock::new(HashMap::new())),
            states: Arc::new(RwLock::new(HashMap::new())),
            accounts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_credentials(&self, credentials: OAuthCredentials) {
        let mut creds = self.credentials.write().await;
        creds.insert(credentials.platform, credentials);
    }

    pub async fn get_credentials(&self, platform: SocialPlatform) -> Option<OAuthCredentials> {
        let creds = self.credentials.read().await;
        creds.get(&platform).cloned()
    }

    pub async fn generate_authorization_url(
        &self,
        platform: SocialPlatform,
        organization_id: Uuid,
        user_id: Uuid,
        redirect_after: Option<String>,
        additional_scopes: Option<Vec<String>>,
    ) -> Result<AuthorizationUrl, OAuthServiceError> {
        let credentials = self
            .get_credentials(platform)
            .await
            .ok_or(OAuthServiceError::CredentialsNotFound)?;

        let auth_url = platform
            .authorization_url()
            .ok_or(OAuthServiceError::PlatformNotSupported)?;

        let state = OAuthState::new(platform, organization_id, user_id, redirect_after);
        let state_encoded = state.encode();

        {
            let mut states = self.states.write().await;
            states.insert(state_encoded.clone(), state);
        }

        let mut scopes: Vec<String> = platform
            .default_scopes()
            .into_iter()
            .map(String::from)
            .collect();

        if let Some(additional) = additional_scopes {
            scopes.extend(additional);
        }

        let scope_string = scopes.join(" ");

        let url = format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}",
            auth_url,
            urlencoding::encode(&credentials.client_id),
            urlencoding::encode(&credentials.redirect_uri),
            urlencoding::encode(&scope_string),
            urlencoding::encode(&state_encoded)
        );

        Ok(AuthorizationUrl {
            url,
            state: state_encoded,
        })
    }

    pub async fn exchange_code(
        &self,
        request: TokenExchangeRequest,
    ) -> Result<OAuthToken, OAuthServiceError> {
        let state = {
            let mut states = self.states.write().await;
            states
                .remove(&request.state)
                .ok_or(OAuthServiceError::InvalidState)?
        };

        if state.is_expired() {
            return Err(OAuthServiceError::StateExpired);
        }

        let credentials = self
            .get_credentials(state.platform)
            .await
            .ok_or(OAuthServiceError::CredentialsNotFound)?;

        let token_url = state
            .platform
            .token_url()
            .ok_or(OAuthServiceError::PlatformNotSupported)?;

        let client = reqwest::Client::new();
        let response = client
            .post(token_url)
            .form(&[
                ("client_id", credentials.client_id.as_str()),
                ("client_secret", credentials.client_secret.as_str()),
                ("code", request.code.as_str()),
                ("redirect_uri", credentials.redirect_uri.as_str()),
                ("grant_type", "authorization_code"),
            ])
            .send()
            .await
            .map_err(|e| OAuthServiceError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(OAuthServiceError::TokenExchangeFailed(error_text));
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| OAuthServiceError::ParseError(e.to_string()))?;

        let now = Utc::now();
        let expires_at = token_response
            .expires_in
            .map(|secs| now + chrono::Duration::seconds(secs));

        let scopes = token_response
            .scope
            .map(|s| s.split(' ').map(String::from).collect())
            .unwrap_or_default();

        let token = OAuthToken {
            id: Uuid::new_v4(),
            organization_id: state.organization_id,
            platform: state.platform,
            access_token: token_response.access_token,
            refresh_token: token_response.refresh_token,
            token_type: token_response.token_type,
            expires_at,
            scopes,
            account_id: None,
            account_name: None,
            created_at: now,
            updated_at: now,
        };

        {
            let mut tokens = self.tokens.write().await;
            tokens.insert(token.id, token.clone());
        }

        Ok(token)
    }

    pub async fn refresh_token(&self, token_id: Uuid) -> Result<OAuthToken, OAuthServiceError> {
        let existing_token = {
            let tokens = self.tokens.read().await;
            tokens
                .get(&token_id)
                .cloned()
                .ok_or(OAuthServiceError::TokenNotFound)?
        };

        let refresh_token = existing_token
            .refresh_token
            .as_ref()
            .ok_or(OAuthServiceError::NoRefreshToken)?;

        let credentials = self
            .get_credentials(existing_token.platform)
            .await
            .ok_or(OAuthServiceError::CredentialsNotFound)?;

        let token_url = existing_token
            .platform
            .token_url()
            .ok_or(OAuthServiceError::PlatformNotSupported)?;

        let client = reqwest::Client::new();
        let response = client
            .post(token_url)
            .form(&[
                ("client_id", credentials.client_id.as_str()),
                ("client_secret", credentials.client_secret.as_str()),
                ("refresh_token", refresh_token.as_str()),
                ("grant_type", "refresh_token"),
            ])
            .send()
            .await
            .map_err(|e| OAuthServiceError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(OAuthServiceError::TokenRefreshFailed(error_text));
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| OAuthServiceError::ParseError(e.to_string()))?;

        let now = Utc::now();
        let expires_at = token_response
            .expires_in
            .map(|secs| now + chrono::Duration::seconds(secs));

        let updated_token = OAuthToken {
            id: existing_token.id,
            organization_id: existing_token.organization_id,
            platform: existing_token.platform,
            access_token: token_response.access_token,
            refresh_token: token_response
                .refresh_token
                .or(existing_token.refresh_token),
            token_type: token_response.token_type,
            expires_at,
            scopes: existing_token.scopes,
            account_id: existing_token.account_id,
            account_name: existing_token.account_name,
            created_at: existing_token.created_at,
            updated_at: now,
        };

        {
            let mut tokens = self.tokens.write().await;
            tokens.insert(updated_token.id, updated_token.clone());
        }

        Ok(updated_token)
    }

    pub async fn revoke_token(&self, token_id: Uuid) -> Result<(), OAuthServiceError> {
        let mut tokens = self.tokens.write().await;
        tokens
            .remove(&token_id)
            .ok_or(OAuthServiceError::TokenNotFound)?;
        Ok(())
    }

    pub async fn get_token(&self, token_id: Uuid) -> Option<OAuthToken> {
        let tokens = self.tokens.read().await;
        tokens.get(&token_id).cloned()
    }

    pub async fn get_organization_tokens(&self, organization_id: Uuid) -> Vec<OAuthToken> {
        let tokens = self.tokens.read().await;
        tokens
            .values()
            .filter(|t| t.organization_id == organization_id)
            .cloned()
            .collect()
    }

    pub async fn get_platform_token(
        &self,
        organization_id: Uuid,
        platform: SocialPlatform,
    ) -> Option<OAuthToken> {
        let tokens = self.tokens.read().await;
        tokens
            .values()
            .find(|t| t.organization_id == organization_id && t.platform == platform)
            .cloned()
    }

    pub async fn get_valid_token(
        &self,
        organization_id: Uuid,
        platform: SocialPlatform,
    ) -> Result<OAuthToken, OAuthServiceError> {
        let token = self
            .get_platform_token(organization_id, platform)
            .await
            .ok_or(OAuthServiceError::TokenNotFound)?;

        if token.needs_refresh() {
            if token.refresh_token.is_some() {
                return self.refresh_token(token.id).await;
            }
            return Err(OAuthServiceError::TokenExpired);
        }

        Ok(token)
    }

    pub async fn add_connected_account(&self, account: ConnectedAccount) {
        let mut accounts = self.accounts.write().await;
        accounts.insert(account.id, account);
    }

    pub async fn get_connected_accounts(&self, organization_id: Uuid) -> Vec<ConnectedAccount> {
        let accounts = self.accounts.read().await;
        accounts
            .values()
            .filter(|a| a.organization_id == organization_id)
            .cloned()
            .collect()
    }

    pub async fn get_account_by_platform(
        &self,
        organization_id: Uuid,
        platform: SocialPlatform,
    ) -> Option<ConnectedAccount> {
        let accounts = self.accounts.read().await;
        accounts
            .values()
            .find(|a| a.organization_id == organization_id && a.platform == platform)
            .cloned()
    }

    pub async fn disconnect_account(&self, account_id: Uuid) -> Result<(), OAuthServiceError> {
        let mut accounts = self.accounts.write().await;
        accounts
            .remove(&account_id)
            .ok_or(OAuthServiceError::AccountNotFound)?;
        Ok(())
    }

    pub async fn cleanup_expired_states(&self) {
        let mut states = self.states.write().await;
        states.retain(|_, state| !state.is_expired());
    }
}

impl Default for SocialOAuthService {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub enum OAuthServiceError {
    CredentialsNotFound,
    PlatformNotSupported,
    InvalidState,
    StateExpired,
    TokenNotFound,
    TokenExpired,
    NoRefreshToken,
    AccountNotFound,
    NetworkError(String),
    TokenExchangeFailed(String),
    TokenRefreshFailed(String),
    ParseError(String),
}

impl std::fmt::Display for OAuthServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CredentialsNotFound => write!(f, "OAuth credentials not found for platform"),
            Self::PlatformNotSupported => write!(f, "Platform does not support OAuth"),
            Self::InvalidState => write!(f, "Invalid OAuth state parameter"),
            Self::StateExpired => write!(f, "OAuth state has expired"),
            Self::TokenNotFound => write!(f, "OAuth token not found"),
            Self::TokenExpired => write!(f, "OAuth token has expired"),
            Self::NoRefreshToken => write!(f, "No refresh token available"),
            Self::AccountNotFound => write!(f, "Connected account not found"),
            Self::NetworkError(msg) => write!(f, "Network error: {msg}"),
            Self::TokenExchangeFailed(msg) => write!(f, "Token exchange failed: {msg}"),
            Self::TokenRefreshFailed(msg) => write!(f, "Token refresh failed: {msg}"),
            Self::ParseError(msg) => write!(f, "Parse error: {msg}"),
        }
    }
}

impl std::error::Error for OAuthServiceError {}
