use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

use crate::engine::VideoEngine;
use crate::requests::*;
use crate::routes::AppState;
use crate::safe_error::SafeErrorResponse;

pub async fn get_clips(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<Uuid>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.conn.clone());
    match engine.get_clips(project_id).await {
        Ok(clips) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({ "clips": clips })),
        ),
        Err(e) => {
            error!("Failed to get clips: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!(SafeErrorResponse::internal_error())),
            )
        }
    }
}

pub async fn add_clip(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<Uuid>,
    axum::Json(req): axum::Json<AddClipRequest>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.conn.clone());
    match engine.add_clip(project_id, req).await {
        Ok(clip) => (
            StatusCode::CREATED,
            axum::Json(serde_json::json!({ "clip": clip })),
        ),
        Err(e) => {
            error!("Failed to add clip: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!(SafeErrorResponse::internal_error())),
            )
        }
    }
}

pub async fn update_clip(
    State(state): State<Arc<AppState>>,
    Path(clip_id): Path<Uuid>,
    axum::Json(req): axum::Json<UpdateClipRequest>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.conn.clone());
    match engine.update_clip(clip_id, req).await {
        Ok(clip) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({ "clip": clip })),
        ),
        Err(diesel::result::Error::NotFound) => (
            StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "error": "Clip not found" })),
        ),
        Err(e) => {
            error!("Failed to update clip: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!(SafeErrorResponse::internal_error())),
            )
        }
    }
}

pub async fn delete_clip(
    State(state): State<Arc<AppState>>,
    Path(clip_id): Path<Uuid>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.conn.clone());
    match engine.delete_clip(clip_id).await {
        Ok(()) => (
            StatusCode::NO_CONTENT,
            axum::Json(serde_json::json!({})),
        ),
        Err(e) => {
            error!("Failed to delete clip: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!(SafeErrorResponse::internal_error())),
            )
        }
    }
}

pub async fn split_clip_handler(
    State(state): State<Arc<AppState>>,
    Path(clip_id): Path<Uuid>,
    axum::Json(req): axum::Json<SplitClipRequest>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.conn.clone());
    match engine.split_clip(clip_id, req.at_ms).await {
        Ok((first, second)) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({
                "first_clip": first,
                "second_clip": second,
            })),
        ),
        Err(diesel::result::Error::NotFound) => (
            StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({ "error": "Invalid split position or clip not found" })),
        ),
        Err(e) => {
            error!("Failed to split clip: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!(SafeErrorResponse::internal_error())),
            )
        }
    }
}
