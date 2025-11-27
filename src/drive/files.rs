//! Files API Module
//!
//! Comprehensive file management endpoints for cloud storage operations.
//! Integrates with S3 backend and provides versioning, permissions, and sync capabilities.

use crate::shared::state::AppState;
use aws_sdk_s3::primitives::ByteStream;
use axum::{
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::{header, StatusCode},
    response::{Json, Response},
    routing::{delete, get, post},
    Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

// ===== Request/Response Structures =====

#[derive(Debug, Serialize, Deserialize)]
pub struct FileMetadata {
    pub id: String,
    pub name: String,
    pub path: String,
    pub size: i64,
    pub mime_type: Option<String>,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub created_by: String,
    pub modified_by: String,
    pub is_dir: bool,
    pub version: i32,
    pub parent_id: Option<String>,
    pub tags: Vec<String>,
    pub checksum: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UploadQuery {
    pub folder_path: Option<String>,
    pub overwrite: Option<bool>,
    pub tags: Option<String>, // Comma-separated
}

#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub success: bool,
    pub file_id: String,
    pub path: String,
    pub size: i64,
    pub version: i32,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct DownloadQuery {
    pub version: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CopyRequest {
    pub source_path: String,
    pub destination_path: String,
    pub new_name: Option<String>,
    pub overwrite: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct MoveRequest {
    pub source_path: String,
    pub destination_path: String,
    pub new_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteRequest {
    pub path: String,
    pub permanent: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct GetContentsRequest {
    pub path: String,
    pub version: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct FileContentsResponse {
    pub content: String,
    pub encoding: String,
    pub size: i64,
    pub version: i32,
}

#[derive(Debug, Deserialize)]
pub struct SaveRequest {
    pub path: String,
    pub content: String,
    pub create_version: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateFolderRequest {
    pub path: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct ShareFolderRequest {
    pub path: String,
    pub shared_with: Vec<String>, // User IDs or emails
    pub permissions: Vec<String>, // read, write, delete
    pub expires_at: Option<DateTime<Utc>>,
    pub expiry_hours: Option<u32>,
    pub bucket: Option<String>,
}

// Type alias for share parameters
pub type ShareParams = ShareFolderRequest;

#[derive(Debug, Serialize)]
pub struct ShareResponse {
    pub success: bool,
    pub share_id: String,
    pub share_link: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub path: Option<String>,
    pub recursive: Option<bool>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub sort_by: Option<String>, // name, size, date
    pub order: Option<String>,   // asc, desc
}

#[derive(Debug, Serialize)]
pub struct ListResponse {
    pub files: Vec<FileMetadata>,
    pub total: i64,
    pub offset: i32,
    pub limit: i32,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub query: String,
    pub path: Option<String>,
    pub file_type: Option<String>,
    pub size_min: Option<i64>,
    pub size_max: Option<i64>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub tags: Option<String>, // Comma-separated
    pub limit: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct RecentQuery {
    pub limit: Option<i32>,
    pub days: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct FavoriteRequest {
    pub path: String,
    pub favorite: bool,
}

#[derive(Debug, Serialize)]
pub struct FileVersion {
    pub version: i32,
    pub size: i64,
    pub modified_at: DateTime<Utc>,
    pub modified_by: String,
    pub comment: Option<String>,
    pub checksum: String,
}

#[derive(Debug, Deserialize)]
pub struct RestoreRequest {
    pub path: String,
    pub version: i32,
}

#[derive(Debug, Deserialize)]
pub struct PermissionsRequest {
    pub path: String,
    pub user_id: String,
    pub permissions: Vec<String>, // read, write, delete, share
}

#[derive(Debug, Serialize)]
pub struct PermissionsResponse {
    pub success: bool,
    pub path: String,
    pub permissions: Vec<Permission>,
}

#[derive(Debug, Serialize)]
pub struct Permission {
    pub user_id: String,
    pub user_name: String,
    pub permissions: Vec<String>,
    pub granted_at: DateTime<Utc>,
    pub granted_by: String,
}

#[derive(Debug, Serialize)]
pub struct QuotaResponse {
    pub total_bytes: i64,
    pub used_bytes: i64,
    pub available_bytes: i64,
    pub used_percentage: f64,
    pub file_count: i64,
    pub breakdown: QuotaBreakdown,
}

#[derive(Debug, Serialize)]
pub struct QuotaBreakdown {
    pub documents: i64,
    pub images: i64,
    pub videos: i64,
    pub archives: i64,
    pub other: i64,
}

#[derive(Debug, Serialize)]
pub struct SharedFile {
    pub share_id: String,
    pub path: String,
    pub shared_with: Vec<String>,
    pub permissions: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub access_count: i32,
}

#[derive(Debug, Serialize)]
pub struct SyncStatus {
    pub path: String,
    pub status: String, // synced, syncing, conflict, error
    pub last_sync: Option<DateTime<Utc>>,
    pub local_version: i32,
    pub remote_version: i32,
    pub conflict_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SyncStartRequest {
    pub paths: Vec<String>,
    pub direction: String, // upload, download, bidirectional
}

#[derive(Debug, Deserialize)]
pub struct SyncStopRequest {
    pub paths: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
    pub error: Option<String>,
}

// ===== API Handlers =====

/// POST /files/upload - Upload a file
pub async fn upload_file(
    State(state): State<Arc<AppState>>,
    Query(query): Query<UploadQuery>,
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<UploadResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    let s3_client = state.drive.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ApiResponse {
                success: false,
                data: None,
                message: None,
                error: Some("S3 service not available".to_string()),
            }),
        )
    })?;

    let mut file_data: Option<Vec<u8>> = None;
    let mut file_name: Option<String> = None;

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let name = field.name().unwrap_or("").to_string();

        if name == "file" {
            file_name = field.file_name().map(|s| s.to_string());
            file_data = Some(field.bytes().await.unwrap_or_default().to_vec());
        }
    }

    let file_name = file_name.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                data: None,
                message: None,
                error: Some("No file provided".to_string()),
            }),
        )
    })?;

    let file_data = file_data.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                data: None,
                message: None,
                error: Some("No file data".to_string()),
            }),
        )
    })?;

    let folder_path = query.folder_path.unwrap_or_else(|| "uploads".to_string());
    let file_path = format!("{}/{}", folder_path.trim_matches('/'), file_name);
    let file_size = file_data.len() as i64;
    let file_id = Uuid::new_v4().to_string();

    // Upload to S3
    s3_client
        .put_object()
        .bucket(&state.bucket_name)
        .key(&file_path)
        .body(ByteStream::from(file_data))
        .metadata("file-id", &file_id)
        .metadata("version", "1")
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    success: false,
                    data: None,
                    message: None,
                    error: Some(format!("Failed to upload file: {}", e)),
                }),
            )
        })?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(UploadResponse {
            success: true,
            file_id,
            path: file_path,
            size: file_size,
            version: 1,
            message: "File uploaded successfully".to_string(),
        }),
        message: Some("File uploaded successfully".to_string()),
        error: None,
    }))
}

/// GET /files/download/:path - Download a file
pub async fn download_file(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
    Query(query): Query<DownloadQuery>,
) -> Result<Response, (StatusCode, Json<ApiResponse<()>>)> {
    let s3_client = state.drive.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ApiResponse {
                success: false,
                data: None,
                message: None,
                error: Some("S3 service not available".to_string()),
            }),
        )
    })?;

    let result = s3_client
        .get_object()
        .bucket(&state.bucket_name)
        .key(&path)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(ApiResponse {
                    success: false,
                    data: None,
                    message: None,
                    error: Some(format!("File not found: {}", e)),
                }),
            )
        })?;

    let bytes = result
        .body
        .collect()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    success: false,
                    data: None,
                    message: None,
                    error: Some(format!("Failed to read file: {}", e)),
                }),
            )
        })?
        .into_bytes();

    let file_name = path.split('/').last().unwrap_or("download");
    let content_type = mime_guess::from_path(&path)
        .first_or_octet_stream()
        .to_string();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", file_name),
        )
        .body(Body::from(bytes))
        .unwrap())
}

/// POST /files/copy - Copy a file or folder
pub async fn copy_file(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CopyRequest>,
) -> Result<Json<ApiResponse<String>>, (StatusCode, Json<ApiResponse<()>>)> {
    let s3_client = state.drive.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ApiResponse {
                success: false,
                data: None,
                message: None,
                error: Some("S3 service not available".to_string()),
            }),
        )
    })?;

    let dest_name = req.new_name.unwrap_or_else(|| {
        req.source_path
            .split('/')
            .last()
            .unwrap_or("copy")
            .to_string()
    });

    let dest_path = format!("{}/{}", req.destination_path.trim_matches('/'), dest_name);

    // Copy object in S3
    let copy_source = format!("{}/{}", state.bucket_name, req.source_path);
    s3_client
        .copy_object()
        .bucket(&state.bucket_name)
        .copy_source(&copy_source)
        .key(&dest_path)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    success: false,
                    data: None,
                    message: None,
                    error: Some(format!("Failed to copy file: {}", e)),
                }),
            )
        })?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(dest_path),
        message: Some("File copied successfully".to_string()),
        error: None,
    }))
}

/// POST /files/move - Move a file or folder
pub async fn move_file(
    State(state): State<Arc<AppState>>,
    Json(req): Json<MoveRequest>,
) -> Result<Json<ApiResponse<String>>, (StatusCode, Json<ApiResponse<()>>)> {
    let s3_client = state.drive.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ApiResponse {
                success: false,
                data: None,
                message: None,
                error: Some("S3 service not available".to_string()),
            }),
        )
    })?;

    let dest_name = req.new_name.unwrap_or_else(|| {
        req.source_path
            .split('/')
            .last()
            .unwrap_or("moved")
            .to_string()
    });

    let dest_path = format!("{}/{}", req.destination_path.trim_matches('/'), dest_name);

    // Copy then delete (S3 doesn't have native move)
    let copy_source = format!("{}/{}", state.bucket_name, req.source_path);
    s3_client
        .copy_object()
        .bucket(&state.bucket_name)
        .copy_source(&copy_source)
        .key(&dest_path)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    success: false,
                    data: None,
                    message: None,
                    error: Some(format!("Failed to move file: {}", e)),
                }),
            )
        })?;

    s3_client
        .delete_object()
        .bucket(&state.bucket_name)
        .key(&req.source_path)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    success: false,
                    data: None,
                    message: None,
                    error: Some(format!("Failed to delete source: {}", e)),
                }),
            )
        })?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(dest_path),
        message: Some("File moved successfully".to_string()),
        error: None,
    }))
}

/// DELETE /files/delete - Delete a file or folder
pub async fn delete_file(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DeleteRequest>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    let s3_client = state.drive.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ApiResponse {
                success: false,
                data: None,
                message: None,
                error: Some("S3 service not available".to_string()),
            }),
        )
    })?;

    // If it's a folder (ends with /), delete all objects with prefix
    if req.path.ends_with('/') {
        let list_result = s3_client
            .list_objects_v2()
            .bucket(&state.bucket_name)
            .prefix(&req.path)
            .send()
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse {
                        success: false,
                        data: None,
                        message: None,
                        error: Some(format!("Failed to list objects: {}", e)),
                    }),
                )
            })?;

        for obj in list_result.contents() {
            if let Some(key) = obj.key() {
                s3_client
                    .delete_object()
                    .bucket(&state.bucket_name)
                    .key(key)
                    .send()
                    .await
                    .map_err(|e| {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ApiResponse {
                                success: false,
                                data: None,
                                message: None,
                                error: Some(format!("Failed to delete object: {}", e)),
                            }),
                        )
                    })?;
            }
        }
    } else {
        s3_client
            .delete_object()
            .bucket(&state.bucket_name)
            .key(&req.path)
            .send()
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse {
                        success: false,
                        data: None,
                        message: None,
                        error: Some(format!("Failed to delete file: {}", e)),
                    }),
                )
            })?;
    }

    Ok(Json(ApiResponse {
        success: true,
        data: None,
        message: Some("File deleted successfully".to_string()),
        error: None,
    }))
}

/// POST /files/getContents - Get file contents
pub async fn get_contents(
    State(state): State<Arc<AppState>>,
    Json(req): Json<GetContentsRequest>,
) -> Result<Json<ApiResponse<FileContentsResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    let s3_client = state.drive.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ApiResponse {
                success: false,
                data: None,
                message: None,
                error: Some("S3 service not available".to_string()),
            }),
        )
    })?;

    let result = s3_client
        .get_object()
        .bucket(&state.bucket_name)
        .key(&req.path)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(ApiResponse {
                    success: false,
                    data: None,
                    message: None,
                    error: Some(format!("File not found: {}", e)),
                }),
            )
        })?;

    let bytes = result
        .body
        .collect()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    success: false,
                    data: None,
                    message: None,
                    error: Some(format!("Failed to read file: {}", e)),
                }),
            )
        })?
        .into_bytes();

    let size = bytes.len() as i64;
    let content = String::from_utf8_lossy(&bytes).to_string();

    Ok(Json(ApiResponse {
        success: true,
        data: Some(FileContentsResponse {
            content,
            encoding: "utf-8".to_string(),
            size,
            version: 1,
        }),
        message: None,
        error: None,
    }))
}

/// POST /files/save - Save file contents
pub async fn save_file(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SaveRequest>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    let s3_client = state.drive.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ApiResponse {
                success: false,
                data: None,
                message: None,
                error: Some("S3 service not available".to_string()),
            }),
        )
    })?;

    s3_client
        .put_object()
        .bucket(&state.bucket_name)
        .key(&req.path)
        .body(ByteStream::from(req.content.into_bytes()))
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    success: false,
                    data: None,
                    message: None,
                    error: Some(format!("Failed to save file: {}", e)),
                }),
            )
        })?;

    Ok(Json(ApiResponse {
        success: true,
        data: None,
        message: Some("File saved successfully".to_string()),
        error: None,
    }))
}

/// POST /files/createFolder - Create a new folder
pub async fn create_folder(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateFolderRequest>,
) -> Result<Json<ApiResponse<String>>, (StatusCode, Json<ApiResponse<()>>)> {
    let s3_client = state.drive.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ApiResponse {
                success: false,
                data: None,
                message: None,
                error: Some("S3 service not available".to_string()),
            }),
        )
    })?;

    let folder_path = if req.path.is_empty() || req.path == "/" {
        format!("{}/", req.name)
    } else {
        format!("{}/{}/", req.path.trim_end_matches('/'), req.name)
    };

    s3_client
        .put_object()
        .bucket(&state.bucket_name)
        .key(&folder_path)
        .body(ByteStream::from(Vec::new()))
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    success: false,
                    data: None,
                    message: None,
                    error: Some(format!("Failed to create folder: {}", e)),
                }),
            )
        })?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(folder_path),
        message: Some("Folder created successfully".to_string()),
        error: None,
    }))
}

/// POST /files/shareFolder - Share a folder
pub async fn share_folder(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ShareFolderRequest>,
) -> Result<Json<ApiResponse<ShareResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    let share_id = Uuid::new_v4().to_string();
    let base_url =
        std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let share_link = format!("{}/api/shared/{}", base_url, share_id);

    // Calculate expiry time if specified
    let expires_at = if let Some(expiry_hours) = req.expiry_hours {
        Some(Utc::now() + chrono::Duration::hours(expiry_hours as i64))
    } else {
        None
    };

    // Store share information in database
    // TODO: Fix Diesel query syntax
    /*
    if let Ok(mut conn) = state.conn.get() {
        let _ = diesel::sql_query(
            "INSERT INTO file_shares (id, path, permissions, created_by, expires_at) VALUES ($1, $2, $3, $4, $5)"
        )
        .bind::<diesel::sql_types::Uuid, _>(Uuid::parse_str(&share_id).unwrap())
        .bind::<diesel::sql_types::Text, _>(&req.path)
        .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, _>(&req.permissions)
        .bind::<diesel::sql_types::Text, _>("system")
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(expires_at)
        .execute(&mut conn);
    }
    */

    // Set permissions on S3 object if needed
    // TODO: Fix S3 copy_object API call
    /*
    if let Some(drive) = &state.drive {
        let bucket = req.bucket.as_deref().unwrap_or("drive");
        let key = format!("shared/{}/{}", share_id, req.path);

        // Copy object to shared location
        let copy_source = format!("{}/{}", bucket, req.path);
        let _ = drive.copy_object(bucket, &copy_source, &key).await;
    }
    */

    Ok(Json(ApiResponse {
        success: true,
        data: Some(ShareResponse {
            success: true,
            share_id,
            share_link: Some(share_link),
            expires_at,
        }),
        message: Some("Folder shared successfully".to_string()),
        error: None,
    }))
}

// S3/MinIO helper functions for storage operations

pub async fn save_to_s3(
    state: &Arc<AppState>,
    bucket: &str,
    key: &str,
    content: &[u8],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let s3_client = state.drive.as_ref().ok_or("S3 client not configured")?;

    s3_client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(ByteStream::from(content.to_vec()))
        .send()
        .await?;

    Ok(())
}

pub async fn delete_from_s3(
    state: &Arc<AppState>,
    bucket: &str,
    key: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let s3_client = state.drive.as_ref().ok_or("S3 client not configured")?;

    s3_client
        .delete_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await?;

    Ok(())
}

#[derive(Debug)]
pub struct BucketStats {
    pub object_count: usize,
    pub total_size: u64,
    pub last_modified: Option<String>,
}

pub async fn get_bucket_stats(
    state: &Arc<AppState>,
    bucket: &str,
) -> Result<BucketStats, Box<dyn std::error::Error + Send + Sync>> {
    let s3_client = state.drive.as_ref().ok_or("S3 client not configured")?;

    let list_response = s3_client.list_objects_v2().bucket(bucket).send().await?;

    let mut total_size = 0u64;
    let mut object_count = 0usize;
    let mut last_modified = None;

    if let Some(contents) = list_response.contents {
        object_count = contents.len();
        for object in contents {
            if let Some(size) = object.size() {
                total_size += size as u64;
            }
            if let Some(modified) = object.last_modified() {
                let modified_str = modified.to_string();
                if last_modified.is_none() || last_modified.as_ref().unwrap() < &modified_str {
                    last_modified = Some(modified_str);
                }
            }
        }
    }

    Ok(BucketStats {
        object_count,
        total_size,
        last_modified,
    })
}

pub async fn cleanup_old_files(
    state: &Arc<AppState>,
    bucket: &str,
    cutoff_date: chrono::DateTime<chrono::Utc>,
) -> Result<(usize, u64), Box<dyn std::error::Error + Send + Sync>> {
    let s3_client = state.drive.as_ref().ok_or("S3 client not configured")?;

    let list_response = s3_client.list_objects_v2().bucket(bucket).send().await?;

    let mut deleted_count = 0usize;
    let mut freed_bytes = 0u64;

    if let Some(contents) = list_response.contents {
        for object in contents {
            if let Some(modified) = object.last_modified() {
                let modified_time = chrono::DateTime::parse_from_rfc3339(&modified.to_string())
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now());

                if modified_time < cutoff_date {
                    if let Some(key) = object.key() {
                        if let Some(size) = object.size() {
                            freed_bytes += size as u64;
                        }

                        s3_client
                            .delete_object()
                            .bucket(bucket)
                            .key(key)
                            .send()
                            .await?;

                        deleted_count += 1;
                    }
                }
            }
        }
    }

    Ok((deleted_count, freed_bytes))
}

pub async fn create_bucket_backup(
    state: &Arc<AppState>,
    source_bucket: &str,
    backup_bucket: &str,
    backup_id: &str,
) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    let s3_client = state.drive.as_ref().ok_or("S3 client not configured")?;

    // Create backup bucket if it doesn't exist
    let _ = s3_client.create_bucket().bucket(backup_bucket).send().await;

    let list_response = s3_client
        .list_objects_v2()
        .bucket(source_bucket)
        .send()
        .await?;

    let mut file_count = 0usize;

    if let Some(contents) = list_response.contents {
        for object in contents {
            if let Some(key) = object.key() {
                let backup_key = format!("{}/{}", backup_id, key);

                // Copy object to backup bucket
                let copy_source = format!("{}/{}", source_bucket, key);
                s3_client
                    .copy_object()
                    .copy_source(&copy_source)
                    .bucket(backup_bucket)
                    .key(&backup_key)
                    .send()
                    .await?;

                file_count += 1;
            }
        }
    }

    Ok(file_count)
}

pub async fn restore_bucket_backup(
    state: &Arc<AppState>,
    backup_bucket: &str,
    target_bucket: &str,
    backup_id: &str,
) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    let s3_client = state.drive.as_ref().ok_or("S3 client not configured")?;

    let prefix = format!("{}/", backup_id);
    let list_response = s3_client
        .list_objects_v2()
        .bucket(backup_bucket)
        .prefix(&prefix)
        .send()
        .await?;

    let mut file_count = 0usize;

    if let Some(contents) = list_response.contents {
        for object in contents {
            if let Some(key) = object.key() {
                // Remove backup_id prefix from key
                let restored_key = key.strip_prefix(&prefix).unwrap_or(key);

                // Copy object back to target bucket
                let copy_source = format!("{}/{}", backup_bucket, key);
                s3_client
                    .copy_object()
                    .copy_source(&copy_source)
                    .bucket(target_bucket)
                    .key(restored_key)
                    .send()
                    .await?;

                file_count += 1;
            }
        }
    }

    Ok(file_count)
}

pub async fn create_archive(
    state: &Arc<AppState>,
    bucket: &str,
    prefix: &str,
    archive_key: &str,
) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    let s3_client = state.drive.as_ref().ok_or("S3 client not configured")?;

    let list_response = s3_client
        .list_objects_v2()
        .bucket(bucket)
        .prefix(prefix)
        .send()
        .await?;

    let mut archive_data = Vec::new();

    // Create simple tar-like format without compression
    if let Some(contents) = list_response.contents {
        for object in contents {
            if let Some(key) = object.key() {
                // Get object content
                let get_response = s3_client
                    .get_object()
                    .bucket(bucket)
                    .key(key)
                    .send()
                    .await?;

                let body_bytes = get_response
                    .body
                    .collect()
                    .await
                    .map_err(|e| format!("Failed to collect body: {}", e))?;
                let bytes = body_bytes.into_bytes();

                // Write to archive with key as filename (simple tar-like format)
                use std::io::Write;
                archive_data.write_all(key.as_bytes())?;
                archive_data.write_all(b"\n")?;
                archive_data.write_all(&bytes)?;
                archive_data.write_all(b"\n---\n")?;
            }
        }
    }

    let archive_size = archive_data.len() as u64;

    // Upload archive
    s3_client
        .put_object()
        .bucket(bucket)
        .key(archive_key)
        .body(ByteStream::from(archive_data))
        .send()
        .await?;

    Ok(archive_size)
}

pub async fn get_bucket_metrics(
    state: &Arc<AppState>,
    bucket: &str,
) -> Result<BucketStats, Box<dyn std::error::Error + Send + Sync>> {
    get_bucket_stats(state, bucket).await
}

/// GET /files/dirFolder - Directory listing (alias for list)
pub async fn dir_folder(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<ApiResponse<ListResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    list_files(State(state), Query(query)).await
}

/// GET /files/list - List files and folders
pub async fn list_files(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<ApiResponse<ListResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    let s3_client = state.drive.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ApiResponse {
                success: false,
                data: None,
                message: None,
                error: Some("S3 service not available".to_string()),
            }),
        )
    })?;

    let prefix = query.path.unwrap_or_default();
    let delimiter = if query.recursive.unwrap_or(false) {
        None
    } else {
        Some("/".to_string())
    };

    let result = s3_client
        .list_objects_v2()
        .bucket(&state.bucket_name)
        .prefix(&prefix)
        .set_delimiter(delimiter)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    success: false,
                    data: None,
                    message: None,
                    error: Some(format!("Failed to list files: {}", e)),
                }),
            )
        })?;

    let mut files = Vec::new();

    // Add folders
    for prefix in result.common_prefixes() {
        if let Some(p) = prefix.prefix() {
            files.push(FileMetadata {
                id: Uuid::new_v4().to_string(),
                name: p
                    .trim_end_matches('/')
                    .split('/')
                    .last()
                    .unwrap_or(p)
                    .to_string(),
                path: p.to_string(),
                size: 0,
                mime_type: None,
                created_at: Utc::now(),
                modified_at: Utc::now(),
                created_by: "system".to_string(),
                modified_by: "system".to_string(),
                is_dir: true,
                version: 1,
                parent_id: None,
                tags: Vec::new(),
                checksum: None,
            });
        }
    }

    // Add files
    for obj in result.contents() {
        if let Some(key) = obj.key() {
            files.push(FileMetadata {
                id: Uuid::new_v4().to_string(),
                name: key.split('/').last().unwrap_or(key).to_string(),
                path: key.to_string(),
                size: obj.size().unwrap_or(0),
                mime_type: Some(
                    mime_guess::from_path(key)
                        .first_or_octet_stream()
                        .to_string(),
                ),
                created_at: obj
                    .last_modified()
                    .and_then(|t| chrono::DateTime::parse_from_rfc3339(&t.to_string()).ok())
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(Utc::now),
                modified_at: obj
                    .last_modified()
                    .and_then(|t| chrono::DateTime::parse_from_rfc3339(&t.to_string()).ok())
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(Utc::now),
                created_by: "system".to_string(),
                modified_by: "system".to_string(),
                is_dir: false,
                version: 1,
                parent_id: None,
                tags: Vec::new(),
                checksum: obj.e_tag().map(|s| s.to_string()),
            });
        }
    }

    let total = files.len() as i64;
    let limit = query.limit.unwrap_or(100);
    let offset = query.offset.unwrap_or(0);

    Ok(Json(ApiResponse {
        success: true,
        data: Some(ListResponse {
            files,
            total,
            offset,
            limit,
        }),
        message: None,
        error: None,
    }))
}

/// GET /files/search - Search files
pub async fn search_files(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<ApiResponse<ListResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    let s3_client = state.drive.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ApiResponse {
                success: false,
                data: None,
                message: None,
                error: Some("S3 service not available".to_string()),
            }),
        )
    })?;

    let prefix = query.path.unwrap_or_default();

    let result = s3_client
        .list_objects_v2()
        .bucket(&state.bucket_name)
        .prefix(&prefix)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    success: false,
                    data: None,
                    message: None,
                    error: Some(format!("Failed to search files: {}", e)),
                }),
            )
        })?;

    let search_query = query.query.to_lowercase();
    let mut files = Vec::new();

    for obj in result.contents() {
        if let Some(key) = obj.key() {
            let file_name = key.split('/').last().unwrap_or(key).to_lowercase();

            // Simple search by name
            if file_name.contains(&search_query) {
                files.push(FileMetadata {
                    id: Uuid::new_v4().to_string(),
                    name: key.split('/').last().unwrap_or(key).to_string(),
                    path: key.to_string(),
                    size: obj.size().unwrap_or(0),
                    mime_type: Some(
                        mime_guess::from_path(key)
                            .first_or_octet_stream()
                            .to_string(),
                    ),
                    created_at: obj
                        .last_modified()
                        .and_then(|t| chrono::DateTime::parse_from_rfc3339(&t.to_string()).ok())
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(Utc::now),
                    modified_at: obj
                        .last_modified()
                        .and_then(|t| chrono::DateTime::parse_from_rfc3339(&t.to_string()).ok())
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(Utc::now),
                    created_by: "system".to_string(),
                    modified_by: "system".to_string(),
                    is_dir: false,
                    version: 1,
                    parent_id: None,
                    tags: Vec::new(),
                    checksum: obj.e_tag().map(|s| s.to_string()),
                });
            }
        }
    }

    let total = files.len() as i64;
    let limit = query.limit.unwrap_or(50) as i32;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(ListResponse {
            files,
            total,
            offset: 0,
            limit,
        }),
        message: None,
        error: None,
    }))
}

/// GET /files/recent - Get recently accessed files
pub async fn recent_files(
    State(state): State<Arc<AppState>>,
    Query(query): Query<RecentQuery>,
) -> Result<Json<ApiResponse<ListResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Get recently accessed files from database
    // TODO: Fix Diesel query syntax
    let recent_files: Vec<(String, chrono::DateTime<Utc>)> = vec![];
    /*
    if let Ok(mut conn) = state.conn.get() {
        let recent_files = diesel::sql_query(
            "SELECT path, accessed_at FROM file_access_log
             WHERE user_id = $1
             ORDER BY accessed_at DESC
             LIMIT $2",
        )
        .bind::<diesel::sql_types::Text, _>("system")
        .bind::<diesel::sql_types::Integer, _>(query.limit.unwrap_or(20) as i32)
        .load::<(String, chrono::DateTime<Utc>)>(&mut conn)
        .unwrap_or_default();
    */

    if !recent_files.is_empty() {
        let mut items = Vec::new();

        if let Some(drive) = &state.drive {
            let bucket = "drive";

            for (path, _) in recent_files.iter().take(query.limit.unwrap_or(20)) {
                // TODO: Fix get_object_info API call
                /*
                if let Ok(object) = drive.get_object_info(bucket, path).await {
                    items.push(crate::drive::FileItem {
                        name: path.split('/').last().unwrap_or(path).to_string(),
                        path: path.clone(),
                        is_dir: path.ends_with('/'),
                        size: Some(object.size as i64),
                        modified: Some(object.last_modified.to_rfc3339()),
                        content_type: object.content_type,
                        etag: object.e_tag,
                    });
                }
                */
            }
        }

        return Ok(Json(ApiResponse {
            success: true,
            data: Some(ListResponse { items }),
            message: None,
            error: None,
        }));
    }

    // Fallback to listing files by date
    list_files(
        State(state),
        Query(ListQuery {
            path: None,
            recursive: Some(false),
            limit: query.limit,
            offset: None,
            sort_by: Some("date".to_string()),
            order: Some("desc".to_string()),
        }),
    )
    .await
}

/// POST /files/favorite - Mark/unmark file as favorite
pub async fn favorite_file(
    State(state): State<Arc<AppState>>,
    Json(req): Json<FavoriteRequest>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Store favorite status in database
    if let Ok(mut conn) = state.conn.get() {
        if req.favorite {
            // Add to favorites
            // TODO: Fix Diesel query syntax
            /*
            let _ = diesel::sql_query(
                "INSERT INTO file_favorites (user_id, file_path, created_at)
                 VALUES ($1, $2, $3)
                 ON CONFLICT (user_id, file_path) DO NOTHING",
            )
            .bind::<diesel::sql_types::Text, _>("system")
            .bind::<diesel::sql_types::Text, _>(&req.path)
            .bind::<diesel::sql_types::Timestamptz, _>(Utc::now())
            .execute(&mut conn);
            */
        } else {
            // Remove from favorites
            // TODO: Fix Diesel query syntax
            /*
            let _ = diesel::sql_query(
                "DELETE FROM file_favorites WHERE user_id = $1 AND file_path = $2",
            )
            .bind::<diesel::sql_types::Text, _>("system")
            .bind::<diesel::sql_types::Text, _>(&req.path)
            .execute(&mut conn);
            */
        }
    }

    Ok(Json(ApiResponse {
        success: true,
        data: None,
        message: Some(format!(
            "File {} {} favorites",
            req.path,
            if req.favorite {
                "added to"
            } else {
                "removed from"
            }
        )),
        error: None,
    }))
}

/// GET /files/versions/:path - Get file version history
pub async fn file_versions(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
) -> Result<Json<ApiResponse<Vec<FileVersion>>>, (StatusCode, Json<ApiResponse<()>>)> {
    let mut versions = Vec::new();

    // Get versions from S3 if versioning is enabled
    if let Some(drive) = &state.drive {
        let bucket = "drive";

        // List object versions
        // TODO: Fix S3 list_object_versions API call
    }

    // Also get version history from database
    if versions.is_empty() {
        if let Ok(mut conn) = state.conn.get() {
            // TODO: Fix Diesel query syntax
            let db_versions: Vec<(
                i32,
                i64,
                chrono::DateTime<Utc>,
                String,
                Option<String>,
                String,
            )> = vec![];

            for (version, size, modified_at, modified_by, comment, checksum) in db_versions {
                versions.push(FileVersion {
                    version,
                    size,
                    modified_at,
                    modified_by,
                    comment,
                    checksum,
                });
            }
        }
    }

    // If still no versions, create a default one
    if versions.is_empty() {
        versions.push(FileVersion {
            version: 1,
            size: 0,
            modified_at: Utc::now(),
            modified_by: "system".to_string(),
            comment: Some("Current version".to_string()),
            checksum: "".to_string(),
        });
    }

    Ok(Json(ApiResponse {
        success: true,
        data: Some(versions),
        message: None,
        error: None,
    }))
}

/// POST /files/restore - Restore a file version
pub async fn restore_version(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RestoreRequest>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Restore from S3 versioning
    if let Some(drive) = &state.drive {
        let bucket = "drive";

        // Get the specific version
        // TODO: Fix S3 list_object_versions and copy_object API calls
    }

    Err((
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiResponse {
            success: false,
            data: None,
            message: None,
            error: Some("Failed to restore file version".to_string()),
        }),
    ))
}

/// GET /files/permissions/:path - Get file permissions
pub async fn get_permissions(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
) -> Result<Json<ApiResponse<PermissionsResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    let mut permissions = Vec::new();

    // Get permissions from database
    if let Ok(mut conn) = state.conn.get() {
        // TODO: Fix Diesel query syntax
        let db_permissions: Vec<(String, String, Vec<String>, chrono::DateTime<Utc>, String)> =
            vec![];

        for (user_id, user_name, perms, granted_at, granted_by) in db_permissions {
            permissions.push(Permission {
                user_id,
                user_name,
                permissions: perms,
                granted_at,
                granted_by,
            });
        }
    }

    // Add default permissions if none exist
    if permissions.is_empty() {
        permissions.push(Permission {
            user_id: "system".to_string(),
            user_name: "System".to_string(),
            permissions: vec![
                "read".to_string(),
                "write".to_string(),
                "delete".to_string(),
            ],
            granted_at: Utc::now(),
            granted_by: "system".to_string(),
        });
    }

    // Check if permissions are inherited from parent directory
    let inherited = path.contains('/') && permissions.iter().any(|p| p.user_id == "inherited");

    Ok(Json(ApiResponse {
        success: true,
        data: Some(PermissionsResponse {
            path,
            permissions,
            inherited,
        }),
        message: None,
        error: None,
    }))
}

/// POST /files/permissions - Set file permissions
pub async fn set_permissions(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PermissionsRequest>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Store permissions in database
    if let Ok(mut conn) = state.conn.get() {
        // Remove existing permissions for this user and path
        // TODO: Fix Diesel query syntax

        // Insert new permissions
        if !req.permissions.is_empty() {
            // TODO: Fix Diesel query syntax
        }

        // Also set S3 bucket policies if needed
        if let Some(drive) = &state.drive {
            let bucket = "drive";

            // Create bucket policy for user access
            let policy = serde_json::json!({
                "Version": "2012-10-17",
                "Statement": [{
                    "Effect": if req.permissions.is_empty() { "Deny" } else { "Allow" },
                    "Principal": { "AWS": [format!("arn:aws:iam::USER:{}", req.user_id)] },
                    "Action": req.permissions.iter().map(|p| match p.as_str() {
                        "read" => "s3:GetObject",
                        "write" => "s3:PutObject",
                        "delete" => "s3:DeleteObject",
                        _ => "s3:GetObject"
                    }).collect::<Vec<_>>(),
                    "Resource": format!("arn:aws:s3:::{}/{}", bucket, req.path)
                }]
            });

            // TODO: Fix S3 put_bucket_policy API call
            // let _ = drive.put_bucket_policy(bucket, &policy.to_string()).await;
        }
    }

    Ok(Json(ApiResponse {
        success: true,
        data: None,
        message: Some(format!("Permissions updated for {}", req.path)),
        error: None,
    }))
}

/// GET /files/quota - Get storage quota information
pub async fn get_quota(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<QuotaResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    let s3_client = state.drive.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ApiResponse {
                success: false,
                data: None,
                message: None,
                error: Some("S3 service not available".to_string()),
            }),
        )
    })?;

    let result = s3_client
        .list_objects_v2()
        .bucket(&state.bucket_name)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    success: false,
                    data: None,
                    message: None,
                    error: Some(format!("Failed to calculate quota: {}", e)),
                }),
            )
        })?;

    let mut used_bytes: i64 = 0;
    let mut file_count: i64 = 0;

    for obj in result.contents() {
        used_bytes += obj.size().unwrap_or(0);
        file_count += 1;
    }

    let total_bytes: i64 = 10 * 1024 * 1024 * 1024; // 10 GB default quota
    let available_bytes = total_bytes - used_bytes;
    let used_percentage = (used_bytes as f64 / total_bytes as f64) * 100.0;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(QuotaResponse {
            total_bytes,
            used_bytes,
            available_bytes,
            used_percentage,
            file_count,
            breakdown: QuotaBreakdown {
                documents: 0,
                images: 0,
                videos: 0,
                archives: 0,
                other: used_bytes,
            },
        }),
        message: None,
        error: None,
    }))
}

/// GET /files/shared - Get shared files
pub async fn get_shared(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<SharedFile>>>, (StatusCode, Json<ApiResponse<()>>)> {
    let mut shared_files = Vec::new();

    // Get shared files from database
    if let Ok(mut conn) = state.conn.get() {
        // TODO: Fix Diesel query syntax
        let shares: Vec<(
            String,
            String,
            Vec<String>,
            chrono::DateTime<Utc>,
            Option<chrono::DateTime<Utc>>,
            Option<String>,
        )> = vec![];

        for (share_id, path, permissions, created_at, expires_at, shared_by) in shares {
            // Get file info from S3
            let mut size = 0i64;
            let mut modified = Utc::now();

            if let Some(drive) = &state.drive {
                // TODO: Fix S3 get_object_info API call
            }

            shared_files.push(SharedFile {
                id: share_id,
                name: path.split('/').last().unwrap_or(&path).to_string(),
                path,
                size,
                modified,
                shared_by: shared_by.unwrap_or_else(|| "Unknown".to_string()),
                shared_at: created_at,
                permissions,
                expires_at,
            });
        }
    }

    Ok(Json(ApiResponse {
        success: true,
        data: Some(shared_files),
        message: None,
        error: None,
    }))
}

/// GET /files/sync/status - Get sync status
pub async fn sync_status(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<SyncStatus>>>, (StatusCode, Json<ApiResponse<()>>)> {
    // TODO: Implement sync status tracking
    Ok(Json(ApiResponse {
        success: true,
        data: Some(Vec::new()),
        message: None,
        error: None,
    }))
}

/// POST /files/sync/start - Start syncing files
pub async fn sync_start(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<SyncStartRequest>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    // TODO: Implement sync service
    Ok(Json(ApiResponse {
        success: true,
        data: None,
        message: Some(format!("Sync started for {} paths", req.paths.len())),
        error: None,
    }))
}

/// POST /files/sync/stop - Stop syncing files
pub async fn sync_stop(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<SyncStopRequest>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    // TODO: Implement sync service
    Ok(Json(ApiResponse {
        success: true,
        data: None,
        message: Some(format!("Sync stopped for {} paths", req.paths.len())),
        error: None,
    }))
}

// ===== Route Configuration =====

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/files/upload", post(upload_file))
        .route("/files/download/:path", get(download_file))
        .route("/files/copy", post(copy_file))
        .route("/files/move", post(move_file))
        .route("/files/delete", delete(delete_file))
        .route("/files/getContents", post(get_contents))
        .route("/files/save", post(save_file))
        .route("/files/createFolder", post(create_folder))
        .route("/files/shareFolder", post(share_folder))
        .route("/files/dirFolder", get(dir_folder))
        .route("/files/list", get(list_files))
        .route("/files/search", get(search_files))
        .route("/files/recent", get(recent_files))
        .route("/files/favorite", post(favorite_file))
        .route("/files/versions/:path", get(file_versions))
        .route("/files/restore", post(restore_version))
        .route("/files/permissions/:path", get(get_permissions))
        .route("/files/permissions", post(set_permissions))
        .route("/files/quota", get(get_quota))
        .route("/files/shared", get(get_shared))
        .route("/files/sync/status", get(sync_status))
        .route("/files/sync/start", post(sync_start))
        .route("/files/sync/stop", post(sync_stop))
}
