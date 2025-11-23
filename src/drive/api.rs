//! Drive File Management REST API
//!
//! Provides HTTP endpoints for file operations with S3 backend.
//! Works across web, desktop, and mobile platforms.

use crate::shared::state::AppState;
use aws_sdk_s3::primitives::ByteStream;
use axum::{
    extract::{Json, Multipart, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileItem {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub modified: String,
    pub is_dir: bool,
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListFilesQuery {
    pub path: Option<String>,
    pub limit: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFolderRequest {
    pub path: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteFileRequest {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveFileRequest {
    pub source: String,
    pub destination: String,
}

/// GET /api/drive/list
/// List files and folders in a directory
pub async fn list_files(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListFilesQuery>,
) -> impl IntoResponse {
    let path = query.path.unwrap_or_else(|| "/".to_string());
    let prefix = path.trim_start_matches('/');

    info!("Listing files in path: {}", path);

    let mut files = Vec::new();

    if let Some(s3_client) = &state.drive {
        let bucket = &state.bucket_name;

        match s3_client
            .list_objects_v2()
            .bucket(bucket)
            .prefix(prefix)
            .delimiter("/")
            .max_keys(query.limit.unwrap_or(1000))
            .send()
            .await
        {
            Ok(output) => {
                // Add folders (common prefixes)
                let prefixes = output.common_prefixes();
                if !prefixes.is_empty() {
                    for prefix in prefixes {
                        if let Some(p) = prefix.prefix() {
                            let name = p.trim_end_matches('/').split('/').last().unwrap_or(p);
                            files.push(FileItem {
                                name: name.to_string(),
                                path: format!("/{}", p),
                                size: 0,
                                modified: chrono::Utc::now().to_rfc3339(),
                                is_dir: true,
                                mime_type: None,
                            });
                        }
                    }
                }

                // Add files
                let objects = output.contents();
                if !objects.is_empty() {
                    for object in objects {
                        if let Some(key) = object.key() {
                            if key.ends_with('/') {
                                continue; // Skip folder markers
                            }

                            let name = key.split('/').last().unwrap_or(key);
                            let size = object.size().unwrap_or(0) as u64;
                            let modified = object
                                .last_modified()
                                .map(|dt| dt.to_string())
                                .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

                            let mime_type =
                                mime_guess::from_path(name).first().map(|m| m.to_string());

                            files.push(FileItem {
                                name: name.to_string(),
                                path: format!("/{}", key),
                                size,
                                modified,
                                is_dir: false,
                                mime_type,
                            });
                        }
                    }
                }

                info!("Found {} items in {}", files.len(), path);
            }
            Err(e) => {
                error!("Failed to list files: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": format!("Failed to list files: {}", e)
                    })),
                );
            }
        }
    } else {
        error!("S3 client not configured");
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "error": "Storage service not available"
            })),
        );
    }

    (StatusCode::OK, Json(serde_json::json!(files)))
}

/// POST /api/drive/upload
/// Upload a file to S3
pub async fn upload_file(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut file_path = String::new();
    let mut file_data: Vec<u8> = Vec::new();
    let mut file_name = String::new();

    // Parse multipart form
    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let name = field.name().unwrap_or("").to_string();

        if name == "path" {
            if let Ok(value) = field.text().await {
                file_path = value;
            }
        } else if name == "file" {
            file_name = field.file_name().unwrap_or("unnamed").to_string();
            if let Ok(data) = field.bytes().await {
                file_data = data.to_vec();
            }
        }
    }

    if file_data.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "No file data provided"
            })),
        );
    }

    let full_path = if file_path.is_empty() {
        file_name.clone()
    } else {
        format!("{}/{}", file_path.trim_matches('/'), file_name)
    };

    let file_size = file_data.len();
    info!("Uploading file: {} ({} bytes)", full_path, file_size);

    if let Some(s3_client) = &state.drive {
        let bucket = &state.bucket_name;
        let content_type = mime_guess::from_path(&file_name)
            .first()
            .map(|m| m.to_string())
            .unwrap_or_else(|| "application/octet-stream".to_string());

        match s3_client
            .put_object()
            .bucket(bucket)
            .key(&full_path)
            .body(ByteStream::from(file_data))
            .content_type(&content_type)
            .send()
            .await
        {
            Ok(_) => {
                info!("Successfully uploaded: {}", full_path);
                (
                    StatusCode::OK,
                    Json(serde_json::json!({
                        "success": true,
                        "path": format!("/{}", full_path),
                        "size": file_size
                    })),
                )
            }
            Err(e) => {
                error!("Failed to upload file: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": format!("Upload failed: {}", e)
                    })),
                )
            }
        }
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "error": "Storage service not available"
            })),
        )
    }
}

/// POST /api/drive/folder
/// Create a new folder
pub async fn create_folder(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateFolderRequest>,
) -> impl IntoResponse {
    let folder_path = format!("{}/{}/", request.path.trim_matches('/'), request.name);

    info!("Creating folder: {}", folder_path);

    if let Some(s3_client) = &state.drive {
        let bucket = &state.bucket_name;

        // Create folder marker (empty object with trailing slash)
        match s3_client
            .put_object()
            .bucket(bucket)
            .key(&folder_path)
            .body(ByteStream::from(vec![]))
            .send()
            .await
        {
            Ok(_) => {
                info!("Successfully created folder: {}", folder_path);
                (
                    StatusCode::OK,
                    Json(serde_json::json!({
                        "success": true,
                        "path": format!("/{}", folder_path)
                    })),
                )
            }
            Err(e) => {
                error!("Failed to create folder: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": format!("Failed to create folder: {}", e)
                    })),
                )
            }
        }
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "error": "Storage service not available"
            })),
        )
    }
}

/// DELETE /api/drive/file
/// Delete a file or folder
pub async fn delete_file(
    State(state): State<Arc<AppState>>,
    Json(request): Json<DeleteFileRequest>,
) -> impl IntoResponse {
    let path = request.path.trim_start_matches('/');

    info!("Deleting: {}", path);

    if let Some(s3_client) = &state.drive {
        let bucket = &state.bucket_name;

        // Check if it's a folder (ends with /)
        if path.ends_with('/') {
            // Delete all objects with this prefix
            match s3_client
                .list_objects_v2()
                .bucket(bucket)
                .prefix(path)
                .send()
                .await
            {
                Ok(output) => {
                    let objects = output.contents();
                    if !objects.is_empty() {
                        for object in objects {
                            if let Some(key) = object.key() {
                                if let Err(e) = s3_client
                                    .delete_object()
                                    .bucket(bucket)
                                    .key(key)
                                    .send()
                                    .await
                                {
                                    error!("Failed to delete {}: {}", key, e);
                                }
                            }
                        }
                    }
                    info!("Successfully deleted folder: {}", path);
                    return (
                        StatusCode::OK,
                        Json(serde_json::json!({
                            "success": true,
                            "path": request.path
                        })),
                    );
                }
                Err(e) => {
                    error!("Failed to list folder contents: {}", e);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "error": format!("Failed to delete folder: {}", e)
                        })),
                    );
                }
            }
        }

        // Delete single file
        match s3_client
            .delete_object()
            .bucket(bucket)
            .key(path)
            .send()
            .await
        {
            Ok(_) => {
                info!("Successfully deleted file: {}", path);
                (
                    StatusCode::OK,
                    Json(serde_json::json!({
                        "success": true,
                        "path": request.path
                    })),
                )
            }
            Err(e) => {
                error!("Failed to delete file: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": format!("Failed to delete: {}", e)
                    })),
                )
            }
        }
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "error": "Storage service not available"
            })),
        )
    }
}

/// POST /api/drive/move
/// Move or rename a file/folder
pub async fn move_file(
    State(state): State<Arc<AppState>>,
    Json(request): Json<MoveFileRequest>,
) -> impl IntoResponse {
    let source = request.source.trim_start_matches('/');
    let destination = request.destination.trim_start_matches('/');

    info!("Moving {} to {}", source, destination);

    if let Some(s3_client) = &state.drive {
        let bucket = &state.bucket_name;

        // Copy to new location
        let copy_source = format!("{}/{}", bucket, source);

        match s3_client
            .copy_object()
            .bucket(bucket)
            .copy_source(&copy_source)
            .key(destination)
            .send()
            .await
        {
            Ok(_) => {
                // Delete original
                match s3_client
                    .delete_object()
                    .bucket(bucket)
                    .key(source)
                    .send()
                    .await
                {
                    Ok(_) => {
                        info!("Successfully moved {} to {}", source, destination);
                        (
                            StatusCode::OK,
                            Json(serde_json::json!({
                                "success": true,
                                "source": request.source,
                                "destination": request.destination
                            })),
                        )
                    }
                    Err(e) => {
                        error!("Failed to delete source after copy: {}", e);
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(serde_json::json!({
                                "error": format!("Move partially failed: {}", e)
                            })),
                        )
                    }
                }
            }
            Err(e) => {
                error!("Failed to copy file: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": format!("Failed to move: {}", e)
                    })),
                )
            }
        }
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "error": "Storage service not available"
            })),
        )
    }
}

/// GET /api/drive/download/{path}
/// Download a file
pub async fn download_file(
    State(state): State<Arc<AppState>>,
    Path(file_path): Path<String>,
) -> impl IntoResponse {
    let path = file_path.trim_start_matches('/');

    info!("Downloading file: {}", path);

    if let Some(s3_client) = &state.drive {
        let bucket = &state.bucket_name;

        match s3_client.get_object().bucket(bucket).key(path).send().await {
            Ok(output) => {
                let content_type = output
                    .content_type()
                    .unwrap_or("application/octet-stream")
                    .to_string();
                let body = output.body.collect().await.unwrap().into_bytes();

                (
                    StatusCode::OK,
                    [(axum::http::header::CONTENT_TYPE, content_type)],
                    body.to_vec(),
                )
            }
            Err(e) => {
                error!("Failed to download file: {}", e);
                (
                    StatusCode::NOT_FOUND,
                    [(
                        axum::http::header::CONTENT_TYPE,
                        "application/json".to_string(),
                    )],
                    serde_json::json!({
                        "error": format!("File not found: {}", e)
                    })
                    .to_string()
                    .into_bytes()
                    .to_vec(),
                )
            }
        }
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            [(
                axum::http::header::CONTENT_TYPE,
                "application/json".to_string(),
            )],
            serde_json::json!({
                "error": "Storage service not available"
            })
            .to_string()
            .into_bytes()
            .to_vec(),
        )
    }
}
