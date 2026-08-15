// Drive HTTP handlers — Issue #589 per-user file scoping

use botcore::shared::state::AppState;
use crate::drive_types::*;
use crate::user_scope;
use axum::{
    extract::{Query, State, Extension},
    http::StatusCode,
    response::Json,
};
use base64::Engine;
use botsecurity_auth::auth_api::types::AuthenticatedUser;

use diesel::prelude::*;
use diesel::sql_types::{BigInt, Bool, Nullable, Text, Timestamptz};
use log::{info, warn};
use std::collections::HashMap;
use std::sync::Arc;

fn err(status: StatusCode, msg: &str) -> (StatusCode, Json<serde_json::Value>) {
    warn!("drive_handler error ({}): {}", status.as_u16(), msg);
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

fn get_user_id(user: &AuthenticatedUser) -> String {
    if let Some(ref email) = user.email {
        if !email.is_empty() && email != "session-user" {
            return email.clone();
        }
    }
    if user.user_id.is_nil() { "default".to_string() } else { user.user_id.to_string() }
}

fn is_admin_user(user: &AuthenticatedUser) -> bool {
    user.is_admin() || user.is_super_admin()
}

// ====== Handlers ======

pub async fn list_files(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<ListFilesParams>,
) -> Result<Json<Vec<FileListItem>>, (StatusCode, Json<serde_json::Value>)> {
    let drive = get_drive(&state)?;
    let scope = params.scope.unwrap_or_default();
    let uid = params.user_id.as_deref().map(|s| s.to_string()).unwrap_or_else(|| get_user_id(&user));
    let bucket = resolve_bucket(&state, params.bucket.as_deref(), &scope, Some(uid.as_str()), None)?;
    let prefix = resolve_scope_prefix(&scope, &uid);
    let sub_path = normalize_path(params.path.as_deref().unwrap_or(""));
    let full_prefix = if sub_path.is_empty() {
        prefix
    } else {
        format!("{prefix}{sub_path}/")
    };

    // Ensure bucket exists — prevents 500 when user's personal bucket hasn't been created yet
    let _ = drive.create_bucket_if_not_exists(&bucket).await;

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
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<WriteFileBody>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    let drive = get_drive(&state)?;
    let scope = req.scope.unwrap_or_default();
    let uid = req.user_id.as_deref().map(|s| s.to_string()).unwrap_or_else(|| get_user_id(&user));
    let bucket = resolve_bucket(&state, req.bucket.as_deref(), &scope, Some(uid.as_str()), None)?;
    let prefix = resolve_scope_prefix(&scope, &uid);

    // Accept both raw text (designer/editor send the file content verbatim)
    // and base64-encoded payloads (drive uploads). Detect: a strict base64
    // decode that round-trips cleanly (ignoring padding) is treated as
    // encoded; otherwise the content is used as-is so text files never fail
    // on invalid base64.
    let data = match base64::engine::general_purpose::STANDARD.decode(&req.content) {
        Ok(decoded) if !decoded.is_empty() => {
            let re_encoded = base64::engine::general_purpose::STANDARD.encode(&decoded);
            let normalized_input = req.content.trim_end_matches('=');
            let normalized_encoded = re_encoded.trim_end_matches('=');
            if normalized_encoded == normalized_input {
                decoded
            } else {
                req.content.as_bytes().to_vec()
            }
        }
        _ => req.content.as_bytes().to_vec(),
    };

    let key = format!("{prefix}{}", normalize_path(&req.path));
    drive
        .put_object(&bucket, &key, data, None)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("Upload failed: {e}")))?;

    info!("File uploaded: {key} to bucket {bucket}");
    Ok(Json(SuccessResponse { success: true }))
}

/// `POST /api/files/read` — returns a file's content as raw text (`{content}`)
/// for the designer/editor apps. Mirrors `download_file` but returns plain
/// UTF-8 text instead of base64 so BASIC sources render directly.
pub async fn read_file(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<DownloadFileBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let drive = get_drive(&state)?;
    let scope = req.scope.unwrap_or_default();
    let uid = req.user_id.as_deref().map(|s| s.to_string()).unwrap_or_else(|| get_user_id(&user));
    let bucket = resolve_bucket(&state, req.bucket.as_deref(), &scope, Some(uid.as_str()), None)?;
    let prefix = resolve_scope_prefix(&scope, &uid);
    let key = format!("{prefix}{}", normalize_path(&req.path));

    let data = drive
        .get_object(&bucket, &key)
        .await
        .map_err(|e| err(StatusCode::NOT_FOUND, &format!("File not found: {e}")))?;

    let content = String::from_utf8_lossy(&data).to_string();
    Ok(Json(serde_json::json!({ "content": content })))
}

pub async fn download_file(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<DownloadFileBody>,
) -> Result<Json<DownloadFileResponse>, (StatusCode, Json<serde_json::Value>)> {
    let drive = get_drive(&state)?;
    let scope = req.scope.unwrap_or_default();
    let uid = req.user_id.as_deref().map(|s| s.to_string()).unwrap_or_else(|| get_user_id(&user));
    let bucket = resolve_bucket(&state, req.bucket.as_deref(), &scope, Some(uid.as_str()), None)?;
    let prefix = resolve_scope_prefix(&scope, &uid);
    let key = format!("{prefix}{}", normalize_path(&req.path));

    let data = drive
        .get_object(&bucket, &key)
        .await
        .map_err(|e| err(StatusCode::NOT_FOUND, &format!("File not found: {e}")))?;

    let content = base64::engine::general_purpose::STANDARD.encode(&data);
    let file_name = req.path.rsplit('/').next().unwrap_or("download").to_string();

    Ok(Json(DownloadFileResponse { content, file_name }))
}

pub async fn download_file_binary(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<DownloadFileBody>,
) -> Result<axum::response::Response, (StatusCode, Json<serde_json::Value>)> {
    let drive = get_drive(&state)?;
    let scope = req.scope.unwrap_or_default();
    let uid = req.user_id.as_deref().map(|s| s.to_string()).unwrap_or_else(|| get_user_id(&user));
    let bucket = resolve_bucket(&state, req.bucket.as_deref(), &scope, Some(uid.as_str()), None)?;
    let prefix = resolve_scope_prefix(&scope, &uid);
    let key = format!("{prefix}{}", normalize_path(&req.path));

    let data = drive
        .get_object(&bucket, &key)
        .await
        .map_err(|e| err(StatusCode::NOT_FOUND, &format!("File not found: {e}")))?;

    let file_name = req.path.rsplit('/').next().unwrap_or("download").to_string();
    let mime = guess_mime(&file_name);

    use axum::response::IntoResponse;
    let headers = [
        (axum::http::header::CONTENT_TYPE, mime.to_string()),
        (
            axum::http::header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", file_name),
        ),
        (axum::http::header::CONTENT_LENGTH, data.len().to_string()),
    ];
    Ok((headers, data).into_response())
}

fn guess_mime(file_name: &str) -> &'static str {
    let ext = file_name.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        "pdf" => "application/pdf",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "txt" | "log" | "md" => "text/plain; charset=utf-8",
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "application/javascript",
        "json" => "application/json",
        "xml" => "application/xml",
        "zip" => "application/zip",
        _ => "application/octet-stream",
    }
}

pub async fn delete_file(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<DeleteFileBody>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    let drive = get_drive(&state)?;
    let scope = req.scope.unwrap_or_default();
    let uid = req.user_id.as_deref().map(|s| s.to_string()).unwrap_or_else(|| get_user_id(&user));
    let bucket = resolve_bucket(&state, req.bucket.as_deref(), &scope, Some(uid.as_str()), None)?;
    let prefix = resolve_scope_prefix(&scope, &uid);
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
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<CreateFolderBody>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    let drive = get_drive(&state)?;
    let scope = req.scope.unwrap_or_default();
    let uid = req.user_id.as_deref().map(|s| s.to_string()).unwrap_or_else(|| get_user_id(&user));
    let bucket = resolve_bucket(&state, req.bucket.as_deref(), &scope, Some(uid.as_str()), None)?;
    let prefix = resolve_scope_prefix(&scope, &uid);
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
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<CopyFileBody>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    let drive = get_drive(&state)?;
    let scope = req.scope.unwrap_or_default();
    let uid = req.user_id.as_deref().map(|s| s.to_string()).unwrap_or_else(|| get_user_id(&user));
    let prefix = resolve_scope_prefix(&scope, &uid);

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
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<MoveFileBody>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    let drive = get_drive(&state)?;
    let scope = req.scope.unwrap_or_default();
    let uid = req.user_id.as_deref().map(|s| s.to_string()).unwrap_or_else(|| get_user_id(&user));
    let prefix = resolve_scope_prefix(&scope, &uid);

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
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<SearchQueryParams>,
) -> Result<Json<Vec<FileListItem>>, (StatusCode, Json<serde_json::Value>)> {
    let drive = get_drive(&state)?;
    let scope = params.scope.unwrap_or_default();
    let uid = params.user_id.as_deref().map(|s| s.to_string()).unwrap_or_else(|| get_user_id(&user));
    let bucket = resolve_bucket(&state, params.bucket.as_deref(), &scope, Some(uid.as_str()), None)?;
    let prefix = resolve_scope_prefix(&scope, &uid);
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
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<RecentQueryParams>,
) -> Result<Json<Vec<FileListItem>>, (StatusCode, Json<serde_json::Value>)> {
    let drive = get_drive(&state)?;
    let scope = params.scope.unwrap_or_default();
    let uid = params.user_id.as_deref().map(|s| s.to_string()).unwrap_or_else(|| get_user_id(&user));
    let bucket = resolve_bucket(&state, params.bucket.as_deref(), &scope, Some(uid.as_str()), None)?;
    let prefix = resolve_scope_prefix(&scope, &uid);

    if let Ok(mut conn) = state.conn.get() {
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Text)]
            file_path: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            file_type: String,
            #[diesel(sql_type = Nullable<BigInt>)]
            file_size: Option<i64>,
            #[diesel(sql_type = Nullable<Timestamptz>)]
            last_modified: Option<chrono::DateTime<chrono::Utc>>,
        }
        let rows = diesel::sql_query(
            "SELECT file_path, file_type, file_size, last_modified FROM drive_files WHERE file_path LIKE $1 || '/%' AND indexed = true ORDER BY last_modified DESC NULLS LAST LIMIT 50"
        )
        .bind::<Text, _>(&bucket)
        .load::<Row>(&mut conn);

        if let Ok(values) = rows {
            let items: Vec<FileListItem> = values.into_iter().map(|r| {
                let file_path = &r.file_path;
                let relative = if file_path.starts_with(&prefix) {
                    file_path[prefix.len()..].to_string()
                } else {
                    file_path.to_string()
                };
                FileListItem {
                    is_dir: false,
                    name: relative.rsplit('/').next().unwrap_or(relative.as_str()).to_string(),
                    path: relative,
                    size: r.file_size.unwrap_or(0) as u64,
                    modified: r.last_modified.map(|d| d.to_rfc3339()),
                    is_kb: r.file_type == "kb",
                    is_public: false,
                }
            }).collect();
            return Ok(Json(items));
        }
    }

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

    let mut fallback_items = build_file_list_items(&prefix, &keys, &meta_map);
    fallback_items.retain(|item| !item.is_dir);
    fallback_items.sort_by(|a, b| b.name.cmp(&a.name));
    fallback_items.truncate(50);
    Ok(Json(fallback_items))
}

pub async fn list_buckets(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<ListBucketsParams>,
) -> Result<Json<Vec<BucketListItem>>, (StatusCode, Json<serde_json::Value>)> {
    let drive = get_drive(&state)?;

    let bucket_names = drive
        .list_all_buckets()
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to list buckets: {e}")))?;

    let is_admin = is_admin_user(&user);
    let items: Vec<BucketListItem> = bucket_names
        .into_iter()
        .filter(|name| {
            if !is_admin && !name.ends_with(".gbai") {
                return true; // non-admins can see non-gbai buckets
            }
            if !is_admin && name.ends_with(".gbai") {
                return false; // non-admins cannot see .gbai buckets
            }
            if let Some(ref bot) = params.bot {
                let gbai = format!("{bot}.gbai");
                let gborg = format!("{bot}.gborg");
                name == &gbai || name == &gborg
            } else {
                true
            }
        })
        .map(|name| BucketListItem {
            is_gbai: name.ends_with(".gbai"),
            is_gborg: name.ends_with(".gborg"),
            name,
        })
        .collect();

    Ok(Json(items))
}

pub async fn create_bot(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateBotRequest>,
) -> Result<Json<CreateBotResponse>, (StatusCode, Json<serde_json::Value>)> {
    let name = req.name.trim().to_lowercase();
    if name.is_empty() || !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid bot name. Use lowercase letters, numbers, and hyphens (3-50 chars)."));
    }
    if name.len() < 3 || name.len() > 50 {
        return Err(err(StatusCode::BAD_REQUEST, "Bot name must be between 3 and 50 characters."));
    }

    let bucket_name = format!("{}.gbai", name);
    let drive = get_drive(&state)?;

    // Check if already exists
    let existing = drive.list_all_buckets().await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to list buckets: {e}")))?;
    if existing.iter().any(|b| b == &bucket_name) {
        return Err(err(StatusCode::CONFLICT, &format!("Bot '{}' already exists", name)));
    }

    // Create bucket
    drive.create_bucket_if_not_exists(&bucket_name).await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to create bucket: {e}")))?;

    // Seed default folder structure with .keep marker files
    let seed_paths = [
        format!("{name}.gbdialog/.keep"),
        format!("{name}.gbkb/.keep"),
        format!("{name}.gbdrive/.keep"),
        format!("{name}.gbot/.keep"),
        format!("{name}.gbot/config.csv"),
    ];
    for path in &seed_paths {
        let content = if path.ends_with("config.csv") {
            "system-prompt,\nhistory-limit,6\n".as_bytes().to_vec()
        } else {
            Vec::new()
        };
        drive.put_object(&bucket_name, path, content, Some("text/plain")).await
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to seed {path}: {e}")))?;
    }

    Ok(Json(CreateBotResponse {
        name: bucket_name,
        status: "created".to_string(),
    }))
}

pub async fn delete_bot(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<CreateBotRequest>,
) -> Result<Json<DeleteBotResponse>, (StatusCode, Json<serde_json::Value>)> {
    if !is_admin_user(&user) {
        return Err(err(StatusCode::FORBIDDEN, "Administrator access required"));
    }

    let bot_name = req.name.trim().to_lowercase();
    if bot_name.is_empty() || !bot_name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid bot name. Use lowercase letters, numbers, and hyphens (3-50 chars)."));
    }
    if bot_name.len() < 3 || bot_name.len() > 50 {
        return Err(err(StatusCode::BAD_REQUEST, "Bot name must be between 3 and 50 characters."));
    }

    let bucket_name = format!("{}.gbai", bot_name);
    let drive = get_drive(&state)?;

    // List and delete all objects in the bucket
    let objects = drive.list_objects(&bucket_name, None).await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to list bucket objects: {e}")))?;

    if !objects.is_empty() {
        drive.delete_objects(&bucket_name, objects).await
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("Failed to delete bot objects: {e}")))?;
    }

    info!("Bot '{}' deleted by user {}", bot_name, user.user_id);
    Ok(Json(DeleteBotResponse {
        status: "deleted".to_string(),
    }))
}

pub async fn open_file(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<OpenFileBody>,
) -> Result<Json<OpenFileResponse>, (StatusCode, Json<serde_json::Value>)> {
    let path = &req.path;
    let ext = path.rsplit('.').next().unwrap_or("");
    let bucket = req.bucket.as_deref().unwrap_or("");
    let (app, url) = match ext {
        "bas" => (
            "designer".to_string(),
            format!("/suite/designer.html?file={path}&bucket={bucket}"),
        ),
        "txt" | "md" | "json" | "xml" | "html" | "css" | "js" => {
            ("editor".to_string(), format!("/suite/editor.html?file={path}&bucket={bucket}"))
        }
        "pdf" => ("viewer".to_string(), format!("/suite/docs/?file={path}")),
        "csv" | "xls" | "xlsx" | "ods" => (
            "sheets".to_string(),
            format!("/suite/sheet/sheet.html?bucket={bucket}&path={path}"),
        ),
        "docx" | "doc" | "odt" | "rtf" => (
            "docs".to_string(),
            format!("/suite/docs/?file={path}&bucket={bucket}"),
        ),
        _ => ("preview".to_string(), format!("/suite/docs/?file={path}")),
    };
    Ok(Json(OpenFileResponse { app, url }))
}

pub async fn quota(
    State(state): State<Arc<AppState>>,
) -> Result<Json<QuotaResponse>, (StatusCode, Json<serde_json::Value>)> {
    let drive = get_drive(&state)?;

    let mut total_size: u64 = 0;
    let mut total_buckets: u64 = 0;

    if let Ok(bucket_names) = drive.list_all_buckets().await {
        for bname in &bucket_names {
            if let Ok(objects) = drive.list_objects_with_metadata(bname, None).await {
                for obj in &objects {
                    total_size = total_size.saturating_add(obj.size);
                }
                total_buckets = total_buckets.saturating_add(1);
            }
        }
    }

    let total_bytes: u64 = (total_buckets + 1).saturating_mul(10 * 1024 * 1024 * 1024);
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
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<TrashQueryParams>,
) -> Result<Json<Vec<StarItem>>, (StatusCode, Json<serde_json::Value>)> {
    let uid = params.user_id.as_deref().map(|s| s.to_string()).unwrap_or_else(|| get_user_id(&user));
    if let Ok(mut conn) = state.conn.get() {
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type = Text)]
            id: String,
            #[diesel(sql_type = Text)]
            bucket: String,
            #[diesel(sql_type = Text)]
            path: String,
            #[diesel(sql_type = Text)]
            created_at: String,
        }
        let rows = diesel::sql_query(
            "SELECT id::text, bucket, path, created_at::text FROM drive_stars WHERE user_id = $1 ORDER BY created_at DESC"
        )
        .bind::<Text, _>(uid)
        .load::<Row>(&mut conn)
        .unwrap_or_default();

        let items: Vec<StarItem> = rows.into_iter().map(|r| StarItem {
            id: r.id,
            bucket: r.bucket,
            path: r.path,
            created_at: r.created_at,
        }).collect();
        return Ok(Json(items));
    }
    Ok(Json(vec![]))
}

pub async fn toggle_star(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<StarToggleBody>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    let uid = req.user_id.as_deref().map(|s| s.to_string()).unwrap_or_else(|| get_user_id(&user));
    let bucket = req.bucket.as_deref().unwrap_or("default");
    let path = &req.path;

    if let Ok(mut conn) = state.conn.get() {
        if req.starred {
            let _ = diesel::sql_query(
                "INSERT INTO drive_stars (user_id, bucket, path) VALUES ($1, $2, $3) ON CONFLICT (user_id, bucket, path) DO NOTHING"
            )
            .bind::<Text, _>(uid)
            .bind::<Text, _>(bucket)
            .bind::<Text, _>(path)
            .execute(&mut conn);
        } else {
            let _ = diesel::sql_query(
                "DELETE FROM drive_stars WHERE user_id = $1 AND bucket = $2 AND path = $3"
            )
            .bind::<Text, _>(uid)
            .bind::<Text, _>(bucket)
            .bind::<Text, _>(path)
            .execute(&mut conn);
        }
    }

    Ok(Json(SuccessResponse { success: true }))
}

pub async fn list_shared(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<TrashQueryParams>,
) -> Result<Json<Vec<ShareItem>>, (StatusCode, Json<serde_json::Value>)> {
    let uid = params.user_id.as_deref().map(|s| s.to_string()).unwrap_or_else(|| get_user_id(&user));
    if let Ok(mut conn) = state.conn.get() {
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type = Text)]
            id: String,
            #[diesel(sql_type = Text)]
            owner_id: String,
            #[diesel(sql_type = Text)]
            recipient_id: String,
            #[diesel(sql_type = Text)]
            bucket: String,
            #[diesel(sql_type = Text)]
            path: String,
            #[diesel(sql_type = Text)]
            permissions: String,
            #[diesel(sql_type = Text)]
            created_at: String,
        }
        let rows = diesel::sql_query(
            "SELECT id::text, owner_id, recipient_id, bucket, path, permissions, created_at::text FROM drive_shares WHERE recipient_id = $1 ORDER BY created_at DESC"
        )
        .bind::<Text, _>(uid)
        .load::<Row>(&mut conn)
        .unwrap_or_default();

        let items: Vec<ShareItem> = rows.into_iter().map(|r| ShareItem {
            id: r.id,
            owner_id: r.owner_id,
            recipient_id: r.recipient_id,
            bucket: r.bucket,
            path: r.path,
            permissions: r.permissions,
            created_at: r.created_at,
        }).collect();
        return Ok(Json(items));
    }
    Ok(Json(vec![]))
}

pub async fn share_folder(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<CreateShareBody>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    let owner_id = req.user_id.as_deref().map(|s| s.to_string()).unwrap_or_else(|| get_user_id(&user));
    let bucket = req.bucket.as_deref().unwrap_or("default");
    let path = &req.path;
    let recipient_id = &req.recipient_id;
    let permissions = req.permissions.as_deref().unwrap_or("read");

    if let Ok(mut conn) = state.conn.get() {
        let _ = diesel::sql_query(
            "INSERT INTO drive_shares (owner_id, recipient_id, bucket, path, permissions) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (owner_id, recipient_id, bucket, path) DO UPDATE SET permissions = $5"
        )
        .bind::<Text, _>(owner_id)
        .bind::<Text, _>(recipient_id)
        .bind::<Text, _>(bucket)
        .bind::<Text, _>(path)
        .bind::<Text, _>(permissions)
        .execute(&mut conn);
    }

    Ok(Json(SuccessResponse { success: true }))
}

pub async fn list_trash(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<TrashQueryParams>,
) -> Result<Json<Vec<TrashItem>>, (StatusCode, Json<serde_json::Value>)> {
    let uid = params.user_id.as_deref().map(|s| s.to_string()).unwrap_or_else(|| get_user_id(&user));
    if let Ok(mut conn) = state.conn.get() {
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type = Text)]
            id: String,
            #[diesel(sql_type = Text)]
            user_id: String,
            #[diesel(sql_type = Text)]
            bucket: String,
            #[diesel(sql_type = Text)]
            path: String,
            #[diesel(sql_type = Text)]
            original_path: String,
            #[diesel(sql_type = Bool)]
            is_dir: bool,
            #[diesel(sql_type = BigInt)]
            size: i64,
            #[diesel(sql_type = Text)]
            deleted_at: String,
            #[diesel(sql_type = Text)]
            expires_at: String,
        }
        let rows = diesel::sql_query(
            "SELECT id::text, user_id, bucket, path, original_path, is_dir, size::bigint, deleted_at::text, expires_at::text FROM drive_trash WHERE user_id = $1 ORDER BY deleted_at DESC"
        )
        .bind::<Text, _>(uid)
        .load::<Row>(&mut conn)
        .unwrap_or_default();

        let items: Vec<TrashItem> = rows.into_iter().map(|r| TrashItem {
            id: r.id,
            user_id: r.user_id,
            bucket: r.bucket,
            path: r.path,
            original_path: r.original_path,
            is_dir: r.is_dir,
            size: r.size as u64,
            deleted_at: r.deleted_at,
            expires_at: r.expires_at,
        }).collect();
        return Ok(Json(items));
    }
    Ok(Json(vec![]))
}

pub async fn trash_file(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<DeleteFileBody>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    let drive = get_drive(&state)?;
    let scope = req.scope.unwrap_or_default();
    let uid = req.user_id.as_deref().map(|s| s.to_string()).unwrap_or_else(|| get_user_id(&user));
    let bucket = resolve_bucket(&state, req.bucket.as_deref(), &scope, Some(uid.as_str()), None)?;
    let prefix = resolve_scope_prefix(&scope, &uid);
    let key = format!("{prefix}{}", normalize_path(&req.path));

    let data = match drive.get_object(&bucket, &key).await {
        Ok(d) => d,
        Err(e) => return Err(err(StatusCode::NOT_FOUND, &format!("File not found: {e}"))),
    };

    let trash_key = format!(".trash/{}/{}", uid, normalize_path(&req.path));
    let data_len = data.len() as i64;
    drive
        .put_object(&bucket, &trash_key, data, None)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("Trash move failed: {e}")))?;

    let _ = drive.delete_object(&bucket, &key).await;

    if let Ok(mut conn) = state.conn.get() {
        let _ = diesel::sql_query(
            "INSERT INTO drive_trash (user_id, bucket, path, original_path, size) VALUES ($1, $2, $3, $4, $5)"
        )
        .bind::<Text, _>(uid)
        .bind::<Text, _>(&bucket)
        .bind::<Text, _>(&trash_key)
        .bind::<Text, _>(&key)
        .bind::<BigInt, _>(data_len)
        .execute(&mut conn);
    }

    info!("File moved to trash: {key} -> {trash_key}");
    Ok(Json(SuccessResponse { success: true }))
}

pub async fn restore_trash(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<RestoreTrashBody>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    let drive = get_drive(&state)?;
    let uid = req.user_id.as_deref().map(|s| s.to_string()).unwrap_or_else(|| get_user_id(&user));

    if let Ok(mut conn) = state.conn.get() {
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type = Text)]
            bucket: String,
            #[diesel(sql_type = Text)]
            path: String,
            #[diesel(sql_type = Text)]
            original_path: String,
        }
        let rows = diesel::sql_query(
            "SELECT bucket, path, original_path FROM drive_trash WHERE id::text = $1 AND user_id = $2"
        )
        .bind::<Text, _>(&req.id)
        .bind::<Text, _>(&uid)
        .load::<Row>(&mut conn)
        .unwrap_or_default();

        if let Some(row) = rows.into_iter().next() {
            if let Ok(data) = drive.get_object(&row.bucket, &row.path).await {
                let _ = drive.put_object(&row.bucket, &row.original_path, data, None).await;
                let _ = drive.delete_object(&row.bucket, &row.path).await;
            }
            let _ = diesel::sql_query("DELETE FROM drive_trash WHERE id::text = $1 AND user_id = $2")
                .bind::<Text, _>(&req.id)
                .bind::<Text, _>(&uid)
                .execute(&mut conn);
        }
    }

    Ok(Json(SuccessResponse { success: true }))
}

pub async fn empty_trash(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<EmptyTrashBody>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    let drive = get_drive(&state)?;
    let uid = req.user_id.as_deref().map(|s| s.to_string()).unwrap_or_else(|| get_user_id(&user));

    if let Ok(mut conn) = state.conn.get() {
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type = Text)]
            bucket: String,
            #[diesel(sql_type = Text)]
            path: String,
        }
        let rows = diesel::sql_query(
            "SELECT bucket, path FROM drive_trash WHERE user_id = $1"
        )
        .bind::<Text, _>(&uid)
        .load::<Row>(&mut conn)
        .unwrap_or_default();

        for row in &rows {
            let _ = drive.delete_object(&row.bucket, &row.path).await;
        }

        let _ = diesel::sql_query("DELETE FROM drive_trash WHERE user_id = $1")
            .bind::<Text, _>(&uid)
            .execute(&mut conn);
    }

    Ok(Json(SuccessResponse { success: true }))
}

pub async fn upload_file_binary(
    State(state): State<Arc<AppState>>,
    body: String,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    let drive = get_drive(&state)?;

    let data = body.into_bytes();
    let bucket = state.bucket_name.clone();
    let key = format!("uploads/{}", uuid::Uuid::new_v4());

    drive
        .put_object(&bucket, &key, data, None)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("Upload failed: {e}")))?;

    Ok(Json(SuccessResponse { success: true }))
}

#[derive(serde::Deserialize)]
pub struct AIChatBody {
    pub message: String,
    pub file_path: Option<String>,
    pub bucket: Option<String>,
    pub scope: Option<FileScope>,
    pub user_id: Option<String>,
}

#[derive(serde::Serialize)]
pub struct AIChatResponse {
    pub reply: String,
}

pub async fn ai_chat_handler(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<AIChatBody>,
) -> Result<Json<AIChatResponse>, (StatusCode, Json<serde_json::Value>)> {
    let provider = state
        .llm_provider
        .as_ref()
        .ok_or_else(|| err(StatusCode::SERVICE_UNAVAILABLE, "LLM provider not configured"))?;

    let drive = get_drive(&state)?;
    let scope = req.scope.unwrap_or_default();
    let uid = req.user_id.as_deref().map(|s| s.to_string()).unwrap_or_else(|| get_user_id(&user));
    let bucket = resolve_bucket(&state, req.bucket.as_deref(), &scope, Some(uid.as_str()), None)?;

    let mut files_context = String::new();
    if let Ok(objects) = drive.list_objects_with_metadata(&bucket, None).await {
        files_context.push_str("Files currently in this storage:\n");
        for obj in objects.iter().take(20) {
            files_context.push_str(&format!("- Name: {}, Size: {} bytes\n", obj.key, obj.size));
        }
    }

    let mut file_content_context = String::new();
    if let Some(ref path) = req.file_path {
        let prefix = resolve_scope_prefix(&scope, &uid);
        let key = format!("{prefix}{}", normalize_path(path));
        file_content_context.push_str(&format!("\nActive File Selected: {}\n", path));

        let ext = path.split('.').next_back().unwrap_or("").to_lowercase();
        if ext == "txt" || ext == "md" || ext == "json" || ext == "csv" {
            if let Ok(data) = drive.get_object(&bucket, &key).await {
                if data.len() < 10240 {
                    if let Ok(text) = String::from_utf8(data) {
                        file_content_context.push_str("File content preview:\n```\n");
                        file_content_context.push_str(&text.chars().take(2000).collect::<String>());
                        file_content_context.push_str("\n```\n");
                    }
                }
            }
        }
    }

    let prompt = format!(
        "You are the GeneralBots Drive AI Assistant. You have access to the user's file storage metadata and contents.\n\n\
        CONTEXT:\n\
        {}\n\
        {}\n\n\
        USER REQUEST: {}\n\n\
        Respond to the user request. Keep it helpful, precise, and professional. Translate/explain/classify files if requested.",
        files_context,
        file_content_context,
        req.message
    );

    let reply = provider
        .generate_simple(&prompt)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("AI generation error: {e}")))?;

    Ok(Json(AIChatResponse { reply }))
}
