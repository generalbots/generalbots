//! WeChat API client implementation

use super::types::{AccessTokenResponse, CachedToken, WeChatApiResponse};
use crate::channels::ChannelError;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// WeChat API provider for Official Accounts and Mini Programs
pub struct WeChatProvider {
    pub(crate) client: reqwest::Client,
    pub(crate) api_base_url: String,
    /// Cache for access tokens (app_id -> token info)
    pub(crate) token_cache: Arc<RwLock<HashMap<String, CachedToken>>>,
}

impl WeChatProvider {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            api_base_url: "https://api.weixin.qq.com".to_string(),
            token_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get access token (with caching)
    pub async fn get_access_token(
        &self,
        app_id: &str,
        app_secret: &str,
    ) -> Result<String, ChannelError> {
        // Check cache first
        {
            let cache = self.token_cache.read().await;
            if let Some(cached) = cache.get(app_id) {
                if cached.expires_at > chrono::Utc::now() + chrono::Duration::minutes(5) {
                    return Ok(cached.access_token.clone());
                }
            }
        }

        // Fetch new token
        let url = format!(
            "{}/cgi-bin/token?grant_type=client_credential&appid={}&secret={}",
            self.api_base_url, app_id, app_secret
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        let token_response: AccessTokenResponse =
            response.json().await.map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })?;

        if let Some(errcode) = token_response.errcode {
            if errcode != 0 {
                return Err(ChannelError::ApiError {
                    code: Some(errcode.to_string()),
                    message: token_response.errmsg.unwrap_or_default(),
                });
            }
        }

        let access_token = token_response.access_token.ok_or_else(|| {
            ChannelError::ApiError {
                code: None,
                message: "No access token in response".to_string(),
            }
        })?;

        let expires_in = token_response.expires_in.unwrap_or(7200);
        let expires_at = chrono::Utc::now() + chrono::Duration::seconds(expires_in as i64);

        // Cache the token
        {
            let mut cache = self.token_cache.write().await;
            cache.insert(
                app_id.to_string(),
                CachedToken {
                    access_token: access_token.clone(),
                    expires_at,
                },
            );
        }

        Ok(access_token)
    }

    pub(crate) fn check_error<T>(&self, response: &WeChatApiResponse<T>) -> Result<(), ChannelError> {
        if let Some(errcode) = response.errcode {
            if errcode != 0 {
                return Err(ChannelError::ApiError {
                    code: Some(errcode.to_string()),
                    message: response.errmsg.clone().unwrap_or_default(),
                });
            }
        }
        Ok(())
    }

    pub(crate) async fn parse_error_response(&self, response: reqwest::Response) -> ChannelError {
        let status = response.status();

        if status.as_u16() == 401 {
            return ChannelError::AuthenticationFailed("Invalid credentials".to_string());
        }

        let error_text = response.text().await.unwrap_or_default();

        if let Ok(api_response) = serde_json::from_str::<WeChatApiResponse<()>>(&error_text) {
            if let Some(errcode) = api_response.errcode {
                return ChannelError::ApiError {
                    code: Some(errcode.to_string()),
                    message: api_response.errmsg.unwrap_or_default(),
                };
            }
        }

        ChannelError::ApiError {
            code: Some(status.to_string()),
            message: error_text,
        }
    }
}

impl Default for WeChatProvider {
    fn default() -> Self {
        Self::new()
    }
}
