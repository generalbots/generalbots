pub use botwhatsapp::*;

use axum::{
    extract::{State, Json},
    http::StatusCode,
    routing::{get, post},
    Router,
};
use botcore::shared::state::AppState;
use std::sync::Arc;
use serde::Deserialize;

#[derive(Deserialize)]
#[allow(dead_code)]
struct SendRequest {
    to: String,
    message: String,
}

async fn handle_send(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<SendRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    Ok(Json(serde_json::json!({ "status": "ok", "to": req.to })))
}

async fn handle_status(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

async fn handle_sessions(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    Ok(Json(serde_json::json!({ "sessions": [] })))
}

async fn handle_webhook(
    State(_state): State<Arc<AppState>>,
    Json(_body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

async fn handle_webhook_verify() -> Result<String, (StatusCode, String)> {
    Ok("ok".to_string())
}

pub fn configure(app_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/whatsapp/webhook", post(handle_webhook).get(handle_webhook_verify))
        .route("/api/whatsapp/send", post(handle_send))
        .route("/api/whatsapp/status", get(handle_status))
        .route("/api/whatsapp/sessions", get(handle_sessions))
        .with_state(app_state)
}
