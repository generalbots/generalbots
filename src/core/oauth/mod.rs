//! OAuth2 Authentication Module
//!
//! Provides OAuth2 authentication support for multiple providers:
//! - Google
//! - Discord
//! - Reddit
//! - Twitter (X)
//! - Microsoft
//! - Facebook

pub mod providers;
pub mod routes;

use serde::{Deserialize, Serialize};
use std::fmt;

/// Supported OAuth2 providers
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
    /// Get all available providers
    pub fn all() -> Vec<OAuthProvider> {
        vec![
            OAuthProvider::Google,
            OAuthProvider::Discord,
            OAuthProvider::Reddit,
            OAuthProvider::Twitter,
            OAuthProvider::Microsoft,
            OAuthProvider::Facebook,
        ]
    }

    /// Get provider from string
    pub fn from_str(s: &str) -> Option<OAuthProvider> {
        match s.to_lowercase().as_str() {
            "google" => Some(OAuthProvider::Google),
            "discord" => Some(OAuthProvider::Discord),
            "reddit" => Some(OAuthProvider::Reddit),
            "twitter" | "x" => Some(OAuthProvider::Twitter),
            "microsoft" => Some(OAuthProvider::Microsoft),
            "facebook" => Some(OAuthProvider::Facebook),
            _ => None,
        }
    }

    /// Get the config key prefix for this provider
    pub fn config_prefix(&self) -> &'static str {
        match self {
            OAuthProvider::Google => "oauth-google",
            OAuthProvider::Discord => "oauth-discord",
            OAuthProvider::Reddit => "oauth-reddit",
            OAuthProvider::Twitter => "oauth-twitter",
            OAuthProvider::Microsoft => "oauth-microsoft",
            OAuthProvider::Facebook => "oauth-facebook",
        }
    }

    /// Get display name for UI
    pub fn display_name(&self) -> &'static str {
        match self {
            OAuthProvider::Google => "Google",
            OAuthProvider::Discord => "Discord",
            OAuthProvider::Reddit => "Reddit",
            OAuthProvider::Twitter => "Twitter",
            OAuthProvider::Microsoft => "Microsoft",
            OAuthProvider::Facebook => "Facebook",
        }
    }

    /// Get icon/emoji for UI
    pub fn icon(&self) -> &'static str {
        match self {
            OAuthProvider::Google => "",
            OAuthProvider::Discord => "",
            OAuthProvider::Reddit => "",
            OAuthProvider::Twitter => "",
            OAuthProvider::Microsoft => "",
            OAuthProvider::Facebook => "",
        }
    }
}

impl fmt::Display for OAuthProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// OAuth configuration for a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    pub provider: OAuthProvider,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub enabled: bool,
}

impl OAuthConfig {
    /// Create a new OAuth config
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

    /// Check if the config is valid (has required fields)
    pub fn is_valid(&self) -> bool {
        self.enabled
            && !self.client_id.is_empty()
            && !self.client_secret.is_empty()
            && !self.redirect_uri.is_empty()
    }
}

/// User information returned from OAuth provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthUserInfo {
    /// Provider-specific user ID
    pub provider_id: String,
    /// OAuth provider
    pub provider: OAuthProvider,
    /// User's email (if available)
    pub email: Option<String>,
    /// User's display name
    pub name: Option<String>,
    /// User's avatar URL
    pub avatar_url: Option<String>,
    /// Raw response from provider (for debugging/additional fields)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<serde_json::Value>,
}

/// OAuth token response
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

/// OAuth error types
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

/// State parameter for OAuth flow (CSRF protection)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthState {
    /// Random state token
    pub token: String,
    /// Provider being used
    pub provider: OAuthProvider,
    /// Optional redirect URL after login
    pub redirect_after: Option<String>,
    /// Timestamp when state was created
    pub created_at: i64,
}

impl OAuthState {
    /// Create a new OAuth state
    pub fn new(provider: OAuthProvider, redirect_after: Option<String>) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};

        let token = uuid::Uuid::new_v4().to_string();
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        Self {
            token,
            provider,
            redirect_after,
            created_at,
        }
    }

    /// Check if state is expired (default: 10 minutes)
    pub fn is_expired(&self) -> bool {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        now - self.created_at > 600 // 10 minutes
    }

    /// Encode state to URL-safe string
    pub fn encode(&self) -> String {
        use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
        let json = serde_json::to_string(self).unwrap_or_default();
        URL_SAFE_NO_PAD.encode(json.as_bytes())
    }

    /// Decode state from URL-safe string
    pub fn decode(encoded: &str) -> Option<Self> {
        use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
        let bytes = URL_SAFE_NO_PAD.decode(encoded).ok()?;
        let json = String::from_utf8(bytes).ok()?;
        serde_json::from_str(&json).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_from_str() {
        assert_eq!(
            OAuthProvider::from_str("google"),
            Some(OAuthProvider::Google)
        );
        assert_eq!(
            OAuthProvider::from_str("DISCORD"),
            Some(OAuthProvider::Discord)
        );
        assert_eq!(
            OAuthProvider::from_str("Twitter"),
            Some(OAuthProvider::Twitter)
        );
        assert_eq!(OAuthProvider::from_str("x"), Some(OAuthProvider::Twitter));
        assert_eq!(OAuthProvider::from_str("invalid"), None);
    }

    #[test]
    fn test_oauth_state_encode_decode() {
        let state = OAuthState::new(OAuthProvider::Google, Some("/dashboard".to_string()));
        let encoded = state.encode();
        let decoded = OAuthState::decode(&encoded).unwrap();

        assert_eq!(decoded.provider, OAuthProvider::Google);
        assert_eq!(decoded.redirect_after, Some("/dashboard".to_string()));
        assert!(!decoded.is_expired());
    }

    #[test]
    fn test_oauth_config_validation() {
        let valid_config = OAuthConfig::new(
            OAuthProvider::Google,
            "client_id".to_string(),
            "client_secret".to_string(),
            "http://localhost/callback".to_string(),
        );
        assert!(valid_config.is_valid());

        let mut invalid_config = valid_config.clone();
        invalid_config.client_id = String::new();
        assert!(!invalid_config.is_valid());
    }
}
