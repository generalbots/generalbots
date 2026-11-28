//! Raw-body upload endpoint (`POST /api/files/upload-binary`).
//!
//! Split from [`super::list_trash`] so the trash handler stays inside the file
//! size budget (#2132). The body carries no filename: the bytes decide the
//! type and the shared upload policy still refuses executables, markup and
//! unknown payloads.

use super::*;
use botsecurity_core::upload_hostile::macro_strip;
use botsecurity_core::upload_policy::{self, UploadClass};

pub async fn upload_file_binary(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    body: axum::body::Body,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let drive = get_drive(&state)?;

    // #2132 — cap before buffering: `to_bytes` never allocates past the limit,
    // so an oversized body cannot exhaust memory, and the declared
    // `Content-Length` is refused up front when it is already over budget.
    if let Err(error) = upload_policy::check_declared_length(UploadClass::Drive, &headers) {
        botlog::safe_warn!(
            "binary upload refused before buffering: {}",
            error.log_detail()
        );
        return Err(upload_policy::rejection_json(&error, &headers));
    }
    let max = upload_policy::max_bytes(UploadClass::Drive);
    let data = axum::body::to_bytes(body, max).await.map_err(|_| {
        let error = upload_policy::UploadError::TooLarge { max };
        upload_policy::rejection_json(&error, &headers)
    })?;

    // #2927 — this endpoint carries no filename, so the bytes decide the
    // type. Sanitize Office packages before the shared gate and store only the
    // macro-free copy, with a machine-readable notice in the response.
    let prepared = macro_strip::prepare_office_upload(&data).map_err(|error| {
        botlog::safe_warn!("binary upload sanitize refused: {}", error.log_detail());
        upload_policy::rejection_json(&error, &headers)
    })?;
    let macro_strip::PreparedUpload {
        bytes: prepared_bytes,
        notice,
    } = prepared;
    let effective = prepared_bytes.as_ref();
    if let Some(notice) = &notice {
        botlog::safe_warn!(
            "binary upload removed {} executable part(s) before storing",
            notice.count
        );
    }
    let detected = botsecurity_core::file_validation::detect_mime_type(effective);
    let extension = detected
        .as_deref()
        .and_then(upload_policy::extension_for_mime)
        .unwrap_or("bin");
    let filename = format!("upload.{extension}");
    let upload = match upload_policy::validate_upload(
        UploadClass::Drive,
        &filename,
        detected.as_deref().unwrap_or(""),
        effective,
    ) {
        Ok(upload) => upload,
        Err(error) => {
            botlog::safe_warn!("binary upload refused: {}", error.log_detail());
            return Err(upload_policy::rejection_json(&error, &headers));
        }
    };
    let stored = prepared_bytes.into_owned();

    let bucket = state.bucket_name.clone();
    let key = format!("uploads/{}.{}", uuid::Uuid::new_v4(), upload.extension);

    drive
        .put_object(&bucket, &key, stored, None)
        .await
        .map_err(|e| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Upload failed: {e}"),
            )
        })?;
    invalidate_quota();

    Ok(drive_upload_response(notice))
}
