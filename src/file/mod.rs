use crate::config::DriveConfig;
use crate::shared::state::AppState;
use actix_multipart::Multipart;
use actix_web::web;
use actix_web::{post, HttpResponse};
use aws_sdk_s3::{Client as S3Client, config::Builder as S3ConfigBuilder};
use aws_config::BehaviorVersion;
use std::io::Write;
use tempfile::NamedTempFile;
use tokio_stream::StreamExt as TokioStreamExt;
// Removed unused import

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

    let client = state.get_ref().drive.as_ref().ok_or_else(|| {
        actix_web::error::ErrorInternalServerError("S3 client is not initialized")
    })?;

    let s3_key = format!("{}/{}", folder_path, file_name);

    match upload_to_s3(client, &state.get_ref().bucket_name, &s3_key, &temp_file_path).await {
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

pub async fn aws_s3_bucket_delete(
    bucket: &str,
    endpoint: &str,
    access_key: &str,
    secret_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = aws_config::defaults(BehaviorVersion::latest())
        .endpoint_url(endpoint)
        .region("auto")
        .credentials_provider(
            aws_sdk_s3::config::Credentials::new(
                access_key.to_string(),
                secret_key.to_string(),
                None,
                None,
                "static",
            )
        )
        .load()
        .await;

    let client = S3Client::new(&config);
    client.delete_bucket()
        .bucket(bucket)
        .send()
        .await?;
    Ok(())
}

pub async fn aws_s3_bucket_create(
    bucket: &str,
    endpoint: &str,
    access_key: &str,
    secret_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = aws_config::defaults(BehaviorVersion::latest())
        .endpoint_url(endpoint)
        .region("auto")
        .credentials_provider(
            aws_sdk_s3::config::Credentials::new(
                access_key.to_string(),
                secret_key.to_string(),
                None,
                None,
                "static",
            )
        )
        .load()
        .await;

    let client = S3Client::new(&config);
    client.create_bucket()
        .bucket(bucket)
        .send()
        .await?;
    Ok(())
}

pub async fn init_drive(config: &DriveConfig) -> Result<S3Client, Box<dyn std::error::Error>> {
    let endpoint = if !config.server.ends_with('/') {
        format!("{}/", config.server)
    } else {
        config.server.clone()
    };

    let base_config = aws_config::defaults(BehaviorVersion::latest())
        .endpoint_url(endpoint)
        .region("auto")
        .credentials_provider(
            aws_sdk_s3::config::Credentials::new(
                config.access_key.clone(),
                config.secret_key.clone(),
                None,
                None,
                "static",
            )
        )
        .load()
        .await;

    let s3_config = S3ConfigBuilder::from(&base_config)
        .force_path_style(true)
        .build();

    Ok(S3Client::from_conf(s3_config))
}

async fn upload_to_s3(
    client: &S3Client,
    bucket: &str,
    key: &str,
    file_path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let data = std::fs::read(file_path)?;
    client.put_object()
        .bucket(bucket)
        .key(key)
        .body(data.into())
        .send()
        .await?;
    Ok(())
}

async fn create_s3_client(
    
) -> Result<S3Client, Box<dyn std::error::Error>> {
    let config = DriveConfig {
        server: std::env::var("DRIVE_SERVER").expect("DRIVE_SERVER not set"),
        access_key: std::env::var("DRIVE_ACCESS_KEY").expect("DRIVE_ACCESS_KEY not set"),
        secret_key: std::env::var("DRIVE_SECRET_KEY").expect("DRIVE_SECRET_KEY not set"),
        use_ssl: false,
    };
    Ok(init_drive(&config).await?)
}

pub async fn bucket_exists(client: &S3Client, bucket: &str) -> Result<bool, Box<dyn std::error::Error>> {
    match client.head_bucket().bucket(bucket).send().await {
        Ok(_) => Ok(true),
        Err(e) => {
            if e.to_string().contains("NoSuchBucket") {
                Ok(false)
            } else {
                Err(Box::new(e))
            }
        }
    }
}

pub async fn create_bucket(client: &S3Client, bucket: &str) -> Result<(), Box<dyn std::error::Error>> {
    client.create_bucket()
        .bucket(bucket)
        .send()
        .await?;
    Ok(())
}

#[cfg(test)]
mod bucket_tests {
    include!("tests/bucket_tests.rs");
}

#[cfg(test)]
mod tests {
    include!("tests/tests.rs");
}
