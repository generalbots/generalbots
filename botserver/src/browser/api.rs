use axum::{
    extract::{Path, State},
    response::Json,
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::core::shared::state::AppState;

pub fn configure_browser_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/browser/session", post(create_session))
        .route("/api/browser/session/:id/execute", post(run_action))
        .route("/api/browser/session/:id/screenshot", get(capture_screenshot))
        .route("/api/browser/session/:id/record/start", post(start_recording))
        .route("/api/browser/session/:id/record/stop", post(stop_recording))
        .route("/api/browser/session/:id/record/export", get(export_test))
}

#[derive(Deserialize)]
pub struct CreateSessionRequest {
    pub headless: Option<bool>,
}

pub async fn create_session(
    State(_state): State<Arc<AppState>>,
    Json(_payload): Json<CreateSessionRequest>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    Ok(Json(serde_json::json!({
        "id": "mock-session-id-1234",
        "status": "created"
    })))
}

#[derive(Deserialize)]
pub struct ExecuteActionRequest {
    pub action_type: String,
    pub payload: serde_json::Value,
}

pub async fn run_action(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(_payload): Json<ExecuteActionRequest>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    Ok(Json(serde_json::json!({ "status": "success", "session": id })))
}

pub async fn capture_screenshot(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<String>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    Ok(Json(serde_json::json!({ "image_data": "base64_encoded_dummy_screenshot" })))
}

pub async fn start_recording(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<String>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    Ok(Json(serde_json::json!({ "status": "recording_started" })))
}

pub async fn stop_recording(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<String>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    Ok(Json(serde_json::json!({ "status": "recording_stopped", "actions": [] })))
}

pub async fn export_test(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<String>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let script = r#"
import { test, expect } from '@playwright/test';
test('Recorded test', async ({ page }) => {
  await page.goto('');
  // Add actions
});
"#;
    Ok(Json(serde_json::json!({ "script": script })))
}
