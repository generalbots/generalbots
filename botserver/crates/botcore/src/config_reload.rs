use axum::{extract::State, http::StatusCode, response::Json};
use serde_json::Value;
use std::sync::Arc;
use crate::shared::state::AppState;

pub async fn reload_config(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, StatusCode> {
    let _ = state;
    Err(StatusCode::NOT_IMPLEMENTED)
}
