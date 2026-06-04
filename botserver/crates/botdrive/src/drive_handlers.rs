// Drive HTTP handlers — Issue #589 per-user file scoping

use botcore::shared::state::AppState;
use crate::drive_types::*;
use crate::user_scope;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use base64::Engine;
use log::{info, warn};
use std::collections::HashMap;
use std::sync::Arc;

fn err(status: StatusCode, msg: &str) -> (StatusCode, Json<serde_json::Value>) {
    (status, Json(serde_json::json!({"error": msg})))
}

fn resolve_scope_prefix(scope: &FileScope, user_id: &str) -> String {
    match scope {
        FileScope::User => user_scope::resolve_key_prefix(scope, user_id),
        FileScope::Bot => user_scope::resolve_key_prefix(scope, user_id),
    }
}

fn resolve_bucket<'a>(
    state: &'a AppState,
    override_bucket: Option<&'a str>,
    scope: &FileScope,
    user_id: Option<&'a str>,
    bot_name: Option<&'a str>,
) -> Result<String, (StatusCode, Json<serde_json::Value>)> {
    if let Some(b) = override_bucket {
        return Ok(b.to_string());
    }
    let default = &state.bucket_name;
    if default.is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "No bucket specified"));
    }
    Ok(user_scope::resolve_bucket_name(scope, user_id, bot_name, default))
}

fn get_drive(state: &AppState) -> Result<&Arc<dyn botlib::traits::DriveRepository>, (StatusCode, Json<serde_json::Value>)> {
    state
        .drive
        .as_ref()
        .ok_or_else(|| err(StatusCode::SERVICE_UNAVAILABLE, "Drive storage not configured"))
}

fn normalize_path(path: &str) -> String {
    path.trim_matches('/').to_string()
}

fn build_file_list_items(
    prefix: &str,
    keys: &[String],
    metadata_map: &HashMap<String, u64>,
) -> Vec<FileListItem> {
    let prefix_trimmed = prefix.trim_end_matches('/');
    let prefix_len = if prefix_trimmed.is_empty() {
        0
    } else {
        prefix_trimmed.len() + 1
    };

    let mut dirs: HashMap<String, FileListItem> = HashMap::new();
    let mut files: Vec<FileListItem> = Vec::new();

    for key in keys {
        let relative = if key.starts_with(prefix) {
            &key[prefix_len..]
        } else {
            key.as_str()
        };
        if relative.is_empty() {
            continue;
        }

        let parts: Vec<&str> = relative.splitn(2, '/').collect();
        if parts.len() == 2 {
            let dir_name = parts[0];
            dirs.entry(dir_name.to_string()).or_insert_with(|| FileListItem {
                name: dir_name.to_string(),
                path: if prefix_trimmed.is_empty() {
                    dir_name.to_string()
                } else {
                    format!("{prefix_trimmed}/{dir_name}")
                },
                is_dir: true,
                size: 0,
                modified: None,
                is_kb: false,
                is_public: false,
            });
        } else {
            let file_name = parts[0];
            let is_kb = key.contains(".gbkb");
            let size = metadata_map.get(key).copied().unwrap_or(0);
            let modified = None;
            files.push(FileListItem {
                name: file_name.to_string(),
                path: if prefix_trimmed.is_empty() {
                    file_name.to_string()
                } else {
                    format!("{prefix_trimmed}/{file_name}")
                },
                is_dir: false,
                size,
                modified,
                is_kb,
                is_public: false,
            });
        }
    }

    let mut items: Vec<FileListItem> = dirs.into_values().collect();
    items.sort_by(|a, b| a.name.cmp(&b.name));
    files.sort_by(|a, b| a.name.cmp(&b.name));
    items.extend(files);
    items
}

// ====== Handlers ======

pub async fn list_files(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListFilesParams>,
) -> Result<Json<Vec<FileListItem>>, (StatusCode, Json<serde_json::Value>)> {
    let drive = get_drive(&state)?;
    let scope = params.scope.unwrap_or_default();
    let uid = params.user_id.as_deref().unwrap_or("default");
    let bucket = resolve_bucket(&state, params.bucket.as_deref(), &scope, Some(uid), None)?;
    let prefix = resolve_scope_prefix(&scope, uid);
    let sub_path = normalize_path(params.path.as_deref().unwrap_or(""));
    let full_prefix = if sub_path.is_empty() {
        prefix
    } else {
        format!("{prefix}{sub_path}/")
    };

    let keys = drive
        .list_objects(&bucket, Some(&full_prefix))
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to list objects: {e}")))?;

    let mut meta_map: HashMap<String, u64> = HashMap::new();
    if let Ok(objects) = drive.list_objects_with_metadata(&bucket, Some(&full_prefix)).await {
        for obj in objects {
            meta_map.insert(obj.key, obj.size);
        }
    }

    let items = build_file_list_items(&full_prefix, &keys, &meta_map);
    Ok(Json(items))
}

pub async fn upload_file_to_drive(
    State(state): State<Arc<AppState>>,
    Json(req): Json<WriteFileBody>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    let drive = get_drive(&state)?;
    let scope = req.scope.unwrap_or_default();
    let uid = req.user_id.as_deref().unwrap_or("default");
    let bucket = resolve_bucket(&state, req.bucket.as_deref(), &scope, Some(uid), None)?;
    let prefix = resolve_scope_prefix(&scope, uid);

    let data = base64::engine::general_purpose::STANDARD
        .decode(&req.content)
        .map_err(|e| err(StatusCode::BAD_REQUEST, &format!("Invalid base64 content: {e}")))?;

    let key = format!("{prefix}{}", normalize_path(&req.path));
    drive
        .put_object(&bucket, &key, data, None)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("Upload failed: {e}")))?;

    info!("File uploaded: {key} to bucket {bucket}");
    Ok(Json(SuccessResponse { success: true }))
}

pub async fn download_file(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DownloadFileBody>,
) -> Result<Json<DownloadFileResponse>, (StatusCode, Json<serde_json::Value>)> {
    let drive = get_drive(&state)?;
    let scope = req.scope.unwrap_or_default();
    let uid = req.user_id.as_deref().unwrap_or("default");
    let bucket = resolve_bucket(&state, req.bucket.as_deref(), &scope, Some(uid), None)?;
    let prefix = resolve_scope_prefix(&scope, uid);
    let key = format!("{prefix}{}", normalize_path(&req.path));

    let data = drive
        .get_object(&bucket, &key)
        .await
        .map_err(|e| err(StatusCode::NOT_FOUND, &format!("File not found: {e}")))?;

    let content = base64::engine::general_purpose::STANDARD.encode(&data);
    let file_name = req.path.rsplit('/').next().unwrap_or("download").to_string();

    Ok(Json(DownloadFileResponse { content, file_name }))
}

pub async fn delete_file(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DeleteFileBody>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    let drive = get_drive(&state)?;
    let scope = req.scope.unwrap_or_default();
    let uid = req.user_id.as_deref().unwrap_or("default");
    let bucket = resolve_bucket(&state, req.bucket.as_deref(), &scope, Some(uid), None)?;
    let prefix = resolve_scope_prefix(&scope, uid);
    let key = format!("{prefix}{}", normalize_path(&req.path));

    drive
        .delete_object(&bucket, &key)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("Delete failed: {e}")))?;

    info!("File deleted: {key} from bucket {bucket}");
    Ok(Json(SuccessResponse { success: true }))
}

pub async fn create_folder(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateFolderBody>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    let drive = get_drive(&state)?;
    let scope = req.scope.unwrap_or_default();
    let uid = req.user_id.as_deref().unwrap_or("default");
    let bucket = resolve_bucket(&state, req.bucket.as_deref(), &scope, Some(uid), None)?;
    let prefix = resolve_scope_prefix(&scope, uid);
    let parent = normalize_path(&req.path);
    let folder_name = normalize_path(&req.name);
    let key = if parent.is_empty() {
        format!("{prefix}{folder_name}/")
    } else {
        format!("{prefix}{parent}/{folder_name}/")
    };

    drive
        .put_object(&bucket, &key, vec![], None)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("Create folder failed: {e}")))?;

    info!("Folder created: {key} in bucket {bucket}");
    Ok(Json(SuccessResponse { success: true }))
}

pub async fn copy_file(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CopyFileBody>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    let drive = get_drive(&state)?;
    let scope = req.scope.unwrap_or_default();
    let uid = req.user_id.as_deref().unwrap_or("default");
    let prefix = resolve_scope_prefix(&scope, uid);

    let src_bucket = req.source_bucket.as_deref().unwrap_or(&state.bucket_name);
    let dest_bucket = req.dest_bucket.as_deref().unwrap_or(&state.bucket_name);
    let from_key = format!("{prefix}{}", normalize_path(&req.source_path));
    let to_key = format!("{prefix}{}", normalize_path(&req.dest_path));

    let data = drive
        .get_object(src_bucket, &from_key)
        .await
        .map_err(|e| err(StatusCode::NOT_FOUND, &format!("Source file not found: {e}")))?;

    drive
        .put_object(dest_bucket, &to_key, data, None)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("Copy failed: {e}")))?;

    info!("Copied {from_key} -> {to_key} in bucket {dest_bucket}");
    Ok(Json(SuccessResponse { success: true }))
}

pub async fn move_file(
    State(state): State<Arc<AppState>>,
    Json(req): Json<MoveFileBody>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    let drive = get_drive(&state)?;
    let scope = req.scope.unwrap_or_default();
    let uid = req.user_id.as_deref().unwrap_or("default");
    let prefix = resolve_scope_prefix(&scope, uid);

    let src_bucket = req.source_bucket.as_deref().unwrap_or(&state.bucket_name);
    let dest_bucket = req.dest_bucket.as_deref().unwrap_or(&state.bucket_name);
    let from_key = format!("{prefix}{}", normalize_path(&req.source_path));
    let to_key = format!("{prefix}{}", normalize_path(&req.dest_path));

    let data = drive
        .get_object(src_bucket, &from_key)
        .await
        .map_err(|e| err(StatusCode::NOT_FOUND, &format!("Source not found: {e}")))?;

    drive
        .put_object(dest_bucket, &to_key, data, None)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("Move (write) failed: {e}")))?;

    let _ = drive
        .delete_object(src_bucket, &from_key)
        .await
        .map_err(|e| {
            warn!("Move: failed to delete source {from_key}: {e}");
        });

    info!("Moved {from_key} -> {to_key}");
    Ok(Json(SuccessResponse { success: true }))
}

pub async fn search_files(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchQueryParams>,
) -> Result<Json<Vec<FileListItem>>, (StatusCode, Json<serde_json::Value>)> {
    let drive = get_drive(&state)?;
    let scope = params.scope.unwrap_or_default();
    let uid = params.user_id.as_deref().unwrap_or("default");
    let bucket = resolve_bucket(&state, params.bucket.as_deref(), &scope, Some(uid), None)?;
    let prefix = resolve_scope_prefix(&scope, uid);
    let query = params.query.unwrap_or_default();
    let query_lower = query.to_lowercase();

    let keys = drive
        .list_objects(&bucket, Some(&prefix))
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("Search failed: {e}")))?;

    let mut meta_map: HashMap<String, u64> = HashMap::new();
    if let Ok(objects) = drive.list_objects_with_metadata(&bucket, Some(&prefix)).await {
        for obj in objects {
            meta_map.insert(obj.key, obj.size);
        }
    }

    let all_items = build_file_list_items(&prefix, &keys, &meta_map);
    let filtered: Vec<FileListItem> = all_items
        .into_iter()
        .filter(|item| item.name.to_lowercase().contains(&query_lower))
        .collect();

    Ok(Json(filtered))
}

pub async fn recent_files(
    State(state): State<Arc<AppState>>,
    Query(params): Query<RecentQueryParams>,
) -> Result<Json<Vec<FileListItem>>, (StatusCode, Json<serde_json::Value>)> {
    let drive = get_drive(&state)?;
    let scope = params.scope.unwrap_or_default();
    let uid = params.user_id.as_deref().unwrap_or("default");
    let bucket = resolve_bucket(&state, params.bucket.as_deref(), &scope, Some(uid), None)?;
    let prefix = resolve_scope_prefix(&scope, uid);

    let keys = drive
        .list_objects(&bucket, Some(&prefix))
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to list recent: {e}")))?;

    let mut meta_map: HashMap<String, u64> = HashMap::new();
    if let Ok(objects) = drive.list_objects_with_metadata(&bucket, Some(&prefix)).await {
        for obj in objects {
            meta_map.insert(obj.key, obj.size);
        }
    }

    let mut items = build_file_list_items(&prefix, &keys, &meta_map);
    items.retain(|item| !item.is_dir);
    items.sort_by(|a, b| b.name.cmp(&a.name));
    items.truncate(50);

    Ok(Json(items))
}

pub async fn list_buckets(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<BucketListItem>>, (StatusCode, Json<serde_json::Value>)> {
    let drive = get_drive(&state)?;

    let bucket_names = drive
        .list_all_buckets()
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to list buckets: {e}")))?;

    let items: Vec<BucketListItem> = bucket_names
        .into_iter()
        .map(|name| BucketListItem {
            is_gbai: name.ends_with(".gbai"),
            name,
        })
        .collect();

    Ok(Json(items))
}

pub async fn open_file(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<OpenFileBody>,
) -> Result<Json<OpenFileResponse>, (StatusCode, Json<serde_json::Value>)> {
    let path = &req.path;
    let ext = path.rsplit('.').next().unwrap_or("");
    let (app, url) = match ext {
        "txt" | "md" | "bas" | "json" | "csv" | "xml" | "html" | "css" | "js" => {
            ("editor".to_string(), format!("/suite/docs/?file={path}"))
        }
        "pdf" => ("viewer".to_string(), format!("/suite/docs/?file={path}")),
        _ => ("preview".to_string(), format!("/suite/docs/?file={path}")),
    };
    Ok(Json(OpenFileResponse { app, url }))
}

pub async fn quota(
    State(state): State<Arc<AppState>>,
) -> Result<Json<QuotaResponse>, (StatusCode, Json<serde_json::Value>)> {
    let drive = get_drive(&state)?;
    let bucket = &state.bucket_name;

    let mut total_size: u64 = 0;
    let objects = drive
        .list_objects_with_metadata(bucket, None)
        .await
        .unwrap_or_default();
    for obj in &objects {
        total_size += obj.size;
    }

    let total_bytes: u64 = 10 * 1024 * 1024 * 1024;
    let used = total_size;
    let available = total_bytes.saturating_sub(used);
    let percentage = if total_bytes > 0 {
        (used as f64 / total_bytes as f64) * 100.0
    } else {
        0.0
    };

    Ok(Json(QuotaResponse {
        used_bytes: used,
        total_bytes,
        available_bytes: available,
        percentage_used: percentage,
    }))
}

pub async fn list_favorites(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<Vec<FileListItem>>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(vec![]))
}

pub async fn list_shared(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<Vec<FileListItem>>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(vec![]))
}

pub async fn share_folder(
    State(_state): State<Arc<AppState>>,
    Json(_req): Json<ShareRequest>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(SuccessResponse { success: true }))
}
