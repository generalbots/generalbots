pub use crate::core::bot::channels::instagram::*;

use crate::shared::state::AppState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct WebhookVerifyQuery {
    #[serde(rename = "hub.mode")]
    pub mode: Option<String>,
    #[serde(rename = "hub.verify_token")]
    pub verify_token: Option<String>,
    #[serde(rename = "hub.challenge")]
    pub challenge: Option<String>,
}

pub fn configure() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/api/instagram/webhook",
            get(verify_webhook).post(handle_webhook),
        )
        .route("/api/instagram/send", post(send_message))
}

async fn verify_webhook(Query(query): Query<WebhookVerifyQuery>) -> impl IntoResponse {
    let adapter = InstagramAdapter::new();

    match (
        query.mode.as_deref(),
        query.verify_token.as_deref(),
        query.challenge,
    ) {
        (Some(mode), Some(token), Some(challenge)) => adapter
            .handle_webhook_verification(mode, token, &challenge)
            .map_or_else(
                || (StatusCode::FORBIDDEN, "Verification failed".to_string()),
                |response| (StatusCode::OK, response),
            ),
        _ => (StatusCode::BAD_REQUEST, "Missing parameters".to_string()),
    }
}

async fn handle_webhook(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<InstagramWebhookPayload>,
) -> impl IntoResponse {
    for entry in payload.entry {
        if let Some(messaging_list) = entry.messaging {
            for messaging in messaging_list {
                if let Some(message) = messaging.message {
                    if let Some(text) = message.text {
                        log::info!(
                            "Instagram message from={} text={}",
                            messaging.sender.id,
                            text
                        );
                    }
                }
            }
        }
    }

    StatusCode::OK
}

async fn send_message(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<serde_json::Value>,
) -> impl IntoResponse {
    let adapter = InstagramAdapter::new();
    let recipient = request.get("to").and_then(|v| v.as_str()).unwrap_or("");
    let message = request
        .get("message")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    match adapter.send_instagram_message(recipient, message).await {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({"success": true}))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"success": false, "error": e.to_string()})),
        ),
    }
}
