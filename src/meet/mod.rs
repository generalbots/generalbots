use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use log::{error, info};
use serde_json::Value;
use std::sync::Arc;

use crate::shared::state::AppState;

pub async fn voice_start(
    State(data): State<Arc<AppState>>,
    Json(info): Json<Value>,
) -> impl IntoResponse {
    let session_id = info
        .get("session_id")
        .and_then(|s| s.as_str())
        .unwrap_or("");
    let user_id = info
        .get("user_id")
        .and_then(|u| u.as_str())
        .unwrap_or("user");

    info!(
        "Voice session start request - session: {}, user: {}",
        session_id, user_id
    );

    match data
        .voice_adapter
        .start_voice_session(session_id, user_id)
        .await
    {
        Ok(token) => {
            info!(
                "Voice session started successfully for session {}",
                session_id
            );
            (
                StatusCode::OK,
                Json(serde_json::json!({"token": token, "status": "started"})),
            )
        }
        Err(e) => {
            error!(
                "Failed to start voice session for session {}: {}",
                session_id, e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        }
    }
}

pub async fn voice_stop(
    State(data): State<Arc<AppState>>,
    Json(info): Json<Value>,
) -> impl IntoResponse {
    let session_id = info
        .get("session_id")
        .and_then(|s| s.as_str())
        .unwrap_or("");

    match data.voice_adapter.stop_voice_session(session_id).await {
        Ok(()) => {
            info!(
                "Voice session stopped successfully for session {}",
                session_id
            );
            (
                StatusCode::OK,
                Json(serde_json::json!({"status": "stopped"})),
            )
        }
        Err(e) => {
            error!(
                "Failed to stop voice session for session {}: {}",
                session_id, e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        }
    }
}
