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

pub async fn get_layers(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<Uuid>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.conn.clone());
    match engine.get_layers(project_id).await {
        Ok(layers) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({ "layers": layers })),
        ),
        Err(e) => {
            error!("Failed to get layers: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!(SafeErrorResponse::internal_error())),
            )
        }
    }
}

pub async fn add_layer(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<Uuid>,
    axum::Json(req): axum::Json<AddLayerRequest>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.conn.clone());
    match engine.add_layer(project_id, req).await {
        Ok(layer) => (
            StatusCode::CREATED,
            axum::Json(serde_json::json!({ "layer": layer })),
        ),
        Err(e) => {
            error!("Failed to add layer: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!(SafeErrorResponse::internal_error())),
            )
        }
    }
}

pub async fn update_layer(
    State(state): State<Arc<AppState>>,
    Path(layer_id): Path<Uuid>,
    axum::Json(req): axum::Json<UpdateLayerRequest>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.conn.clone());
    match engine.update_layer(layer_id, req).await {
        Ok(layer) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({ "layer": layer })),
        ),
        Err(diesel::result::Error::NotFound) => (
            StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "error": "Layer not found" })),
        ),
        Err(e) => {
            error!("Failed to update layer: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!(SafeErrorResponse::internal_error())),
            )
        }
    }
}

pub async fn delete_layer(
    State(state): State<Arc<AppState>>,
    Path(layer_id): Path<Uuid>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.conn.clone());
    match engine.delete_layer(layer_id).await {
        Ok(()) => (
            StatusCode::NO_CONTENT,
            axum::Json(serde_json::json!({})),
        ),
        Err(e) => {
            error!("Failed to delete layer: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!(SafeErrorResponse::internal_error())),
            )
        }
    }
}
