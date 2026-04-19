//! Snapchat Marketing API Integration
//!
//! Provides Snap Ads management, audience targeting, and content publishing capabilities.
//! Supports OAuth 2.0 authentication flow for Snapchat Marketing API.

use crate::channels::{
    ChannelAccount, ChannelCredentials, ChannelError, ChannelProvider, ChannelType, PostContent,
    PostResult,
};
use serde::{Deserialize, Serialize};

/// Snapchat Marketing API provider
pub struct SnapchatProvider {
    client: reqwest::Client,
    api_base_url: String,
    oauth_base_url: String,
}

impl SnapchatProvider {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            api_base_url: "https://adsapi.snapchat.com/v1".to_string(),
            oauth_base_url: "https://accounts.snapchat.com".to_string(),
        }
    }

    /// Get authenticated user info
    pub async fn get_me(&self, access_token: &str) -> Result<SnapchatUser, ChannelError> {
        let url = format!("{}/me", self.api_base_url);

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

        let result: MeResponse = response.json().await.map_err(|e| ChannelError::ApiError {
            code: None,
            message: e.to_string(),
        })?;

        result.me.ok_or_else(|| ChannelError::ApiError {
            code: None,
            message: "No user data in response".to_string(),
        })
    }

    /// List organizations for the authenticated user
    pub async fn list_organizations(
        &self,
        access_token: &str,
    ) -> Result<Vec<Organization>, ChannelError> {
        let url = format!("{}/me/organizations", self.api_base_url);

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

        let result: OrganizationsResponse =
            response.json().await.map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })?;

        Ok(result
            .organizations
            .into_iter()
            .filter_map(|wrapper| wrapper.organization)
            .collect())
    }

    /// List ad accounts for an organization
    pub async fn list_ad_accounts(
        &self,
        access_token: &str,
        organization_id: &str,
    ) -> Result<Vec<AdAccount>, ChannelError> {
        let url = format!(
            "{}/organizations/{}/adaccounts",
            self.api_base_url, organization_id
        );

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

        let result: AdAccountsResponse =
            response.json().await.map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })?;

        Ok(result
            .adaccounts
            .into_iter()
            .filter_map(|wrapper| wrapper.adaccount)
            .collect())
    }

    /// Create a campaign
    pub async fn create_campaign(
        &self,
        access_token: &str,
        ad_account_id: &str,
        campaign: &CampaignCreateRequest,
    ) -> Result<Campaign, ChannelError> {
        let url = format!(
            "{}/adaccounts/{}/campaigns",
            self.api_base_url, ad_account_id
        );

        let request_body = serde_json::json!({
            "campaigns": [{
                "name": campaign.name,
                "status": campaign.status,
                "objective": campaign.objective,
                "start_time": campaign.start_time,
                "end_time": campaign.end_time,
                "daily_budget_micro": campaign.daily_budget_micro,
                "lifetime_spend_cap_micro": campaign.lifetime_spend_cap_micro
            }]
        });

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

        let result: CampaignsResponse =
            response.json().await.map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })?;

        result
            .campaigns
            .into_iter()
            .next()
            .and_then(|wrapper| wrapper.campaign)
            .ok_or_else(|| ChannelError::ApiError {
                code: None,
                message: "No campaign in response".to_string(),
            })
    }

    /// List campaigns for an ad account
    pub async fn list_campaigns(
        &self,
        access_token: &str,
        ad_account_id: &str,
    ) -> Result<Vec<Campaign>, ChannelError> {
        let url = format!(
            "{}/adaccounts/{}/campaigns",
            self.api_base_url, ad_account_id
        );

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

        let result: CampaignsResponse =
            response.json().await.map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })?;

        Ok(result
            .campaigns
            .into_iter()
            .filter_map(|wrapper| wrapper.campaign)
            .collect())
    }

    /// Create an ad squad (ad set)
    pub async fn create_ad_squad(
        &self,
        access_token: &str,
        campaign_id: &str,
        ad_squad: &AdSquadCreateRequest,
    ) -> Result<AdSquad, ChannelError> {
        let url = format!("{}/campaigns/{}/adsquads", self.api_base_url, campaign_id);

        let request_body = serde_json::json!({
            "adsquads": [{
                "name": ad_squad.name,
                "status": ad_squad.status,
                "type": ad_squad.squad_type,
                "placement_v2": ad_squad.placement,
                "billing_event": ad_squad.billing_event,
                "bid_micro": ad_squad.bid_micro,
                "daily_budget_micro": ad_squad.daily_budget_micro,
                "start_time": ad_squad.start_time,
                "end_time": ad_squad.end_time,
                "optimization_goal": ad_squad.optimization_goal,
                "targeting": ad_squad.targeting
            }]
        });

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

        let result: AdSquadsResponse =
            response.json().await.map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })?;

        result
            .adsquads
            .into_iter()
            .next()
            .and_then(|wrapper| wrapper.adsquad)
            .ok_or_else(|| ChannelError::ApiError {
                code: None,
                message: "No ad squad in response".to_string(),
            })
    }

    /// Create a creative
    pub async fn create_creative(
        &self,
        access_token: &str,
        ad_account_id: &str,
        creative: &CreativeCreateRequest,
    ) -> Result<Creative, ChannelError> {
        let url = format!(
            "{}/adaccounts/{}/creatives",
            self.api_base_url, ad_account_id
        );

        let request_body = serde_json::json!({
            "creatives": [{
                "name": creative.name,
                "type": creative.creative_type,
                "headline": creative.headline,
                "brand_name": creative.brand_name,
                "shareable": creative.shareable,
                "call_to_action": creative.call_to_action,
                "top_snap_media_id": creative.top_snap_media_id,
                "top_snap_crop_position": creative.top_snap_crop_position,
                "longform_video_properties": creative.longform_video_properties,
                "web_view_properties": creative.web_view_properties
            }]
        });

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

        let result: CreativesResponse =
            response.json().await.map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })?;

        result
            .creatives
            .into_iter()
            .next()
            .and_then(|wrapper| wrapper.creative)
            .ok_or_else(|| ChannelError::ApiError {
                code: None,
                message: "No creative in response".to_string(),
            })
    }

    /// Create an ad
    pub async fn create_ad(
        &self,
        access_token: &str,
        ad_squad_id: &str,
        ad: &AdCreateRequest,
    ) -> Result<Ad, ChannelError> {
        let url = format!("{}/adsquads/{}/ads", self.api_base_url, ad_squad_id);

        let request_body = serde_json::json!({
            "ads": [{
                "name": ad.name,
                "status": ad.status,
                "creative_id": ad.creative_id,
                "type": ad.ad_type
            }]
        });

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

        let result: AdsResponse = response.json().await.map_err(|e| ChannelError::ApiError {
            code: None,
            message: e.to_string(),
        })?;

        result
            .ads
            .into_iter()
            .next()
            .and_then(|wrapper| wrapper.ad)
            .ok_or_else(|| ChannelError::ApiError {
                code: None,
                message: "No ad in response".to_string(),
            })
    }

    /// Upload media (initialize upload)
    pub async fn init_media_upload(
        &self,
        access_token: &str,
        ad_account_id: &str,
        media: &MediaUploadRequest,
    ) -> Result<Media, ChannelError> {
        let url = format!("{}/adaccounts/{}/media", self.api_base_url, ad_account_id);

        let request_body = serde_json::json!({
            "media": [{
                "name": media.name,
                "type": media.media_type,
                "ad_account_id": ad_account_id
            }]
        });

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

        let result: MediaResponse = response.json().await.map_err(|e| ChannelError::ApiError {
            code: None,
            message: e.to_string(),
        })?;

        result
            .media
            .into_iter()
            .next()
            .and_then(|wrapper| wrapper.media)
            .ok_or_else(|| ChannelError::ApiError {
                code: None,
                message: "No media in response".to_string(),
            })
    }

    /// Upload media chunk
    pub async fn upload_media_chunk(
        &self,
        access_token: &str,
        ad_account_id: &str,
        media_id: &str,
        chunk_data: &[u8],
        chunk_number: u32,
    ) -> Result<(), ChannelError> {
        let url = format!(
            "{}/adaccounts/{}/media/{}/upload",
            self.api_base_url, ad_account_id, media_id
        );

        let part = reqwest::multipart::Part::bytes(chunk_data.to_vec())
            .file_name("chunk")
            .mime_str("application/octet-stream")
            .map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })?;

        let form = reqwest::multipart::Form::new()
            .part("file", part)
            .text("chunk_number", chunk_number.to_string());

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .multipart(form)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        Ok(())
    }

    /// Complete media upload
    pub async fn complete_media_upload(
        &self,
        access_token: &str,
        ad_account_id: &str,
        media_id: &str,
        total_chunks: u32,
    ) -> Result<Media, ChannelError> {
        let url = format!(
            "{}/adaccounts/{}/media/{}/complete",
            self.api_base_url, ad_account_id, media_id
        );

        let request_body = serde_json::json!({
            "total_chunks": total_chunks
        });

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

        let result: SingleMediaResponse =
            response.json().await.map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })?;

        result.media.ok_or_else(|| ChannelError::ApiError {
            code: None,
            message: "No media in response".to_string(),
        })
    }

    /// Get media status
    pub async fn get_media(
        &self,
        access_token: &str,
        ad_account_id: &str,
        media_id: &str,
    ) -> Result<Media, ChannelError> {
        let url = format!(
            "{}/adaccounts/{}/media/{}",
            self.api_base_url, ad_account_id, media_id
        );

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

        let result: SingleMediaResponse =
            response.json().await.map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })?;

        result.media.ok_or_else(|| ChannelError::ApiError {
            code: None,
            message: "No media in response".to_string(),
        })
    }

    /// Create a custom audience
    pub async fn create_audience(
        &self,
        access_token: &str,
        ad_account_id: &str,
        audience: &AudienceCreateRequest,
    ) -> Result<Audience, ChannelError> {
        let url = format!(
            "{}/adaccounts/{}/segments",
            self.api_base_url, ad_account_id
        );

        let request_body = serde_json::json!({
            "segments": [{
                "name": audience.name,
                "description": audience.description,
                "source_type": audience.source_type,
                "retention_in_days": audience.retention_in_days,
                "ad_account_id": ad_account_id
            }]
        });

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

        let result: AudiencesResponse =
            response.json().await.map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })?;

        result
            .segments
            .into_iter()
            .next()
            .and_then(|wrapper| wrapper.segment)
            .ok_or_else(|| ChannelError::ApiError {
                code: None,
                message: "No audience in response".to_string(),
            })
    }

    /// Get campaign stats
    pub async fn get_campaign_stats(
        &self,
        access_token: &str,
        campaign_id: &str,
        granularity: &str,
        start_time: &str,
        end_time: &str,
    ) -> Result<CampaignStats, ChannelError> {
        let url = format!("{}/campaigns/{}/stats", self.api_base_url, campaign_id);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .query(&[
                ("granularity", granularity),
                ("start_time", start_time),
                ("end_time", end_time),
            ])
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        response
            .json::<CampaignStats>()
            .await
            .map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })
    }

    /// Refresh OAuth token
    pub async fn refresh_oauth_token(
        &self,
        client_id: &str,
        client_secret: &str,
        refresh_token: &str,
    ) -> Result<OAuthTokenResponse, ChannelError> {
        let url = format!("{}/login/oauth2/access_token", self.oauth_base_url);

        let response = self
            .client
            .post(&url)
            .form(&[
                ("client_id", client_id),
                ("client_secret", client_secret),
                ("refresh_token", refresh_token),
                ("grant_type", "refresh_token"),
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
            "{}/login/oauth2/authorize?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}",
            self.oauth_base_url,
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
        let url = format!("{}/login/oauth2/access_token", self.oauth_base_url);

        let response = self
            .client
            .post(&url)
            .form(&[
                ("client_id", client_id),
                ("client_secret", client_secret),
                ("code", code),
                ("grant_type", "authorization_code"),
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
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok());
            return ChannelError::RateLimited { retry_after };
        }

        let error_text = response.text().await.unwrap_or_default();

        if let Ok(error_response) = serde_json::from_str::<SnapchatErrorResponse>(&error_text) {
            return ChannelError::ApiError {
                code: error_response.request_status.clone(),
                message: error_response
                    .debug_message
                    .or(error_response.display_message)
                    .unwrap_or_else(|| error_response.request_status.unwrap_or_default()),
            };
        }

        ChannelError::ApiError {
            code: Some(status.to_string()),
            message: error_text,
        }
    }
}

impl Default for SnapchatProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl ChannelProvider for SnapchatProvider {
    fn channel_type(&self) -> ChannelType {
        ChannelType::Snapchat
    }

    fn max_text_length(&self) -> usize {
        34 // Headline character limit for ads
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
                    "OAuth credentials required for Snapchat".to_string(),
                ))
            }
        };

        // Get required IDs from settings
        let ad_account_id = account
            .settings
            .custom
            .get("ad_account_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ChannelError::ApiError {
                code: None,
                message: "ad_account_id required in settings".to_string(),
            })?;

        account
            .settings
            .custom
            .get("campaign_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ChannelError::ApiError {
                code: None,
                message: "campaign_id required in settings".to_string(),
            })?;

        let ad_squad_id = account
            .settings
            .custom
            .get("ad_squad_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ChannelError::ApiError {
                code: None,
                message: "ad_squad_id required in settings".to_string(),
            })?;

        let text = content.text.as_deref().unwrap_or("");

        // For Snapchat Ads, we need a media file first
        // This is a simplified flow - real implementation would handle media upload
        let media_id = content
            .metadata
            .get("media_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ChannelError::ApiError {
                code: None,
                message: "media_id required for Snapchat ads".to_string(),
            })?;

        // Create creative
        let creative_request = CreativeCreateRequest {
            name: format!("Creative - {}", chrono::Utc::now().format("%Y%m%d%H%M%S")),
            creative_type: "SNAP_AD".to_string(),
            headline: Some(text.chars().take(34).collect()),
            brand_name: content
                .metadata
                .get("brand_name")
                .and_then(|v| v.as_str())
                .map(String::from),
            shareable: Some(true),
            call_to_action: content
                .metadata
                .get("cta")
                .and_then(|v| v.as_str())
                .map(String::from),
            top_snap_media_id: media_id.to_string(),
            top_snap_crop_position: Some("MIDDLE".to_string()),
            longform_video_properties: None,
            web_view_properties: content.link.as_ref().map(|url| WebViewProperties {
                url: url.clone(),
                allow_snap_javascript_sdk: Some(false),
                use_immersive_mode: Some(false),
                deep_link_urls: None,
            }),
        };

        let creative = self
            .create_creative(&access_token, ad_account_id, &creative_request)
            .await?;

        // Create ad
        let ad_request = AdCreateRequest {
            name: format!("Ad - {}", chrono::Utc::now().format("%Y%m%d%H%M%S")),
            status: "ACTIVE".to_string(),
            creative_id: creative.id.clone(),
            ad_type: "SNAP_AD".to_string(),
        };

        let ad = self
            .create_ad(&access_token, ad_squad_id, &ad_request)
            .await?;

        Ok(PostResult::success(ChannelType::Snapchat, ad.id, None))
    }

    async fn validate_credentials(
        &self,
        credentials: &ChannelCredentials,
    ) -> Result<bool, ChannelError> {
        match credentials {
            ChannelCredentials::OAuth { access_token, .. } => {
                match self.get_me(access_token).await {
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
pub struct CampaignCreateRequest {
    pub name: String,
    pub status: String,
    pub objective: String,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub daily_budget_micro: Option<i64>,
    pub lifetime_spend_cap_micro: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdSquadCreateRequest {
    pub name: String,
    pub status: String,
    pub squad_type: String,
    pub placement: Option<PlacementV2>,
    pub billing_event: String,
    pub bid_micro: i64,
    pub daily_budget_micro: Option<i64>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub optimization_goal: String,
    pub targeting: Option<Targeting>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacementV2 {
    pub config: String, // "AUTOMATIC" or "CUSTOM"
    pub platforms: Option<Vec<String>>,
    pub snap_positions: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Targeting {
    pub geos: Option<Vec<GeoTarget>>,
    pub demographics: Option<Vec<Demographic>>,
    pub interests: Option<Vec<Interest>>,
    pub devices: Option<Vec<Device>>,
    pub segments: Option<Vec<SegmentTarget>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoTarget {
    pub country_code: String,
    pub region_id: Option<String>,
    pub metro_id: Option<String>,
    pub postal_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Demographic {
    pub age_groups: Option<Vec<String>>,
    pub genders: Option<Vec<String>>,
    pub languages: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interest {
    pub category_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub os_type: Option<String>,
    pub os_version_min: Option<String>,
    pub os_version_max: Option<String>,
    pub make: Option<Vec<String>>,
    pub connection_type: Option<Vec<String>>,
    pub carrier: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentTarget {
    pub segment_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreativeCreateRequest {
    pub name: String,
    pub creative_type: String,
    pub headline: Option<String>,
    pub brand_name: Option<String>,
    pub shareable: Option<bool>,
    pub call_to_action: Option<String>,
    pub top_snap_media_id: String,
    pub top_snap_crop_position: Option<String>,
    pub longform_video_properties: Option<LongformVideoProperties>,
    pub web_view_properties: Option<WebViewProperties>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongformVideoProperties {
    pub video_media_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebViewProperties {
    pub url: String,
    pub allow_snap_javascript_sdk: Option<bool>,
    pub use_immersive_mode: Option<bool>,
    pub deep_link_urls: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdCreateRequest {
    pub name: String,
    pub status: String,
    pub creative_id: String,
    pub ad_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaUploadRequest {
    pub name: String,
    pub media_type: String, // "VIDEO" or "IMAGE"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudienceCreateRequest {
    pub name: String,
    pub description: Option<String>,
    pub source_type: String,
    pub retention_in_days: Option<i32>,
}

// ============================================================================
// Response Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapchatErrorResponse {
    pub request_status: Option<String>,
    pub request_id: Option<String>,
    pub debug_message: Option<String>,
    pub display_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeResponse {
    pub request_status: Option<String>,
    pub request_id: Option<String>,
    pub me: Option<SnapchatUser>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapchatUser {
    pub id: String,
    pub updated_at: Option<String>,
    pub created_at: Option<String>,
    pub email: Option<String>,
    pub organization_id: Option<String>,
    pub display_name: Option<String>,
    pub member_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationsResponse {
    pub request_status: Option<String>,
    pub request_id: Option<String>,
    pub organizations: Vec<OrganizationWrapper>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationWrapper {
    pub organization: Option<Organization>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub id: String,
    pub updated_at: Option<String>,
    pub created_at: Option<String>,
    pub name: String,
    pub address_line_1: Option<String>,
    pub locality: Option<String>,
    pub administrative_district_level_1: Option<String>,
    pub country: Option<String>,
    pub postal_code: Option<String>,
    pub organization_type: Option<String>,
    pub state: Option<String>,
    pub roles: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdAccountsResponse {
    pub request_status: Option<String>,
    pub request_id: Option<String>,
    pub adaccounts: Vec<AdAccountWrapper>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdAccountWrapper {
    pub adaccount: Option<AdAccount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdAccount {
    pub id: String,
    pub updated_at: Option<String>,
    pub created_at: Option<String>,
    pub name: String,
    pub organization_id: String,
    pub currency: String,
    pub timezone: String,
    pub advertiser: Option<String>,
    pub status: Option<String>,
    pub funding_source_ids: Option<Vec<String>>,
    pub lifetime_spend_cap_micro: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignsResponse {
    pub request_status: Option<String>,
    pub request_id: Option<String>,
    pub campaigns: Vec<CampaignWrapper>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignWrapper {
    pub campaign: Option<Campaign>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Campaign {
    pub id: String,
    pub updated_at: Option<String>,
    pub created_at: Option<String>,
    pub name: String,
    pub ad_account_id: String,
    pub status: String,
    pub objective: String,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub daily_budget_micro: Option<i64>,
    pub lifetime_spend_cap_micro: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdSquadsResponse {
    pub request_status: Option<String>,
    pub request_id: Option<String>,
    pub adsquads: Vec<AdSquadWrapper>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdSquadWrapper {
    pub adsquad: Option<AdSquad>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdSquad {
    pub id: String,
    pub updated_at: Option<String>,
    pub created_at: Option<String>,
    pub name: String,
    pub campaign_id: String,
    pub status: String,
    pub squad_type: Option<String>,
    pub billing_event: Option<String>,
    pub bid_micro: Option<i64>,
    pub daily_budget_micro: Option<i64>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub optimization_goal: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreativesResponse {
    pub request_status: Option<String>,
    pub request_id: Option<String>,
    pub creatives: Vec<CreativeWrapper>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreativeWrapper {
    pub creative: Option<Creative>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Creative {
    pub id: String,
    pub updated_at: Option<String>,
    pub created_at: Option<String>,
    pub name: String,
    pub ad_account_id: String,
    pub creative_type: Option<String>,
    pub headline: Option<String>,
    pub brand_name: Option<String>,
    pub shareable: Option<bool>,
    pub call_to_action: Option<String>,
    pub top_snap_media_id: Option<String>,
    pub top_snap_crop_position: Option<String>,
    pub review_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdsResponse {
    pub request_status: Option<String>,
    pub request_id: Option<String>,
    pub ads: Vec<AdWrapper>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdWrapper {
    pub ad: Option<Ad>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ad {
    pub id: String,
    pub updated_at: Option<String>,
    pub created_at: Option<String>,
    pub name: String,
    pub ad_squad_id: String,
    pub creative_id: String,
    pub status: String,
    pub ad_type: Option<String>,
    pub review_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaResponse {
    pub request_status: Option<String>,
    pub request_id: Option<String>,
    pub media: Vec<MediaWrapper>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaWrapper {
    pub media: Option<Media>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingleMediaResponse {
    pub request_status: Option<String>,
    pub request_id: Option<String>,
    pub media: Option<Media>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Media {
    pub id: String,
    pub updated_at: Option<String>,
    pub created_at: Option<String>,
    pub name: String,
    pub ad_account_id: String,
    pub media_type: String,
    pub media_status: Option<String>,
    pub file_name: Option<String>,
    pub download_link: Option<String>,
    pub duration_secs: Option<f64>,
    pub upload_link: Option<String>,
}

impl Media {
    pub fn is_ready(&self) -> bool {
        self.media_status.as_deref() == Some("READY")
    }

    pub fn is_pending(&self) -> bool {
        self.media_status.as_deref() == Some("PENDING_UPLOAD")
    }

    pub fn is_processing(&self) -> bool {
        self.media_status.as_deref() == Some("PROCESSING")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudiencesResponse {
    pub request_status: Option<String>,
    pub request_id: Option<String>,
    pub segments: Vec<AudienceWrapper>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudienceWrapper {
    pub segment: Option<Audience>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Audience {
    pub id: String,
    pub updated_at: Option<String>,
    pub created_at: Option<String>,
    pub name: String,
    pub ad_account_id: String,
    pub description: Option<String>,
    pub source_type: String,
    pub retention_in_days: Option<i32>,
    pub status: Option<String>,
    pub approximate_number_users: Option<i64>,
    pub upload_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignStats {
    pub request_status: Option<String>,
    pub request_id: Option<String>,
    pub total_stats: Option<Vec<StatsEntry>>,
    pub timeseries_stats: Option<Vec<TimeseriesStats>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsEntry {
    pub id: Option<String>,
    pub stats: Option<Stats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    pub impressions: Option<i64>,
    pub swipes: Option<i64>,
    pub spend: Option<i64>,
    pub quartile_1: Option<i64>,
    pub quartile_2: Option<i64>,
    pub quartile_3: Option<i64>,
    pub view_completion: Option<i64>,
    pub screen_time_millis: Option<i64>,
    pub video_views: Option<i64>,
    pub video_views_time_based: Option<i64>,
    pub video_views_15s: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeseriesStats {
    pub id: Option<String>,
    pub timeseries: Option<Vec<TimeseriesEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeseriesEntry {
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub stats: Option<Stats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub token_type: String,
    pub scope: String,
}

// ============================================================================
// Constants
// ============================================================================

/// Campaign objectives
pub struct CampaignObjectives;

impl CampaignObjectives {
    pub const AWARENESS: &'static str = "AWARENESS";
    pub const APP_INSTALLS: &'static str = "APP_INSTALLS";
    pub const ENGAGEMENT: &'static str = "ENGAGEMENT";
    pub const VIDEO_VIEWS: &'static str = "VIDEO_VIEWS";
    pub const WEB_CONVERSIONS: &'static str = "WEB_CONVERSIONS";
    pub const LEAD_GENERATION: &'static str = "LEAD_GENERATION";
    pub const CATALOG_SALES: &'static str = "CATALOG_SALES";
}

/// Ad status values
pub struct AdStatus;

impl AdStatus {
    pub const ACTIVE: &'static str = "ACTIVE";
    pub const PAUSED: &'static str = "PAUSED";
}

/// Billing events
pub struct BillingEvents;

impl BillingEvents {
    pub const IMPRESSION: &'static str = "IMPRESSION";
    pub const SWIPE: &'static str = "SWIPE";
    pub const VIDEO_VIEW: &'static str = "VIDEO_VIEW";
}

/// Optimization goals
pub struct OptimizationGoals;

impl OptimizationGoals {
    pub const IMPRESSIONS: &'static str = "IMPRESSIONS";
    pub const SWIPES: &'static str = "SWIPES";
    pub const APP_INSTALLS: &'static str = "APP_INSTALLS";
    pub const VIDEO_VIEWS: &'static str = "VIDEO_VIEWS";
    pub const VIDEO_VIEWS_15_SEC: &'static str = "VIDEO_VIEWS_15_SEC";
    pub const USES: &'static str = "USES";
    pub const STORY_OPENS: &'static str = "STORY_OPENS";
    pub const PIXEL_PAGE_VIEW: &'static str = "PIXEL_PAGE_VIEW";
    pub const PIXEL_ADD_TO_CART: &'static str = "PIXEL_ADD_TO_CART";
    pub const PIXEL_PURCHASE: &'static str = "PIXEL_PURCHASE";
    pub const PIXEL_SIGNUP: &'static str = "PIXEL_SIGNUP";
}

/// Call to action types
pub struct CallToAction;

impl CallToAction {
    pub const APPLY_NOW: &'static str = "APPLY_NOW";
    pub const BOOK_NOW: &'static str = "BOOK_NOW";
    pub const BUY_TICKETS: &'static str = "BUY_TICKETS";
    pub const CONTACT_US: &'static str = "CONTACT_US";
    pub const DONATE: &'static str = "DONATE";
    pub const DOWNLOAD: &'static str = "DOWNLOAD";
    pub const GET_NOW: &'static str = "GET_NOW";
    pub const INSTALL_NOW: &'static str = "INSTALL_NOW";
    pub const LEARN_MORE: &'static str = "LEARN_MORE";
    pub const LISTEN: &'static str = "LISTEN";
    pub const MORE: &'static str = "MORE";
    pub const ORDER_NOW: &'static str = "ORDER_NOW";
    pub const PLAY: &'static str = "PLAY";
    pub const READ: &'static str = "READ";
    pub const SHOP_NOW: &'static str = "SHOP_NOW";
    pub const SHOW_TIMES: &'static str = "SHOW_TIMES";
    pub const SIGN_UP: &'static str = "SIGN_UP";
    pub const SUBSCRIBE: &'static str = "SUBSCRIBE";
    pub const USE_APP: &'static str = "USE_APP";
    pub const VIEW: &'static str = "VIEW";
    pub const VIEW_MORE: &'static str = "VIEW_MORE";
    pub const VOTE_NOW: &'static str = "VOTE_NOW";
    pub const WATCH: &'static str = "WATCH";
}

/// Audience source types
pub struct AudienceSourceTypes;

impl AudienceSourceTypes {
    pub const FIRST_PARTY: &'static str = "FIRST_PARTY";
    pub const ENGAGEMENT_SNAPCHAT: &'static str = "ENGAGEMENT_SNAPCHAT";
    pub const PIXEL: &'static str = "PIXEL";
    pub const MOBILE_APP: &'static str = "MOBILE_APP";
    pub const LOOKALIKE: &'static str = "LOOKALIKE";
}
