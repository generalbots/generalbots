pub mod providers;
pub mod routes;

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OAuthProvider {
    Google,
    Discord,
    Reddit,
    Twitter,
    Microsoft,
    Facebook,
}

impl OAuthProvider {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Google,
            Self::Discord,
            Self::Reddit,
            Self::Twitter,
            Self::Microsoft,
            Self::Facebook,
        ]
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "google" => Some(Self::Google),
            "discord" => Some(Self::Discord),
            "reddit" => Some(Self::Reddit),
            "twitter" | "x" => Some(Self::Twitter),
            "microsoft" => Some(Self::Microsoft),
            "facebook" => Some(Self::Facebook),
            _ => None,
        }
    }

    pub fn config_prefix(&self) -> &'static str {
        match self {
            Self::Google => "oauth-google",
            Self::Discord => "oauth-discord",
            Self::Reddit => "oauth-reddit",
            Self::Twitter => "oauth-twitter",
            Self::Microsoft => "oauth-microsoft",
            Self::Facebook => "oauth-facebook",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Google => "Google",
            Self::Discord => "Discord",
            Self::Reddit => "Reddit",
            Self::Twitter => "Twitter",
            Self::Microsoft => "Microsoft",
            Self::Facebook => "Facebook",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Google
            | Self::Discord
            | Self::Reddit
            | Self::Twitter
            | Self::Microsoft
            | Self::Facebook => "",
        }
    }
}

impl fmt::Display for OAuthProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    pub provider: OAuthProvider,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub enabled: bool,
}

impl OAuthConfig {
    pub fn new(
        provider: OAuthProvider,
        client_id: String,
        client_secret: String,
        redirect_uri: String,
    ) -> Self {
        Self {
            provider,
            client_id,
            client_secret,
            redirect_uri,
            enabled: true,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.enabled
            && !self.client_id.is_empty()
            && !self.client_secret.is_empty()
            && !self.redirect_uri.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthUserInfo {
    pub provider_id: String,

    pub provider: OAuthProvider,

    pub email: Option<String>,

    pub name: Option<String>,

    pub avatar_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    #[serde(default)]
    pub token_type: String,
    #[serde(default)]
    pub expires_in: Option<i64>,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthError {
    pub error: String,
    pub error_description: Option<String>,
}

impl fmt::Display for OAuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(desc) = &self.error_description {
            write!(f, "{}: {}", self.error, desc)
        } else {
            write!(f, "{}", self.error)
        }
    }
}

impl std::error::Error for OAuthError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthState {
    pub token: String,

    pub provider: OAuthProvider,

    pub redirect_after: Option<String>,

    pub created_at: i64,
}

impl OAuthState {
    pub fn new(provider: OAuthProvider, redirect_after: Option<String>) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};

        let token = uuid::Uuid::new_v4().to_string();
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        Self {
            token,
            provider,
            redirect_after,
            created_at,
        }
    }

    pub fn is_expired(&self) -> bool {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        now - self.created_at > 600
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
