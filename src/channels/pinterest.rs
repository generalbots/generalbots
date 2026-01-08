//! Pinterest API v5 Integration
//!
//! Provides Pin creation, Board management, and Advertising capabilities.
//! Supports OAuth 2.0 authentication flow for Pinterest API.

use crate::channels::{
    ChannelAccount, ChannelCredentials, ChannelError, ChannelProvider, ChannelType, PostContent,
    PostResult,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Pinterest API v5 provider for pins, boards, and ads
pub struct PinterestProvider {
    client: reqwest::Client,
    api_base_url: String,
    oauth_base_url: String,
}

impl PinterestProvider {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            api_base_url: "https://api.pinterest.com/v5".to_string(),
            oauth_base_url: "https://api.pinterest.com/v5/oauth".to_string(),
        }
    }

    /// Get authenticated user info
    pub async fn get_user(&self, access_token: &str) -> Result<PinterestUser, ChannelError> {
        let url = format!("{}/user_account", self.api_base_url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        response
            .json::<PinterestUser>()
            .await
            .map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })
    }

    /// Create a new pin
    pub async fn create_pin(
        &self,
        access_token: &str,
        pin: &PinCreateRequest,
    ) -> Result<Pin, ChannelError> {
        let url = format!("{}/pins", self.api_base_url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .json(pin)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        response.json::<Pin>().await.map_err(|e| ChannelError::ApiError {
            code: None,
            message: e.to_string(),
        })
    }

    /// Get a pin by ID
    pub async fn get_pin(
        &self,
        access_token: &str,
        pin_id: &str,
    ) -> Result<Pin, ChannelError> {
        let url = format!("{}/pins/{}", self.api_base_url, pin_id);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        response.json::<Pin>().await.map_err(|e| ChannelError::ApiError {
            code: None,
            message: e.to_string(),
        })
    }

    /// Delete a pin
    pub async fn delete_pin(
        &self,
        access_token: &str,
        pin_id: &str,
    ) -> Result<(), ChannelError> {
        let url = format!("{}/pins/{}", self.api_base_url, pin_id);

        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if response.status().as_u16() == 204 {
            return Ok(());
        }

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        Ok(())
    }

    /// Save a pin to a board
    pub async fn save_pin(
        &self,
        access_token: &str,
        pin_id: &str,
        board_id: &str,
        board_section_id: Option<&str>,
    ) -> Result<Pin, ChannelError> {
        let url = format!("{}/pins/{}/save", self.api_base_url, pin_id);

        let mut request_body = serde_json::json!({
            "board_id": board_id
        });

        if let Some(section_id) = board_section_id {
            request_body["board_section_id"] = serde_json::json!(section_id);
        }

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        response.json::<Pin>().await.map_err(|e| ChannelError::ApiError {
            code: None,
            message: e.to_string(),
        })
    }

    /// Create a new board
    pub async fn create_board(
        &self,
        access_token: &str,
        board: &BoardCreateRequest,
    ) -> Result<Board, ChannelError> {
        let url = format!("{}/boards", self.api_base_url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .json(board)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        response
            .json::<Board>()
            .await
            .map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })
    }

    /// List user's boards
    pub async fn list_boards(
        &self,
        access_token: &str,
        options: &BoardListOptions,
    ) -> Result<BoardListResponse, ChannelError> {
        let url = format!("{}/boards", self.api_base_url);

        let mut query_params = vec![];

        if let Some(page_size) = options.page_size {
            query_params.push(("page_size", page_size.to_string()));
        }

        if let Some(bookmark) = &options.bookmark {
            query_params.push(("bookmark", bookmark.clone()));
        }

        if let Some(privacy) = &options.privacy {
            query_params.push(("privacy", privacy.clone()));
        }

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .query(&query_params)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        response
            .json::<BoardListResponse>()
            .await
            .map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })
    }

    /// Get a board by ID
    pub async fn get_board(
        &self,
        access_token: &str,
        board_id: &str,
    ) -> Result<Board, ChannelError> {
        let url = format!("{}/boards/{}", self.api_base_url, board_id);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        response
            .json::<Board>()
            .await
            .map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })
    }

    /// Update a board
    pub async fn update_board(
        &self,
        access_token: &str,
        board_id: &str,
        update: &BoardUpdateRequest,
    ) -> Result<Board, ChannelError> {
        let url = format!("{}/boards/{}", self.api_base_url, board_id);

        let response = self
            .client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .json(update)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        response
            .json::<Board>()
            .await
            .map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })
    }

    /// Delete a board
    pub async fn delete_board(
        &self,
        access_token: &str,
        board_id: &str,
    ) -> Result<(), ChannelError> {
        let url = format!("{}/boards/{}", self.api_base_url, board_id);

        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if response.status().as_u16() == 204 {
            return Ok(());
        }

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        Ok(())
    }

    /// List pins on a board
    pub async fn list_board_pins(
        &self,
        access_token: &str,
        board_id: &str,
        options: &PinListOptions,
    ) -> Result<PinListResponse, ChannelError> {
        let url = format!("{}/boards/{}/pins", self.api_base_url, board_id);

        let mut query_params = vec![];

        if let Some(page_size) = options.page_size {
            query_params.push(("page_size", page_size.to_string()));
        }

        if let Some(bookmark) = &options.bookmark {
            query_params.push(("bookmark", bookmark.clone()));
        }

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .query(&query_params)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        response
            .json::<PinListResponse>()
            .await
            .map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })
    }

    /// Create a board section
    pub async fn create_board_section(
        &self,
        access_token: &str,
        board_id: &str,
        section: &BoardSectionCreateRequest,
    ) -> Result<BoardSection, ChannelError> {
        let url = format!("{}/boards/{}/sections", self.api_base_url, board_id);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .json(section)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        response
            .json::<BoardSection>()
            .await
            .map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })
    }

    /// List board sections
    pub async fn list_board_sections(
        &self,
        access_token: &str,
        board_id: &str,
        options: &PaginationOptions,
    ) -> Result<BoardSectionListResponse, ChannelError> {
        let url = format!("{}/boards/{}/sections", self.api_base_url, board_id);

        let mut query_params = vec![];

        if let Some(page_size) = options.page_size {
            query_params.push(("page_size", page_size.to_string()));
        }

        if let Some(bookmark) = &options.bookmark {
            query_params.push(("bookmark", bookmark.clone()));
        }

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .query(&query_params)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        response
            .json::<BoardSectionListResponse>()
            .await
            .map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })
    }

    /// Search pins
    pub async fn search_pins(
        &self,
        access_token: &str,
        query: &str,
        options: &SearchOptions,
    ) -> Result<PinSearchResponse, ChannelError> {
        let url = format!("{}/search/pins", self.api_base_url);

        let mut query_params = vec![("query", query.to_string())];

        if let Some(page_size) = options.page_size {
            query_params.push(("page_size", page_size.to_string()));
        }

        if let Some(bookmark) = &options.bookmark {
            query_params.push(("bookmark", bookmark.clone()));
        }

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .query(&query_params)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        response
            .json::<PinSearchResponse>()
            .await
            .map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })
    }

    // ========================================================================
    // Advertising API
    // ========================================================================

    /// List ad accounts
    pub async fn list_ad_accounts(
        &self,
        access_token: &str,
        options: &PaginationOptions,
    ) -> Result<AdAccountListResponse, ChannelError> {
        let url = format!("{}/ad_accounts", self.api_base_url);

        let mut query_params = vec![];

        if let Some(page_size) = options.page_size {
            query_params.push(("page_size", page_size.to_string()));
        }

        if let Some(bookmark) = &options.bookmark {
            query_params.push(("bookmark", bookmark.clone()));
        }

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .query(&query_params)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        response
            .json::<AdAccountListResponse>()
            .await
            .map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })
    }

    /// Create a campaign
    pub async fn create_campaign(
        &self,
        access_token: &str,
        ad_account_id: &str,
        campaign: &CampaignCreateRequest,
    ) -> Result<Campaign, ChannelError> {
        let url = format!(
            "{}/ad_accounts/{}/campaigns",
            self.api_base_url, ad_account_id
        );

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .json(campaign)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        response
            .json::<Campaign>()
            .await
            .map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })
    }

    /// List campaigns
    pub async fn list_campaigns(
        &self,
        access_token: &str,
        ad_account_id: &str,
        options: &CampaignListOptions,
    ) -> Result<CampaignListResponse, ChannelError> {
        let url = format!(
            "{}/ad_accounts/{}/campaigns",
            self.api_base_url, ad_account_id
        );

        let mut query_params = vec![];

        if let Some(page_size) = options.page_size {
            query_params.push(("page_size", page_size.to_string()));
        }

        if let Some(bookmark) = &options.bookmark {
            query_params.push(("bookmark", bookmark.clone()));
        }

        if let Some(order) = &options.order {
            query_params.push(("order", order.clone()));
        }

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .query(&query_params)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        response
            .json::<CampaignListResponse>()
            .await
            .map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })
    }

    /// Create an ad group
    pub async fn create_ad_group(
        &self,
        access_token: &str,
        ad_account_id: &str,
        ad_group: &AdGroupCreateRequest,
    ) -> Result<AdGroup, ChannelError> {
        let url = format!(
            "{}/ad_accounts/{}/ad_groups",
            self.api_base_url, ad_account_id
        );

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .json(ad_group)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        response
            .json::<AdGroup>()
            .await
            .map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })
    }

    /// Create an ad (Promoted Pin)
    pub async fn create_ad(
        &self,
        access_token: &str,
        ad_account_id: &str,
        ad: &AdCreateRequest,
    ) -> Result<Ad, ChannelError> {
        let url = format!("{}/ad_accounts/{}/ads", self.api_base_url, ad_account_id);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .json(ad)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        response.json::<Ad>().await.map_err(|e| ChannelError::ApiError {
            code: None,
            message: e.to_string(),
        })
    }

    /// Get analytics for a pin
    pub async fn get_pin_analytics(
        &self,
        access_token: &str,
        pin_id: &str,
        start_date: &str,
        end_date: &str,
        metric_types: &[&str],
    ) -> Result<PinAnalytics, ChannelError> {
        let url = format!("{}/pins/{}/analytics", self.api_base_url, pin_id);

        let metrics = metric_types.join(",");

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .query(&[
                ("start_date", start_date),
                ("end_date", end_date),
                ("metric_types", &metrics),
                ("app_types", "ALL"),
                ("split_field", "NO_SPLIT"),
            ])
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        response
            .json::<PinAnalytics>()
            .await
            .map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })
    }

    /// Get user account analytics
    pub async fn get_user_analytics(
        &self,
        access_token: &str,
        start_date: &str,
        end_date: &str,
        metric_types: &[&str],
    ) -> Result<UserAnalytics, ChannelError> {
        let url = format!("{}/user_account/analytics", self.api_base_url);

        let metrics = metric_types.join(",");

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .query(&[
                ("start_date", start_date),
                ("end_date", end_date),
                ("metric_types", &metrics),
                ("app_types", "ALL"),
                ("split_field", "NO_SPLIT"),
            ])
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        response
            .json::<UserAnalytics>()
            .await
            .map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })
    }

    // ========================================================================
    // OAuth
    // ========================================================================

    /// Refresh OAuth token
    pub async fn refresh_oauth_token(
        &self,
        client_id: &str,
        client_secret: &str,
        refresh_token: &str,
    ) -> Result<OAuthTokenResponse, ChannelError> {
        let url = format!("{}/token", self.oauth_base_url);

        let credentials = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            format!("{}:{}", client_id, client_secret),
        );

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Basic {}", credentials))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token),
            ])
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        response
            .json::<OAuthTokenResponse>()
            .await
            .map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })
    }

    /// Generate OAuth authorization URL
    pub fn get_authorization_url(
        &self,
        client_id: &str,
        redirect_uri: &str,
        scope: &str,
        state: &str,
    ) -> String {
        format!(
            "https://www.pinterest.com/oauth/?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}",
            client_id,
            urlencoding::encode(redirect_uri),
            urlencoding::encode(scope),
            urlencoding::encode(state)
        )
    }

    /// Exchange authorization code for access token
    pub async fn exchange_code_for_token(
        &self,
        client_id: &str,
        client_secret: &str,
        code: &str,
        redirect_uri: &str,
    ) -> Result<OAuthTokenResponse, ChannelError> {
        let url = format!("{}/token", self.oauth_base_url);

        let credentials = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            format!("{}:{}", client_id, client_secret),
        );

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Basic {}", credentials))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", code),
                ("redirect_uri", redirect_uri),
            ])
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        response
            .json::<OAuthTokenResponse>()
            .await
            .map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })
    }

    async fn parse_error_response(&self, response: reqwest::Response) -> ChannelError {
        let status = response.status();

        if status.as_u16() == 401 {
            return ChannelError::AuthenticationFailed("Invalid or expired token".to_string());
        }

        if status.as_u16() == 429 {
            let retry_after = response
                .headers()
                .get("x-ratelimit-reset")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok());
            return ChannelError::RateLimited { retry_after };
        }

        let error_text = response.text().await.unwrap_or_default();

        if let Ok(error_response) = serde_json::from_str::<PinterestErrorResponse>(&error_text) {
            return ChannelError::ApiError {
                code: Some(error_response.code.to_string()),
                message: error_response.message,
            };
        }

        ChannelError::ApiError {
            code: Some(status.to_string()),
            message: error_text,
        }
    }
}

impl Default for PinterestProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl ChannelProvider for PinterestProvider {
    fn channel_type(&self) -> ChannelType {
        ChannelType::Pinterest
    }

    fn max_text_length(&self) -> usize {
        500 // Pin description limit
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
        let access_token = match &account.credentials {
            ChannelCredentials::OAuth { access_token, .. } => access_token.clone(),
            _ => {
                return Err(ChannelError::AuthenticationFailed(
                    "OAuth credentials required for Pinterest".to_string(),
                ))
            }
        };

        let text = content.text.as_deref().unwrap_or("");

        if text.len() > self.max_text_length() {
            return Err(ChannelError::ContentTooLong {
                max_length: self.max_text_length(),
                actual_length: text.len(),
            });
        }

        // Get board_id from settings or metadata
        let board_id = content
            .metadata
            .get("board_id")
            .and_then(|v| v.as_str())
            .or_else(|| {
                account
                    .settings
                    .custom
                    .get("default_board_id")
                    .and_then(|v| v.as_str())
            })
            .ok_or_else(|| ChannelError::ApiError {
                code: None,
                message: "board_id required for Pinterest pin".to_string(),
            })?;

        // Build pin request
        let media_source = if let Some(image_url) = content.image_urls.first() {
            MediaSource::ImageUrl {
                url: image_url.clone(),
            }
        } else if let Some(video_url) = &content.video_url {
            MediaSource::VideoId {
                cover_image_url: content
                    .metadata
                    .get("cover_image_url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                media_id: video_url.clone(),
            }
        } else {
            return Err(ChannelError::ApiError {
                code: None,
                message: "Pinterest requires an image or video".to_string(),
            });
        };

        let pin_request = PinCreateRequest {
            board_id: board_id.to_string(),
            title: content
                .metadata
                .get("title")
                .and_then(|v| v.as_str())
                .map(String::from),
            description: Some(text.to_string()),
            link: content.link.clone(),
            alt_text: content
                .metadata
                .get("alt_text")
                .and_then(|v| v.as_str())
                .map(String::from),
            board_section_id: content
                .metadata
                .get("board_section_id")
                .and_then(|v| v.as_str())
                .map(String::from),
            media_source,
        };

        let pin = self.create_pin(&access_token, &pin_request).await?;

        let url = format!("https://www.pinterest.com/pin/{}/", pin.id);

        Ok(PostResult::success(ChannelType::Pinterest, pin.id, Some(url)))
    }

    async fn validate_credentials(
        &self,
        credentials: &ChannelCredentials,
    ) -> Result<bool, ChannelError> {
        match credentials {
            ChannelCredentials::OAuth { access_token, .. } => {
                match self.get_user(access_token).await {
                    Ok(_) => Ok(true),
                    Err(ChannelError::AuthenticationFailed(_)) => Ok(false),
                    Err(e) => Err(e),
                }
            }
            _ => Ok(false),
        }
    }

    async fn refresh_token(&self, account: &mut ChannelAccount) -> Result<(), ChannelError> {
        let (refresh_token, client_id, client_secret) = match &account.credentials {
            ChannelCredentials::OAuth { refresh_token, .. } => {
                let refresh = refresh_token.as_ref().ok_or_else(|| {
                    ChannelError::AuthenticationFailed("No refresh token available".to_string())
                })?;
                let client_id = account
                    .settings
                    .custom
                    .get("client_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ChannelError::AuthenticationFailed("Missing client_id".to_string())
                    })?;
                let client_secret = account
                    .settings
                    .custom
                    .get("client_secret")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ChannelError::AuthenticationFailed("Missing client_secret".to_string())
                    })?;
                (
                    refresh.clone(),
                    client_id.to_string(),
                    client_secret.to_string(),
                )
            }
            _ => {
                return Err(ChannelError::AuthenticationFailed(
                    "OAuth credentials required".to_string(),
                ))
            }
        };

        let token_response = self
            .refresh_oauth_token(&client_id, &client_secret, &refresh_token)
            .await?;

        let expires_at =
            chrono::Utc::now() + chrono::Duration::seconds(token_response.expires_in as i64);

        account.credentials = ChannelCredentials::OAuth {
            access_token: token_response.access_token,
            refresh_token: Some(token_response.refresh_token),
            expires_at: Some(expires_at),
            scope: Some(token_response.scope),
        };

        Ok(())
    }
}

// ============================================================================
// Request Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinCreateRequest {
    pub board_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub board_section_id: Option<String>,
    pub media_source: MediaSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "source_type", rename_all = "snake_case")]
pub enum MediaSource {
    #[serde(rename = "image_url")]
    ImageUrl { url: String },
    #[serde(rename = "video_id")]
    VideoId {
        cover_image_url: String,
        media_id: String,
    },
    #[serde(rename = "multiple_image_urls")]
    MultipleImageUrls { items: Vec<ImageItem> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageItem {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardCreateRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privacy: Option<String>, // "PUBLIC" or "SECRET"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardUpdateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privacy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardSectionCreateRequest {
    pub name: String,
}

#[derive(Debug, Clone, Default)]
pub struct BoardListOptions {
    pub page_size: Option<u32>,
    pub bookmark: Option<String>,
    pub privacy: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct PinListOptions {
    pub page_size: Option<u32>,
    pub bookmark: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct PaginationOptions {
    pub page_size: Option<u32>,
    pub bookmark: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SearchOptions {
    pub page_size: Option<u32>,
    pub bookmark: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct CampaignListOptions {
    pub page_size: Option<u32>,
    pub bookmark: Option<String>,
    pub order: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignCreateRequest {
    pub ad_account_id: String,
    pub name: String,
    pub status: String,
    pub objective_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub daily_spend_cap: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifetime_spend_cap: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdGroupCreateRequest {
    pub ad_account_id: String,
    pub campaign_id: String,
    pub name: String,
    pub status: String,
    pub budget_in_micro_currency: i64,
    pub bid_in_micro_currency: Option<i64>,
    pub optimization_goal_metadata: Option<serde_json::Value>,
    pub targeting_spec: Option<TargetingSpec>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetingSpec {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age_bucket: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geo: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interest: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyword: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdCreateRequest {
    pub ad_group_id: String,
    pub creative_type: String,
    pub pin_id: String,
    pub name: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking_urls: Option<TrackingUrls>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingUrls {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub impression: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub click: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engagement: Option<Vec<String>>,
}

// ============================================================================
// Response Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinterestErrorResponse {
    pub code: i32,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinterestUser {
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pin {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub board_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub board_section_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media: Option<PinMedia>,
}

impl Pin {
    pub fn url(&self) -> String {
        format!("https://www.pinterest.com/pin/{}/", self.id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinMedia {
    pub media_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<HashMap<String, ImageInfo>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageInfo {
    pub url: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privacy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pin_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub follower_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<BoardOwner>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardOwner {
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardSection {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardListResponse {
    pub items: Vec<Board>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bookmark: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinListResponse {
    pub items: Vec<Pin>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bookmark: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardSectionListResponse {
    pub items: Vec<BoardSection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bookmark: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinSearchResponse {
    pub items: Vec<Pin>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bookmark: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdAccount {
    pub id: String,
    pub name: String,
    pub currency: String,
    pub country: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<AdAccountOwner>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdAccountOwner {
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdAccountListResponse {
    pub items: Vec<AdAccount>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bookmark: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Campaign {
    pub id: String,
    pub ad_account_id: String,
    pub name: String,
    pub status: String,
    pub objective_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub daily_spend_cap: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifetime_spend_cap: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_time: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignListResponse {
    pub items: Vec<Campaign>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bookmark: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdGroup {
    pub id: String,
    pub ad_account_id: String,
    pub campaign_id: String,
    pub name: String,
    pub status: String,
    pub budget_in_micro_currency: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bid_in_micro_currency: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_time: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ad {
    pub id: String,
    pub ad_group_id: String,
    pub creative_type: String,
    pub pin_id: String,
    pub name: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_time: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinAnalytics {
    pub all: Option<AnalyticsData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAnalytics {
    pub all: Option<AnalyticsData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsData {
    pub daily_metrics: Option<Vec<DailyMetric>>,
    pub summary_metrics: Option<HashMap<String, f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyMetric {
    pub date: String,
    pub data_status: String,
    pub metrics: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub scope: String,
}

// ============================================================================
// Constants
// ============================================================================

/// Board privacy options
pub struct BoardPrivacy;

impl BoardPrivacy {
    pub const PUBLIC: &'static str = "PUBLIC";
    pub const SECRET: &'static str = "SECRET";
}

/// Campaign objective types
pub struct CampaignObjective;

impl CampaignObjective {
    pub const AWARENESS: &'static str = "AWARENESS";
    pub const CONSIDERATION: &'static str = "CONSIDERATION";
    pub const VIDEO_VIEW: &'static str = "VIDEO_VIEW";
    pub const WEB_CONVERSION: &'static str = "WEB_CONVERSION";
    pub const CATALOG_SALES: &'static str = "CATALOG_SALES";
    pub const SHOPPING: &'static str = "SHOPPING";
}

/// Ad status values
pub struct AdStatus;

impl AdStatus {
    pub const ACTIVE: &'static str = "ACTIVE";
    pub const PAUSED: &'static str = "PAUSED";
    pub const ARCHIVED: &'static str = "ARCHIVED";
}

/// Creative types
pub struct CreativeType;

impl CreativeType {
    pub const REGULAR: &'static str = "REGULAR";
    pub const VIDEO: &'static str = "VIDEO";
    pub const SHOPPING: &'static str = "SHOPPING";
    pub const CAROUSEL: &'static str = "CAROUSEL";
    pub const COLLECTION: &'static str = "COLLECTION";
    pub const IDEA: &'static str = "IDEA";
}

/// Analytics metric types
pub struct AnalyticsMetrics;

impl AnalyticsMetrics {
    pub const IMPRESSION: &'static str = "IMPRESSION";
    pub const SAVE: &'static str = "SAVE";
    pub const PIN_CLICK: &'static str = "PIN_CLICK";
    pub const OUTBOUND_CLICK: &'static str = "OUTBOUND_CLICK";
    pub const VIDEO_MRC_VIEW: &'static str = "VIDEO_MRC_VIEW";
    pub const VIDEO_AVG_WATCH_TIME: &'static str = "VIDEO_AVG_WATCH_TIME";
    pub const VIDEO_V50_WATCH_TIME: &'static str = "VIDEO_V50_WATCH_TIME";
    pub const QUARTILE_95_PERCENT_VIEW: &'static str = "QUARTILE_95_PERCENT_VIEW";
}

/// OAuth scopes
pub struct PinterestScopes;

impl PinterestScopes {
    pub const BOARDS_READ: &'static str = "boards:read";
    pub const BOARDS_WRITE: &'static str = "boards:write";
    pub const PINS_READ: &'static str = "pins:read";
    pub const PINS_WRITE: &'static str = "pins:write";
    pub const USER_ACCOUNTS_READ: &'static str = "user_accounts:read";
    pub const ADS_READ: &'static str = "ads:read";
    pub const ADS_WRITE: &'static str = "ads:write";

    /// Get recommended scopes for content posting
    pub fn content_posting_scopes() -> &'static str {
        "boards:read,boards:write,pins:read,pins:write,user_accounts:read"
    }

    /// Get all available scopes
    pub fn all_scopes() -> &'static str {
        "boards:read,boards:write,pins:read,pins:write,user_accounts:read,ads:read,ads:write"
    }
}
