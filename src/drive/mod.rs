#[cfg(feature = "console")]
use crate::console::file_tree::FileTree;
use crate::shared::state::AppState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};

use serde::{Deserialize, Serialize};

use std::sync::Arc;

pub mod document_processing;
pub mod drive_monitor;
pub mod vectordb;

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
    pub is_desktop: bool,
    pub message: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct VersionsQuery {
    pub bucket: Option<String>,
    pub path: String,
}

#[derive(Debug, Serialize)]
pub struct FileVersion {
    pub version_id: String,
    pub modified: String,
    pub size: i64,
    pub is_latest: bool,
    pub etag: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct VersionsResponse {
    pub path: String,
    pub versions: Vec<FileVersion>,
}

#[derive(Debug, Deserialize)]
pub struct RestoreRequest {
    pub bucket: Option<String>,
    pub path: String,
    pub version_id: String,
}

#[derive(Debug, Serialize)]
pub struct RestoreResponse {
    pub success: bool,
    pub message: String,
    pub restored_version: String,
    pub new_version_id: Option<String>,
}

#[allow(unused)]
pub fn configure() -> Router<Arc<AppState>> {
    Router::new()
        .route("/files/list", get(list_files))
        .route("/files/read", post(read_file))
        .route("/files/write", post(write_file))
        .route("/files/save", post(write_file))
        .route("/files/getContents", post(read_file))
        .route("/files/delete", post(delete_file))
        .route("/files/upload", post(upload_file_to_drive))
        .route("/files/download", post(download_file))
        .route("/files/copy", post(copy_file))
        .route("/files/move", post(move_file))
        .route("/files/createFolder", post(create_folder))
        .route("/files/create-folder", post(create_folder))
        .route("/files/dirFolder", post(list_folder_contents))
        .route("/files/search", get(search_files))
        .route("/files/recent", get(recent_files))
        .route("/files/favorite", get(list_favorites))
        .route("/files/shareFolder", post(share_folder))
        .route("/files/shared", get(list_shared))
        .route("/files/permissions", get(get_permissions))
        .route("/files/quota", get(get_quota))
        .route("/files/sync/status", get(sync_status))
        .route("/files/sync/start", post(start_sync))
        .route("/files/sync/stop", post(stop_sync))
        .route("/files/versions", get(list_versions))
        .route("/files/restore", post(restore_version))
        .route("/docs/merge", post(document_processing::merge_documents))
        .route("/docs/convert", post(document_processing::convert_document))
        .route("/docs/fill", post(document_processing::fill_document))
        .route("/docs/export", post(document_processing::export_document))
        .route("/docs/import", post(document_processing::import_document))
}

pub async fn list_files(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListQuery>,
) -> Result<Json<Vec<FileItem>>, (StatusCode, Json<serde_json::Value>)> {
    #[cfg(feature = "console")]
    let result = {
        let mut tree = FileTree::new(state.clone());
        if let Some(bucket) = &params.bucket {
            if let Some(path) = &params.path {
                tree.enter_folder(bucket.clone(), path.clone()).await.ok();
            } else {
                tree.enter_bucket(bucket.clone()).await.ok();
            }
        } else {
            tree.load_root().await.ok();
        }

        Ok::<Vec<FileItem>, (StatusCode, Json<serde_json::Value>)>(vec![])
    };

    #[cfg(not(feature = "console"))]
    let result: Result<Vec<FileItem>, (StatusCode, Json<serde_json::Value>)> = {
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

            let mut stream = paginator;
            while let Some(result) = stream.try_next().await.map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": e.to_string()})),
                )
            })? {
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
pub fn convert_tree_to_items(tree: &FileTree) -> Vec<FileItem> {
    let mut items = Vec::new();

    for (display_name, node) in tree.get_items() {
        match node {
            crate::console::file_tree::TreeNode::Bucket { name } => {
                if !name.is_empty() {
                    items.push(FileItem {
                        name: display_name.clone(),
                        path: format!("/{}", name),
                        is_dir: true,
                        size: None,
                        modified: None,
                        icon: if name.to_ascii_lowercase().ends_with(".gbai") {
                            "".to_string()
                        } else {
                            "📦".to_string()
                        },
                    });
                }
            }
            crate::console::file_tree::TreeNode::Folder { bucket, path } => {
                let folder_name = path.split('/').next_back().unwrap_or(&display_name);
                items.push(FileItem {
                    name: folder_name.to_string(),
                    path: format!("/{}/{}", bucket, path),
                    is_dir: true,
                    size: None,
                    modified: None,
                    icon: "📁".to_string(),
                });
            }
            crate::console::file_tree::TreeNode::File { bucket, path } => {
                let file_name = path.split('/').next_back().unwrap_or(&display_name);
                items.push(FileItem {
                    name: file_name.to_string(),
                    path: format!("/{}/{}", bucket, path),
                    is_dir: false,
                    size: None,
                    modified: None,
                    icon: "📄".to_string(),
                });
            }
        }
    }

    items
}

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

fn get_file_icon(path: &str) -> String {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    match ext.as_deref() {
        Some("bas") => "".to_string(),
        Some("ast") => "".to_string(),
        Some("csv") => "".to_string(),
        Some("gbkb") => "".to_string(),
        Some("json") => "🔖".to_string(),
        Some("txt" | "md") => "📃".to_string(),
        Some("pdf") => "📕".to_string(),
        Some("zip" | "tar" | "gz") => "📦".to_string(),
        Some("jpg" | "png" | "gif") | _ => "📄".to_string(),
    }
}

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

pub async fn upload_file_to_drive(
    State(state): State<Arc<AppState>>,
    Json(req): Json<WriteRequest>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    write_file(State(state), Json(req)).await
}

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
    let buckets = if let Some(bucket) = params.bucket.as_ref() {
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
                let name = key.split('/').next_back().unwrap_or(key).to_lowercase();
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
                    name: key.split('/').next_back().unwrap_or(key).to_string(),
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

pub async fn list_favorites(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<Vec<FileItem>>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(Vec::new()))
}

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
                .checked_add_signed(chrono::Duration::hours(24))
                .unwrap()
                .to_rfc3339(),
        ),
    }))
}

pub async fn list_shared(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<Vec<FileItem>>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(Vec::new()))
}

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

pub async fn sync_status(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<SyncStatus>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(SyncStatus {
        status: "unavailable".to_string(),
        last_sync: None,
        files_synced: 0,
        bytes_synced: 0,
        is_desktop: false,
        message: Some(
            "File sync requires the General Bots desktop app with rclone installed".to_string(),
        ),
    }))
}

pub async fn start_sync(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(SuccessResponse {
        success: false,
        message: Some("File sync requires the General Bots desktop app. Install rclone and use the desktop app to sync files.".to_string()),
    }))
}

pub async fn stop_sync(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(SuccessResponse {
        success: false,
        message: Some("File sync requires the General Bots desktop app".to_string()),
    }))
}

pub async fn list_versions(
    State(state): State<Arc<AppState>>,
    Query(params): Query<VersionsQuery>,
) -> Result<Json<VersionsResponse>, (StatusCode, Json<serde_json::Value>)> {
    let bucket = params.bucket.unwrap_or_else(|| "default".to_string());
    let path = params.path;

    let s3_client = state.s3_client.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "S3 storage not configured" })),
        )
    })?;

    let versions_result = s3_client
        .list_object_versions()
        .bucket(&bucket)
        .prefix(&path)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to list versions: {}", e) })),
            )
        })?;

    let mut versions: Vec<FileVersion> = Vec::new();

    for version in versions_result.versions() {
        if version.key().unwrap_or_default() == path {
            versions.push(FileVersion {
                version_id: version.version_id().unwrap_or("null").to_string(),
                modified: version
                    .last_modified()
                    .map(|t| t.to_string())
                    .unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
                size: version.size().unwrap_or(0),
                is_latest: version.is_latest().unwrap_or(false),
                etag: version.e_tag().map(|s| s.to_string()),
            });
        }
    }

    versions.sort_by(|a, b| b.modified.cmp(&a.modified));

    Ok(Json(VersionsResponse {
        path: path.clone(),
        versions,
    }))
}

pub async fn restore_version(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RestoreRequest>,
) -> Result<Json<RestoreResponse>, (StatusCode, Json<serde_json::Value>)> {
    let bucket = payload.bucket.unwrap_or_else(|| "default".to_string());
    let path = payload.path;
    let version_id = payload.version_id;

    let s3_client = state.s3_client.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "S3 storage not configured" })),
        )
    })?;

    let copy_source = format!("{}/{}?versionId={}", bucket, path, version_id);

    let copy_result = s3_client
        .copy_object()
        .bucket(&bucket)
        .key(&path)
        .copy_source(&copy_source)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to restore version: {}", e) })),
            )
        })?;

    let new_version_id = copy_result.version_id().map(|s| s.to_string());

    Ok(Json(RestoreResponse {
        success: true,
        message: format!("Successfully restored {} to version {}", path, version_id),
        restored_version: version_id,
        new_version_id,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    // Test structures for MinIO/S3-like storage service tests from bottest/services/minio.rs

    #[derive(Debug, Clone)]
    struct MinioTestConfig {
        api_port: u16,
        console_port: u16,
        data_dir: PathBuf,
        access_key: String,
        secret_key: String,
    }

    impl Default for MinioTestConfig {
        fn default() -> Self {
            Self {
                api_port: 9000,
                console_port: 10000,
                data_dir: PathBuf::from("/tmp/test"),
                access_key: "minioadmin".to_string(),
                secret_key: "minioadmin".to_string(),
            }
        }
    }

    impl MinioTestConfig {
        fn endpoint(&self) -> String {
            format!("http://127.0.0.1:{}", self.api_port)
        }

        fn console_url(&self) -> String {
            format!("http://127.0.0.1:{}", self.console_port)
        }

        fn credentials(&self) -> (String, String) {
            (self.access_key.clone(), self.secret_key.clone())
        }

        fn s3_config(&self) -> HashMap<String, String> {
            let mut config = HashMap::new();
            config.insert("endpoint_url".to_string(), self.endpoint());
            config.insert("access_key_id".to_string(), self.access_key.clone());
            config.insert("secret_access_key".to_string(), self.secret_key.clone());
            config.insert("region".to_string(), "us-east-1".to_string());
            config.insert("force_path_style".to_string(), "true".to_string());
            config
        }
    }

    #[test]
    fn test_endpoint_format() {
        let config = MinioTestConfig {
            api_port: 9000,
            console_port: 10000,
            ..Default::default()
        };

        assert_eq!(config.endpoint(), "http://127.0.0.1:9000");
        assert_eq!(config.console_url(), "http://127.0.0.1:10000");
    }

    #[test]
    fn test_credentials() {
        let config = MinioTestConfig {
            access_key: "mykey".to_string(),
            secret_key: "mysecret".to_string(),
            ..Default::default()
        };

        let (key, secret) = config.credentials();
        assert_eq!(key, "mykey");
        assert_eq!(secret, "mysecret");
    }

    #[test]
    fn test_s3_config() {
        let config = MinioTestConfig {
            api_port: 9000,
            access_key: "access".to_string(),
            secret_key: "secret".to_string(),
            ..Default::default()
        };

        let s3_config = config.s3_config();
        assert_eq!(
            s3_config.get("endpoint_url"),
            Some(&"http://127.0.0.1:9000".to_string())
        );
        assert_eq!(s3_config.get("access_key_id"), Some(&"access".to_string()));
        assert_eq!(s3_config.get("force_path_style"), Some(&"true".to_string()));
    }

    #[test]
    fn test_file_item_creation() {
        let item = FileItem {
            name: "test.txt".to_string(),
            path: "/documents/test.txt".to_string(),
            is_dir: false,
            size: Some(1024),
            modified: Some("2024-01-15T10:30:00Z".to_string()),
            icon: "file-text".to_string(),
        };

        assert_eq!(item.name, "test.txt");
        assert!(!item.is_dir);
        assert_eq!(item.size, Some(1024));
    }

    #[test]
    fn test_file_item_directory() {
        let item = FileItem {
            name: "documents".to_string(),
            path: "/documents".to_string(),
            is_dir: true,
            size: None,
            modified: Some("2024-01-15T10:30:00Z".to_string()),
            icon: "folder".to_string(),
        };

        assert!(item.is_dir);
        assert!(item.size.is_none());
    }

    #[test]
    fn test_list_query() {
        let query = ListQuery {
            path: Some("/documents".to_string()),
            bucket: Some("my-bucket".to_string()),
        };

        assert_eq!(query.path, Some("/documents".to_string()));
        assert_eq!(query.bucket, Some("my-bucket".to_string()));
    }

    #[test]
    fn test_read_request() {
        let request = ReadRequest {
            bucket: "test-bucket".to_string(),
            path: "folder/file.txt".to_string(),
        };

        assert_eq!(request.bucket, "test-bucket");
        assert_eq!(request.path, "folder/file.txt");
    }

    #[test]
    fn test_write_request() {
        let request = WriteRequest {
            bucket: "test-bucket".to_string(),
            path: "folder/newfile.txt".to_string(),
            content: "Hello, World!".to_string(),
        };

        assert_eq!(request.bucket, "test-bucket");
        assert_eq!(request.content, "Hello, World!");
    }

    #[test]
    fn test_delete_request() {
        let request = DeleteRequest {
            bucket: "test-bucket".to_string(),
            path: "folder/delete-me.txt".to_string(),
        };

        assert_eq!(request.bucket, "test-bucket");
        assert_eq!(request.path, "folder/delete-me.txt");
    }

    #[test]
    fn test_create_folder_request() {
        let request = CreateFolderRequest {
            bucket: "test-bucket".to_string(),
            path: "/documents".to_string(),
            name: "new-folder".to_string(),
        };

        assert_eq!(request.name, "new-folder");
        assert_eq!(request.path, "/documents");
    }

    #[test]
    fn test_copy_request() {
        let request = CopyRequest {
            source_bucket: "bucket-a".to_string(),
            source_path: "file.txt".to_string(),
            dest_bucket: "bucket-b".to_string(),
            dest_path: "copied-file.txt".to_string(),
        };

        assert_eq!(request.source_bucket, "bucket-a");
        assert_eq!(request.dest_bucket, "bucket-b");
    }

    #[test]
    fn test_move_request() {
        let request = MoveRequest {
            source_bucket: "bucket-a".to_string(),
            source_path: "old-location/file.txt".to_string(),
            dest_bucket: "bucket-a".to_string(),
            dest_path: "new-location/file.txt".to_string(),
        };

        assert_eq!(request.source_path, "old-location/file.txt");
        assert_eq!(request.dest_path, "new-location/file.txt");
    }

    #[test]
    fn test_search_query() {
        let query = SearchQuery {
            bucket: Some("test-bucket".to_string()),
            query: "report".to_string(),
            file_type: Some("pdf".to_string()),
        };

        assert_eq!(query.query, "report");
        assert_eq!(query.file_type, Some("pdf".to_string()));
    }

    #[test]
    fn test_share_request() {
        let request = ShareRequest {
            bucket: "test-bucket".to_string(),
            path: "shared-folder".to_string(),
            users: vec![
                "user1@example.com".to_string(),
                "user2@example.com".to_string(),
            ],
            permissions: "read".to_string(),
        };

        assert_eq!(request.users.len(), 2);
        assert_eq!(request.permissions, "read");
    }

    #[test]
    fn test_success_response() {
        let response = SuccessResponse {
            success: true,
            message: Some("Operation completed successfully".to_string()),
        };

        assert!(response.success);
        assert!(response
            .message
            .as_ref()
            .is_some_and(|m| m.contains("successfully")));
    }

    #[test]
    fn test_quota_response() {
        let response = QuotaResponse {
            total_bytes: 1_073_741_824,   // 1 GB
            used_bytes: 536_870_912,      // 512 MB
            available_bytes: 536_870_912, // 512 MB
            percentage_used: 50.0,
        };

        assert_eq!(response.percentage_used, 50.0);
        assert_eq!(
            response.total_bytes,
            response.used_bytes + response.available_bytes
        );
    }

    #[test]
    fn test_share_response() {
        let response = ShareResponse {
            share_id: "share-12345".to_string(),
            url: "https://example.com/share/share-12345".to_string(),
            expires_at: Some("2024-12-31T23:59:59Z".to_string()),
        };

        assert!(response.url.contains("share-12345"));
        assert!(response.expires_at.is_some());
    }

    #[test]
    fn test_sync_status() {
        let status = SyncStatus {
            status: "syncing".to_string(),
            last_sync: Some("2024-01-15T10:30:00Z".to_string()),
            files_synced: 150,
            bytes_synced: 52_428_800,
            is_desktop: true,
            message: Some("Syncing in progress...".to_string()),
        };

        assert_eq!(status.status, "syncing");
        assert_eq!(status.files_synced, 150);
        assert!(status.is_desktop);
    }

    #[test]
    fn test_file_version() {
        let version = FileVersion {
            version_id: "v1234567890".to_string(),
            modified: "2024-01-15T10:30:00Z".to_string(),
            size: 2048,
            is_latest: true,
            etag: Some("abc123def456".to_string()),
        };

        assert!(version.is_latest);
        assert_eq!(version.size, 2048);
    }

    #[test]
    fn test_versions_response() {
        let versions = vec![
            FileVersion {
                version_id: "v2".to_string(),
                modified: "2024-01-15T12:00:00Z".to_string(),
                size: 2048,
                is_latest: true,
                etag: Some("etag2".to_string()),
            },
            FileVersion {
                version_id: "v1".to_string(),
                modified: "2024-01-15T10:00:00Z".to_string(),
                size: 1024,
                is_latest: false,
                etag: Some("etag1".to_string()),
            },
        ];

        let response = VersionsResponse {
            path: "documents/report.pdf".to_string(),
            versions,
        };

        assert_eq!(response.versions.len(), 2);
        assert!(response.versions[0].is_latest);
        assert!(!response.versions[1].is_latest);
    }

    #[test]
    fn test_restore_request() {
        let request = RestoreRequest {
            bucket: Some("test-bucket".to_string()),
            path: "documents/file.txt".to_string(),
            version_id: "v1234567890".to_string(),
        };

        assert_eq!(request.version_id, "v1234567890");
    }

    #[test]
    fn test_restore_response() {
        let response = RestoreResponse {
            success: true,
            message: "File restored successfully".to_string(),
            restored_version: "v1234567890".to_string(),
            new_version_id: Some("v9876543210".to_string()),
        };

        assert!(response.success);
        assert!(response.new_version_id.is_some());
    }

    #[test]
    fn test_get_file_icon() {
        assert_eq!(get_file_icon("document.pdf"), "file-text");
        assert_eq!(get_file_icon("image.png"), "image");
        assert_eq!(get_file_icon("image.jpg"), "image");
        assert_eq!(get_file_icon("video.mp4"), "video");
        assert_eq!(get_file_icon("music.mp3"), "music");
        assert_eq!(get_file_icon("archive.zip"), "archive");
        assert_eq!(get_file_icon("unknown.xyz"), "file");
    }

    #[test]
    fn test_default_minio_credentials() {
        let config = MinioTestConfig::default();
        assert_eq!(config.access_key, "minioadmin");
        assert_eq!(config.secret_key, "minioadmin");
    }

    #[test]
    fn test_custom_port_configuration() {
        let config = MinioTestConfig {
            api_port: 19000,
            console_port: 19001,
            ..Default::default()
        };

        assert!(config.endpoint().contains("19000"));
        assert!(config.console_url().contains("19001"));
    }

    #[test]
    fn test_download_request() {
        let request = DownloadRequest {
            bucket: "my-bucket".to_string(),
            path: "downloads/file.zip".to_string(),
        };

        assert_eq!(request.bucket, "my-bucket");
        assert!(request.path.ends_with(".zip"));
    }
}
