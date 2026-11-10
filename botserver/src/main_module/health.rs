//! Health check and client error handlers

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use std::sync::Arc;

use botcore::shared::state::AppState;

pub async fn health_check(
    State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<serde_json::Value>) {
    let pool = state.conn.clone();
    let db_ok = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        tokio::task::spawn_blocking(move || pool.get().is_ok()),
    )
    .await
    .unwrap_or(Ok(false))
    .unwrap_or(false);

    let status = if db_ok { "healthy" } else { "degraded" };
    let code = if db_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    // #2103 — one build metadata source for every endpoint.
    let build_date = botcore::product::build_info::build_date();
    let commit = botcore::product::build_info::commit();

    (
        code,
        Json(serde_json::json!({
            "status": status,
            "service": "botserver",
            "version": botcore::product::build_info::version(),
            "build_date": build_date,
            "commit": commit,
            "database": db_ok
        })),
    )
}

pub async fn health_check_simple() -> (StatusCode, Json<serde_json::Value>) {
    let commit = botcore::product::build_info::commit();
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
            "service": "botserver",
            "version": botcore::product::build_info::version(),
            "commit": commit
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
