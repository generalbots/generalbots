pub mod binding;
pub mod cookies;
pub mod nonce_store;
pub mod oauth_identity;
pub mod providers;
pub mod redirect;
pub mod routes;
pub mod schema;
pub mod state;

// GitHub email ownership lookup (#2134).
mod github_email;

pub use binding::{consume_bound_nonce, consume_bound_user_nonce, BindingError};
pub use providers::{ProviderEndpoints, get_enabled_providers, load_oauth_config};
pub use redirect::{safe_redirect_target, RedirectPolicy, DEFAULT_REDIRECT};
pub use routes::{
    configure, EnabledProvidersResponse, OAuthCallbackParams, OAuthStartParams, ProviderInfo,
};
pub use state::{server_signer, OAuthState, StateError, StateSigner, STATE_TTL_SECS};

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
    GitHub,
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
            Self::GitHub,
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
            "github" => Some(Self::GitHub),
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
            Self::GitHub => "oauth-github",
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
            Self::GitHub => "GitHub",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Google
            | Self::Discord
            | Self::Reddit
            | Self::Twitter
            | Self::Microsoft
            | Self::Facebook
            | Self::GitHub => "",
        }
    }
}

impl fmt::Display for OAuthProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Generates a random PKCE code verifier (43–128 chars, unreserved ASCII).
/// The S256 challenge is derived from it during the authorization request;
/// the verifier itself is presented at the token exchange.
pub fn generate_pkce_verifier() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 48];
    rand::rng().fill_bytes(&mut bytes);
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Derives the S256 PKCE code challenge from a verifier.
pub fn pkce_s256_challenge(verifier: &str) -> String {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(digest)
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
    /// Whether the provider asserted that it verified `email` (#2134). `None`
    /// means the provider does not report it, which is treated as unverified:
    /// such an address may never be matched to an existing local account.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email_verified: Option<bool>,
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

#[cfg(test)]
#[path = "pkce_tests.rs"]
mod pkce_tests;

#[cfg(test)]
#[path = "state_tests.rs"]
mod state_tests;

#[cfg(test)]
#[path = "redirect_tests.rs"]
mod redirect_tests;

#[cfg(test)]
#[path = "cookies_tests.rs"]
mod cookies_tests;

#[cfg(test)]
#[path = "oauth_identity_tests.rs"]
mod oauth_identity_tests;
