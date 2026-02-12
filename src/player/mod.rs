use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::core::shared::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaInfo {
    pub path: String,
    pub filename: String,
    pub mime_type: String,
    pub size: u64,
    pub duration: Option<f64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThumbnailInfo {
    pub path: String,
    pub width: u32,
    pub height: u32,
    pub format: String,
}

#[derive(Debug, Deserialize)]
pub struct StreamQuery {
    pub quality: Option<String>,
    pub start: Option<f64>,
    pub end: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct ThumbnailQuery {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub time: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct PlayerError {
    pub error: String,
    pub code: String,
}

impl IntoResponse for PlayerError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": self.error, "code": self.code})),
        )
            .into_response()
    }
}

fn get_mime_type(path: &str) -> &'static str {
    let ext = path.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "ogv" => "video/ogg",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "m4a" => "audio/mp4",
        "flac" => "audio/flac",
        "pdf" => "application/pdf",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        _ => "application/octet-stream",
    }
}

fn get_format(path: &str) -> String {
    path.rsplit('.')
        .next()
        .unwrap_or("unknown")
        .to_uppercase()
}

async fn get_file_info(
    State(_state): State<Arc<AppState>>,
    Path((bot_id, path)): Path<(String, String)>,
) -> Result<Json<MediaInfo>, PlayerError> {
    let filename = path.rsplit('/').next().unwrap_or(&path).to_string();
    let mime_type = get_mime_type(&path).to_string();
    let format = get_format(&path);

    let info = MediaInfo {
        path: format!("{bot_id}/{path}"),
        filename,
        mime_type,
        size: 0,
        duration: None,
        width: None,
        height: None,
        format,
    };

    Ok(Json(info))
}

async fn stream_file(
    State(state): State<Arc<AppState>>,
    Path((bot_id, path)): Path<(String, String)>,
    Query(_query): Query<StreamQuery>,
) -> Result<Response<Body>, PlayerError> {
    let mime_type = get_mime_type(&path);
    let full_path = format!("{bot_id}.gbdrive/{path}");

    let s3 = state.drive.as_ref().ok_or_else(|| PlayerError {
        error: "Storage not configured".to_string(),
        code: "STORAGE_NOT_CONFIGURED".to_string(),
    })?;

    let result = s3
        .get_object()
        .bucket(&format!("{bot_id}.gbai"))
        .key(&full_path)
        .send()
        .await
        .map_err(|e| PlayerError {
            error: format!("Failed to get file: {e}"),
            code: "FILE_NOT_FOUND".to_string(),
        })?;

    let body = result.body.collect().await.map_err(|e| PlayerError {
        error: format!("Failed to read file: {e}"),
        code: "READ_ERROR".to_string(),
    })?;

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime_type)
        .header(header::ACCEPT_RANGES, "bytes")
        .body(Body::from(body.into_bytes()))
        .map_err(|e| PlayerError {
            error: format!("Failed to build response: {e}"),
            code: "RESPONSE_ERROR".to_string(),
        })?;

    Ok(response)
}

async fn get_thumbnail(
    State(_state): State<Arc<AppState>>,
    Path((bot_id, path)): Path<(String, String)>,
    Query(query): Query<ThumbnailQuery>,
) -> Result<Response<Body>, PlayerError> {
    let width = query.width.unwrap_or(320);
    let height = query.height.unwrap_or(180);

    let filename = path.rsplit('/').next().unwrap_or(&path);
    let placeholder = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">
            <rect width="100%" height="100%" fill="#374151"/>
            <text x="50%" y="50%" text-anchor="middle" dy="0.3em" fill="#9CA3AF" font-family="sans-serif" font-size="14">
                {}
            </text>
        </svg>"##,
        width, height, width, height, filename
    );

    let _ = bot_id;

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "image/svg+xml")
        .header(header::CACHE_CONTROL, "public, max-age=3600")
        .body(Body::from(placeholder))
        .map_err(|e| PlayerError {
            error: format!("Failed to build response: {e}"),
            code: "RESPONSE_ERROR".to_string(),
        })?;

    Ok(response)
}

async fn get_supported_formats(
    State(_state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "video": ["mp4", "webm", "ogv"],
        "audio": ["mp3", "wav", "ogg", "m4a", "flac"],
        "document": ["pdf", "txt", "md", "html"],
        "image": ["png", "jpg", "jpeg", "gif", "svg", "webp"],
        "presentation": ["pptx", "odp"]
    }))
}

pub fn configure_player_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/player/formats", get(get_supported_formats))
        .route("/api/player/:bot_id/info/*path", get(get_file_info))
        .route("/api/player/:bot_id/stream/*path", get(stream_file))
        .route("/api/player/:bot_id/thumbnail/*path", get(get_thumbnail))
}
