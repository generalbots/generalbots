// Drive HTTP handlers - stub for when drive feature is disabled
#[cfg(not(feature = "drive"))]

use crate::core::shared::state::AppState;
use crate::drive::drive_types::*;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use std::sync::Arc;

pub async fn open_file(
    State(_): State<Arc<AppState>>,
    Path(_file_id): Path<String>,
) -> Result<Json<FileItem>, (StatusCode, Json<serde_json::Value>)> {
    Err((
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "Drive feature not enabled"})),
    ))
}

pub async fn list_buckets(
    State(_): State<Arc<AppState>>,
) -> Result<Json<Vec<BucketInfo>>, (StatusCode, Json<serde_json::Value>)> {
    Err((
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "Drive feature not enabled"})),
    ))
}

pub async fn list_files(
    State(_): State<Arc<AppState>>,
    Json(_req): Json<SearchQuery>,
) -> Result<Json<Vec<FileItem>>, (StatusCode, Json<serde_json::Value>)> {
    Err((
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "Drive feature not enabled"})),
    ))
}

pub async fn read_file(
    State(_): State<Arc<AppState>>,
    Path(_file_id): Path<String>,
) -> Result<Json<super::ReadResponse>, (StatusCode, Json<serde_json::Value>)> {
    Err((
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "Drive feature not enabled"})),
    ))
}

pub async fn write_file(
    State(_): State<Arc<AppState>>,
    Json(_req): Json<WriteRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    Err((
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "Drive feature not enabled"})),
    ))
}

pub async fn delete_file(
    State(_): State<Arc<AppState>>,
    Path(_file_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    Err((
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "Drive feature not enabled"})),
    ))
}

pub async fn create_folder(
    State(_): State<Arc<AppState>>,
    Json(_req): Json<CreateFolderRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    Err((
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "Drive feature not enabled"})),
    ))
}

pub async fn download_file(
    State(_): State<Arc<AppState>>,
    Path(_file_id): Path<String>,
) -> Result<axum::response::Response, (StatusCode, Json<serde_json::Value>)> {
    Err((
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "Drive feature not enabled"})),
    ))
}

pub async fn copy_file(State(_): State<Arc<AppState>>, Json(_): Json<CopyFileRequest>) -> impl axum::response::IntoResponse {
    (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({"error": "Drive feature not enabled"})))
}

pub async fn upload_file_to_drive(State(_): State<Arc<AppState>>, Json(_): Json<UploadRequest>) -> impl axum::response::IntoResponse {
    (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({"error": "Drive feature not enabled"})))
}

pub async fn list_folder_contents(State(_): State<Arc<AppState>>, Json(_): Json<SearchQuery>) -> impl axum::response::IntoResponse {
    (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({"error": "Drive feature not enabled"})))
}

pub async fn search_files(State(_): State<Arc<AppState>>, Json(_): Json<SearchQuery>) -> impl axum::response::IntoResponse {
    (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({"error": "Drive feature not enabled"})))
}

pub async fn recent_files(State(_): State<Arc<AppState>>) -> impl axum::response::IntoResponse {
    (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({"error": "Drive feature not enabled"})))
}

pub async fn list_favorites(State(_): State<Arc<AppState>>) -> impl axum::response::IntoResponse {
    (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({"error": "Drive feature not enabled"})))
}

pub async fn share_folder(State(_): State<Arc<AppState>>, Json(_): Json<ShareRequest>) -> impl axum::response::IntoResponse {
    (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({"error": "Drive feature not enabled"})))
}

pub async fn list_shared(State(_): State<Arc<AppState>>) -> impl axum::response::IntoResponse {
    (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({"error": "Drive feature not enabled"})))
}
