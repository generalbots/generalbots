use crate::adapter::InstagramAdapter;
use crate::state::ChannelState;
use crate::types::InstagramWebhookPayload;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use log::info;
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

pub fn configure() -> Router<Arc<ChannelState>> {
    Router::new()
        .route(
            "/api/instagram/webhook",
            get(verify_webhook).post(handle_webhook),
        )
        .route("/api/instagram/send", post(crate::handlers::send_message))
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

pub async fn handle_webhook(
    State(state): State<Arc<ChannelState>>,
    Json(payload): Json<InstagramWebhookPayload>,
) -> impl IntoResponse {
    for entry in &payload.entry {
        if let Some(messaging_list) = &entry.messaging {
            for messaging in messaging_list {
                if let Some(message) = &messaging.message {
                    if let Some(text) = &message.text {
                        info!(
                            "Instagram message from={} text={}",
                            messaging.sender.id, text
                        );

                        let user_id = &messaging.sender.id;
                        let user_message = botlib::models::UserMessage::text(
                            String::new(),
                            user_id.clone(),
                            String::new(),
                            "instagram".to_string(),
                            text.clone(),
                        );

                        let (tx, mut rx) = tokio::sync::mpsc::channel::<botlib::models::BotResponse>(10);
                        let _handle = (state.stream_response)(user_message, tx);

                        let adapter = InstagramAdapter::with_config(
                            &state.get_config,
                            "",
                        );
                        let recipient = user_id.clone();

                        tokio::spawn(async move {
                            let mut accumulated = String::new();
                            while let Some(response) = rx.recv().await {
                                if !response.content.is_empty() {
                                    accumulated.push_str(&response.content);
                                }
                                if response.is_complete {
                                    if !accumulated.is_empty() {
                                        let _ = adapter
                                            .send_instagram_message(&recipient, &accumulated)
                                            .await;
                                        accumulated.clear();
                                    }
                                }
                            }
                            if !accumulated.is_empty() {
                                let _ = adapter
                                    .send_instagram_message(&recipient, &accumulated)
                                    .await;
                            }
                        });
                    }
                }
            }
        }
    }

    StatusCode::OK
}
