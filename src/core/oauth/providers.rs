//! OAuth2 Provider Configurations
//!
//! This module contains the configuration for each OAuth2 provider including:
//! - Authorization URLs
//! - Token exchange URLs
//! - User info endpoints
//! - Required scopes

use super::{OAuthConfig, OAuthProvider, OAuthTokenResponse, OAuthUserInfo};
use anyhow::{anyhow, Result};
use reqwest::Client;
use std::collections::HashMap;

/// Provider-specific OAuth2 endpoints and configuration
#[derive(Debug, Clone)]
pub struct ProviderEndpoints {
    /// Authorization URL (where user is redirected to login)
    pub auth_url: &'static str,
    /// Token exchange URL
    pub token_url: &'static str,
    /// User info endpoint URL
    pub userinfo_url: &'static str,
    /// Required scopes for basic user info
    pub scopes: &'static [&'static str],
    /// Whether to use Basic auth for token exchange
    pub use_basic_auth: bool,
}

impl OAuthProvider {
    /// Get the endpoints configuration for this provider
    pub fn endpoints(&self) -> ProviderEndpoints {
        match self {
            OAuthProvider::Google => ProviderEndpoints {
                auth_url: "https://accounts.google.com/o/oauth2/v2/auth",
                token_url: "https://oauth2.googleapis.com/token",
                userinfo_url: "https://www.googleapis.com/oauth2/v2/userinfo",
                scopes: &["openid", "email", "profile"],
                use_basic_auth: false,
            },
            OAuthProvider::Discord => ProviderEndpoints {
                auth_url: "https://discord.com/api/oauth2/authorize",
                token_url: "https://discord.com/api/oauth2/token",
                userinfo_url: "https://discord.com/api/users/@me",
                scopes: &["identify", "email"],
                use_basic_auth: true,
            },
            OAuthProvider::Reddit => ProviderEndpoints {
                auth_url: "https://www.reddit.com/api/v1/authorize",
                token_url: "https://www.reddit.com/api/v1/access_token",
                userinfo_url: "https://oauth.reddit.com/api/v1/me",
                scopes: &["identity"],
                use_basic_auth: true,
            },
            OAuthProvider::Twitter => ProviderEndpoints {
                auth_url: "https://twitter.com/i/oauth2/authorize",
                token_url: "https://api.twitter.com/2/oauth2/token",
                userinfo_url: "https://api.twitter.com/2/users/me",
                scopes: &["users.read", "tweet.read"],
                use_basic_auth: true,
            },
            OAuthProvider::Microsoft => ProviderEndpoints {
                auth_url: "https://login.microsoftonline.com/common/oauth2/v2.0/authorize",
                token_url: "https://login.microsoftonline.com/common/oauth2/v2.0/token",
                userinfo_url: "https://graph.microsoft.com/v1.0/me",
                scopes: &["openid", "email", "profile", "User.Read"],
                use_basic_auth: false,
            },
            OAuthProvider::Facebook => ProviderEndpoints {
                auth_url: "https://www.facebook.com/v18.0/dialog/oauth",
                token_url: "https://graph.facebook.com/v18.0/oauth/access_token",
                userinfo_url: "https://graph.facebook.com/v18.0/me",
                scopes: &["email", "public_profile"],
                use_basic_auth: false,
            },
        }
    }

    /// Build the authorization URL for this provider
    pub fn build_auth_url(&self, config: &OAuthConfig, state: &str) -> String {
        let endpoints = self.endpoints();
        let scopes = endpoints.scopes.join(" ");

        let mut params = vec![
            ("client_id", config.client_id.as_str()),
            ("redirect_uri", config.redirect_uri.as_str()),
            ("response_type", "code"),
            ("state", state),
            ("scope", &scopes),
        ];

        // Provider-specific parameters
        match self {
            OAuthProvider::Google => {
                params.push(("access_type", "offline"));
                params.push(("prompt", "consent"));
            }
            OAuthProvider::Discord => {
                // Discord uses space-separated scopes in the URL
            }
            OAuthProvider::Reddit => {
                params.push(("duration", "temporary"));
            }
            OAuthProvider::Twitter => {
                params.push(("code_challenge", "challenge"));
                params.push(("code_challenge_method", "plain"));
            }
            OAuthProvider::Microsoft => {
                params.push(("response_mode", "query"));
            }
            OAuthProvider::Facebook => {
                // Facebook uses comma-separated scopes, but also accepts space
            }
        }

        let query = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        format!("{}?{}", endpoints.auth_url, query)
    }

    /// Exchange authorization code for access token
    pub async fn exchange_code(
        &self,
        config: &OAuthConfig,
        code: &str,
        client: &Client,
    ) -> Result<OAuthTokenResponse> {
        let endpoints = self.endpoints();

        let mut params = HashMap::new();
        params.insert("grant_type", "authorization_code");
        params.insert("code", code);
        params.insert("redirect_uri", config.redirect_uri.as_str());
        params.insert("client_id", config.client_id.as_str());

        // Twitter requires code_verifier for PKCE
        if matches!(self, OAuthProvider::Twitter) {
            params.insert("code_verifier", "challenge");
        }

        let mut request = client.post(endpoints.token_url);

        if endpoints.use_basic_auth {
            request = request.basic_auth(&config.client_id, Some(&config.client_secret));
        } else {
            params.insert("client_secret", config.client_secret.as_str());
        }

        // Reddit requires a custom User-Agent
        if matches!(self, OAuthProvider::Reddit) {
            request = request.header("User-Agent", "BotServer/1.0");
        }

        let response = request
            .form(&params)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to exchange code: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Token exchange failed: {}", error_text));
        }

        let token: OAuthTokenResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse token response: {}", e))?;

        Ok(token)
    }

    /// Fetch user info from the provider
    pub async fn fetch_user_info(
        &self,
        access_token: &str,
        client: &Client,
    ) -> Result<OAuthUserInfo> {
        let endpoints = self.endpoints();

        let mut request = client.get(endpoints.userinfo_url);

        // Provider-specific headers and query params
        match self {
            OAuthProvider::Reddit => {
                request = request
                    .header("User-Agent", "BotServer/1.0")
                    .bearer_auth(access_token);
            }
            OAuthProvider::Twitter => {
                request = request
                    .query(&[("user.fields", "id,name,username,profile_image_url")])
                    .bearer_auth(access_token);
            }
            OAuthProvider::Facebook => {
                request = request.query(&[
                    ("fields", "id,name,email,picture.type(large)"),
                    ("access_token", access_token),
                ]);
            }
            _ => {
                request = request.bearer_auth(access_token);
            }
        }

        let response = request
            .send()
            .await
            .map_err(|e| anyhow!("Failed to fetch user info: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to fetch user info: {}", error_text));
        }

        let raw: serde_json::Value = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse user info: {}", e))?;

        // Parse provider-specific response into common format
        let user_info = self.parse_user_info(&raw)?;

        Ok(user_info)
    }

    /// Parse provider-specific user info response into common format
    fn parse_user_info(&self, raw: &serde_json::Value) -> Result<OAuthUserInfo> {
        match self {
            OAuthProvider::Google => Ok(OAuthUserInfo {
                provider_id: raw["id"].as_str().unwrap_or_default().to_string(),
                provider: *self,
                email: raw["email"].as_str().map(String::from),
                name: raw["name"].as_str().map(String::from),
                avatar_url: raw["picture"].as_str().map(String::from),
                raw: Some(raw.clone()),
            }),
            OAuthProvider::Discord => Ok(OAuthUserInfo {
                provider_id: raw["id"].as_str().unwrap_or_default().to_string(),
                provider: *self,
                email: raw["email"].as_str().map(String::from),
                name: raw["username"].as_str().map(String::from),
                avatar_url: raw["avatar"].as_str().map(|avatar| {
                    let user_id = raw["id"].as_str().unwrap_or_default();
                    format!(
                        "https://cdn.discordapp.com/avatars/{}/{}.png",
                        user_id, avatar
                    )
                }),
                raw: Some(raw.clone()),
            }),
            OAuthProvider::Reddit => Ok(OAuthUserInfo {
                provider_id: raw["id"].as_str().unwrap_or_default().to_string(),
                provider: *self,
                email: None, // Reddit doesn't provide email with basic scope
                name: raw["name"].as_str().map(String::from),
                avatar_url: raw["icon_img"]
                    .as_str()
                    .map(|s| s.split('?').next().unwrap_or(s).to_string()),
                raw: Some(raw.clone()),
            }),
            OAuthProvider::Twitter => {
                let data = raw.get("data").unwrap_or(raw);
                Ok(OAuthUserInfo {
                    provider_id: data["id"].as_str().unwrap_or_default().to_string(),
                    provider: *self,
                    email: None, // Twitter requires elevated access for email
                    name: data["name"].as_str().map(String::from),
                    avatar_url: data["profile_image_url"].as_str().map(String::from),
                    raw: Some(raw.clone()),
                })
            }
            OAuthProvider::Microsoft => Ok(OAuthUserInfo {
                provider_id: raw["id"].as_str().unwrap_or_default().to_string(),
                provider: *self,
                email: raw["mail"]
                    .as_str()
                    .or_else(|| raw["userPrincipalName"].as_str())
                    .map(String::from),
                name: raw["displayName"].as_str().map(String::from),
                avatar_url: None, // Microsoft Graph requires separate endpoint for photo
                raw: Some(raw.clone()),
            }),
            OAuthProvider::Facebook => Ok(OAuthUserInfo {
                provider_id: raw["id"].as_str().unwrap_or_default().to_string(),
                provider: *self,
                email: raw["email"].as_str().map(String::from),
                name: raw["name"].as_str().map(String::from),
                avatar_url: raw["picture"]["data"]["url"].as_str().map(String::from),
                raw: Some(raw.clone()),
            }),
        }
    }
}

/// Load OAuth configuration from bot config
pub fn load_oauth_config(
    provider: OAuthProvider,
    bot_config: &HashMap<String, String>,
    base_url: &str,
) -> Option<OAuthConfig> {
    let prefix = provider.config_prefix();

    let enabled = bot_config
        .get(&format!("{}-enabled", prefix))
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false);

    if !enabled {
        return None;
    }

    let client_id = bot_config.get(&format!("{}-client-id", prefix))?.clone();
    let client_secret = bot_config
        .get(&format!("{}-client-secret", prefix))?
        .clone();

    // Use configured redirect URI or build default
    let redirect_uri = bot_config
        .get(&format!("{}-redirect-uri", prefix))
        .cloned()
        .unwrap_or_else(|| {
            format!(
                "{}/auth/oauth/{}/callback",
                base_url,
                provider.to_string().to_lowercase()
            )
        });

    if client_id.is_empty() || client_secret.is_empty() {
        return None;
    }

    Some(OAuthConfig {
        provider,
        client_id,
        client_secret,
        redirect_uri,
        enabled,
    })
}

/// Get all enabled OAuth providers from config
pub fn get_enabled_providers(
    bot_config: &HashMap<String, String>,
    base_url: &str,
) -> Vec<OAuthConfig> {
    OAuthProvider::all()
        .into_iter()
        .filter_map(|provider| load_oauth_config(provider, bot_config, base_url))
        .filter(|config| config.is_valid())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_auth_url() {
        let config = OAuthConfig::new(
            OAuthProvider::Google,
            "test_client_id".to_string(),
            "test_secret".to_string(),
            "http://localhost:8300/callback".to_string(),
        );

        let url = OAuthProvider::Google.build_auth_url(&config, "test_state");

        assert!(url.starts_with("https://accounts.google.com/o/oauth2/v2/auth?"));
        assert!(url.contains("client_id=test_client_id"));
        assert!(url.contains("state=test_state"));
        assert!(url.contains("response_type=code"));
    }

    #[test]
    fn test_load_oauth_config() {
        let mut bot_config = HashMap::new();
        bot_config.insert("oauth-google-enabled".to_string(), "true".to_string());
        bot_config.insert(
            "oauth-google-client-id".to_string(),
            "my_client_id".to_string(),
        );
        bot_config.insert(
            "oauth-google-client-secret".to_string(),
            "my_secret".to_string(),
        );

        let config = load_oauth_config(OAuthProvider::Google, &bot_config, "http://localhost:8300");

        assert!(config.is_some());
        let config = config.unwrap();
        assert_eq!(config.client_id, "my_client_id");
        assert!(config.redirect_uri.contains("/auth/oauth/google/callback"));
    }

    #[test]
    fn test_disabled_provider() {
        let mut bot_config = HashMap::new();
        bot_config.insert("oauth-google-enabled".to_string(), "false".to_string());
        bot_config.insert(
            "oauth-google-client-id".to_string(),
            "my_client_id".to_string(),
        );
        bot_config.insert(
            "oauth-google-client-secret".to_string(),
            "my_secret".to_string(),
        );

        let config = load_oauth_config(OAuthProvider::Google, &bot_config, "http://localhost:8300");

        assert!(config.is_none());
    }
}
