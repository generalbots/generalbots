//! Drive Module - S3-based File Storage
//!
//! Provides file management operations using S3 as backend storage.
//! Supports bot storage and provides REST API endpoints for desktop frontend.
//!
//! API Endpoints:
//! - GET /files/list - List files and folders
//! - POST /files/read - Read file content
//! - POST /files/write - Write file content
//! - POST /files/delete - Delete file/folder
//! - POST /files/create-folder - Create new folder

#[cfg(feature = "console")]
use crate::console::file_tree::{FileTree, TreeNode};
use crate::shared::state::AppState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use futures_util::stream::StreamExt;
use serde::{Deserialize, Serialize};
// use serde_json::json; // Unused import
use std::sync::Arc;

pub mod document_processing;
pub mod drive_monitor;
pub mod files;
pub mod vectordb;

// ===== Request/Response Structures =====

#[derive(Debug, Serialize, Deserialize)]
pub struct FileItem {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: Option<i64>,
    pub modified: Option<String>,
    pub icon: String,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub path: Option<String>,
    pub bucket: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReadRequest {
    pub bucket: String,
    pub path: String,
}

#[derive(Debug, Serialize)]
pub struct ReadResponse {
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct WriteRequest {
    pub bucket: String,
    pub path: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteRequest {
    pub bucket: String,
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateFolderRequest {
    pub bucket: String,
    pub path: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct CopyRequest {
    pub source_bucket: String,
    pub source_path: String,
    pub dest_bucket: String,
    pub dest_path: String,
}

#[derive(Debug, Deserialize)]
pub struct MoveRequest {
    pub source_bucket: String,
    pub source_path: String,
    pub dest_bucket: String,
    pub dest_path: String,
}

#[derive(Debug, Deserialize)]
pub struct DownloadRequest {
    pub bucket: String,
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub bucket: Option<String>,
    pub query: String,
    pub file_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ShareRequest {
    pub bucket: String,
    pub path: String,
    pub users: Vec<String>,
    pub permissions: String,
}

#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct QuotaResponse {
    pub total_bytes: i64,
    pub used_bytes: i64,
    pub available_bytes: i64,
    pub percentage_used: f64,
}

#[derive(Debug, Serialize)]
pub struct ShareResponse {
    pub share_id: String,
    pub url: String,
    pub expires_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SyncStatus {
    pub status: String,
    pub last_sync: Option<String>,
    pub files_synced: i64,
    pub bytes_synced: i64,
}

// ===== API Configuration =====

/// Configure drive API routes
pub fn configure() -> Router<Arc<AppState>> {
    Router::new()
        // Basic file operations
        .route("/files/list", get(list_files))
        .route("/files/read", post(read_file))
        .route("/files/write", post(write_file))
        .route("/files/save", post(write_file))
        .route("/files/getContents", post(read_file))
        .route("/files/delete", post(delete_file))
        .route("/files/upload", post(upload_file_to_drive))
        .route("/files/download", post(download_file))
        // File management
        .route("/files/copy", post(copy_file))
        .route("/files/move", post(move_file))
        .route("/files/createFolder", post(create_folder))
        .route("/files/create-folder", post(create_folder))
        .route("/files/dirFolder", post(list_folder_contents))
        // Search and discovery
        .route("/files/search", get(search_files))
        .route("/files/recent", get(recent_files))
        .route("/files/favorite", get(list_favorites))
        // Sharing and permissions
        .route("/files/shareFolder", post(share_folder))
        .route("/files/shared", get(list_shared))
        .route("/files/permissions", get(get_permissions))
        // Storage management
        .route("/files/quota", get(get_quota))
        // Sync operations
        .route("/files/sync/status", get(sync_status))
        .route("/files/sync/start", post(start_sync))
        .route("/files/sync/stop", post(stop_sync))
        // Document processing
        .route("/docs/merge", post(document_processing::merge_documents))
        .route("/docs/convert", post(document_processing::convert_document))
        .route("/docs/fill", post(document_processing::fill_document))
        .route("/docs/export", post(document_processing::export_document))
        .route("/docs/import", post(document_processing::import_document))
}

// ===== API Handlers =====

/// GET /files/list - List files and folders in S3 bucket
pub async fn list_files(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListQuery>,
) -> Result<Json<Vec<FileItem>>, (StatusCode, Json<serde_json::Value>)> {
    // Use FileTree for hierarchical navigation when console feature is enabled
    #[cfg(feature = "console")]
    let result = {
        let mut tree = FileTree::new(state.clone());
        if let Some(bucket) = &params.bucket {
            if let Some(path) = &params.path {
                tree.enter_folder(bucket.clone(), path.clone()).await
            } else {
                tree.list_root(bucket.clone()).await
            }
        } else {
            tree.list_buckets().await
        }
    };

    #[cfg(not(feature = "console"))]
    let result: Result<Vec<FileItem>, (StatusCode, Json<serde_json::Value>)> = {
        // Fallback implementation without FileTree
        let s3_client = state.drive.as_ref().ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "S3 client not configured"})),
            )
        })?;

        if let Some(bucket) = &params.bucket {
            let mut items = Vec::new();
            let prefix = params.path.as_deref().unwrap_or("");

            let paginator = s3_client
                .list_objects_v2()
                .bucket(bucket)
                .prefix(prefix)
                .delimiter("/")
                .into_paginator()
                .send();

            use futures_util::TryStreamExt;

            let mut stream = paginator;
            while let Some(result) = stream.try_next().await.map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": e.to_string()})),
                )
            })? {
                // Add directories
                if let Some(prefixes) = result.common_prefixes {
                    for prefix in prefixes {
                        if let Some(dir) = prefix.prefix {
                            let name = dir
                                .trim_end_matches('/')
                                .split('/')
                                .last()
                                .unwrap_or(&dir)
                                .to_string();
                            items.push(FileItem {
                                name,
                                path: dir.clone(),
                                is_dir: true,
                                size: None,
                                modified: None,
                                icon: get_file_icon(&dir),
                            });
                        }
                    }
                }

                // Add files
                if let Some(contents) = result.contents {
                    for object in contents {
                        if let Some(key) = object.key {
                            if !key.ends_with('/') {
                                let name = key.split('/').last().unwrap_or(&key).to_string();
                                items.push(FileItem {
                                    name,
                                    path: key.clone(),
                                    is_dir: false,
                                    size: object.size.map(|s| s as i64),
                                    modified: object.last_modified.map(|t| t.to_string()),
                                    icon: get_file_icon(&key),
                                });
                            }
                        }
                    }
                }
            }
            Ok(items)
        } else {
            Ok(vec![])
        }
    };

    match result {
        Ok(items) => Ok(Json(items)),
        Err(e) => Err(e),
    }
}

#[cfg(feature = "console")]
fn convert_tree_to_items(_tree: &FileTree) -> Vec<FileItem> {
    // TODO: Implement tree conversion when console feature is available
    vec![]
}

/// POST /files/read - Read file content from S3
pub async fn read_file(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ReadRequest>,
) -> Result<Json<ReadResponse>, (StatusCode, Json<serde_json::Value>)> {
    let s3_client = state.drive.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "S3 service not available" })),
        )
    })?;

    let result = s3_client
        .get_object()
        .bucket(&req.bucket)
        .key(&req.path)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to read file: {}", e) })),
            )
        })?;

    let bytes = result
        .body
        .collect()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to read file body: {}", e) })),
            )
        })?
        .into_bytes();

    let content = String::from_utf8(bytes.to_vec()).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("File is not valid UTF-8: {}", e) })),
        )
    })?;

    Ok(Json(ReadResponse { content }))
}

/// POST /files/write - Write file content to S3
pub async fn write_file(
    State(state): State<Arc<AppState>>,
    Json(req): Json<WriteRequest>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    let s3_client = state.drive.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "S3 service not available" })),
        )
    })?;

    s3_client
        .put_object()
        .bucket(&req.bucket)
        .key(&req.path)
        .body(req.content.into_bytes().into())
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to write file: {}", e) })),
            )
        })?;

    Ok(Json(SuccessResponse {
        success: true,
        message: Some("File written successfully".to_string()),
    }))
}

/// POST /files/delete - Delete file or folder from S3
pub async fn delete_file(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DeleteRequest>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    let s3_client = state.drive.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "S3 service not available" })),
        )
    })?;

    // If path ends with /, it's a folder - delete all objects with this prefix
    if req.path.ends_with('/') {
        let result = s3_client
            .list_objects_v2()
            .bucket(&req.bucket)
            .prefix(&req.path)
            .send()
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": format!("Failed to list objects for deletion: {}", e) })),
                )
            })?;

        for obj in result.contents() {
            if let Some(key) = obj.key() {
                s3_client
                    .delete_object()
                    .bucket(&req.bucket)
                    .key(key)
                    .send()
                    .await
                    .map_err(|e| {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(serde_json::json!({ "error": format!("Failed to delete object: {}", e) })),
                        )
                    })?;
            }
        }
    } else {
        s3_client
            .delete_object()
            .bucket(&req.bucket)
            .key(&req.path)
            .send()
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": format!("Failed to delete file: {}", e) })),
                )
            })?;
    }

    Ok(Json(SuccessResponse {
        success: true,
        message: Some("Deleted successfully".to_string()),
    }))
}

/// POST /files/create-folder - Create new folder in S3
pub async fn create_folder(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateFolderRequest>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    let s3_client = state.drive.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "S3 service not available" })),
        )
    })?;

    // S3 doesn't have real folders, create an empty object with trailing /
    let folder_path = if req.path.is_empty() || req.path == "/" {
        format!("{}/", req.name)
    } else {
        format!("{}/{}/", req.path.trim_end_matches('/'), req.name)
    };

    s3_client
        .put_object()
        .bucket(&req.bucket)
        .key(&folder_path)
        .body(Vec::new().into())
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to create folder: {}", e) })),
            )
        })?;

    Ok(Json(SuccessResponse {
        success: true,
        message: Some("Folder created successfully".to_string()),
    }))
}

// ===== Helper Functions =====

/// Get appropriate icon for file based on extension
fn get_file_icon(path: &str) -> String {
    if path.ends_with(".bas") {
        "⚙️".to_string()
    } else if path.ends_with(".ast") {
        "🔧".to_string()
    } else if path.ends_with(".csv") {
        "📊".to_string()
    } else if path.ends_with(".gbkb") {
        "📚".to_string()
    } else if path.ends_with(".json") {
        "🔖".to_string()
    } else if path.ends_with(".txt") || path.ends_with(".md") {
        "📃".to_string()
    } else if path.ends_with(".pdf") {
        "📕".to_string()
    } else if path.ends_with(".zip") || path.ends_with(".tar") || path.ends_with(".gz") {
        "📦".to_string()
    } else if path.ends_with(".jpg") || path.ends_with(".png") || path.ends_with(".gif") {
        "🖼️".to_string()
    } else {
        "📄".to_string()
    }
}

// ===== Extended File Operations =====

/// POST /files/copy - Copy file or folder within S3
pub async fn copy_file(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CopyRequest>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    let s3_client = state.drive.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "S3 service not available" })),
        )
    })?;

    let copy_source = format!("{}/{}", req.source_bucket, req.source_path);

    s3_client
        .copy_object()
        .copy_source(&copy_source)
        .bucket(&req.dest_bucket)
        .key(&req.dest_path)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to copy file: {}", e) })),
            )
        })?;

    Ok(Json(SuccessResponse {
        success: true,
        message: Some("File copied successfully".to_string()),
    }))
}

/// POST /files/move - Move file or folder within S3
pub async fn move_file(
    State(state): State<Arc<AppState>>,
    Json(req): Json<MoveRequest>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    let s3_client = state.drive.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "S3 service not available" })),
        )
    })?;

    let copy_source = format!("{}/{}", req.source_bucket, req.source_path);

    s3_client
        .copy_object()
        .copy_source(&copy_source)
        .bucket(&req.dest_bucket)
        .key(&req.dest_path)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to move file: {}", e) })),
            )
        })?;

    s3_client
        .delete_object()
        .bucket(&req.source_bucket)
        .key(&req.source_path)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    serde_json::json!({ "error": format!("Failed to delete source file: {}", e) }),
                ),
            )
        })?;

    Ok(Json(SuccessResponse {
        success: true,
        message: Some("File moved successfully".to_string()),
    }))
}

/// POST /files/upload - Upload file to S3
pub async fn upload_file_to_drive(
    State(state): State<Arc<AppState>>,
    Json(req): Json<WriteRequest>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    write_file(State(state), Json(req)).await
}

/// POST /files/download - Download file from S3
pub async fn download_file(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DownloadRequest>,
) -> Result<Json<ReadResponse>, (StatusCode, Json<serde_json::Value>)> {
    read_file(
        State(state),
        Json(ReadRequest {
            bucket: req.bucket,
            path: req.path,
        }),
    )
    .await
}

/// POST /files/dirFolder - List folder contents
pub async fn list_folder_contents(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ReadRequest>,
) -> Result<Json<Vec<FileItem>>, (StatusCode, Json<serde_json::Value>)> {
    list_files(
        State(state),
        Query(ListQuery {
            path: Some(req.path),
            bucket: Some(req.bucket),
        }),
    )
    .await
}

/// GET /files/search - Search for files
pub async fn search_files(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<Vec<FileItem>>, (StatusCode, Json<serde_json::Value>)> {
    let s3_client = state.drive.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "S3 service not available" })),
        )
    })?;

    let mut all_items = Vec::new();
    let buckets = if let Some(bucket) = &params.bucket {
        vec![bucket.clone()]
    } else {
        let result = s3_client.list_buckets().send().await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to list buckets: {}", e) })),
            )
        })?;
        result
            .buckets()
            .iter()
            .filter_map(|b| b.name().map(String::from))
            .collect()
    };

    for bucket in buckets {
        let result = s3_client
            .list_objects_v2()
            .bucket(&bucket)
            .send()
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": format!("Failed to list objects: {}", e) })),
                )
            })?;

        for obj in result.contents() {
            if let Some(key) = obj.key() {
                let name = key.split('/').last().unwrap_or(key).to_lowercase();
                let query_lower = params.query.to_lowercase();

                if name.contains(&query_lower) {
                    if let Some(file_type) = &params.file_type {
                        if key.ends_with(file_type) {
                            all_items.push(FileItem {
                                name: name.to_string(),
                                path: key.to_string(),
                                is_dir: false,
                                size: obj.size(),
                                modified: obj.last_modified().map(|t| t.to_string()),
                                icon: get_file_icon(key),
                            });
                        }
                    } else {
                        all_items.push(FileItem {
                            name: name.to_string(),
                            path: key.to_string(),
                            is_dir: false,
                            size: obj.size(),
                            modified: obj.last_modified().map(|t| t.to_string()),
                            icon: get_file_icon(key),
                        });
                    }
                }
            }
        }
    }

    Ok(Json(all_items))
}

/// GET /files/recent - Get recently modified files
pub async fn recent_files(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListQuery>,
) -> Result<Json<Vec<FileItem>>, (StatusCode, Json<serde_json::Value>)> {
    let s3_client = state.drive.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "S3 service not available" })),
        )
    })?;

    let mut all_items = Vec::new();
    let buckets = if let Some(bucket) = &params.bucket {
        vec![bucket.clone()]
    } else {
        let result = s3_client.list_buckets().send().await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to list buckets: {}", e) })),
            )
        })?;
        result
            .buckets()
            .iter()
            .filter_map(|b| b.name().map(String::from))
            .collect()
    };

    for bucket in buckets {
        let result = s3_client
            .list_objects_v2()
            .bucket(&bucket)
            .send()
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": format!("Failed to list objects: {}", e) })),
                )
            })?;

        for obj in result.contents() {
            if let Some(key) = obj.key() {
                all_items.push(FileItem {
                    name: key.split('/').last().unwrap_or(key).to_string(),
                    path: key.to_string(),
                    is_dir: false,
                    size: obj.size(),
                    modified: obj.last_modified().map(|t| t.to_string()),
                    icon: get_file_icon(key),
                });
            }
        }
    }

    all_items.sort_by(|a, b| b.modified.cmp(&a.modified));
    all_items.truncate(50);

    Ok(Json(all_items))
}

/// GET /files/favorite - List favorite files
pub async fn list_favorites(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<Vec<FileItem>>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(Vec::new()))
}

/// POST /files/shareFolder - Share folder with users
pub async fn share_folder(
    State(_state): State<Arc<AppState>>,
    Json(_req): Json<ShareRequest>,
) -> Result<Json<ShareResponse>, (StatusCode, Json<serde_json::Value>)> {
    let share_id = uuid::Uuid::new_v4().to_string();
    let url = format!("https://share.example.com/{}", share_id);

    Ok(Json(ShareResponse {
        share_id,
        url,
        expires_at: Some(
            chrono::Utc::now()
                .checked_add_signed(chrono::Duration::days(7))
                .unwrap()
                .to_rfc3339(),
        ),
    }))
}

/// GET /files/shared - List shared files and folders
pub async fn list_shared(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<Vec<FileItem>>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(Vec::new()))
}

/// GET /files/permissions - Get file/folder permissions
pub async fn get_permissions(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<ReadRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(serde_json::json!({
        "bucket": params.bucket,
        "path": params.path,
        "permissions": {
            "read": true,
            "write": true,
            "delete": true,
            "share": true
        },
        "shared_with": []
    })))
}

/// GET /files/quota - Get storage quota information
pub async fn get_quota(
    State(state): State<Arc<AppState>>,
) -> Result<Json<QuotaResponse>, (StatusCode, Json<serde_json::Value>)> {
    let s3_client = state.drive.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "S3 service not available" })),
        )
    })?;

    let mut total_size = 0i64;

    let result = s3_client.list_buckets().send().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to list buckets: {}", e) })),
        )
    })?;

    let buckets: Vec<String> = result
        .buckets()
        .iter()
        .filter_map(|b| b.name().map(String::from))
        .collect();

    for bucket in buckets {
        let list_result = s3_client
            .list_objects_v2()
            .bucket(&bucket)
            .send()
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": format!("Failed to list objects: {}", e) })),
                )
            })?;

        for obj in list_result.contents() {
            total_size += obj.size().unwrap_or(0);
        }
    }

    let total_bytes = 100_000_000_000i64;
    let used_bytes = total_size;
    let available_bytes = total_bytes - used_bytes;
    let percentage_used = (used_bytes as f64 / total_bytes as f64) * 100.0;

    Ok(Json(QuotaResponse {
        total_bytes,
        used_bytes,
        available_bytes,
        percentage_used,
    }))
}

/// GET /files/sync/status - Get sync status
pub async fn sync_status(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<SyncStatus>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(SyncStatus {
        status: "idle".to_string(),
        last_sync: Some(chrono::Utc::now().to_rfc3339()),
        files_synced: 0,
        bytes_synced: 0,
    }))
}

/// POST /files/sync/start - Start file synchronization
pub async fn start_sync(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(SuccessResponse {
        success: true,
        message: Some("Sync started".to_string()),
    }))
}

/// POST /files/sync/stop - Stop file synchronization
pub async fn stop_sync(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(SuccessResponse {
        success: true,
        message: Some("Sync stopped".to_string()),
    }))
}
