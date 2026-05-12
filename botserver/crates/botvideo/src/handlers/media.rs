use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

use crate::engine::VideoEngine;
use crate::requests::*;
use crate::responses::*;
use crate::routes::AppState;
use crate::safe_error::SafeErrorResponse;

pub async fn upload_media(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<Uuid>,
    mut multipart: axum::extract::Multipart,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.conn.clone());
    let _ = engine;
    let upload_dir =
        std::env::var("VIDEO_UPLOAD_DIR").unwrap_or_else(|_| "./uploads/video".to_string());

    if let Err(e) = std::fs::create_dir_all(&upload_dir) {
        error!("Failed to create upload directory: {e}");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": SafeErrorResponse::internal_error() })),
        );
    }

    while let Ok(Some(field)) = multipart.next_field().await {
        let file_name = field
            .file_name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{}.mp4", Uuid::new_v4()));

        let content_type = field
            .content_type()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "video/mp4".to_string());

        let data = match field.bytes().await {
            Ok(d) => d,
            Err(e) => {
                error!("Failed to read upload data: {e}");
                return (
                    StatusCode::BAD_REQUEST,
                    axum::Json(serde_json::json!({ "error": "Failed to read upload" })),
                );
            }
        };

        let file_size = data.len() as u64;
        let safe_name = format!("{}_{}", project_id, sanitize_filename(&file_name));
        let file_path = format!("{}/{}", upload_dir, safe_name);

        if let Err(e) = std::fs::write(&file_path, &data) {
            error!("Failed to write uploaded file: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({ "error": SafeErrorResponse::internal_error() })),
            );
        }

        let file_url = format!("/video/uploads/{}", safe_name);

        return (
            StatusCode::OK,
            axum::Json(serde_json::json!(UploadResponse {
                file_url,
                file_name: safe_name,
                file_size,
                mime_type: content_type,
            })),
        );
    }

    (
        StatusCode::BAD_REQUEST,
        axum::Json(serde_json::json!({ "error": "No file provided" })),
    )
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

pub async fn get_preview_frame(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<Uuid>,
    Query(params): Query<PreviewFrameRequest>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.conn.clone());
    let at_ms = params.at_ms.unwrap_or(0);
    let width = params.width.unwrap_or(640);
    let height = params.height.unwrap_or(360);

    let output_dir =
        std::env::var("VIDEO_PREVIEW_DIR").unwrap_or_else(|_| "./previews/video".to_string());

    if let Err(e) = std::fs::create_dir_all(&output_dir) {
        error!("Failed to create preview directory: {e}");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": SafeErrorResponse::internal_error() })),
        );
    }

    match engine
        .generate_preview_frame(project_id, at_ms, width, height, &output_dir)
        .await
    {
        Ok(url) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({ "preview_url": url, "at_ms": at_ms })),
        ),
        Err(e) => {
            error!("Failed to generate preview: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({ "error": SafeErrorResponse::internal_error() })),
            )
        }
    }
}
