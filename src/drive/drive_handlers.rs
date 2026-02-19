// Drive HTTP handlers implementation using S3
// Extracted from drive/mod.rs and using aws-sdk-s3

use crate::core::shared::state::AppState;
use crate::drive::drive_types::*;
use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Json, Response},
    body::Body,
};
use aws_sdk_s3::primitives::ByteStream;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;


// Import ReadResponse from parent mod if not in drive_types
use super::ReadResponse; 

/// Open a file (get metadata)
pub async fn open_file(
    State(state): State<Arc<AppState>>,
    Path(file_id): Path<String>,
) -> Result<Json<FileItem>, (StatusCode, Json<serde_json::Value>)> {
    read_metadata(state, file_id).await
}

/// Helper to get file metadata
async fn read_metadata(
    state: Arc<AppState>,
    file_id: String,
) -> Result<Json<FileItem>, (StatusCode, Json<serde_json::Value>)> {
    let client = state.drive.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "Drive not configured"})),
    ))?;
    let bucket = &state.bucket_name;

    let resp = client.head_object()
        .bucket(bucket)
        .key(&file_id)
        .send()
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": e.to_string()}))))?;

    let item = FileItem {
        id: file_id.clone(),
        name: file_id.split('/').last().unwrap_or(&file_id).to_string(),
        file_type: if file_id.ends_with('/') { "folder".to_string() } else { "file".to_string() },
        size: resp.content_length.unwrap_or(0),
        mime_type: resp.content_type.unwrap_or_else(|| "application/octet-stream".to_string()),
        created_at: Utc::now(), // S3 doesn't track creation time easily
        modified_at: Utc::now(), // Simplified
        parent_id: None,
        url: None,
        thumbnail_url: None,
        is_favorite: false, // Not implemented in S3
        tags: vec![],
        metadata: HashMap::new(),
    };
    Ok(Json(item))
}

/// List all buckets (or configured one)
pub async fn list_buckets(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<BucketInfo>>, (StatusCode, Json<serde_json::Value>)> {
    let client = state.drive.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "Drive not configured"})),
    ))?;

    match client.list_buckets().send().await {
        Ok(resp) => {
            let buckets = resp.buckets.unwrap_or_default().iter().map(|b| {
                BucketInfo {
                    id: b.name.clone().unwrap_or_default(),
                    name: b.name.clone().unwrap_or_default(),
                    created_at: Utc::now(),
                    file_count: 0,
                    total_size: 0,
                }
            }).collect();
            Ok(Json(buckets))
        },
        Err(_) => {
            // Fallback
            Ok(Json(vec![BucketInfo {
                id: state.bucket_name.clone(),
                name: state.bucket_name.clone(),
                created_at: Utc::now(),
                file_count: 0,
                total_size: 0,
            }]))
        }
    }
}

/// List files in a bucket
pub async fn list_files(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SearchQuery>,
) -> Result<Json<Vec<FileItem>>, (StatusCode, Json<serde_json::Value>)> {
    let client = state.drive.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "Drive not configured"})),
    ))?;
    let bucket = req.bucket.clone().unwrap_or_else(|| state.bucket_name.clone());
    let prefix = req.parent_path.clone().unwrap_or_default();

    let resp = client.list_objects_v2()
        .bucket(&bucket)
        .prefix(&prefix)
        .send()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))))?;

    let files = resp.contents.unwrap_or_default().iter().map(|obj| {
        let key = obj.key().unwrap_or_default();
        let name = key.split('/').last().unwrap_or(key).to_string();
        FileItem {
            id: key.to_string(),
            name,
            file_type: if key.ends_with('/') { "folder".to_string() } else { "file".to_string() },
            size: obj.size.unwrap_or(0),
            mime_type: "application/octet-stream".to_string(),
            created_at: Utc::now(),
            modified_at: Utc::now(),
            parent_id: Some(prefix.clone()),
            url: None,
            thumbnail_url: None,
            is_favorite: false,
            tags: vec![],
            metadata: HashMap::new(),
        }
    }).collect();

    Ok(Json(files))
}

/// Read file content (as text)
pub async fn read_file(
    State(state): State<Arc<AppState>>,
    Path(file_id): Path<String>,
) -> Result<Json<ReadResponse>, (StatusCode, Json<serde_json::Value>)> {
    let client = state.drive.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "Drive not configured"})),
    ))?;
    let bucket = &state.bucket_name;

    let resp = client.get_object()
        .bucket(bucket)
        .key(&file_id)
        .send()
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": e.to_string()}))))?;

    let data = resp.body.collect().await.map_err(|e| 
        (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))
    )?.into_bytes();

    let content = String::from_utf8(data.to_vec()).unwrap_or_else(|_| "[Binary Content]".to_string());

    Ok(Json(ReadResponse { content }))
}

/// Write file content
pub async fn write_file(
    State(state): State<Arc<AppState>>,
    Json(req): Json<WriteRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let client = state.drive.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "Drive not configured"})),
    ))?;
    let bucket = &state.bucket_name;
    let key = req.file_id.ok_or((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Missing file_id"}))))?;

    client.put_object()
        .bucket(bucket)
        .key(&key)
        .body(ByteStream::from(req.content.into_bytes()))
        .send()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))))?;

    Ok(Json(serde_json::json!({"success": true})))
}

/// Delete a file
pub async fn delete_file(
    State(state): State<Arc<AppState>>,
    Path(file_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let client = state.drive.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "Drive not configured"})),
    ))?;
    let bucket = &state.bucket_name;

    client.delete_object()
        .bucket(bucket)
        .key(&file_id)
        .send()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))))?;

    Ok(Json(serde_json::json!({"success": true})))
}

/// Create a folder
pub async fn create_folder(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateFolderRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let client = state.drive.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "Drive not configured"})),
    ))?;
    let bucket = &state.bucket_name;
    
    // Construct folder path/key
    let mut key = req.parent_id.unwrap_or_default();
    if !key.ends_with('/') && !key.is_empty() {
        key.push('/');
    }
    key.push_str(&req.name);
    if !key.ends_with('/') {
        key.push('/');
    }

    client.put_object()
        .bucket(bucket)
        .key(&key)
        .body(ByteStream::from_static(&[]))
        .send()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))))?;

    Ok(Json(serde_json::json!({"success": true})))
}

/// Download file (stream)
pub async fn download_file(
    State(state): State<Arc<AppState>>,
    Path(file_id): Path<String>,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let client = state.drive.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "Drive not configured"})),
    ))?;
    let bucket = &state.bucket_name;

    let resp = client.get_object()
        .bucket(bucket)
        .key(&file_id)
        .send()
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": e.to_string()}))))?;

    let stream = Body::from_stream(resp.body);

    Ok(Response::builder()
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", file_id.split('/').last().unwrap_or("file")))
        .body(stream)
        .unwrap())
}

// Stubs for others (list_shared, etc.)
pub async fn copy_file(State(_): State<Arc<AppState>>, Json(_): Json<CopyFileRequest>) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, Json(serde_json::json!({"error": "Not implemented"})))
}
pub async fn upload_file_to_drive(State(_): State<Arc<AppState>>, Json(_): Json<UploadRequest>) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, Json(serde_json::json!({"error": "Not implemented"})))
}
pub async fn list_folder_contents(State(_): State<Arc<AppState>>, Json(_): Json<SearchQuery>) -> impl IntoResponse {
    (StatusCode::OK, Json(Vec::<FileItem>::new()))
}
pub async fn search_files(State(_): State<Arc<AppState>>, Json(_): Json<SearchQuery>) -> impl IntoResponse {
    (StatusCode::OK, Json(Vec::<FileItem>::new()))
}
pub async fn recent_files(State(_): State<Arc<AppState>>) -> impl IntoResponse {
    (StatusCode::OK, Json(Vec::<FileItem>::new()))
}
pub async fn list_favorites(State(_): State<Arc<AppState>>) -> impl IntoResponse {
    (StatusCode::OK, Json(Vec::<FileItem>::new()))
}
pub async fn share_folder(State(_): State<Arc<AppState>>, Json(_): Json<ShareRequest>) -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({"success": true})))
}
pub async fn list_shared(State(_): State<Arc<AppState>>) -> impl IntoResponse {
    (StatusCode::OK, Json(Vec::<FileItem>::new()))
}
