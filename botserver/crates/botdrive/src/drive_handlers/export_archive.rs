use super::*;

use serde::Deserialize;

use std::io::Write;

use axum::http::{header, HeaderValue};
use axum::response::{IntoResponse, Response};

#[derive(Debug, Deserialize)]
pub struct ExportArchiveBody {
    /// Individual files (paths relative to the caller's scope).
    pub paths: Option<Vec<String>>,
    /// Or a whole folder, exported recursively.
    pub folder: Option<String>,
    pub bucket: Option<String>,
    pub scope: Option<FileScope>,
    pub user_id: Option<String>,
}

/// `POST /api/files/export` — one zip archive for a selection of files or a
/// folder (#1639). The archive keeps the folder structure below the requested
/// folder and never escapes the caller's scope prefix.
pub async fn export_archive(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<ExportArchiveBody>,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let drive = get_drive(&state)?;
    let scope = req.scope.unwrap_or_default();
    let uid = effective_uid(&state, &user, req.user_id.as_deref());
    let bucket =
        resolve_bucket(&state, req.bucket.as_deref(), &scope, Some(uid.as_str()), None, &user)?;
    let prefix = resolve_scope_prefix(&scope, &uid);

    let mut entries: Vec<String> = Vec::new();
    if let Some(folder) = req.folder.as_deref() {
        let folder = normalize_path(folder);
        if folder.is_empty() {
            return Err(err(StatusCode::BAD_REQUEST, "folder is empty"));
        }
        let folder_prefix = format!("{prefix}{folder}/");
        let keys = drive
            .list_objects(&bucket, Some(&folder_prefix))
            .await
            .map_err(|e| err(StatusCode::BAD_REQUEST, &format!("Folder not found: {e}")))?;
        for key in keys {
            let relative = key.strip_prefix(&folder_prefix).unwrap_or(&key);
            if relative.is_empty() || relative.ends_with('/') {
                continue;
            }
            entries.push(format!("{folder}/{relative}"));
        }
    }
    for path in req.paths.unwrap_or_default() {
        let path = normalize_path(&path);
        if !path.is_empty() && !path.ends_with('/') {
            entries.push(path);
        }
    }
    entries.sort();
    entries.dedup();
    if entries.is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "Nothing to export"));
    }

    let mut cursor = std::io::Cursor::new(Vec::new());
    {
        let mut archive = zip::ZipWriter::new(&mut cursor);
        for entry in &entries {
            let key = format!("{prefix}{entry}");
            let bytes = match drive.get_object(&bucket, &key).await {
                Ok(bytes) => bytes,
                Err(e) => {
                    warn!("export: skipping {entry}: {e}");
                    continue;
                }
            };
            let options = zip::write::FileOptions::<()>::default();
            archive
                .start_file(entry.replace('\\', "/"), options)
                .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("zip: {e}")))?;
            archive
                .write_all(&bytes)
                .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("zip: {e}")))?;
        }
        archive
            .finish()
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("zip: {e}")))?;
    }
    let archive_bytes = cursor.into_inner();
    botlog::safe_info!(
        "exported {} entries from {bucket} as a {} byte archive",
        entries.len(),
        archive_bytes.len()
    );

    let mut response = archive_bytes.into_response();
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("application/zip"));
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=\"general-bots-export.zip\""),
    );
    Ok(response)
}
