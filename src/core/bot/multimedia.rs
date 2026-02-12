












use crate::core::shared::message_types::MessageType;
use crate::core::shared::models::{BotResponse, UserMessage};
use anyhow::Result;
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MultimediaMessage {
    Text {
        content: String,
    },
    Image {
        url: String,
        caption: Option<String>,
        mime_type: String,
    },
    Video {
        url: String,
        thumbnail_url: Option<String>,
        caption: Option<String>,
        duration: Option<u32>,
        mime_type: String,
    },
    Audio {
        url: String,
        duration: Option<u32>,
        mime_type: String,
    },
    Document {
        url: String,
        filename: String,
        mime_type: String,
    },
    WebSearch {
        query: String,
        results: Vec<SearchResult>,
    },
    Location {
        latitude: f64,
        longitude: f64,
        name: Option<String>,
        address: Option<String>,
    },
    MeetingInvite {
        meeting_id: String,
        meeting_url: String,
        start_time: Option<String>,
        duration: Option<u32>,
        participants: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub thumbnail: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MediaUploadRequest {
    pub file_name: String,
    pub content_type: String,
    pub data: Vec<u8>,
    pub user_id: String,
    pub session_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MediaUploadResponse {
    pub media_id: String,
    pub url: String,
    pub thumbnail_url: Option<String>,
}


#[async_trait]
pub trait MultimediaHandler: Send + Sync {

    async fn process_multimedia(
        &self,
        message: MultimediaMessage,
        user_id: &str,
        session_id: &str,
    ) -> Result<BotResponse>;


    async fn upload_media(&self, request: MediaUploadRequest) -> Result<MediaUploadResponse>;


    async fn download_media(&self, url: &str) -> Result<Vec<u8>>;


    async fn web_search(&self, query: &str, max_results: usize) -> Result<Vec<SearchResult>>;


    async fn generate_thumbnail(&self, media_url: &str) -> Result<String>;
}


#[derive(Debug)]
pub struct DefaultMultimediaHandler {
    storage_client: Option<aws_sdk_s3::Client>,
    search_api_key: Option<String>,
}

impl DefaultMultimediaHandler {
    pub fn new(storage_client: Option<aws_sdk_s3::Client>, search_api_key: Option<String>) -> Self {
        Self {
            storage_client,
            search_api_key,
        }
    }

    pub fn storage_client(&self) -> &Option<aws_sdk_s3::Client> {
        &self.storage_client
    }

    pub fn search_api_key(&self) -> &Option<String> {
        &self.search_api_key
    }
}

#[async_trait]
impl MultimediaHandler for DefaultMultimediaHandler {
    async fn process_multimedia(
        &self,
        message: MultimediaMessage,
        user_id: &str,
        session_id: &str,
    ) -> Result<BotResponse> {
        match message {
            MultimediaMessage::Text { content } => {

                Ok(BotResponse {
                    bot_id: "default".to_string(),
                    user_id: user_id.to_string(),
                    session_id: session_id.to_string(),
                    channel: "multimedia".to_string(),
                    content,
                    message_type: MessageType::EXTERNAL,
                    stream_token: None,
                    is_complete: true,
                    suggestions: Vec::new(),
                    context_name: None,
                    context_length: 0,
                    context_max_length: 0,
                })
            }
            MultimediaMessage::Image { url, caption, .. } => {

                log::debug!("Processing image from URL: {}", url);
                let response_content = format!(
                    "I see you've shared an image from {}{}. {}",
                    url,
                    caption
                        .as_ref()
                        .map(|c| format!(" with caption: {}", c))
                        .unwrap_or_default(),
                    "Let me analyze this for you."
                );

                Ok(BotResponse {
                    bot_id: "default".to_string(),
                    user_id: user_id.to_string(),
                    session_id: session_id.to_string(),
                    channel: "multimedia".to_string(),
                    content: response_content,
                    message_type: MessageType::EXTERNAL,
                    stream_token: None,
                    is_complete: true,
                    suggestions: Vec::new(),
                    context_name: None,
                    context_length: 0,
                    context_max_length: 0,
                })
            }
            MultimediaMessage::Video {
                url,
                caption,
                duration,
                ..
            } => {

                log::debug!("Processing video from URL: {}", url);
                let response_content = format!(
                    "You've shared a video from {}{}{}. Processing video content...",
                    url,
                    duration.map(|d| format!(" ({}s)", d)).unwrap_or_default(),
                    caption
                        .as_ref()
                        .map(|c| format!(" - {}", c))
                        .unwrap_or_default()
                );

                Ok(BotResponse {
                    bot_id: "default".to_string(),
                    user_id: user_id.to_string(),
                    session_id: session_id.to_string(),
                    channel: "multimedia".to_string(),
                    content: response_content,
                    message_type: MessageType::EXTERNAL,
                    stream_token: None,
                    is_complete: true,
                    suggestions: Vec::new(),
                    context_name: None,
                    context_length: 0,
                    context_max_length: 0,
                })
            }
            MultimediaMessage::WebSearch { query, .. } => {

                let results = self.web_search(&query, 5).await?;
                let response_content = if results.is_empty() {
                    format!("No results found for: {}", query)
                } else {
                    let results_text = results
                        .iter()
                        .enumerate()
                        .map(|(i, r)| {
                            format!("{}. [{}]({})\n   {}", i + 1, r.title, r.url, r.snippet)
                        })
                        .collect::<Vec<_>>()
                        .join("\n\n");

                    format!("Search results for \"{}\":\n\n{}", query, results_text)
                };

                Ok(BotResponse {
                    bot_id: "default".to_string(),
                    user_id: user_id.to_string(),
                    session_id: session_id.to_string(),
                    channel: "multimedia".to_string(),
                    content: response_content,
                    message_type: MessageType::EXTERNAL,
                    stream_token: None,
                    is_complete: true,
                    suggestions: Vec::new(),
                    context_name: None,
                    context_length: 0,
                    context_max_length: 0,
                })
            }
            MultimediaMessage::MeetingInvite {
                meeting_url,
                start_time,
                ..
            } => {
                let response_content = format!(
                    "Meeting invite received. Join at: {}{}",
                    meeting_url,
                    start_time
                        .as_ref()
                        .map(|t| format!("\nScheduled for: {}", t))
                        .unwrap_or_default()
                );

                Ok(BotResponse {
                    bot_id: "default".to_string(),
                    user_id: user_id.to_string(),
                    session_id: session_id.to_string(),
                    channel: "multimedia".to_string(),
                    content: response_content,
                    message_type: MessageType::EXTERNAL,
                    stream_token: None,
                    is_complete: true,
                    suggestions: Vec::new(),
                    context_name: None,
                    context_length: 0,
                    context_max_length: 0,
                })
            }
            _ => {

                Ok(BotResponse {
                    bot_id: "default".to_string(),
                    user_id: user_id.to_string(),
                    session_id: session_id.to_string(),
                    channel: "multimedia".to_string(),
                    content: "Message received and processing...".to_string(),
                    message_type: MessageType::EXTERNAL,
                    stream_token: None,
                    is_complete: true,
                    suggestions: Vec::new(),
                    context_name: None,
                    context_length: 0,
                    context_max_length: 0,
                })
            }
        }
    }

    async fn upload_media(&self, request: MediaUploadRequest) -> Result<MediaUploadResponse> {
        let media_id = Uuid::new_v4().to_string();
        let key = format!(
            "media/{}/{}/{}",
            request.user_id, request.session_id, request.file_name
        );

        if let Some(client) = &self.storage_client {

            client
                .put_object()
                .bucket("botserver-media")
                .key(&key)
                .body(request.data.into())
                .content_type(&request.content_type)
                .send()
                .await?;

            let url = format!("https://storage.botserver.com/{}", key);

            Ok(MediaUploadResponse {
                media_id,
                url,
                thumbnail_url: None,
            })
        } else {

            let local_path = format!("./media/{}", key);
            if let Some(parent) = std::path::Path::new(&local_path).parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&local_path, request.data)?;

            Ok(MediaUploadResponse {
                media_id,
                url: format!("file://{}", local_path),
                thumbnail_url: None,
            })
        }
    }

    async fn download_media(&self, url: &str) -> Result<Vec<u8>> {
        if url.starts_with("http://") || url.starts_with("https://") {
            let response = reqwest::get(url).await?;
            Ok(response.bytes().await?.to_vec())
        } else if url.starts_with("file://") {
            let path = url.strip_prefix("file://").unwrap_or_default();
            Ok(std::fs::read(path)?)
        } else {
            Err(anyhow::anyhow!("Unsupported URL scheme: {}", url))
        }
    }

    async fn web_search(&self, query: &str, max_results: usize) -> Result<Vec<SearchResult>> {


        let mock_results = vec![
            SearchResult {
                title: format!("Result 1 for: {}", query),
                url: "https://example.com/1".to_string(),
                snippet: "This is a sample search result snippet...".to_string(),
                thumbnail: None,
            },
            SearchResult {
                title: format!("Result 2 for: {}", query),
                url: "https://example.com/2".to_string(),
                snippet: "Another sample search result...".to_string(),
                thumbnail: None,
            },
        ];

        Ok(mock_results.into_iter().take(max_results).collect())
    }

    async fn generate_thumbnail(&self, media_url: &str) -> Result<String> {


        Ok(media_url.to_string())
    }
}


pub trait UserMessageMultimedia {
    fn to_multimedia(&self) -> MultimediaMessage;
}

impl UserMessageMultimedia for UserMessage {
    fn to_multimedia(&self) -> MultimediaMessage {

        if self.content.starts_with("http") {

            if self.content.contains(".jpg")
                || self.content.contains(".png")
                || self.content.contains(".gif")
            {
                MultimediaMessage::Image {
                    url: self.content.clone(),
                    caption: None,
                    mime_type: "image/jpeg".to_string(),
                }
            } else if self.content.contains(".mp4")
                || self.content.contains(".webm")
                || self.content.contains(".mov")
            {
                MultimediaMessage::Video {
                    url: self.content.clone(),
                    thumbnail_url: None,
                    caption: None,
                    duration: None,
                    mime_type: "video/mp4".to_string(),
                }
            } else {
                MultimediaMessage::Text {
                    content: self.content.clone(),
                }
            }
        } else if self.content.starts_with("/search ") {
            let query = self
                .content
                .strip_prefix("/search ")
                .unwrap_or(&self.content);
            MultimediaMessage::WebSearch {
                query: query.to_string(),
                results: Vec::new(),
            }
        } else {
            MultimediaMessage::Text {
                content: self.content.clone(),
            }
        }
    }
}


use crate::core::shared::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;


pub async fn upload_media_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<MediaUploadRequest>,
) -> impl IntoResponse {
    let handler = DefaultMultimediaHandler::new(state.drive.clone(), None);

    match handler.upload_media(request).await {
        Ok(response) => (StatusCode::OK, Json(serde_json::json!(response))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}


pub async fn download_media_handler(
    State(state): State<Arc<AppState>>,
    Path(media_id): Path<String>,
) -> impl IntoResponse {
    let handler = DefaultMultimediaHandler::new(state.drive.clone(), None);


    let url = format!("https://storage.botserver.com/media/{}", media_id);

    match handler.download_media(&url).await {
        Ok(data) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "media_id": media_id,
                "size": data.len(),
                "data": STANDARD.encode(&data)
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}


pub async fn generate_thumbnail_handler(
    State(state): State<Arc<AppState>>,
    Path(media_id): Path<String>,
) -> impl IntoResponse {
    let handler = DefaultMultimediaHandler::new(state.drive.clone(), None);


    let url = format!("https://storage.botserver.com/media/{}", media_id);

    match handler.generate_thumbnail(&url).await {
        Ok(thumbnail_url) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "media_id": media_id,
                "thumbnail_url": thumbnail_url
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}


pub async fn web_search_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let query = payload.get("query").and_then(|q| q.as_str()).unwrap_or("");
    let max_results = payload
        .get("max_results")
        .and_then(|m| m.as_u64())
        .unwrap_or(10) as usize;

    let handler = DefaultMultimediaHandler::new(state.drive.clone(), None);

    match handler.web_search(query, max_results).await {
        Ok(results) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "query": query,
                "results": results
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}
