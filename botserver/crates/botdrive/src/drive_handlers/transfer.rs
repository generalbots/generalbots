//! Drive transfer handlers: write (`/api/files/write`), read
//! (`/api/files/read`) and the three download variants (base64, attachment,
//! inline).
//!
//! Split from [`super::list_files`] to keep both files inside the size budget
//! (#2132). Every write goes through the shared upload policy first, and
//! inline serving is limited to passive media.

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
            attachment_disposition(&file_name),
        ),
        (axum::http::header::CONTENT_LENGTH, data.len().to_string()),
        (
            axum::http::header::X_CONTENT_TYPE_OPTIONS,
            "nosniff".to_string(),
        ),
    ];
    Ok((headers, data).into_response())
}

/// `Content-Disposition` value with the client name stripped of characters
/// that would break the quoted string (a `"` closes it early, CR/LF inject a
/// header). #2132.
fn attachment_disposition(file_name: &str) -> String {
    let safe: String = file_name
        .chars()
        .filter(|c| !matches!(c, '"' | '\\' | '\r' | '\n' | '\0'))
        .collect();
    format!("attachment; filename=\"{safe}\"")
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

    // #2132 — inline serving is only safe for types the browser renders as
    // passive media. HTML, SVG and scripts are downgraded to an opaque
    // attachment so an upload can never execute on the suite origin.
    let inline_ok = matches!(
        mime,
        "image/png" | "image/jpeg" | "image/gif" | "image/webp" | "application/pdf"
    );
    let content_type = if inline_ok {
        mime
    } else {
        "application/octet-stream"
    };

    let mut response = axum::response::Response::new(axum::body::Body::from(data));
    let headers = response.headers_mut();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static(content_type),
    );
    headers.insert(
        axum::http::header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static("no-store"),
    );
    headers.insert(
        axum::http::header::X_CONTENT_TYPE_OPTIONS,
        axum::http::HeaderValue::from_static("nosniff"),
    );
    if !inline_ok {
        if let Ok(value) = axum::http::HeaderValue::from_str(&attachment_disposition(&file_name)) {
            headers.insert(axum::http::header::CONTENT_DISPOSITION, value);
        }
    }
    Ok(response)
}
