use crate::config::DriveConfig;
use crate::shared::state::AppState;
use actix_multipart::Multipart;
use actix_web::web;
use actix_web::{post, HttpResponse};
use opendal::Operator;
use std::io::Write;
use tempfile::NamedTempFile;
use tokio_stream::StreamExt as TokioStreamExt;

#[post("/files/upload/{folder_path}")]
pub async fn upload_file(
    folder_path: web::Path<String>,
    mut payload: Multipart,
    state: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    let folder_path = folder_path.into_inner();
    let mut temp_file = NamedTempFile::new().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to create temp file: {}", e))
    })?;

    let mut file_name: Option<String> = None;
    while let Some(mut field) = payload.try_next().await? {
        if let Some(disposition) = field.content_disposition() {
            if let Some(name) = disposition.get_filename() {
                file_name = Some(name.to_string());
            }
        }

        while let Some(chunk) = field.try_next().await? {
            temp_file.write_all(&chunk).map_err(|e| {
                actix_web::error::ErrorInternalServerError(format!(
                    "Failed to write to temp file: {}",
                    e
                ))
            })?;
        }
    }

    let file_name = file_name.unwrap_or_else(|| "unnamed_file".to_string());
    let temp_file_path = temp_file.into_temp_path();

    let op = state.get_ref().s3_operator.as_ref().ok_or_else(|| {
        actix_web::error::ErrorInternalServerError("S3 operator is not initialized")
    })?;

    let s3_key = format!("{}/{}", folder_path, file_name);

    match upload_to_s3(op, &s3_key, &temp_file_path).await {
        Ok(_) => {
            let _ = std::fs::remove_file(&temp_file_path);
            Ok(HttpResponse::Ok().body(format!(
                "Uploaded file '{}' to folder '{}'",
                file_name, folder_path
            )))
        }
        Err(e) => {
            let _ = std::fs::remove_file(&temp_file_path);
            Err(actix_web::error::ErrorInternalServerError(format!(
                "Failed to upload file to S3: {}",
                e
            )))
        }
    }
}

pub async fn init_drive(config: &DriveConfig) -> Result<Operator, Box<dyn std::error::Error>> {
    use opendal::services::S3;
    use opendal::Operator;
    let client = Operator::new(
        S3::default()
            .root("/")
            .endpoint(&config.server)
            .access_key_id(&config.access_key)
            .secret_access_key(&config.secret_key),
    )?
    .finish();

    Ok(client)
}

async fn upload_to_s3(
    op: &Operator,
    key: &str,
    file_path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let data = std::fs::read(file_path)?;
    op.write(key, data).await?;
    Ok(())
}
