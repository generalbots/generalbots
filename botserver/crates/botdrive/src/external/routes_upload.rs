//! Upload handler — the write half of the external-drive API (#2047).
//!
//! Split from [`super::routes_files`] (reads) and [`super::routes`] (connect
//! lifecycle) so each surface stays small and testable. One entry point for
//! both providers: resolve the caller's branch, prove the connection consented
//! to a write scope, build the provider request and mirror the result into the
//! local listing so it appears in the Drive app immediately.

use super::routes::{caller_branch, err, parse_provider, ApiError, ApiResult};
use crate::external::{
    config, db, engine, files,
    types::{ExternalFile, Provider, SyncError},
    upload::{self, UploadError, UploadRequest, MAX_UPLOAD_BYTES},
};
use axum::{extract::State, http::StatusCode, Extension, Json};
use base64::Engine;
use botcore::shared::state::AppState;
use botsecurity_auth::auth_api::types::AuthenticatedUser;
use botsecurity_core::upload_policy::{self, UploadClass};
use serde::Deserialize;
use std::sync::Arc;

/// Request body. Mirrors the shape `/api/files/write` already uses (base64
/// `content`) so the chat save path can switch target without a new encoder.
#[derive(Debug, Deserialize)]
pub struct UploadBody {
    /// `onedrive` or `gdrive`.
    pub provider: String,
    /// Destination folder relative to the provider root (optional).
    #[serde(default)]
    pub path: Option<String>,
    /// Google Drive parent folder id (optional; takes precedence over `path`
    /// for the metadata parent).
    #[serde(default)]
    pub folder_id: Option<String>,
    /// File name.
    pub name: String,
    /// MIME type; optional.
    #[serde(default)]
    pub mime_type: Option<String>,
    /// Base64 file bytes.
    pub content: String,
    /// Replace this provider file instead of creating one.
    #[serde(default)]
    pub replace_remote_id: Option<String>,
}

fn upload_error(error: UploadError) -> ApiError {
    let status = match error {
        UploadError::NotConfigured => StatusCode::NOT_IMPLEMENTED,
        UploadError::NotConnected => StatusCode::CONFLICT,
        UploadError::MissingWriteScope => StatusCode::FORBIDDEN,
        UploadError::TooLarge(_) => StatusCode::PAYLOAD_TOO_LARGE,
        UploadError::Provider(_) => StatusCode::BAD_GATEWAY,
    };
    (
        status,
        Json(serde_json::json!({
            "error": error.message(),
            "kind": error.kind(),
        })),
    )
}

/// Provider rejections carry their own meaning: an authentication failure is
/// a read-only/missing grant in practice, everything else is upstream.
fn sync_error(provider: Provider, error: SyncError) -> ApiError {
    match error {
        SyncError::Auth(detail) => (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": format!("{} refused the write: {detail}", provider.display_name()),
                "kind": "missing_write_scope",
            })),
        ),
        other => {
            log::warn!(
                "external drive upload to {} failed: {}",
                provider.as_str(),
                other
            );
            (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({
                    "error": other.to_string(),
                    "kind": "provider_error",
                })),
            )
        }
    }
}

fn decode_content(content: &str) -> Result<Vec<u8>, ApiError> {
    let trimmed = content.trim();
    // Accept both the standard and the URL-safe alphabet: the chat encoder and
    // the API clients do not necessarily agree on `+`/`-` and `/`/`_`.
    base64::engine::general_purpose::STANDARD
        .decode(trimmed)
        .or_else(|_| base64::engine::general_purpose::URL_SAFE.decode(trimmed))
        .map_err(|_| {
            err(
                StatusCode::BAD_REQUEST,
                "content must be base64-encoded file bytes",
            )
        })
}

/// `POST /api/external-drives/upload` — save an artifact to a connected drive.
pub async fn upload_file(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    headers: axum::http::HeaderMap,
    Json(body): Json<UploadBody>,
) -> ApiResult<Json<serde_json::Value>> {
    let provider = parse_provider(&body.provider)?;
    if config::client_for(provider).is_none() {
        return Err(upload_error(UploadError::NotConfigured));
    }
    if body.name.trim().is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "name is required"));
    }
    let content = decode_content(&body.content)?;
    if content.len() > MAX_UPLOAD_BYTES {
        return Err(upload_error(UploadError::TooLarge(content.len())));
    }

    // #2132 — the declared type and the bytes must agree with the name before
    // anything leaves for the provider.
    let declared = body.mime_type.as_deref().unwrap_or_default();
    let upload = upload_policy::validate_upload(UploadClass::Drive, &body.name, declared, &content)
        .map_err(|error| {
            log::warn!("external drive upload refused: {}", error.log_detail());
            upload_policy::rejection_json(&error, &headers)
        })?;

    let branch_id = caller_branch(&state, &user)?;
    let (connection, granted) = {
        let mut conn = db::conn(&state.conn).map_err(|e| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("database unavailable: {e}"),
            )
        })?;
        let connection = db::get_connection(&mut conn, branch_id, provider)
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e))?
            .ok_or_else(|| upload_error(UploadError::NotConnected))?;
        if connection.status == "disconnected" {
            return Err(upload_error(UploadError::NotConnected));
        }
        let granted = connection.scopes.clone().unwrap_or_default();
        (connection, granted)
    };
    if !upload::allows_write(provider, &granted) {
        return Err(upload_error(UploadError::MissingWriteScope));
    }

    let request = UploadRequest {
        provider,
        path: body.path.unwrap_or_default(),
        name: upload.filename,
        mime_type: upload.mime,
        content,
        replace_remote_id: body.replace_remote_id,
        parent_id: body.folder_id,
    };
    let prepared = upload::build_target(&request, &granted).map_err(upload_error)?;

    let http = engine::http_client()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let token = engine::connection_access_token(&http, &state.conn, branch_id, &connection)
        .await
        .map_err(|e| sync_error(provider, e))?;
    let uploaded = upload::send(&http, provider, &token, &request.name, prepared)
        .await
        .map_err(|e| sync_error(provider, e))?;

    // Mirror the new file so the Drive app lists it without waiting for a sync
    // pass. A mirror failure must not fail the upload: the bytes are already
    // stored at the provider.
    let mirror = ExternalFile {
        remote_id: uploaded.remote_id.clone(),
        name: uploaded.name.clone(),
        path: upload::mirror_path(&request.path, &uploaded.name),
        mime_type: Some(request.mime_type.clone()).filter(|mime| !mime.is_empty()),
        size_bytes: uploaded.bytes as i64,
        modified_at: Some(chrono::Utc::now()),
        is_folder: false,
        revision: None,
        parent_remote_id: request.parent_id.clone(),
    };
    if let Ok(mut conn) = db::conn(&state.conn) {
        if let Err(e) = files::upsert_file(&mut conn, branch_id, provider, &mirror) {
            log::warn!(
                "external drive upload to {} stored but the mirror failed: {e}",
                provider.as_str()
            );
        }
    }

    Ok(Json(serde_json::json!({
        "provider": uploaded.provider,
        "remote_id": uploaded.remote_id,
        "name": uploaded.name,
        "web_view_link": uploaded.web_view_link,
        "bytes": uploaded.bytes,
        "path": mirror.path,
    })))
}
