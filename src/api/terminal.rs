use axum::{
    extract::State,
    response::Json,
    routing::get,
    Router,
};
use std::sync::Arc;

use crate::core::shared::state::AppState;

pub fn configure_terminal_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/terminal/ws", get(terminal_ws))
}

pub async fn terminal_ws(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    // Note: Mock websocket connection upgrade logic
    Ok(Json(serde_json::json!({ "status": "Upgrade required" })))
}
