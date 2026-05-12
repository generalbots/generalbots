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

pub async fn get_keyframes(
    State(state): State<Arc<AppState>>,
    Path(layer_id): Path<Uuid>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.conn.clone());
    match engine.get_keyframes(layer_id).await {
        Ok(keyframes) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({ "keyframes": keyframes })),
        ),
        Err(e) => {
            error!("Failed to get keyframes: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!(SafeErrorResponse::internal_error())),
            )
        }
    }
}

pub async fn add_keyframe(
    State(state): State<Arc<AppState>>,
    Path(layer_id): Path<Uuid>,
    axum::Json(req): axum::Json<AddKeyframeRequest>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.conn.clone());
    match engine.add_keyframe(layer_id, req).await {
        Ok(keyframe) => (
            StatusCode::CREATED,
            axum::Json(serde_json::json!({ "keyframe": keyframe })),
        ),
        Err(e) => {
            error!("Failed to add keyframe: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!(SafeErrorResponse::internal_error())),
            )
        }
    }
}

pub async fn delete_keyframe(
    State(state): State<Arc<AppState>>,
    Path(keyframe_id): Path<Uuid>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.conn.clone());
    match engine.delete_keyframe(keyframe_id).await {
        Ok(()) => (
            StatusCode::NO_CONTENT,
            axum::Json(serde_json::json!({})),
        ),
        Err(e) => {
            error!("Failed to delete keyframe: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!(SafeErrorResponse::internal_error())),
            )
        }
    }
}
