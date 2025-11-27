use crate::shared::state::AppState;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::{Delete, ObjectIdentifier};
use axum::{
    extract::{Json, Multipart, Path, Query, State},
    response::IntoResponse,
};

use chrono::Utc;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileItem {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub modified: String,
    pub is_dir: bool,
    pub mime_type: Option<String>,
    pub icon: String,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub path: Option<String>,
    pub bucket: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperation {
    pub source_bucket: String,
    pub source_path: String,
    pub dest_bucket: String,
    pub dest_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaInfo {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub percentage_used: f32,
}

pub async fn list_files(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    let bucket = query.bucket.unwrap_or_else(|| "default".to_string());
    let path = query.path.unwrap_or_else(|| "/".to_string());
    let limit = query.limit.unwrap_or(100);
    let _offset = query.offset.unwrap_or(0);

    let prefix = if path == "/" {
        String::new()
    } else {
        path.trim_start_matches('/').to_string()
    };

    let mut items = Vec::new();

    let s3 = match state.s3_client.as_ref() {
        Some(client) => client,
        None => {
            return Json(FileResponse {
                success: false,
                message: "S3 client not configured".to_string(),
                data: None,
            })
        }
    };

    match s3
        .list_objects_v2()
        .bucket(&bucket)
        .prefix(&prefix)
        .max_keys(limit)
        .send()
        .await
    {
        Ok(response) => {
            if let Some(contents) = response.contents {
                for obj in contents {
                    let key = obj.key.clone().unwrap_or_default();
                    let name = key.split('/').last().unwrap_or(&key).to_string();
                    let size = obj.size.unwrap_or(0) as u64;
                    let modified = obj
                        .last_modified
                        .map(|d| d.to_string())
                        .unwrap_or_else(|| Utc::now().to_rfc3339());

                    items.push(FileItem {
                        name,
                        path: key.clone(),
                        size,
                        modified,
                        is_dir: key.ends_with('/'),
                        mime_type: mime_guess::from_path(&key).first().map(|m| m.to_string()),
                        icon: get_file_icon(&key),
                    });
                }
            }

            Json(FileResponse {
                success: true,
                message: format!("Found {} items", items.len()),
                data: Some(serde_json::to_value(items).unwrap()),
            })
        }
        Err(e) => {
            error!("Failed to list files: {:?}", e);
            Json(FileResponse {
                success: false,
                message: format!("Failed to list files: {}", e),
                data: None,
            })
        }
    }
}

pub async fn read_file(
    State(state): State<Arc<AppState>>,
    Path((bucket, path)): Path<(String, String)>,
) -> impl IntoResponse {
    let s3 = match state.s3_client.as_ref() {
        Some(client) => client,
        None => {
            return Json(FileResponse {
                success: false,
                message: "S3 client not configured".to_string(),
                data: None,
            })
        }
    };

    match s3.get_object().bucket(&bucket).key(&path).send().await {
        Ok(response) => {
            let body = response.body.collect().await.unwrap();
            let bytes = body.to_vec();
            let content = String::from_utf8(bytes.clone()).unwrap_or_else(|_| {
                base64::Engine::encode(&base64::engine::general_purpose::STANDARD, bytes)
            });

            Json(FileResponse {
                success: true,
                message: "File read successfully".to_string(),
                data: Some(serde_json::json!({
                    "content": content,
                    "content_type": response.content_type,
                    "content_length": response.content_length,
                })),
            })
        }
        Err(e) => {
            error!("Failed to read file: {:?}", e);
            Json(FileResponse {
                success: false,
                message: format!("Failed to read file: {}", e),
                data: None,
            })
        }
    }
}

pub async fn write_file(
    State(state): State<Arc<AppState>>,
    Path((bucket, path)): Path<(String, String)>,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    let content_type = mime_guess::from_path(&path)
        .first()
        .map(|m| m.to_string())
        .unwrap_or_else(|| "application/octet-stream".to_string());

    let s3 = match state.s3_client.as_ref() {
        Some(client) => client,
        None => {
            return Json(FileResponse {
                success: false,
                message: "S3 client not configured".to_string(),
                data: None,
            })
        }
    };

    match s3
        .put_object()
        .bucket(&bucket)
        .key(&path)
        .body(ByteStream::from(body.to_vec()))
        .content_type(content_type)
        .send()
        .await
    {
        Ok(_) => {
            info!("File written successfully: {}/{}", bucket, path);
            Json(FileResponse {
                success: true,
                message: "File uploaded successfully".to_string(),
                data: Some(serde_json::json!({
                    "bucket": bucket,
                    "path": path,
                    "size": body.len(),
                })),
            })
        }
        Err(e) => {
            error!("Failed to write file: {:?}", e);
            Json(FileResponse {
                success: false,
                message: format!("Failed to write file: {}", e),
                data: None,
            })
        }
    }
}

pub async fn delete_file(
    State(state): State<Arc<AppState>>,
    Path((bucket, path)): Path<(String, String)>,
) -> impl IntoResponse {
    if path.ends_with('/') {
        let prefix = path.trim_end_matches('/');
        let mut continuation_token = None;
        let mut objects_to_delete = Vec::new();

        let s3 = match state.s3_client.as_ref() {
            Some(client) => client,
            None => {
                return Json(FileResponse {
                    success: false,
                    message: "S3 client not configured".to_string(),
                    data: None,
                })
            }
        };

        loop {
            let mut list_req = s3.list_objects_v2().bucket(&bucket).prefix(prefix);

            if let Some(token) = continuation_token {
                list_req = list_req.continuation_token(token);
            }

            match list_req.send().await {
                Ok(response) => {
                    if let Some(contents) = response.contents {
                        for obj in contents {
                            if let Some(key) = obj.key {
                                objects_to_delete
                                    .push(ObjectIdentifier::builder().key(key).build().unwrap());
                            }
                        }
                    }

                    if response.is_truncated.unwrap_or(false) {
                        continuation_token = response.next_continuation_token;
                    } else {
                        break;
                    }
                }
                Err(e) => {
                    error!("Failed to list objects for deletion: {:?}", e);
                    return Json(FileResponse {
                        success: false,
                        message: format!("Failed to list objects: {}", e),
                        data: None,
                    });
                }
            }
        }

        if !objects_to_delete.is_empty() {
            let delete = Delete::builder()
                .set_objects(Some(objects_to_delete.clone()))
                .build()
                .unwrap();

            match s3
                .delete_objects()
                .bucket(&bucket)
                .delete(delete)
                .send()
                .await
            {
                Ok(_) => {
                    info!(
                        "Deleted {} objects from {}/{}",
                        objects_to_delete.len(),
                        bucket,
                        path
                    );
                    Json(FileResponse {
                        success: true,
                        message: format!("Deleted {} files", objects_to_delete.len()),
                        data: None,
                    })
                }
                Err(e) => {
                    error!("Failed to delete objects: {:?}", e);
                    Json(FileResponse {
                        success: false,
                        message: format!("Failed to delete: {}", e),
                        data: None,
                    })
                }
            }
        } else {
            Json(FileResponse {
                success: true,
                message: "No files to delete".to_string(),
                data: None,
            })
        }
    } else {
        let s3 = match state.s3_client.as_ref() {
            Some(client) => client,
            None => {
                return Json(FileResponse {
                    success: false,
                    message: "S3 client not configured".to_string(),
                    data: None,
                })
            }
        };

        match s3.delete_object().bucket(&bucket).key(&path).send().await {
            Ok(_) => {
                info!("File deleted: {}/{}", bucket, path);
                Json(FileResponse {
                    success: true,
                    message: "File deleted successfully".to_string(),
                    data: None,
                })
            }
            Err(e) => {
                error!("Failed to delete file: {:?}", e);
                Json(FileResponse {
                    success: false,
                    message: format!("Failed to delete file: {}", e),
                    data: None,
                })
            }
        }
    }
}

pub async fn create_folder(
    State(state): State<Arc<AppState>>,
    Path((bucket, path)): Path<(String, String)>,
    Json(folder_name): Json<String>,
) -> impl IntoResponse {
    let folder_path = format!("{}/{}/", path.trim_end_matches('/'), folder_name);

    let s3 = match state.s3_client.as_ref() {
        Some(client) => client,
        None => {
            return Json(FileResponse {
                success: false,
                message: "S3 client not configured".to_string(),
                data: None,
            })
        }
    };

    match s3
        .put_object()
        .bucket(&bucket)
        .key(&folder_path)
        .body(ByteStream::from(vec![]))
        .send()
        .await
    {
        Ok(_) => {
            info!("Folder created: {}/{}", bucket, folder_path);
            Json(FileResponse {
                success: true,
                message: "Folder created successfully".to_string(),
                data: Some(serde_json::json!({
                    "bucket": bucket,
                    "path": folder_path,
                })),
            })
        }
        Err(e) => {
            error!("Failed to create folder: {:?}", e);
            Json(FileResponse {
                success: false,
                message: format!("Failed to create folder: {}", e),
                data: None,
            })
        }
    }
}

pub async fn copy_file(
    State(state): State<Arc<AppState>>,
    Json(operation): Json<FileOperation>,
) -> impl IntoResponse {
    let copy_source = format!("{}/{}", operation.source_bucket, operation.source_path);

    let s3 = match state.s3_client.as_ref() {
        Some(client) => client,
        None => {
            return Json(FileResponse {
                success: false,
                message: "S3 client not configured".to_string(),
                data: None,
            })
        }
    };

    match s3
        .copy_object()
        .copy_source(&copy_source)
        .bucket(&operation.dest_bucket)
        .key(&operation.dest_path)
        .send()
        .await
    {
        Ok(_) => {
            info!(
                "File copied from {} to {}/{}",
                copy_source, operation.dest_bucket, operation.dest_path
            );
            Json(FileResponse {
                success: true,
                message: "File copied successfully".to_string(),
                data: Some(serde_json::json!({
                    "source": copy_source,
                    "destination": format!("{}/{}", operation.dest_bucket, operation.dest_path),
                })),
            })
        }
        Err(e) => {
            error!("Failed to copy file: {:?}", e);
            Json(FileResponse {
                success: false,
                message: format!("Failed to copy file: {}", e),
                data: None,
            })
        }
    }
}

pub async fn move_file(
    State(state): State<Arc<AppState>>,
    Json(operation): Json<FileOperation>,
) -> impl IntoResponse {
    let copy_source = format!("{}/{}", operation.source_bucket, operation.source_path);

    let s3 = match state.s3_client.as_ref() {
        Some(client) => client,
        None => {
            return Json(FileResponse {
                success: false,
                message: "S3 client not configured".to_string(),
                data: None,
            })
        }
    };

    match s3
        .copy_object()
        .copy_source(&copy_source)
        .bucket(&operation.dest_bucket)
        .key(&operation.dest_path)
        .send()
        .await
    {
        Ok(_) => {
            match s3
                .delete_object()
                .bucket(&operation.source_bucket)
                .key(&operation.source_path)
                .send()
                .await
            {
                Ok(_) => {
                    info!(
                        "File moved from {} to {}/{}",
                        copy_source, operation.dest_bucket, operation.dest_path
                    );
                    Json(FileResponse {
                        success: true,
                        message: "File moved successfully".to_string(),
                        data: Some(serde_json::json!({
                            "source": copy_source,
                            "destination": format!("{}/{}", operation.dest_bucket, operation.dest_path),
                        })),
                    })
                }
                Err(e) => {
                    error!("Failed to delete source after copy: {:?}", e);
                    Json(FileResponse {
                        success: false,
                        message: format!("File copied but failed to delete source: {}", e),
                        data: None,
                    })
                }
            }
        }
        Err(e) => {
            error!("Failed to copy file for move: {:?}", e);
            Json(FileResponse {
                success: false,
                message: format!("Failed to move file: {}", e),
                data: None,
            })
        }
    }
}

pub async fn search_files(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let bucket = params
        .get("bucket")
        .cloned()
        .unwrap_or_else(|| "default".to_string());
    let query = params.get("query").cloned().unwrap_or_default();
    let file_type = params.get("file_type").cloned();

    let mut results = Vec::new();
    let mut continuation_token = None;

    loop {
        let s3 = match state.s3_client.as_ref() {
            Some(client) => client,
            None => {
                return Json(FileResponse {
                    success: false,
                    message: "S3 client not configured".to_string(),
                    data: None,
                })
            }
        };
        let mut list_req = s3.list_objects_v2().bucket(&bucket).max_keys(1000);

        if let Some(token) = continuation_token {
            list_req = list_req.continuation_token(token);
        }

        match list_req.send().await {
            Ok(response) => {
                if let Some(contents) = response.contents {
                    for obj in contents {
                        let key = obj.key.unwrap_or_default();
                        let name = key.split('/').last().unwrap_or(&key);

                        let matches_query =
                            query.is_empty() || name.to_lowercase().contains(&query.to_lowercase());

                        let matches_type = file_type.as_ref().map_or(true, |ft| {
                            key.to_lowercase()
                                .ends_with(&format!(".{}", ft.to_lowercase()))
                        });

                        if matches_query && matches_type && !key.ends_with('/') {
                            results.push(FileItem {
                                name: name.to_string(),
                                path: key.clone(),
                                size: obj.size.unwrap_or(0) as u64,
                                modified: obj
                                    .last_modified
                                    .map(|d| d.to_string())
                                    .unwrap_or_else(|| Utc::now().to_rfc3339()),
                                is_dir: false,
                                mime_type: mime_guess::from_path(&key)
                                    .first()
                                    .map(|m| m.to_string()),
                                icon: get_file_icon(&key),
                            });
                        }
                    }
                }

                if response.is_truncated.unwrap_or(false) {
                    continuation_token = response.next_continuation_token;
                } else {
                    break;
                }
            }
            Err(e) => {
                error!("Failed to search files: {:?}", e);
                return Json(FileResponse {
                    success: false,
                    message: format!("Search failed: {}", e),
                    data: None,
                });
            }
        }
    }

    Json(FileResponse {
        success: true,
        message: format!("Found {} files", results.len()),
        data: Some(serde_json::to_value(results).unwrap()),
    })
}

pub async fn get_quota(
    State(state): State<Arc<AppState>>,
    Path(bucket): Path<String>,
) -> impl IntoResponse {
    let mut total_size = 0u64;
    let mut _total_objects = 0u64;
    let mut continuation_token = None;

    loop {
        let s3 = match state.s3_client.as_ref() {
            Some(client) => client,
            None => {
                return Json(FileResponse {
                    success: false,
                    message: "S3 client not configured".to_string(),
                    data: None,
                })
            }
        };
        let mut list_req = s3.list_objects_v2().bucket(&bucket).max_keys(1000);

        if let Some(token) = continuation_token {
            list_req = list_req.continuation_token(token);
        }

        match list_req.send().await {
            Ok(response) => {
                if let Some(contents) = response.contents {
                    for obj in contents {
                        total_size += obj.size.unwrap_or(0) as u64;
                        _total_objects += 1;
                    }
                }

                if response.is_truncated.unwrap_or(false) {
                    continuation_token = response.next_continuation_token;
                } else {
                    break;
                }
            }
            Err(e) => {
                error!("Failed to calculate quota: {:?}", e);
                return Json(FileResponse {
                    success: false,
                    message: format!("Failed to get quota: {}", e),
                    data: None,
                });
            }
        }
    }

    let total_bytes: u64 = 10 * 1024 * 1024 * 1024; // 10GB limit
    let available_bytes = total_bytes.saturating_sub(total_size);
    let percentage_used = (total_size as f32 / total_bytes as f32) * 100.0;

    Json(FileResponse {
        success: true,
        message: "Quota calculated".to_string(),
        data: Some(serde_json::json!(QuotaInfo {
            total_bytes,
            used_bytes: total_size,
            available_bytes,
            percentage_used,
        })),
    })
}

pub async fn upload_multipart(
    State(state): State<Arc<AppState>>,
    Path((bucket, path)): Path<(String, String)>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let file_name = field
            .file_name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let content_type = field
            .content_type()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "application/octet-stream".to_string());

        let data = field.bytes().await.unwrap();
        let file_path = format!("{}/{}", path.trim_end_matches('/'), file_name);

        let s3 = match state.s3_client.as_ref() {
            Some(client) => client,
            None => {
                return Json(FileResponse {
                    success: false,
                    message: "S3 client not configured".to_string(),
                    data: None,
                })
            }
        };

        match s3
            .put_object()
            .bucket(&bucket)
            .key(&file_path)
            .body(ByteStream::from(data.to_vec()))
            .content_type(&content_type)
            .send()
            .await
        {
            Ok(_) => {
                info!("Uploaded file: {}/{}", bucket, file_path);
                return Json(FileResponse {
                    success: true,
                    message: "File uploaded successfully".to_string(),
                    data: Some(serde_json::json!({
                        "bucket": bucket,
                        "path": file_path,
                        "size": data.len(),
                        "content_type": content_type,
                    })),
                });
            }
            Err(e) => {
                error!("Failed to upload file: {:?}", e);
                return Json(FileResponse {
                    success: false,
                    message: format!("Upload failed: {}", e),
                    data: None,
                });
            }
        }
    }

    Json(FileResponse {
        success: false,
        message: "No file received".to_string(),
        data: None,
    })
}

pub async fn recent_files(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let bucket = params
        .get("bucket")
        .cloned()
        .unwrap_or_else(|| "default".to_string());
    let limit = params
        .get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(20);

    let mut all_files = Vec::new();
    let mut continuation_token = None;

    loop {
        let s3 = match state.s3_client.as_ref() {
            Some(client) => client,
            None => {
                return Json(FileResponse {
                    success: false,
                    message: "S3 client not configured".to_string(),
                    data: None,
                })
            }
        };
        let mut list_req = s3.list_objects_v2().bucket(&bucket).max_keys(1000);

        if let Some(token) = continuation_token {
            list_req = list_req.continuation_token(token);
        }

        match list_req.send().await {
            Ok(response) => {
                if let Some(contents) = response.contents {
                    for obj in contents {
                        let key = obj.key.unwrap_or_default();
                        if !key.ends_with('/') {
                            all_files.push((
                                obj.last_modified.unwrap(),
                                FileItem {
                                    name: key.split('/').last().unwrap_or(&key).to_string(),
                                    path: key.clone(),
                                    size: obj.size.unwrap_or(0) as u64,
                                    modified: obj.last_modified.unwrap().to_string(),
                                    is_dir: false,
                                    mime_type: mime_guess::from_path(&key)
                                        .first()
                                        .map(|m| m.to_string()),
                                    icon: get_file_icon(&key),
                                },
                            ));
                        }
                    }
                }

                if response.is_truncated.unwrap_or(false) {
                    continuation_token = response.next_continuation_token;
                } else {
                    break;
                }
            }
            Err(e) => {
                error!("Failed to get recent files: {:?}", e);
                return Json(FileResponse {
                    success: false,
                    message: format!("Failed to get recent files: {}", e),
                    data: None,
                });
            }
        }
    }

    all_files.sort_by(|a, b| b.0.cmp(&a.0));
    let recent: Vec<FileItem> = all_files
        .into_iter()
        .take(limit)
        .map(|(_, item)| item)
        .collect();

    Json(FileResponse {
        success: true,
        message: format!("Found {} recent files", recent.len()),
        data: Some(serde_json::to_value(recent).unwrap()),
    })
}

fn get_file_icon(path: &str) -> String {
    let extension = path.split('.').last().unwrap_or("").to_lowercase();
    match extension.as_str() {
        "pdf" => "📄",
        "doc" | "docx" => "📝",
        "xls" | "xlsx" => "📊",
        "ppt" | "pptx" => "📽️",
        "jpg" | "jpeg" | "png" | "gif" | "bmp" => "🖼️",
        "mp4" | "avi" | "mov" | "mkv" => "🎥",
        "mp3" | "wav" | "flac" | "aac" => "🎵",
        "zip" | "rar" | "7z" | "tar" | "gz" => "📦",
        "js" | "ts" | "jsx" | "tsx" => "📜",
        "rs" => "🦀",
        "py" => "🐍",
        "json" | "xml" | "yaml" | "yml" => "📋",
        "txt" | "md" => "📃",
        "html" | "css" => "🌐",
        _ => "📎",
    }
    .to_string()
}

pub fn configure() -> axum::routing::Router<Arc<AppState>> {
    use axum::routing::{delete, get, post, Router};

    Router::new()
        .route("/api/drive/list", get(list_files))
        .route("/api/drive/read/:bucket/*path", get(read_file))
        .route("/api/drive/write/:bucket/*path", post(write_file))
        .route("/api/drive/delete/:bucket/*path", delete(delete_file))
        .route("/api/drive/folder/:bucket/*path", post(create_folder))
        .route("/api/drive/copy", post(copy_file))
        .route("/api/drive/move", post(move_file))
        .route("/api/drive/search", get(search_files))
        .route("/api/drive/quota/:bucket", get(get_quota))
        .route("/api/drive/upload/:bucket/*path", post(upload_multipart))
        .route("/api/drive/recent", get(recent_files))
}
