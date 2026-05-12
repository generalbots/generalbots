use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

use crate::engine::VideoEngine;
use crate::requests::*;
use crate::routes::AppState;
use crate::safe_error::SafeErrorResponse;

pub async fn list_projects(
    State(state): State<Arc<AppState>>,
    Query(filters): Query<ProjectFilters>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.conn.clone());
    match engine.list_projects(None, filters).await {
        Ok(projects) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({ "projects": projects })),
        ),
        Err(e) => {
            error!("Failed to list video projects: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!(SafeErrorResponse::internal_error())),
            )
        }
    }
}

pub async fn create_project(
    State(state): State<Arc<AppState>>,
    axum::Json(req): axum::Json<CreateProjectRequest>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.conn.clone());
    match engine.create_project(None, None, req).await {
        Ok(project) => (
            StatusCode::CREATED,
            axum::Json(serde_json::json!({ "project": project })),
        ),
        Err(e) => {
            error!("Failed to create video project: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!(SafeErrorResponse::internal_error())),
            )
        }
    }
}

pub async fn get_project(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.conn.clone());
    match engine.get_project_detail(id).await {
        Ok(detail) => (StatusCode::OK, axum::Json(serde_json::json!(detail))),
        Err(diesel::result::Error::NotFound) => (
            StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "error": "Project not found" })),
        ),
        Err(e) => {
            error!("Failed to get video project: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!(SafeErrorResponse::internal_error())),
            )
        }
    }
}

pub async fn update_project(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    axum::Json(req): axum::Json<UpdateProjectRequest>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.conn.clone());
    match engine.update_project(id, req).await {
        Ok(project) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({ "project": project })),
        ),
        Err(diesel::result::Error::NotFound) => (
            StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "error": "Project not found" })),
        ),
        Err(e) => {
            error!("Failed to update video project: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!(SafeErrorResponse::internal_error())),
            )
        }
    }
}

pub async fn delete_project(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let engine = VideoEngine::new(state.conn.clone());
    match engine.delete_project(id).await {
        Ok(()) => (
            StatusCode::NO_CONTENT,
            axum::Json(serde_json::json!({})),
        ),
        Err(e) => {
            error!("Failed to delete video project: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!(SafeErrorResponse::internal_error())),
            )
        }
    }
}
