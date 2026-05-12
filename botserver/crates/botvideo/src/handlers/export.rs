use axum::extract::{Path, State};
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

pub async fn start_export(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<Uuid>,
    axum::Json(req): axum::Json<ExportRequest>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.conn.clone());
    match engine.start_export(project_id, req, state.cache.as_ref()).await {
        Ok(export) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({ "export": export })),
        ),
        Err(e) => {
            error!("Start export failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!(SafeErrorResponse::internal_error())),
            )
        }
    }
}

pub async fn get_export_status(
    State(state): State<Arc<AppState>>,
    Path(export_id): Path<Uuid>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.conn.clone());
    match engine.get_export_status(export_id).await {
        Ok(export) => (
            StatusCode::OK,
            axum::Json(serde_json::json!(ExportStatusResponse {
                id: export.id,
                status: export.status,
                progress: export.progress,
                output_url: export.output_url,
                gbdrive_path: export.gbdrive_path,
                error_message: export.error_message,
            })),
        ),
        Err(diesel::result::Error::NotFound) => (
            StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "error": "Export not found" })),
        ),
        Err(e) => {
            error!("Get export status failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!(SafeErrorResponse::internal_error())),
            )
        }
    }
}

pub async fn add_transition_handler(
    State(state): State<Arc<AppState>>,
    Path((from_id, to_id)): Path<(Uuid, Uuid)>,
    axum::Json(req): axum::Json<TransitionRequest>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.conn.clone());
    match engine
        .add_transition(from_id, to_id, &req.transition_type, req.duration_ms.unwrap_or(500))
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({ "success": true })),
        ),
        Err(e) => {
            error!("Add transition failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!(SafeErrorResponse::internal_error())),
            )
        }
    }
}
