use super::*;
use botsecurity_core::upload_policy::{self, UploadClass};

/// Local principal for single-user desktop installs (`no_security=true`).
/// Normal deployments always answer `None`, so a missing auth extension can
/// never silently become an authenticated request there.
fn desktop_local_fallback() -> Option<AuthenticatedUser> {
    let no_security = botcore::product::PRODUCT_CONFIG
        .read()
        .map(|product| product.no_security)
        .unwrap_or(false);
    no_security.then(AuthenticatedUser::desktop_local)
}

pub async fn list_files(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<ListFilesParams>,
) -> Result<Json<Vec<FileListItem>>, (StatusCode, Json<serde_json::Value>)> {
    // #1801 — one file API over local FS and MinIO. A desktop request sets
    // backend=local (or passes a `local:<root>` bucket) and never reaches S3.
    if crate::local_fs::is_local_backend(params.backend.as_deref(), params.bucket.as_deref()) {
        let root =
            crate::local_fs::requested_root(params.root.as_deref(), params.bucket.as_deref());
        let items = crate::local_fs::list_local_files(root, params.path.as_deref())
            .await
            .map_err(|e| err(StatusCode::BAD_REQUEST, &e))?;
        return Ok(Json(items));
    }
    let drive = get_drive(&state)?;
    let scope = params.scope.unwrap_or_default();
    let uid = effective_uid(&state, &user, params.user_id.as_deref());
    let bucket = resolve_bucket(
        &state,
        params.bucket.as_deref(),
        &scope,
        Some(uid.as_str()),
        None,
        &user,
    )?;
    let prefix = resolve_scope_prefix(&scope, &uid);
    let sub_path = normalize_path(params.path.as_deref().unwrap_or(""));
    let full_prefix = if sub_path.is_empty() {
        prefix
    } else {
        format!("{}/", scoped_key(&prefix, &sub_path).trim_end_matches('/'))
    };

    // One LIST carries key + size, so the second, identical listing was pure
    // waste: `list_objects` is just `list_objects_with_metadata` mapped to
    // keys, meaning every folder navigation hit MinIO twice for the same
    // prefix. `create_bucket_if_not_exists` moved to the error path too — the
    // HEAD it performs on every listing only matters when the bucket is
    // actually missing (a user's personal bucket), and paying for it on every
    // request starved the CPU the handler needs.
    let objects = match drive
        .list_objects_with_metadata(&bucket, Some(&full_prefix))
        .await
    {
        Ok(objects) => objects,
        Err(list_err) => {
            // Ensures a personal bucket that has never been written to still
            // renders as an empty folder instead of erroring.
            botlog::safe_warn!("drive: listing '{full_prefix}' failed ({list_err}); ensuring bucket '{bucket}' exists");
            let _ = drive.create_bucket_if_not_exists(&bucket).await;
            drive
                .list_objects_with_metadata(&bucket, Some(&full_prefix))
                .await
                .map_err(|e| {
                    err(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        &format!("Failed to list objects: {e}"),
                    )
                })?
        }
    };

    // Internal prefixes are not user files: versions and trash would otherwise
    // appear as folders in the listing.
    let keys: Vec<String> = objects
        .iter()
        .filter(|o| !o.key.starts_with(".versions/") && !o.key.starts_with(".trash/"))
        .map(|o| o.key.clone())
        .collect();
    let meta_map: HashMap<String, u64> = objects.into_iter().map(|o| (o.key, o.size)).collect();

    let items = build_file_list_items(&full_prefix, &keys, &meta_map);
    Ok(Json(items))
}
pub async fn upload_file_to_drive(
    State(state): State<Arc<AppState>>,
    user: Option<Extension<AuthenticatedUser>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<WriteFileBody>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    // The global auth middleware always inserts a principal, including the
    // local desktop one when `no_security=true`. The fallback below keeps the
    // endpoint itself safe if it is ever mounted without that middleware:
    // only a single-user desktop profile may write without a bearer identity.
    let user = match user {
        Some(Extension(user)) => user,
        None => match desktop_local_fallback() {
            Some(user) => user,
            None => return Err(err(StatusCode::UNAUTHORIZED, "Authentication required")),
        },
    };
    // Defence in depth: a normal deployment must never reach the write path
    // as the anonymous fallback, even if the route is accidentally made
    // public. Only the explicit local desktop principal may write token-less.
    if !user.is_authenticated() && !user.metadata.contains_key("desktop_local") {
        return Err(err(StatusCode::UNAUTHORIZED, "Authentication required"));
    }
    let drive = get_drive(&state)?;
    let scope = req.scope.unwrap_or_default();
    let uid = effective_uid(&state, &user, req.user_id.as_deref());
    let bucket = resolve_bucket(
        &state,
        req.bucket.as_deref(),
        &scope,
        Some(uid.as_str()),
        None,
        &user,
    )?;
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

    // #2132 — every write is an upload: the bytes must agree with the
    // extension they are stored under, the filename must be safe and the size
    // must fit the endpoint budget.
    if let Err(error) = upload_policy::check_declared_length(UploadClass::Drive, &headers) {
        botlog::safe_warn!(
            "drive write refused before buffering: {}",
            error.log_detail()
        );
        return Err(upload_policy::rejection_json(&error, &headers));
    }
    let upload = match upload_policy::validate_upload(UploadClass::Drive, &req.path, "", &data) {
        Ok(upload) => upload,
        Err(error) => {
            botlog::safe_warn!(
                "drive write refused for '{}': {}",
                req.path,
                error.log_detail()
            );
            return Err(upload_policy::rejection_json(&error, &headers));
        }
    };

    // The client name is only a hint: the stored key keeps the sanitized
    // basename and the (already normalized) parent folder.
    let normalized = normalize_path(&req.path);
    let parent = normalized
        .rsplit_once('/')
        .map(|(dir, _)| dir)
        .unwrap_or_default();
    let stored_path = if parent.is_empty() {
        upload.filename.clone()
    } else {
        format!("{parent}/{}", upload.filename)
    };
    let key = scoped_key(&prefix, &stored_path);
    // GB7-14 — the previous revision becomes restorable history before the
    // overwrite. The snapshot never blocks the write.
    snapshot_before_write(drive, &bucket, &key, &scope, &uid).await;
    // A personal bucket may not exist yet (the local/email user id produces
    // user-<slug>, created on first use). Mirrors list_files: create on
    // NoSuchBucket and retry once instead of failing the upload (#1709).
    let mut created = false;
    loop {
        match drive.put_object(&bucket, &key, data.clone(), None).await {
            Ok(()) => break,
            Err(e) if !created && e.contains("NoSuchBucket") => {
                let _ = drive.create_bucket_if_not_exists(&bucket).await;
                created = true;
            }
            Err(e) => {
                return Err(err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    &format!("Upload failed: {e}"),
                ))
            }
        }
    }

    botlog::safe_info!("File uploaded: {key} to bucket {bucket}");
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
    let data = if crate::local_fs::is_local_backend(req.backend.as_deref(), req.bucket.as_deref()) {
        let root = crate::local_fs::requested_root(req.root.as_deref(), req.bucket.as_deref());
        crate::local_fs::read_local_file(root, &req.path)
            .await
            .map_err(|e| err(StatusCode::NOT_FOUND, &e))?
    } else {
        let drive = get_drive(&state)?;
        let scope = req.scope.unwrap_or_default();
        let uid = effective_uid(&state, &user, req.user_id.as_deref());
        let bucket = resolve_bucket(
            &state,
            req.bucket.as_deref(),
            &scope,
            Some(uid.as_str()),
            None,
            &user,
        )?;
        let prefix = resolve_scope_prefix(&scope, &uid);
        let key = scoped_key(&prefix, &req.path);
        drive
            .get_object(&bucket, &key)
            .await
            .map_err(|e| err(StatusCode::NOT_FOUND, &format!("File not found: {e}")))?
    };

    let content = String::from_utf8_lossy(&data).to_string();
    Ok(Json(serde_json::json!({ "content": content })))
}
pub async fn download_file(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<DownloadFileBody>,
) -> Result<Json<DownloadFileResponse>, (StatusCode, Json<serde_json::Value>)> {
    let data = if crate::local_fs::is_local_backend(req.backend.as_deref(), req.bucket.as_deref()) {
        let root = crate::local_fs::requested_root(req.root.as_deref(), req.bucket.as_deref());
        crate::local_fs::read_local_file(root, &req.path)
            .await
            .map_err(|e| err(StatusCode::NOT_FOUND, &e))?
    } else {
        let drive = get_drive(&state)?;
        let scope = req.scope.unwrap_or_default();
        let uid = effective_uid(&state, &user, req.user_id.as_deref());
        let bucket = resolve_bucket(
            &state,
            req.bucket.as_deref(),
            &scope,
            Some(uid.as_str()),
            None,
            &user,
        )?;
        let prefix = resolve_scope_prefix(&scope, &uid);
        let key = scoped_key(&prefix, &req.path);
        drive
            .get_object(&bucket, &key)
            .await
            .map_err(|e| err(StatusCode::NOT_FOUND, &format!("File not found: {e}")))?
    };

    let content = base64::engine::general_purpose::STANDARD.encode(&data);
    let file_name = req
        .path
        .rsplit('/')
        .next()
        .unwrap_or("download")
        .to_string();

    Ok(Json(DownloadFileResponse { content, file_name }))
}
pub async fn download_file_binary(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<DownloadFileBody>,
) -> Result<axum::response::Response, (StatusCode, Json<serde_json::Value>)> {
    let data = if crate::local_fs::is_local_backend(req.backend.as_deref(), req.bucket.as_deref()) {
        let root = crate::local_fs::requested_root(req.root.as_deref(), req.bucket.as_deref());
        crate::local_fs::read_local_file(root, &req.path)
            .await
            .map_err(|e| err(StatusCode::NOT_FOUND, &e))?
    } else {
        let drive = get_drive(&state)?;
        let scope = req.scope.unwrap_or_default();
        let uid = effective_uid(&state, &user, req.user_id.as_deref());
        let bucket = resolve_bucket(
            &state,
            req.bucket.as_deref(),
            &scope,
            Some(uid.as_str()),
            None,
            &user,
        )?;
        let prefix = resolve_scope_prefix(&scope, &uid);
        let key = scoped_key(&prefix, &req.path);
        drive
            .get_object(&bucket, &key)
            .await
            .map_err(|e| err(StatusCode::NOT_FOUND, &format!("File not found: {e}")))?
    };

    let file_name = req
        .path
        .rsplit('/')
        .next()
        .unwrap_or("download")
        .to_string();
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
/// #1306 — raw file bytes served INLINE (no Content-Disposition attachment),
/// so apps like the Photo Editor can load Drive images into a canvas with
/// `fetch` + `createObjectURL` instead of base64 round-trips.
pub async fn download_file_inline(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<DownloadFileBody>,
) -> Result<axum::response::Response, (StatusCode, Json<serde_json::Value>)> {
    let data = if crate::local_fs::is_local_backend(req.backend.as_deref(), req.bucket.as_deref()) {
        let root = crate::local_fs::requested_root(req.root.as_deref(), req.bucket.as_deref());
        crate::local_fs::read_local_file(root, &req.path)
            .await
            .map_err(|e| err(StatusCode::NOT_FOUND, &e))?
    } else {
        let drive = get_drive(&state)?;
        let scope = req.scope.unwrap_or_default();
        let uid = effective_uid(&state, &user, req.user_id.as_deref());
        let bucket = resolve_bucket(
            &state,
            req.bucket.as_deref(),
            &scope,
            Some(uid.as_str()),
            None,
            &user,
        )?;
        let prefix = resolve_scope_prefix(&scope, &uid);
        let key = scoped_key(&prefix, &req.path);
        drive
            .get_object(&bucket, &key)
            .await
            .map_err(|e| err(StatusCode::NOT_FOUND, &format!("File not found: {e}")))?
    };

    let file_name = req
        .path
        .rsplit('/')
        .next()
        .unwrap_or("download")
        .to_string();
    let mime = guess_mime(&file_name);

    use axum::response::IntoResponse;
    let headers = [
        (axum::http::header::CONTENT_TYPE, mime.to_string()),
        (axum::http::header::CONTENT_LENGTH, data.len().to_string()),
        (axum::http::header::CACHE_CONTROL, "no-store".to_string()),
    ];
    Ok((headers, data).into_response())
}
pub(super) fn guess_mime(file_name: &str) -> &'static str {
    let ext = file_name
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
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
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "doc" => "application/msword",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "xls" => "application/vnd.ms-excel",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "ppt" => "application/vnd.ms-powerpoint",
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
    let uid = effective_uid(&state, &user, req.user_id.as_deref());
    let bucket = resolve_bucket(
        &state,
        req.bucket.as_deref(),
        &scope,
        Some(uid.as_str()),
        None,
        &user,
    )?;
    let prefix = resolve_scope_prefix(&scope, &uid);
    let key = scoped_key(&prefix, &req.path);

    drive.delete_object(&bucket, &key).await.map_err(|e| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("Delete failed: {e}"),
        )
    })?;

    botlog::safe_info!("File deleted: {key} from bucket {bucket}");
    Ok(Json(SuccessResponse { success: true }))
}
pub async fn create_folder(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<CreateFolderBody>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    let drive = get_drive(&state)?;
    let scope = req.scope.unwrap_or_default();
    let uid = effective_uid(&state, &user, req.user_id.as_deref());
    let bucket = resolve_bucket(
        &state,
        req.bucket.as_deref(),
        &scope,
        Some(uid.as_str()),
        None,
        &user,
    )?;
    let prefix = resolve_scope_prefix(&scope, &uid);
    let parent = normalize_path(&req.path);
    let folder_name = normalize_path(&req.name);
    let key = if parent.is_empty() {
        format!(
            "{}/",
            scoped_key(&prefix, &folder_name).trim_end_matches('/')
        )
    } else {
        format!(
            "{}/",
            scoped_key(&prefix, &format!("{parent}/{folder_name}")).trim_end_matches('/')
        )
    };

    drive
        .put_object(&bucket, &key, vec![], None)
        .await
        .map_err(|e| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Create folder failed: {e}"),
            )
        })?;

    botlog::safe_info!("Folder created: {key} in bucket {bucket}");
    Ok(Json(SuccessResponse { success: true }))
}
pub async fn copy_file(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<CopyFileBody>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    let drive = get_drive(&state)?;
    let scope = req.scope.unwrap_or_default();
    let uid = effective_uid(&state, &user, req.user_id.as_deref());
    let prefix = resolve_scope_prefix(&scope, &uid);

    // Scope decides the bucket: a user-scope folder lives in the personal
    // bucket, not in the workspace bucket the client sent (folder copies used
    // to 404 because the listing ran against the wrong store).
    let src_bucket = resolve_bucket(
        &state,
        req.source_bucket.as_deref(),
        &scope,
        Some(uid.as_str()),
        None,
        &user,
    )?;
    let dest_bucket = resolve_bucket(
        &state,
        req.dest_bucket.as_deref(),
        &scope,
        Some(uid.as_str()),
        None,
        &user,
    )?;
    let from_key = scoped_key(&prefix, &req.source_path);
    let to_key = scoped_key(&prefix, &req.dest_path);

    // A folder copy must clone every child; the old code copied only the
    // folder marker key (a zero-byte object) and left the contents behind.
    // Listings omit the trailing slash, so probe the marker/children too.
    let source_is_folder = req.source_path.ends_with('/') || {
        let marker = format!("{}/", from_key.trim_end_matches('/'));
        if drive.get_object(&src_bucket, &marker).await.is_ok() {
            true
        } else if drive.get_object(&src_bucket, &from_key).await.is_ok() {
            // A real object wins over a same-named prefix: never turn a file
            // into a folder move/copy.
            false
        } else {
            drive
                .list_objects(&src_bucket, Some(&marker))
                .await
                .map(|keys| !keys.is_empty())
                .unwrap_or(false)
        }
    };
    if source_is_folder {
        let source_prefix = format!("{}/", from_key.trim_end_matches('/'));
        let dest_prefix = format!("{}/", to_key.trim_end_matches('/'));
        let objects = drive
            .list_objects_with_metadata(&src_bucket, Some(&source_prefix))
            .await
            .map_err(|e| {
                err(
                    StatusCode::NOT_FOUND,
                    &format!("Source folder not found: {e}"),
                )
            })?;
        if objects.is_empty() {
            return Err(err(
                StatusCode::NOT_FOUND,
                "Source folder is empty or missing",
            ));
        }
        for object in objects {
            let relative = object
                .key
                .strip_prefix(&source_prefix)
                .unwrap_or(object.key.as_str());
            let data = drive
                .get_object(&src_bucket, &object.key)
                .await
                .map_err(|e| {
                    err(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        &format!("Folder copy read failed: {e}"),
                    )
                })?;
            drive
                .put_object(
                    &dest_bucket,
                    &format!("{dest_prefix}{relative}"),
                    data,
                    None,
                )
                .await
                .map_err(|e| {
                    err(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        &format!("Folder copy write failed: {e}"),
                    )
                })?;
        }
        botlog::safe_info!("Copied folder {from_key} -> {to_key} in bucket {dest_bucket}");
        return Ok(Json(SuccessResponse { success: true }));
    }

    let data = drive
        .get_object(&src_bucket, &from_key)
        .await
        .map_err(|e| {
            err(
                StatusCode::NOT_FOUND,
                &format!("Source file not found: {e}"),
            )
        })?;

    drive
        .put_object(&dest_bucket, &to_key, data, None)
        .await
        .map_err(|e| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Copy failed: {e}"),
            )
        })?;

    botlog::safe_info!("Copied {from_key} -> {to_key} in bucket {dest_bucket}");
    Ok(Json(SuccessResponse { success: true }))
}
