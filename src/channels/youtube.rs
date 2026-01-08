//! YouTube Data API v3 Integration
//!
//! Provides video upload, community posts, and channel management capabilities.
//! Supports OAuth 2.0 authentication flow.

use crate::channels::{
    ChannelAccount, ChannelCredentials, ChannelError, ChannelProvider, ChannelType, PostContent,
    PostResult,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

        let metadata = VideoMetadata {
            snippet: VideoSnippet {
                title: video.title.clone(),
                description: video.description.clone(),
                tags: video.tags.clone(),
                category_id: video.category_id.clone().unwrap_or_else(|| "22".to_string()), // 22 = People & Blogs
                default_language: video.default_language.clone(),
                default_audio_language: video.default_audio_language.clone(),
            },
            status: VideoStatus {
                privacy_status: video.privacy_status.clone(),
                embeddable: video.embeddable.unwrap_or(true),
                license: video.license.clone().unwrap_or_else(|| "youtube".to_string()),
                public_stats_viewable: video.public_stats_viewable.unwrap_or(true),
                publish_at: video.scheduled_publish_at.clone(),
                self_declared_made_for_kids: video.made_for_kids.unwrap_or(false),
            },
        };

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
            return Err(self.parse_error_response(init_response).await);
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
            return Err(self.parse_error_response(upload_response).await);
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
            return Err(self.parse_error_response(response).await);
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
            return Err(self.parse_error_response(response).await);
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
            return Err(self.parse_error_response(response).await);
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
            return Err(self.parse_error_response(response).await);
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
            return Err(self.parse_error_response(response).await);
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
            return Err(self.parse_error_response(response).await);
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
            return Err(self.parse_error_response(response).await);
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
            return Err(self.parse_error_response(response).await);
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
            return Err(self.parse_error_response(response).await);
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
            return Err(self.parse_error_response(response).await);
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
            return Err(self.parse_error_response(response).await);
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
            return Err(self.parse_error_response(response).await);
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
            return Err(self.parse_error_response(response).await);
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
            return Err(self.parse_error_response(response).await);
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
            return Err(self.parse_error_response(response).await);
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
            return Err(self.parse_error_response(response).await);
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
            return Err(self.parse_error_response(response).await);
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
            return Err(self.parse_error_response(response).await);
        }

        response.json().await.map_err(|e| ChannelError::ApiError {
            code: None,
            message: e.to_string(),
        })
    }

    async fn parse_error_response(&self, response: reqwest::Response) -> ChannelError {
        let status = response.status();

        if status.as_u16() == 401 {
            return ChannelError::AuthenticationFailed("Invalid or expired token".to_string());
        }

        if status.as_u16() == 403 {
            return ChannelError::AuthenticationFailed("Insufficient permissions".to_string());
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

        if let Ok(error_response) = serde_json::from_str::<YouTubeErrorResponse>(&error_text) {
            return ChannelError::ApiError {
                code: Some(error_response.error.code.to_string()),
                message: error_response.error.message,
            };
        }

        ChannelError::ApiError {
            code: Some(status.to_string()),
            message: error_text,
        }
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

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoUploadRequest {
    pub title: String,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub category_id: Option<String>,
    pub privacy_status: String, // "private", "public", "unlisted"
    pub content_type: String,   // e.g., "video/mp4"
    pub default_language: Option<String>,
    pub default_audio_language: Option<String>,
    pub embeddable: Option<bool>,
    pub license: Option<String>,
    pub public_stats_viewable: Option<bool>,
    pub scheduled_publish_at: Option<String>,
    pub made_for_kids: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityPostRequest {
    pub channel_id: String,
    pub text: String,
    pub attached_video_id: Option<String>,
    pub image_urls: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoListOptions {
    pub channel_id: Option<String>,
    pub for_mine: Option<bool>,
    pub order: Option<String>, // "date", "rating", "relevance", "title", "viewCount"
    pub page_token: Option<String>,
    pub published_after: Option<String>,
    pub published_before: Option<String>,
    pub max_results: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoUpdateRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub category_id: Option<String>,
    pub privacy_status: Option<String>,
    pub embeddable: Option<bool>,
    pub public_stats_viewable: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistCreateRequest {
    pub title: String,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub default_language: Option<String>,
    pub privacy_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsRequest {
    pub channel_id: String,
    pub start_date: String,
    pub end_date: String,
    pub metrics: Option<String>,
    pub dimensions: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveBroadcastRequest {
    pub title: String,
    pub description: Option<String>,
    pub scheduled_start_time: String,
    pub privacy_status: String,
    pub enable_auto_start: Option<bool>,
    pub enable_auto_stop: Option<bool>,
    pub enable_dvr: Option<bool>,
    pub enable_embed: Option<bool>,
    pub record_from_start: Option<bool>,
}

// ============================================================================
// API Response Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YouTubeVideo {
    pub id: String,
    pub kind: String,
    pub etag: String,
    pub snippet: Option<VideoSnippetResponse>,
    pub content_details: Option<VideoContentDetails>,
    pub statistics: Option<VideoStatistics>,
    pub status: Option<VideoStatusResponse>,
    pub player: Option<VideoPlayer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoSnippetResponse {
    pub title: String,
    pub description: String,
    pub published_at: String,
    pub channel_id: String,
    pub channel_title: String,
    pub thumbnails: Option<Thumbnails>,
    pub tags: Option<Vec<String>>,
    pub category_id: Option<String>,
    pub live_broadcast_content: Option<String>,
    pub default_language: Option<String>,
    pub default_audio_language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoContentDetails {
    pub duration: String,
    pub dimension: String,
    pub definition: String,
    pub caption: Option<String>,
    pub licensed_content: bool,
    pub projection: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoStatistics {
    pub view_count: Option<String>,
    pub like_count: Option<String>,
    pub dislike_count: Option<String>,
    pub favorite_count: Option<String>,
    pub comment_count: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoStatusResponse {
    pub upload_status: String,
    pub privacy_status: String,
    pub license: Option<String>,
    pub embeddable: Option<bool>,
    pub public_stats_viewable: Option<bool>,
    pub made_for_kids: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoPlayer {
    pub embed_html: Option<String>,
    pub embed_width: Option<i64>,
    pub embed_height: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thumbnails {
    pub default: Option<Thumbnail>,
    pub medium: Option<Thumbnail>,
    pub high: Option<Thumbnail>,
    pub standard: Option<Thumbnail>,
    pub maxres: Option<Thumbnail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thumbnail {
    pub url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YouTubeChannel {
    pub id: String,
    pub kind: String,
    pub etag: String,
    pub snippet: Option<ChannelSnippet>,
    pub content_details: Option<ChannelContentDetails>,
    pub statistics: Option<ChannelStatistics>,
    pub status: Option<ChannelStatus>,
    pub branding_settings: Option<BrandingSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelSnippet {
    pub title: String,
    pub description: String,
    pub custom_url: Option<String>,
    pub published_at: String,
    pub thumbnails: Option<Thumbnails>,
    pub default_language: Option<String>,
    pub country: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelContentDetails {
    pub related_playlists: Option<RelatedPlaylists>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelatedPlaylists {
    pub likes: Option<String>,
    pub uploads: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelStatistics {
    pub view_count: Option<String>,
    pub subscriber_count: Option<String>,
    pub hidden_subscriber_count: bool,
    pub video_count: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelStatus {
    pub privacy_status: String,
    pub is_linked: Option<bool>,
    pub long_uploads_status: Option<String>,
    pub made_for_kids: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrandingSettings {
    pub channel: Option<ChannelBranding>,
    pub image: Option<ImageBranding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelBranding {
    pub title: Option<String>,
    pub description: Option<String>,
    pub keywords: Option<String>,
    pub default_tab: Option<String>,
    pub country: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageBranding {
    pub banner_external_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YouTubePlaylist {
    pub id: String,
    pub kind: String,
    pub etag: String,
    pub snippet: Option<PlaylistSnippet>,
    pub status: Option<PlaylistStatus>,
    pub content_details: Option<PlaylistContentDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistSnippet {
    pub title: String,
    pub description: String,
    pub published_at: String,
    pub channel_id: String,
    pub channel_title: String,
    pub thumbnails: Option<Thumbnails>,
    pub default_language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistStatus {
    pub privacy_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistContentDetails {
    pub item_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistItem {
    pub id: String,
    pub kind: String,
    pub etag: String,
    pub snippet: Option<PlaylistItemSnippet>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistItemSnippet {
    pub playlist_id: String,
    pub position: u32,
    pub resource_id: ResourceId,
    pub title: String,
    pub description: String,
    pub thumbnails: Option<Thumbnails>,
    pub channel_id: String,
    pub channel_title: String,
    pub published_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceId {
    pub kind: String,
    pub video_id: Option<String>,
    pub channel_id: Option<String>,
    pub playlist_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommunityPost {
    pub id: String,
    pub kind: String,
    pub etag: String,
    pub snippet: Option<CommunityPostSnippet>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommunityPostSnippet {
    pub channel_id: String,
    pub description: String,
    pub published_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentThread {
    pub id: String,
    pub kind: String,
    pub etag: String,
    pub snippet: Option<CommentThreadSnippet>,
    pub replies: Option<CommentReplies>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentThreadSnippet {
    pub channel_id: String,
    pub video_id: String,
    pub top_level_comment: Comment,
    pub can_reply: bool,
    pub total_reply_count: u32,
    pub is_public: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    pub id: String,
    pub kind: String,
    pub etag: String,
    pub snippet: Option<CommentSnippet>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentSnippet {
    pub video_id: Option<String>,
    pub text_display: String,
    pub text_original: String,
    pub author_display_name: String,
    pub author_profile_image_url: Option<String>,
    pub author_channel_url: Option<String>,
    pub author_channel_id: Option<AuthorChannelId>,
    pub can_rate: bool,
    pub viewer_rating: Option<String>,
    pub like_count: u32,
    pub published_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorChannelId {
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentReplies {
    pub comments: Vec<Comment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Subscription {
    pub id: String,
    pub kind: String,
    pub etag: String,
    pub snippet: Option<SubscriptionSnippet>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionSnippet {
    pub published_at: String,
    pub title: String,
    pub description: String,
    pub resource_id: ResourceId,
    pub channel_id: String,
    pub thumbnails: Option<Thumbnails>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveBroadcast {
    pub id: String,
    pub kind: String,
    pub etag: String,
    pub snippet: Option<LiveBroadcastSnippet>,
    pub status: Option<LiveBroadcastStatus>,
    pub content_details: Option<LiveBroadcastContentDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveBroadcastSnippet {
    pub published_at: String,
    pub channel_id: String,
    pub title: String,
    pub description: String,
    pub thumbnails: Option<Thumbnails>,
    pub scheduled_start_time: Option<String>,
    pub scheduled_end_time: Option<String>,
    pub actual_start_time: Option<String>,
    pub actual_end_time: Option<String>,
    pub is_default_broadcast: bool,
    pub live_chat_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveBroadcastStatus {
    pub life_cycle_status: String,
    pub privacy_status: String,
    pub recording_status: Option<String>,
    pub made_for_kids: Option<bool>,
    pub self_declared_made_for_kids: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveBroadcastContentDetails {
    pub bound_stream_id: Option<String>,
    pub bound_stream_last_update_time_ms: Option<String>,
    pub enable_closed_captions: Option<bool>,
    pub enable_content_encryption: Option<bool>,
    pub enable_dvr: Option<bool>,
    pub enable_embed: Option<bool>,
    pub enable_auto_start: Option<bool>,
    pub enable_auto_stop: Option<bool>,
    pub record_from_start: Option<bool>,
    pub start_with_slate: Option<bool>,
    pub projection: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailSetResponse {
    pub kind: String,
    pub etag: String,
    pub items: Vec<ThumbnailItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThumbnailItem {
    pub default: Option<Thumbnail>,
    pub medium: Option<Thumbnail>,
    pub high: Option<Thumbnail>,
    pub standard: Option<Thumbnail>,
    pub maxres: Option<Thumbnail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsResponse {
    pub kind: String,
    pub column_headers: Vec<ColumnHeader>,
    pub rows: Option<Vec<Vec<serde_json::Value>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnHeader {
    pub name: String,
    pub column_type: String,
    pub data_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<u64>,
    pub token_type: String,
    pub scope: Option<String>,
}

// ============================================================================
// List Response Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelListResponse {
    pub kind: String,
    pub etag: String,
    pub page_info: Option<PageInfo>,
    pub items: Vec<YouTubeChannel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YouTubeVideoListResponse {
    pub kind: String,
    pub etag: String,
    pub page_info: Option<PageInfo>,
    pub items: Vec<YouTubeVideo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoListResponse {
    pub kind: String,
    pub etag: String,
    pub next_page_token: Option<String>,
    pub prev_page_token: Option<String>,
    pub page_info: Option<PageInfo>,
    pub items: Vec<VideoSearchResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoSearchResult {
    pub kind: String,
    pub etag: String,
    pub id: VideoSearchId,
    pub snippet: Option<VideoSnippetResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoSearchId {
    pub kind: String,
    pub video_id: Option<String>,
    pub channel_id: Option<String>,
    pub playlist_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentThreadListResponse {
    pub kind: String,
    pub etag: String,
    pub next_page_token: Option<String>,
    pub page_info: Option<PageInfo>,
    pub items: Vec<CommentThread>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageInfo {
    pub total_results: u32,
    pub results_per_page: u32,
}

// ============================================================================
// Internal Types
// ============================================================================

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct VideoMetadata {
    snippet: VideoSnippet,
    status: VideoStatus,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct VideoSnippet {
    title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<Vec<String>>,
    category_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    default_language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default_audio_language: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct VideoStatus {
    privacy_status: String,
    embeddable: bool,
    license: String,
    public_stats_viewable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    publish_at: Option<String>,
    self_declared_made_for_kids: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct YouTubeErrorResponse {
    error: YouTubeError,
}

#[derive(Debug, Clone, Deserialize)]
struct YouTubeError {
    code: u16,
    message: String,
    #[serde(default)]
    errors: Vec<YouTubeErrorDetail>,
}

#[derive(Debug, Clone, Deserialize)]
struct YouTubeErrorDetail {
    message: String,
    domain: String,
    reason: String,
}

// ============================================================================
// Helper Functions
// ============================================================================

impl YouTubeVideo {
    /// Get the video URL
    pub fn url(&self) -> String {
        format!("https://www.youtube.com/watch?v={}", self.id)
    }

    /// Get the embed URL
    pub fn embed_url(&self) -> String {
        format!("https://www.youtube.com/embed/{}", self.id)
    }

    /// Get the thumbnail URL (high quality)
    pub fn thumbnail_url(&self) -> Option<String> {
        self.snippet
            .as_ref()
            .and_then(|s| s.thumbnails.as_ref())
            .and_then(|t| {
                t.high
                    .as_ref()
                    .or(t.medium.as_ref())
                    .or(t.default.as_ref())
            })
            .map(|t| t.url.clone())
    }
}

impl YouTubeChannel {
    /// Get the channel URL
    pub fn url(&self) -> String {
        if let Some(snippet) = &self.snippet {
            if let Some(custom_url) = &snippet.custom_url {
                return format!("https://www.youtube.com/{}", custom_url);
            }
        }
        format!("https://www.youtube.com/channel/{}", self.id)
    }
}

/// Video categories commonly used on YouTube
pub struct VideoCategories;

impl VideoCategories {
    pub const FILM_AND_ANIMATION: &'static str = "1";
    pub const AUTOS_AND_VEHICLES: &'static str = "2";
    pub const MUSIC: &'static str = "10";
    pub const PETS_AND_ANIMALS: &'static str = "15";
    pub const SPORTS: &'static str = "17";
    pub const TRAVEL_AND_EVENTS: &'static str = "19";
    pub const GAMING: &'static str = "20";
    pub const PEOPLE_AND_BLOGS: &'static str = "22";
    pub const COMEDY: &'static str = "23";
    pub const ENTERTAINMENT: &'static str = "24";
    pub const NEWS_AND_POLITICS: &'static str = "25";
    pub const HOWTO_AND_STYLE: &'static str = "26";
    pub const EDUCATION: &'static str = "27";
    pub const SCIENCE_AND_TECHNOLOGY: &'static str = "28";
    pub const NONPROFITS_AND_ACTIVISM: &'static str = "29";
}

/// Privacy status options for videos and playlists
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivacyStatus {
    Public,
    Private,
    Unlisted,
}

impl PrivacyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Private => "private",
            Self::Unlisted => "unlisted",
        }
    }
}

impl std::fmt::Display for PrivacyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
