use crate::shared::state::AppState;
use aws_sdk_s3::Client;
use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::io::Write;
use std::sync::Arc;
use tempfile::NamedTempFile;

pub async fn upload_file(
    Path(folder_path): Path<String>,
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut temp_file = NamedTempFile::new().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create temp file: {}", e),
        )
    })?;

    let mut file_name: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("Failed to read multipart field: {}", e),
        )
    })? {
        if let Some(name) = field.file_name() {
            file_name = Some(name.to_string());
        }

        let data = field.bytes().await.map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                format!("Failed to read field data: {}", e),
            )
        })?;

        temp_file.write_all(&data).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to write to temp file: {}", e),
            )
        })?;
    }

    let file_name = file_name.unwrap_or_else(|| "unnamed_file".to_string());
    let temp_file_path = temp_file.into_temp_path();

    let client = state.drive.as_ref().ok_or_else(|| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "S3 client is not initialized".to_string(),
        )
    })?;

    let s3_key = format!("{}/{}", folder_path, file_name);

    match upload_to_s3(client, &state.bucket_name, &s3_key, &temp_file_path).await {
        Ok(_) => {
            let _ = std::fs::remove_file(&temp_file_path);
            Ok((
                StatusCode::OK,
                format!("Uploaded file '{}' to folder '{}'", file_name, folder_path),
            ))
        }
        Err(e) => {
            let _ = std::fs::remove_file(&temp_file_path);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to upload file to S3: {}", e),
            ))
        }
    }
}

async fn upload_to_s3(
    client: &Client,
    bucket: &str,
    key: &str,
    file_path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let data = std::fs::read(file_path)?;
    client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(data.into())
        .send()
        .await?;
    Ok(())
}
