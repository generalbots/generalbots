//! TikTok Content Posting API Integration
//!
//! Provides video upload, publishing, and user management capabilities.
//! Supports OAuth 2.0 authentication flow for TikTok Login Kit.

use crate::channels::{
    ChannelAccount, ChannelCredentials, ChannelError, ChannelProvider, ChannelType, PostContent,
    PostResult,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// TikTok API provider for video uploads and content posting
pub struct TikTokProvider {
    client: reqwest::Client,
    api_base_url: String,
    oauth_base_url: String,
}

impl TikTokProvider {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            api_base_url: "https://open.tiktokapis.com/v2".to_string(),
            oauth_base_url: "https://open.tiktokapis.com/v2/oauth".to_string(),
        }
    }

    /// Initialize a video upload (Direct Post)
    /// Returns upload URL and publish_id for tracking
    pub async fn init_video_upload(
        &self,
        access_token: &str,
        request: &VideoUploadRequest,
    ) -> Result<VideoUploadInit, ChannelError> {
        let url = format!("{}/post/publish/video/init/", self.api_base_url);

        let post_info = serde_json::json!({
            "title": request.title,
            "privacy_level": request.privacy_level,
            "disable_duet": request.disable_duet.unwrap_or(false),
            "disable_comment": request.disable_comment.unwrap_or(false),
            "disable_stitch": request.disable_stitch.unwrap_or(false),
            "video_cover_timestamp_ms": request.video_cover_timestamp_ms.unwrap_or(0),
            "brand_content_toggle": request.brand_content_toggle.unwrap_or(false),
            "brand_organic_toggle": request.brand_organic_toggle.unwrap_or(false)
        });

        let source_info = if let Some(url) = &request.video_url {
            serde_json::json!({
                "source": "PULL_FROM_URL",
                "video_url": url
            })
        } else {
            serde_json::json!({
                "source": "FILE_UPLOAD",
                "video_size": request.video_size.unwrap_or(0),
                "chunk_size": request.chunk_size.unwrap_or(10_000_000),
                "total_chunk_count": request.total_chunk_count.unwrap_or(1)
            })
        };

        let request_body = serde_json::json!({
            "post_info": post_info,
            "source_info": source_info
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json; charset=UTF-8")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        let api_response: TikTokApiResponse<VideoUploadInit> =
            response.json().await.map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })?;

        api_response.data.ok_or_else(|| ChannelError::ApiError {
            code: api_response.error.as_ref().map(|e| e.code.clone()),
            message: api_response
                .error
                .map(|e| e.message)
                .unwrap_or_else(|| "Unknown error".to_string()),
        })
    }

    /// Upload video chunk (for FILE_UPLOAD source)
    pub async fn upload_video_chunk(
        &self,
        upload_url: &str,
        chunk_data: &[u8],
        chunk_index: u32,
        total_chunks: u32,
    ) -> Result<(), ChannelError> {
        let content_range = if total_chunks == 1 {
            format!("bytes 0-{}/{}", chunk_data.len() - 1, chunk_data.len())
        } else {
            let start = chunk_index as usize * chunk_data.len();
            let end = start + chunk_data.len() - 1;
            let total = total_chunks as usize * chunk_data.len();
            format!("bytes {}-{}/{}", start, end, total)
        };

        let response = self
            .client
            .put(upload_url)
            .header("Content-Type", "video/mp4")
            .header("Content-Length", chunk_data.len().to_string())
            .header("Content-Range", content_range)
            .body(chunk_data.to_vec())
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        Ok(())
    }

    /// Check the status of a video upload/publish
    pub async fn get_publish_status(
        &self,
        access_token: &str,
        publish_id: &str,
    ) -> Result<PublishStatus, ChannelError> {
        let url = format!("{}/post/publish/status/fetch/", self.api_base_url);

        let request_body = serde_json::json!({
            "publish_id": publish_id
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json; charset=UTF-8")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        let api_response: TikTokApiResponse<PublishStatus> =
            response.json().await.map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })?;

        api_response.data.ok_or_else(|| ChannelError::ApiError {
            code: api_response.error.as_ref().map(|e| e.code.clone()),
            message: api_response
                .error
                .map(|e| e.message)
                .unwrap_or_else(|| "Unknown error".to_string()),
        })
    }

    /// Initialize a photo post
    pub async fn init_photo_post(
        &self,
        access_token: &str,
        request: &PhotoPostRequest,
    ) -> Result<PhotoUploadInit, ChannelError> {
        let url = format!("{}/post/publish/content/init/", self.api_base_url);

        let request_body = serde_json::json!({
            "post_info": {
                "title": request.title,
                "description": request.description,
                "privacy_level": request.privacy_level,
                "disable_comment": request.disable_comment.unwrap_or(false),
                "auto_add_music": request.auto_add_music.unwrap_or(true)
            },
            "source_info": {
                "source": "PULL_FROM_URL",
                "photo_cover_index": request.photo_cover_index.unwrap_or(0),
                "photo_images": request.photo_urls
            },
            "post_mode": "DIRECT_POST",
            "media_type": "PHOTO"
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json; charset=UTF-8")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        let api_response: TikTokApiResponse<PhotoUploadInit> =
            response.json().await.map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })?;

        api_response.data.ok_or_else(|| ChannelError::ApiError {
            code: api_response.error.as_ref().map(|e| e.code.clone()),
            message: api_response
                .error
                .map(|e| e.message)
                .unwrap_or_else(|| "Unknown error".to_string()),
        })
    }

    /// Get user info (requires user.info.basic or user.info.profile scope)
    pub async fn get_user_info(
        &self,
        access_token: &str,
        fields: &[&str],
    ) -> Result<TikTokUser, ChannelError> {
        let fields_param = fields.join(",");
        let url = format!("{}/user/info/", self.api_base_url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .query(&[("fields", fields_param)])
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        let api_response: TikTokApiResponse<UserInfoData> =
            response.json().await.map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })?;

        api_response
            .data
            .and_then(|d| d.user)
            .ok_or_else(|| ChannelError::ApiError {
                code: api_response.error.as_ref().map(|e| e.code.clone()),
                message: api_response
                    .error
                    .map(|e| e.message)
                    .unwrap_or_else(|| "User not found".to_string()),
            })
    }

    /// List user videos (requires video.list scope)
    pub async fn list_videos(
        &self,
        access_token: &str,
        options: &VideoListOptions,
    ) -> Result<VideoListResponse, ChannelError> {
        let url = format!("{}/video/list/", self.api_base_url);

        let fields = options.fields.as_deref().unwrap_or(
            "id,create_time,cover_image_url,share_url,video_description,duration,title",
        );

        let mut request_body = serde_json::json!({
            "max_count": options.max_count.unwrap_or(20)
        });

        if let Some(cursor) = &options.cursor {
            request_body["cursor"] = serde_json::json!(cursor);
        }

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json; charset=UTF-8")
            .query(&[("fields", fields)])
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        let api_response: TikTokApiResponse<VideoListResponse> =
            response.json().await.map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })?;

        api_response.data.ok_or_else(|| ChannelError::ApiError {
            code: api_response.error.as_ref().map(|e| e.code.clone()),
            message: api_response
                .error
                .map(|e| e.message)
                .unwrap_or_else(|| "Failed to list videos".to_string()),
        })
    }

    /// Query specific videos by IDs (requires video.list scope)
    pub async fn query_videos(
        &self,
        access_token: &str,
        video_ids: &[String],
        fields: Option<&str>,
    ) -> Result<Vec<TikTokVideo>, ChannelError> {
        let url = format!("{}/video/query/", self.api_base_url);

        let fields_param = fields.unwrap_or(
            "id,create_time,cover_image_url,share_url,video_description,duration,title,like_count,comment_count,share_count,view_count",
        );

        let request_body = serde_json::json!({
            "filters": {
                "video_ids": video_ids
            }
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json; charset=UTF-8")
            .query(&[("fields", fields_param)])
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        let api_response: TikTokApiResponse<VideoQueryResponse> =
            response.json().await.map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })?;

        api_response
            .data
            .map(|d| d.videos)
            .ok_or_else(|| ChannelError::ApiError {
                code: api_response.error.as_ref().map(|e| e.code.clone()),
                message: api_response
                    .error
                    .map(|e| e.message)
                    .unwrap_or_else(|| "Failed to query videos".to_string()),
            })
    }

    /// Get creator info for content posting (requires video.publish scope)
    pub async fn get_creator_info(
        &self,
        access_token: &str,
    ) -> Result<CreatorInfo, ChannelError> {
        let url = format!("{}/post/publish/creator_info/", self.api_base_url);

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

        let api_response: TikTokApiResponse<CreatorInfo> =
            response.json().await.map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })?;

        api_response.data.ok_or_else(|| ChannelError::ApiError {
            code: api_response.error.as_ref().map(|e| e.code.clone()),
            message: api_response
                .error
                .map(|e| e.message)
                .unwrap_or_else(|| "Failed to get creator info".to_string()),
        })
    }

    /// Refresh OAuth access token
    pub async fn refresh_oauth_token(
        &self,
        client_key: &str,
        client_secret: &str,
        refresh_token: &str,
    ) -> Result<OAuthTokenResponse, ChannelError> {
        let url = format!("{}/token/", self.oauth_base_url);

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&[
                ("client_key", client_key),
                ("client_secret", client_secret),
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

    /// Revoke OAuth access token
    pub async fn revoke_token(
        &self,
        client_key: &str,
        client_secret: &str,
        access_token: &str,
    ) -> Result<(), ChannelError> {
        let url = format!("{}/revoke/", self.oauth_base_url);

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&[
                ("client_key", client_key),
                ("client_secret", client_secret),
                ("token", access_token),
            ])
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        Ok(())
    }

    /// Generate OAuth authorization URL
    pub fn get_authorization_url(
        &self,
        client_key: &str,
        redirect_uri: &str,
        scope: &str,
        state: &str,
    ) -> String {
        format!(
            "https://www.tiktok.com/v2/auth/authorize/?client_key={}&scope={}&response_type=code&redirect_uri={}&state={}",
            client_key,
            urlencoding::encode(scope),
            urlencoding::encode(redirect_uri),
            urlencoding::encode(state)
        )
    }

    /// Exchange authorization code for access token
    pub async fn exchange_code_for_token(
        &self,
        client_key: &str,
        client_secret: &str,
        code: &str,
        redirect_uri: &str,
    ) -> Result<OAuthTokenResponse, ChannelError> {
        let url = format!("{}/token/", self.oauth_base_url);

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&[
                ("client_key", client_key),
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
                .get("x-ratelimit-reset")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok());
            return ChannelError::RateLimited { retry_after };
        }

        let error_text = response.text().await.unwrap_or_default();

        if let Ok(api_response) = serde_json::from_str::<TikTokApiResponse<()>>(&error_text) {
            if let Some(error) = api_response.error {
                return ChannelError::ApiError {
                    code: Some(error.code),
                    message: error.message,
                };
            }
        }

        ChannelError::ApiError {
            code: Some(status.to_string()),
            message: error_text,
        }
    }
}

impl Default for TikTokProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl ChannelProvider for TikTokProvider {
    fn channel_type(&self) -> ChannelType {
        ChannelType::TikTok
    }

    fn max_text_length(&self) -> usize {
        2200 // Max caption length
    }

    fn supports_images(&self) -> bool {
        true // Photo mode
    }

    fn supports_video(&self) -> bool {
        true
    }

    fn supports_links(&self) -> bool {
        false // Links not clickable in TikTok captions
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
                    "OAuth credentials required for TikTok".to_string(),
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

        // Determine privacy level from settings or default
        let privacy_level = account
            .settings
            .custom
            .get("default_privacy")
            .and_then(|v| v.as_str())
            .unwrap_or("SELF_ONLY")
            .to_string();

        // Check if we're posting a video or photos
        if let Some(video_url) = &content.video_url {
            // Video post via URL
            let request = VideoUploadRequest {
                title: text.to_string(),
                privacy_level,
                video_url: Some(video_url.clone()),
                disable_duet: Some(false),
                disable_comment: Some(false),
                disable_stitch: Some(false),
                video_cover_timestamp_ms: None,
                brand_content_toggle: None,
                brand_organic_toggle: None,
                video_size: None,
                chunk_size: None,
                total_chunk_count: None,
            };

            let init_result = self.init_video_upload(&access_token, &request).await?;

            Ok(PostResult::success(
                ChannelType::TikTok,
                init_result.publish_id,
                None, // URL not available until processing complete
            ))
        } else if !content.image_urls.is_empty() {
            // Photo post
            let request = PhotoPostRequest {
                title: text.to_string(),
                description: content
                    .metadata
                    .get("description")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                privacy_level,
                photo_urls: content.image_urls.clone(),
                photo_cover_index: Some(0),
                disable_comment: Some(false),
                auto_add_music: Some(true),
            };

            let init_result = self.init_photo_post(&access_token, &request).await?;

            Ok(PostResult::success(
                ChannelType::TikTok,
                init_result.publish_id,
                None,
            ))
        } else {
            Err(ChannelError::ApiError {
                code: None,
                message: "TikTok requires either a video or photos to post".to_string(),
            })
        }
    }

    async fn validate_credentials(
        &self,
        credentials: &ChannelCredentials,
    ) -> Result<bool, ChannelError> {
        match credentials {
            ChannelCredentials::OAuth { access_token, .. } => {
                match self
                    .get_user_info(access_token, &["open_id", "display_name"])
                    .await
                {
                    Ok(_) => Ok(true),
                    Err(ChannelError::AuthenticationFailed(_)) => Ok(false),
                    Err(e) => Err(e),
                }
            }
            _ => Ok(false),
        }
    }

    async fn refresh_token(&self, account: &mut ChannelAccount) -> Result<(), ChannelError> {
        let (refresh_token, client_key, client_secret) = match &account.credentials {
            ChannelCredentials::OAuth { refresh_token, .. } => {
                let refresh = refresh_token.as_ref().ok_or_else(|| {
                    ChannelError::AuthenticationFailed("No refresh token available".to_string())
                })?;
                let client_key = account
                    .settings
                    .custom
                    .get("client_key")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ChannelError::AuthenticationFailed("Missing client_key".to_string())
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
                    client_key.to_string(),
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
            .refresh_oauth_token(&client_key, &client_secret, &refresh_token)
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
pub struct VideoUploadRequest {
    pub title: String,
    pub privacy_level: String, // "PUBLIC_TO_EVERYONE", "MUTUAL_FOLLOW_FRIENDS", "SELF_ONLY", "FOLLOWER_OF_CREATOR"
    pub video_url: Option<String>,
    pub disable_duet: Option<bool>,
    pub disable_comment: Option<bool>,
    pub disable_stitch: Option<bool>,
    pub video_cover_timestamp_ms: Option<u64>,
    pub brand_content_toggle: Option<bool>,
    pub brand_organic_toggle: Option<bool>,
    // For FILE_UPLOAD source
    pub video_size: Option<u64>,
    pub chunk_size: Option<u64>,
    pub total_chunk_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhotoPostRequest {
    pub title: String,
    pub description: Option<String>,
    pub privacy_level: String,
    pub photo_urls: Vec<String>,
    pub photo_cover_index: Option<u32>,
    pub disable_comment: Option<bool>,
    pub auto_add_music: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoListOptions {
    pub cursor: Option<i64>,
    pub max_count: Option<u32>,
    pub fields: Option<String>,
}

// ============================================================================
// Response Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TikTokApiResponse<T> {
    pub data: Option<T>,
    pub error: Option<TikTokError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TikTokError {
    pub code: String,
    pub message: String,
    pub log_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoUploadInit {
    pub publish_id: String,
    pub upload_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhotoUploadInit {
    pub publish_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishStatus {
    pub status: String, // "PROCESSING_UPLOAD", "PROCESSING_DOWNLOAD", "SEND_TO_USER_INBOX", "PUBLISH_COMPLETE", "FAILED"
    #[serde(default)]
    pub fail_reason: Option<String>,
    pub publicly_available_post_id: Option<Vec<String>>,
    pub uploaded_bytes: Option<u64>,
}

impl PublishStatus {
    pub fn is_complete(&self) -> bool {
        self.status == "PUBLISH_COMPLETE"
    }

    pub fn is_failed(&self) -> bool {
        self.status == "FAILED"
    }

    pub fn is_processing(&self) -> bool {
        matches!(
            self.status.as_str(),
            "PROCESSING_UPLOAD" | "PROCESSING_DOWNLOAD" | "SEND_TO_USER_INBOX"
        )
    }

    pub fn get_video_id(&self) -> Option<&str> {
        self.publicly_available_post_id
            .as_ref()
            .and_then(|ids| ids.first())
            .map(|s| s.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfoData {
    pub user: Option<TikTokUser>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TikTokUser {
    pub open_id: String,
    pub union_id: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub avatar_url_100: Option<String>,
    pub avatar_large_url: Option<String>,
    pub bio_description: Option<String>,
    pub profile_deep_link: Option<String>,
    pub is_verified: Option<bool>,
    pub follower_count: Option<u64>,
    pub following_count: Option<u64>,
    pub likes_count: Option<u64>,
    pub video_count: Option<u64>,
}

impl TikTokUser {
    /// Get the user's TikTok profile URL
    pub fn profile_url(&self) -> Option<String> {
        self.profile_deep_link.clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoListResponse {
    pub videos: Vec<TikTokVideo>,
    pub cursor: Option<i64>,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoQueryResponse {
    pub videos: Vec<TikTokVideo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TikTokVideo {
    pub id: String,
    pub create_time: Option<i64>,
    pub cover_image_url: Option<String>,
    pub share_url: Option<String>,
    pub video_description: Option<String>,
    pub duration: Option<u32>,
    pub title: Option<String>,
    pub height: Option<u32>,
    pub width: Option<u32>,
    pub like_count: Option<u64>,
    pub comment_count: Option<u64>,
    pub share_count: Option<u64>,
    pub view_count: Option<u64>,
    pub embed_html: Option<String>,
    pub embed_link: Option<String>,
}

impl TikTokVideo {
    /// Get the video URL
    pub fn url(&self) -> Option<&str> {
        self.share_url.as_deref()
    }

    /// Get video creation time as DateTime
    pub fn created_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.create_time
            .map(|ts| chrono::DateTime::from_timestamp(ts, 0))
            .flatten()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatorInfo {
    pub creator_avatar_url: Option<String>,
    pub creator_username: Option<String>,
    pub creator_nickname: Option<String>,
    pub privacy_level_options: Option<Vec<String>>,
    pub comment_disabled: Option<bool>,
    pub duet_disabled: Option<bool>,
    pub stitch_disabled: Option<bool>,
    pub max_video_post_duration_sec: Option<u32>,
}

impl CreatorInfo {
    /// Check if public posting is available
    pub fn can_post_public(&self) -> bool {
        self.privacy_level_options
            .as_ref()
            .map(|opts| opts.contains(&"PUBLIC_TO_EVERYONE".to_string()))
            .unwrap_or(false)
    }

    /// Get max video duration in seconds
    pub fn max_video_duration(&self) -> u32 {
        self.max_video_post_duration_sec.unwrap_or(60)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub refresh_expires_in: u64,
    pub open_id: String,
    pub scope: String,
    pub token_type: String,
}

// ============================================================================
// Privacy Level Constants
// ============================================================================

/// Privacy levels for TikTok posts
pub struct PrivacyLevel;

impl PrivacyLevel {
    /// Video visible to everyone
    pub const PUBLIC: &'static str = "PUBLIC_TO_EVERYONE";
    /// Video visible only to mutual followers
    pub const FRIENDS: &'static str = "MUTUAL_FOLLOW_FRIENDS";
    /// Video visible only to creator
    pub const PRIVATE: &'static str = "SELF_ONLY";
    /// Video visible to followers only
    pub const FOLLOWERS: &'static str = "FOLLOWER_OF_CREATOR";
}

// ============================================================================
// Scopes
// ============================================================================

/// OAuth scopes for TikTok API
pub struct TikTokScopes;

impl TikTokScopes {
    /// Basic user info (open_id, union_id, avatar)
    pub const USER_INFO_BASIC: &'static str = "user.info.basic";
    /// Extended user profile (bio, verified status, stats)
    pub const USER_INFO_PROFILE: &'static str = "user.info.profile";
    /// User's email address
    pub const USER_INFO_STATS: &'static str = "user.info.stats";
    /// List user's videos
    pub const VIDEO_LIST: &'static str = "video.list";
    /// Upload and publish videos
    pub const VIDEO_PUBLISH: &'static str = "video.publish";
    /// Upload videos to user's inbox
    pub const VIDEO_UPLOAD: &'static str = "video.upload";

    /// Get recommended scopes for content posting
    pub fn content_posting_scopes() -> &'static str {
        "user.info.basic,video.publish"
    }

    /// Get all available scopes
    pub fn all_scopes() -> &'static str {
        "user.info.basic,user.info.profile,user.info.stats,video.list,video.publish,video.upload"
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Build hashtag string from list
pub fn build_hashtags(tags: &[String]) -> String {
    tags.iter()
        .map(|t| {
            if t.starts_with('#') {
                t.clone()
            } else {
                format!("#{}", t)
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Validate video file for TikTok upload
pub fn validate_video_file(size_bytes: u64, duration_seconds: u32) -> Result<(), ChannelError> {
    // TikTok limits: 4GB max, typically 60s for regular users, up to 10 min for some
    const MAX_SIZE: u64 = 4 * 1024 * 1024 * 1024; // 4GB
    const MAX_DURATION: u32 = 600; // 10 minutes

    if size_bytes > MAX_SIZE {
        return Err(ChannelError::ApiError {
            code: Some("VIDEO_TOO_LARGE".to_string()),
            message: format!(
                "Video file too large: {} bytes (max: {} bytes)",
                size_bytes, MAX_SIZE
            ),
        });
    }

    if duration_seconds > MAX_DURATION {
        return Err(ChannelError::ApiError {
            code: Some("VIDEO_TOO_LONG".to_string()),
            message: format!(
                "Video too long: {} seconds (max: {} seconds)",
                duration_seconds, MAX_DURATION
            ),
        });
    }

    Ok(())
}

/// Supported video formats for TikTok
pub struct VideoFormats;

impl VideoFormats {
    pub const MP4: &'static str = "video/mp4";
    pub const WEBM: &'static str = "video/webm";
    pub const MOV: &'static str = "video/quicktime";

    pub fn is_supported(content_type: &str) -> bool {
        matches!(
            content_type.to_lowercase().as_str(),
            "video/mp4" | "video/webm" | "video/quicktime"
        )
    }
}

/// Supported image formats for TikTok photo posts
pub struct ImageFormats;

impl ImageFormats {
    pub const JPEG: &'static str = "image/jpeg";
    pub const PNG: &'static str = "image/png";
    pub const WEBP: &'static str = "image/webp";

    pub fn is_supported(content_type: &str) -> bool {
        matches!(
            content_type.to_lowercase().as_str(),
            "image/jpeg" | "image/png" | "image/webp"
        )
    }
}
