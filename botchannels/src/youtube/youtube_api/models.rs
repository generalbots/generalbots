//! Internal Models for YouTube API
//!
//! Contains internal types used for API requests that are not exposed publicly.

use serde::{Deserialize, Serialize};
use super::types::VideoUploadRequest;

/// Internal metadata structure for video uploads
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoMetadata {
    pub snippet: VideoSnippet,
    pub status: VideoStatus,
}

impl VideoMetadata {
    /// Create VideoMetadata from a VideoUploadRequest
    pub fn from_request(request: &VideoUploadRequest) -> Self {
        Self {
            snippet: VideoSnippet {
                title: request.title.clone(),
                description: request.description.clone(),
                tags: request.tags.clone(),
                category_id: request
                    .category_id
                    .clone()
                    .unwrap_or_else(|| "22".to_string()), // 22 = People & Blogs
                default_language: request.default_language.clone(),
                default_audio_language: request.default_audio_language.clone(),
            },
            status: VideoStatus {
                privacy_status: request.privacy_status.clone(),
                embeddable: request.embeddable.unwrap_or(true),
                license: request.license.clone().unwrap_or_else(|| "youtube".to_string()),
                public_stats_viewable: request.public_stats_viewable.unwrap_or(true),
                publish_at: request.scheduled_publish_at.clone(),
                self_declared_made_for_kids: request.made_for_kids.unwrap_or(false),
            },
        }
    }
}

/// Internal snippet structure for video uploads
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoSnippet {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    pub category_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_audio_language: Option<String>,
}

/// Internal status structure for video uploads
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoStatus {
    pub privacy_status: String,
    pub embeddable: bool,
    pub license: String,
    pub public_stats_viewable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publish_at: Option<String>,
    pub self_declared_made_for_kids: bool,
}

/// Error response from YouTube API
#[derive(Debug, Clone, Deserialize)]
pub struct YouTubeErrorResponse {
    pub error: YouTubeError,
}

/// YouTube error details
#[derive(Debug, Clone, Deserialize)]
pub struct YouTubeError {
    pub code: u16,
    pub message: String,
}
