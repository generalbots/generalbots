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

use crate::shared::state::AppState;
use crate::ui_tree::file_tree::{FileTree, TreeNode};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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

#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub success: bool,
    pub message: Option<String>,
}

// ===== API Configuration =====

/// Configure drive API routes
pub fn configure() -> Router<Arc<AppState>> {
    Router::new()
        .route("/files/list", get(list_files))
        .route("/files/read", post(read_file))
        .route("/files/write", post(write_file))
        .route("/files/delete", post(delete_file))
        .route("/files/create-folder", post(create_folder))
}

// ===== API Handlers =====

/// GET /files/list - List files and folders in S3 bucket
pub async fn list_files(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListQuery>,
) -> Result<Json<Vec<FileItem>>, (StatusCode, Json<serde_json::Value>)> {
    // Use FileTree for hierarchical navigation
    let mut tree = FileTree::new(state.clone());

    let result = if let Some(bucket) = &params.bucket {
        if let Some(path) = &params.path {
            tree.enter_folder(bucket.clone(), path.clone()).await
        } else {
            tree.enter_bucket(bucket.clone()).await
        }
    } else {
        tree.load_root().await
    };

    if let Err(e) = result {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ));
    }

    let items: Vec<FileItem> = tree
        .render_items()
        .iter()
        .map(|(display, node)| {
            let (name, path, is_dir, icon) = match node {
                TreeNode::Bucket { name } => {
                    let icon = if name.ends_with(".gbai") {
                        "🤖"
                    } else {
                        "📦"
                    };
                    (name.clone(), name.clone(), true, icon.to_string())
                }
                TreeNode::Folder { bucket, path } => {
                    let name = path.split('/').last().unwrap_or(path).to_string();
                    (name, path.clone(), true, "📁".to_string())
                }
                TreeNode::File { bucket, path } => {
                    let name = path.split('/').last().unwrap_or(path).to_string();
                    let icon = get_file_icon(path);
                    (name, path.clone(), false, icon)
                }
            };

            FileItem {
                name,
                path,
                is_dir,
                size: None,
                modified: None,
                icon,
            }
        })
        .collect();

    Ok(Json(items))
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
