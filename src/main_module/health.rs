//! Health check and client error handlers

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use std::sync::Arc;

use crate::core::shared::state::AppState;

pub async fn health_check(State(state): State<Arc<AppState>>) -> (StatusCode, Json<serde_json::Value>) {
    let db_ok = state.conn.get().is_ok();

    let status = if db_ok { "healthy" } else { "degraded" };
    let code = if db_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        code,
        Json(serde_json::json!({
            "status": status,
            "service": "botserver",
            "version": env!("CARGO_PKG_VERSION"),
            "database": db_ok
        })),
    )
}

pub async fn health_check_simple() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
            "service": "botserver",
            "version": env!("CARGO_PKG_VERSION")
        })),
    )
}

#[derive(serde::Deserialize)]
pub struct ClientErrorsRequest {
    errors: Vec<ClientErrorData>,
}

#[derive(serde::Deserialize)]
pub struct ClientErrorData {
    #[serde(default)]
    r#type: String,
    #[serde(default)]
    message: String,
    #[serde(default)]
    stack: Option<String>,
    #[serde(default)]
    url: String,
    #[serde(default)]
    timestamp: String,
}

pub async fn receive_client_errors(
    Json(payload): Json<ClientErrorsRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    for error in &payload.errors {
        log::error!(
            "[CLIENT ERROR] {} | {} | {} | URL: {} | Stack: {}",
            error.timestamp,
            error.r#type,
            error.message,
            error.url,
            error.stack.as_deref().unwrap_or("<no stack>")
        );
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "received",
            "count": payload.errors.len()
        })),
    )
}
