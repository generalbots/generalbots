//! Raw-body upload endpoint (`POST /api/files/upload-binary`).
//!
//! Split from [`super::list_trash`] so the trash handler stays inside the file
//! size budget (#2132). The body carries no filename: the bytes decide the
//! type and the shared upload policy still refuses executables, markup and
//! unknown payloads.

use super::*;
use botsecurity_core::upload_policy::{self, UploadClass};

pub async fn upload_file_binary(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    body: axum::body::Body,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    let drive = get_drive(&state)?;

    // #2132 — cap before buffering: `to_bytes` never allocates past the limit,
    // so an oversized body cannot exhaust memory, and the declared
    // `Content-Length` is refused up front when it is already over budget.
    if let Err(error) = upload_policy::check_declared_length(UploadClass::Drive, &headers) {
        botlog::safe_warn!("binary upload refused before buffering: {}", error.log_detail());
        return Err(upload_policy::rejection_json(&error, &headers));
    }
    let max = upload_policy::max_bytes(UploadClass::Drive);
    let data = axum::body::to_bytes(body, max).await.map_err(|_| {
        let error = upload_policy::UploadError::TooLarge { max };
        upload_policy::rejection_json(&error, &headers)
    })?;

    // This endpoint carries no filename: the bytes decide the type, and the
    // validator still refuses executables, markup and unknown payloads.
    let detected = botsecurity_core::file_validation::detect_mime_type(&data);
    let extension = detected
        .as_deref()
        .and_then(upload_policy::extension_for_mime)
        .unwrap_or("bin");
    let filename = format!("upload.{extension}");
    let upload = match upload_policy::validate_upload(
        UploadClass::Drive,
        &filename,
        detected.as_deref().unwrap_or_default(),
        &data,
    ) {
        Ok(upload) => upload,
        Err(error) => {
            botlog::safe_warn!("binary upload refused: {}", error.log_detail());
            return Err(upload_policy::rejection_json(&error, &headers));
        }
    };

    let bucket = state.bucket_name.clone();
    let key = format!("uploads/{}.{}", uuid::Uuid::new_v4(), upload.extension);

    drive
        .put_object(&bucket, &key, data.to_vec(), None)
        .await
        .map_err(|e| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Upload failed: {e}"),
            )
        })?;

    Ok(Json(SuccessResponse { success: true }))
}
