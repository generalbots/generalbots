use axum::extract::State;
use axum::response::Json;
use std::sync::Arc;
use serde_json::{json, Value};
use crate::state::FacebookState;

pub async fn handle_send_message(
    State(_state): State<Arc<FacebookState>>,
    Json(payload): Json<serde_json::Value>,
) -> Json<Value> {
    let recipient = payload.get("recipient").and_then(|r| r.as_str()).unwrap_or("");
    let message = payload.get("message").and_then(|m| m.as_str()).unwrap_or("");
    let bot_name = payload.get("bot_name").and_then(|b| b.as_str()).unwrap_or("default");

    let (bot_id, _) = (_state.get_default_bot)();
    let user_id = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_DNS, format!("fb:{}", recipient).as_bytes());
    let session_id = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_DNS, format!("fb-session:{}", recipient).as_bytes());

    let _ = (_state.process_message)(
        &bot_id.to_string(),
        recipient,
        message,
        &session_id.to_string(),
        bot_name,
    ).await;

    Json(json!({"status": "sent"}))
}

pub async fn handle_status() -> Json<Value> {
    Json(json!({"status": "ok", "service": "facebook"}))
}

pub async fn handle_sessions() -> Json<Value> {
    Json(json!({"sessions": [], "count": 0}))
}
