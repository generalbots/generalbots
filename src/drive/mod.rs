













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
    pub _bucket: String,
    pub _path: String,
    pub _users: Vec<String>,
    pub _permissions: String,
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
                        icon: if name.ends_with(".gbai") {
                            "".to_string()
                        } else {
                            "📦".to_string()
                        },
                    });
                }
            }
            crate::console::file_tree::TreeNode::Folder { bucket, path } => {
                let folder_name = path.split('/').last().unwrap_or(&display_name);
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
                let file_name = path.split('/').last().unwrap_or(&display_name);
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
    if path.ends_with(".bas") {
        "".to_string()
    } else if path.ends_with(".ast") {
        "".to_string()
    } else if path.ends_with(".csv") {
        "".to_string()
    } else if path.ends_with(".gbkb") {
        "".to_string()
    } else if path.ends_with(".json") {
        "🔖".to_string()
    } else if path.ends_with(".txt") || path.ends_with(".md") {
        "📃".to_string()
    } else if path.ends_with(".pdf") {
        "📕".to_string()
    } else if path.ends_with(".zip") || path.ends_with(".tar") || path.ends_with(".gz") {
        "📦".to_string()
    } else if path.ends_with(".jpg") || path.ends_with(".png") || path.ends_with(".gif") {
        "".to_string()
    } else {
        "📄".to_string()
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
        percentage_used: percentage_used as f64,
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
