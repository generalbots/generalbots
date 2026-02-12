//! YouTube Data API v3 Provider Implementation
//!
//! Provides video upload, community posts, and channel management capabilities.
//! Supports OAuth 2.0 authentication flow.

use crate::channels::{
    ChannelAccount, ChannelCredentials, ChannelError, ChannelProvider, ChannelType, PostContent,
    PostResult,
};
use super::types::*;
use super::models::VideoMetadata;
use super::client::parse_error_response;

/// YouTube API provider for video uploads and community posts
pub struct YouTubeProvider {
    client: reqwest::Client,
    api_base_url: String,
    upload_base_url: String,
    oauth_base_url: String,
}

impl YouTubeProvider {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            api_base_url: "https://www.googleapis.com/youtube/v3".to_string(),
            upload_base_url: "https://www.googleapis.com/upload/youtube/v3".to_string(),
            oauth_base_url: "https://oauth2.googleapis.com".to_string(),
        }
    }

    /// Upload a video to YouTube
    pub async fn upload_video(
        &self,
        access_token: &str,
        video: &VideoUploadRequest,
        video_data: &[u8],
    ) -> Result<YouTubeVideo, ChannelError> {
        // Step 1: Initialize resumable upload
        let init_url = format!(
            "{}/videos?uploadType=resumable&part=snippet,status,contentDetails",
            self.upload_base_url
        );

        let metadata = VideoMetadata::from_request(video);

        let init_response = self
            .client
            .post(&init_url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .header("X-Upload-Content-Type", &video.content_type)
            .header("X-Upload-Content-Length", video_data.len().to_string())
            .json(&metadata)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !init_response.status().is_success() {
            return Err(parse_error_response(init_response).await);
        }

        let upload_url = init_response
            .headers()
            .get("location")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| ChannelError::ApiError {
                code: None,
                message: "Missing upload URL in response".to_string(),
            })?
            .to_string();

        // Step 2: Upload video data
        let upload_response = self
            .client
            .put(&upload_url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", &video.content_type)
            .header("Content-Length", video_data.len().to_string())
            .body(video_data.to_vec())
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !upload_response.status().is_success() {
            return Err(parse_error_response(upload_response).await);
        }

        upload_response
            .json::<YouTubeVideo>()
            .await
            .map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })
    }

    /// Create a community post (text, poll, image, or video)
    pub async fn create_community_post(
        &self,
        access_token: &str,
        post: &CommunityPostRequest,
    ) -> Result<CommunityPost, ChannelError> {
        // Note: Community Posts API is limited and may require additional permissions
        let url = format!("{}/activities", self.api_base_url);

        let request_body = serde_json::json!({
            "snippet": {
                "description": post.text,
                "channelId": post.channel_id
            },
            "contentDetails": {
                "bulletin": {
                    "resourceId": post.attached_video_id.as_ref().map(|vid| {
                        serde_json::json!({
                            "kind": "youtube#video",
                            "videoId": vid
                        })
                    })
                }
            }
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .query(&[("part", "snippet,contentDetails")])
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(parse_error_response(response).await);
        }

        response
            .json::<CommunityPost>()
            .await
            .map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })
    }

    /// Get channel information
    pub async fn get_channel(&self, access_token: &str) -> Result<YouTubeChannel, ChannelError> {
        let url = format!("{}/channels", self.api_base_url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .query(&[
                ("part", "snippet,contentDetails,statistics,status,brandingSettings"),
                ("mine", "true"),
            ])
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(parse_error_response(response).await);
        }

        let list_response: ChannelListResponse = response.json().await.map_err(|e| {
            ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            }
        })?;

        list_response.items.into_iter().next().ok_or_else(|| {
            ChannelError::ApiError {
                code: None,
                message: "No channel found".to_string(),
            }
        })
    }

    /// Get channel by ID
    pub async fn get_channel_by_id(
        &self,
        access_token: &str,
        channel_id: &str,
    ) -> Result<YouTubeChannel, ChannelError> {
        let url = format!("{}/channels", self.api_base_url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .query(&[
                ("part", "snippet,contentDetails,statistics,status"),
                ("id", channel_id),
            ])
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(parse_error_response(response).await);
        }

        let list_response: ChannelListResponse = response.json().await.map_err(|e| {
            ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            }
        })?;

        list_response.items.into_iter().next().ok_or_else(|| {
            ChannelError::ApiError {
                code: None,
                message: "Channel not found".to_string(),
            }
        })
    }

    /// List videos from a channel or playlist
    pub async fn list_videos(
        &self,
        access_token: &str,
        options: &VideoListOptions,
    ) -> Result<VideoListResponse, ChannelError> {
        let url = format!("{}/search", self.api_base_url);

        let mut query_params = vec![
            ("part", "snippet".to_string()),
            ("type", "video".to_string()),
            ("maxResults", options.max_results.unwrap_or(25).to_string()),
        ];

        if let Some(channel_id) = &options.channel_id {
            query_params.push(("channelId", channel_id.clone()));
        }

        if options.for_mine.unwrap_or(false) {
            query_params.push(("forMine", "true".to_string()));
        }

        if let Some(order) = &options.order {
            query_params.push(("order", order.clone()));
        }

        if let Some(page_token) = &options.page_token {
            query_params.push(("pageToken", page_token.clone()));
        }

        if let Some(published_after) = &options.published_after {
            query_params.push(("publishedAfter", published_after.clone()));
        }

        if let Some(published_before) = &options.published_before {
            query_params.push(("publishedBefore", published_before.clone()));
        }

        let query_refs: Vec<(&str, &str)> = query_params
            .iter()
            .map(|(k, v)| (*k, v.as_str()))
            .collect();

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .query(&query_refs)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(parse_error_response(response).await);
        }

        response.json().await.map_err(|e| ChannelError::ApiError {
            code: None,
            message: e.to_string(),
        })
    }

    /// Get video details by ID
    pub async fn get_video(
        &self,
        access_token: &str,
        video_id: &str,
    ) -> Result<YouTubeVideo, ChannelError> {
        let url = format!("{}/videos", self.api_base_url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .query(&[
                ("part", "snippet,contentDetails,statistics,status,player"),
                ("id", video_id),
            ])
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(parse_error_response(response).await);
        }

        let list_response: YouTubeVideoListResponse =
            response.json().await.map_err(|e| ChannelError::ApiError {
                code: None,
                message: e.to_string(),
            })?;

        list_response.items.into_iter().next().ok_or_else(|| {
            ChannelError::ApiError {
                code: None,
                message: "Video not found".to_string(),
            }
        })
    }

    /// Update video metadata
    pub async fn update_video(
        &self,
        access_token: &str,
        video_id: &str,
        update: &VideoUpdateRequest,
    ) -> Result<YouTubeVideo, ChannelError> {
        let url = format!("{}/videos", self.api_base_url);

        let update_body = serde_json::json!({
            "id": video_id,
            "snippet": {
                "title": update.title,
                "description": update.description,
                "tags": update.tags,
                "categoryId": update.category_id
            },
            "status": {
                "privacyStatus": update.privacy_status,
                "embeddable": update.embeddable,
                "publicStatsViewable": update.public_stats_viewable
            }
        });

        let response = self
            .client
            .put(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .query(&[("part", "snippet,status")])
            .json(&update_body)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(parse_error_response(response).await);
        }

        response.json().await.map_err(|e| ChannelError::ApiError {
            code: None,
            message: e.to_string(),
        })
    }

    /// Delete a video
    pub async fn delete_video(
        &self,
        access_token: &str,
        video_id: &str,
    ) -> Result<(), ChannelError> {
        let url = format!("{}/videos", self.api_base_url);

        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .query(&[("id", video_id)])
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if response.status().as_u16() == 204 {
            return Ok(());
        }

        if !response.status().is_success() {
            return Err(parse_error_response(response).await);
        }

        Ok(())
    }

    /// Create a playlist
    pub async fn create_playlist(
        &self,
        access_token: &str,
        playlist: &PlaylistCreateRequest,
    ) -> Result<YouTubePlaylist, ChannelError> {
        let url = format!("{}/playlists", self.api_base_url);

        let request_body = serde_json::json!({
            "snippet": {
                "title": playlist.title,
                "description": playlist.description,
                "tags": playlist.tags,
                "defaultLanguage": playlist.default_language
            },
            "status": {
                "privacyStatus": playlist.privacy_status
            }
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .query(&[("part", "snippet,status")])
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(parse_error_response(response).await);
        }

        response.json().await.map_err(|e| ChannelError::ApiError {
            code: None,
            message: e.to_string(),
        })
    }

    /// Add video to playlist
    pub async fn add_video_to_playlist(
        &self,
        access_token: &str,
        playlist_id: &str,
        video_id: &str,
        position: Option<u32>,
    ) -> Result<PlaylistItem, ChannelError> {
        let url = format!("{}/playlistItems", self.api_base_url);

        let mut request_body = serde_json::json!({
            "snippet": {
                "playlistId": playlist_id,
                "resourceId": {
                    "kind": "youtube#video",
                    "videoId": video_id
                }
            }
        });

        if let Some(pos) = position {
            request_body["snippet"]["position"] = serde_json::json!(pos);
        }

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .query(&[("part", "snippet")])
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(parse_error_response(response).await);
        }

        response.json().await.map_err(|e| ChannelError::ApiError {
            code: None,
            message: e.to_string(),
        })
    }

    /// Remove video from playlist
    pub async fn remove_from_playlist(
        &self,
        access_token: &str,
        playlist_item_id: &str,
    ) -> Result<(), ChannelError> {
        let url = format!("{}/playlistItems", self.api_base_url);

        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .query(&[("id", playlist_item_id)])
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if response.status().as_u16() == 204 {
            return Ok(());
        }

        if !response.status().is_success() {
            return Err(parse_error_response(response).await);
        }

        Ok(())
    }

    /// Set video thumbnail
    pub async fn set_thumbnail(
        &self,
        access_token: &str,
        video_id: &str,
        image_data: &[u8],
        content_type: &str,
    ) -> Result<ThumbnailSetResponse, ChannelError> {
        let url = format!("{}/thumbnails/set", self.upload_base_url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", content_type)
            .query(&[("videoId", video_id)])
            .body(image_data.to_vec())
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(parse_error_response(response).await);
        }

        response.json().await.map_err(|e| ChannelError::ApiError {
            code: None,
            message: e.to_string(),
        })
    }

    /// Add a comment to a video
    pub async fn add_comment(
        &self,
        access_token: &str,
        video_id: &str,
        comment_text: &str,
    ) -> Result<CommentThread, ChannelError> {
        let url = format!("{}/commentThreads", self.api_base_url);

        let request_body = serde_json::json!({
            "snippet": {
                "videoId": video_id,
                "topLevelComment": {
                    "snippet": {
                        "textOriginal": comment_text
                    }
                }
            }
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .query(&[("part", "snippet")])
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(parse_error_response(response).await);
        }

        response.json().await.map_err(|e| ChannelError::ApiError {
            code: None,
            message: e.to_string(),
        })
    }

    /// Reply to a comment
    pub async fn reply_to_comment(
        &self,
        access_token: &str,
        parent_id: &str,
        reply_text: &str,
    ) -> Result<Comment, ChannelError> {
        let url = format!("{}/comments", self.api_base_url);

        let request_body = serde_json::json!({
            "snippet": {
                "parentId": parent_id,
                "textOriginal": reply_text
            }
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .query(&[("part", "snippet")])
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(parse_error_response(response).await);
        }

        response.json().await.map_err(|e| ChannelError::ApiError {
            code: None,
            message: e.to_string(),
        })
    }

    /// Get video comments
    pub async fn get_comments(
        &self,
        access_token: &str,
        video_id: &str,
        page_token: Option<&str>,
        max_results: Option<u32>,
    ) -> Result<CommentThreadListResponse, ChannelError> {
        let url = format!("{}/commentThreads", self.api_base_url);

        let mut query_params = vec![
            ("part", "snippet,replies"),
            ("videoId", video_id),
        ];

        let max_results_str = max_results.unwrap_or(20).to_string();
        query_params.push(("maxResults", &max_results_str));

        if let Some(token) = page_token {
            query_params.push(("pageToken", token));
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
            return Err(parse_error_response(response).await);
        }

        response.json().await.map_err(|e| ChannelError::ApiError {
            code: None,
            message: e.to_string(),
        })
    }

    /// Get channel analytics (requires YouTube Analytics API)
    pub async fn get_analytics(
        &self,
        access_token: &str,
        options: &AnalyticsRequest,
    ) -> Result<AnalyticsResponse, ChannelError> {
        let url = "https://youtubeanalytics.googleapis.com/v2/reports";

        let metrics = options
            .metrics
            .as_deref()
            .unwrap_or("views,estimatedMinutesWatched,averageViewDuration,subscribersGained");

        let response = self
            .client
            .get(url)
            .header("Authorization", format!("Bearer {}", access_token))
            .query(&[
                ("ids", format!("channel=={}", options.channel_id).as_str()),
                ("startDate", &options.start_date),
                ("endDate", &options.end_date),
                ("metrics", metrics),
                ("dimensions", options.dimensions.as_deref().unwrap_or("day")),
            ])
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(parse_error_response(response).await);
        }

        response.json().await.map_err(|e| ChannelError::ApiError {
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
        let url = format!("{}/token", self.oauth_base_url);

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
            return Err(parse_error_response(response).await);
        }

        response.json().await.map_err(|e| ChannelError::ApiError {
            code: None,
            message: e.to_string(),
        })
    }

    /// Subscribe to a channel
    pub async fn subscribe(
        &self,
        access_token: &str,
        channel_id: &str,
    ) -> Result<Subscription, ChannelError> {
        let url = format!("{}/subscriptions", self.api_base_url);

        let request_body = serde_json::json!({
            "snippet": {
                "resourceId": {
                    "kind": "youtube#channel",
                    "channelId": channel_id
                }
            }
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .query(&[("part", "snippet")])
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(parse_error_response(response).await);
        }

        response.json().await.map_err(|e| ChannelError::ApiError {
            code: None,
            message: e.to_string(),
        })
    }

    /// Create a live broadcast
    pub async fn create_live_broadcast(
        &self,
        access_token: &str,
        broadcast: &LiveBroadcastRequest,
    ) -> Result<LiveBroadcast, ChannelError> {
        let url = format!("{}/liveBroadcasts", self.api_base_url);

        let request_body = serde_json::json!({
            "snippet": {
                "title": broadcast.title,
                "description": broadcast.description,
                "scheduledStartTime": broadcast.scheduled_start_time
            },
            "status": {
                "privacyStatus": broadcast.privacy_status
            },
            "contentDetails": {
                "enableAutoStart": broadcast.enable_auto_start,
                "enableAutoStop": broadcast.enable_auto_stop,
                "enableDvr": broadcast.enable_dvr,
                "enableEmbed": broadcast.enable_embed,
                "recordFromStart": broadcast.record_from_start
            }
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .query(&[("part", "snippet,status,contentDetails")])
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ChannelError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(parse_error_response(response).await);
        }

        response.json().await.map_err(|e| ChannelError::ApiError {
            code: None,
            message: e.to_string(),
        })
    }
}

impl Default for YouTubeProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl ChannelProvider for YouTubeProvider {
    fn channel_type(&self) -> ChannelType {
        ChannelType::YouTube
    }

    fn max_text_length(&self) -> usize {
        5000 // Max description length for videos
    }

    fn supports_images(&self) -> bool {
        true // Thumbnails
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
                    "OAuth credentials required for YouTube".to_string(),
                ))
            }
        };

        let text = content.text.as_deref().unwrap_or("");

        // Get channel ID for community post
        let channel = self.get_channel(&access_token).await?;

        // Create community post with the content
        let post_request = CommunityPostRequest {
            channel_id: channel.id.clone(),
            text: text.to_string(),
            attached_video_id: content
                .metadata
                .get("video_id")
                .and_then(|v| v.as_str())
                .map(String::from),
            image_urls: content.image_urls.clone(),
        };

        let post = self.create_community_post(&access_token, &post_request).await?;

        let url = format!("https://www.youtube.com/post/{}", post.id);

        Ok(PostResult::success(ChannelType::YouTube, post.id, Some(url)))
    }

    async fn validate_credentials(
        &self,
        credentials: &ChannelCredentials,
    ) -> Result<bool, ChannelError> {
        match credentials {
            ChannelCredentials::OAuth { access_token, .. } => {
                match self.get_channel(access_token).await {
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
                (refresh.clone(), client_id.to_string(), client_secret.to_string())
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

        let expires_at = chrono::Utc::now()
            + chrono::Duration::seconds(token_response.expires_in.unwrap_or(3600) as i64);

        account.credentials = ChannelCredentials::OAuth {
            access_token: token_response.access_token,
            refresh_token: token_response.refresh_token.or(Some(refresh_token)),
            expires_at: Some(expires_at),
            scope: token_response.scope,
        };

        Ok(())
    }
}
