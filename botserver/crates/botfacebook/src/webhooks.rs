use axum::extract::{Query, State};
use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;
use axum::Json;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::collections::HashSet;

use crate::models::{VerifyParams, WebhookPayload};
use crate::state::FacebookState;
use crate::message_processing;

static SEEN_IDS: LazyLock<Mutex<HashSet<String>>> = LazyLock::new(|| Mutex::new(HashSet::new()));

pub async fn handle_webhook_verify(
    State(_state): State<Arc<FacebookState>>,
    Query(params): Query<VerifyParams>,
) -> Response {
    let mode = params.hub_mode.as_deref().unwrap_or("");
    let token = params.hub_verify_token.as_deref().unwrap_or("");
    let challenge = params.hub_challenge.unwrap_or_default();

    if mode != "subscribe" {
        return (StatusCode::BAD_REQUEST, "Invalid mode").into_response();
    }

    let expected_token = "fb-verify-2026";
    if token != expected_token {
        return (StatusCode::FORBIDDEN, "Invalid verify token").into_response();
    }

    (StatusCode::OK, challenge).into_response()
}

pub async fn handle_webhook(
    State(state): State<Arc<FacebookState>>,
    body: String,
) -> Response {
    let payload: WebhookPayload = match serde_json::from_str(&body) {
        Ok(p) => p,
        Err(e) => {
            log::error!("Facebook webhook parse error: {}", e);
            return (StatusCode::BAD_REQUEST, "Invalid payload").into_response();
        }
    };

    if payload.object != "page" {
        return (StatusCode::OK, "Event received").into_response();
    }

    for entry in payload.entry {
        for event in entry.messaging {
            if let Some(msg) = event.message {
                let sender_id = event.sender.id.clone();
                let msg_id = msg.mid.clone();

                {
                    let mut seen = SEEN_IDS.lock().unwrap();
                    if !seen.insert(msg_id.clone()) {
                        continue;
                    }
                    if seen.len() > 10000 {
                        seen.clear();
                    }
                }

                if let Some(text) = msg.text {
                    if text.is_empty() { continue; }

                    let db_id = sender_id.clone();
                    let content = text.clone();

                    let state_clone = state.clone();
                    tokio::spawn(async move {
                        let _ = message_processing::process_incoming_message(
                            &state_clone, &db_id, &content, &sender_id, "fb",
                        ).await;
                    });
                }
            }

            if let Some(postback) = event.postback {
                if let Some(payload) = postback.payload {
                    let state_clone = state.clone();
                    let sender_id = event.sender.id.clone();
                    tokio::spawn(async move {
                        let _ = message_processing::process_incoming_message(
                            &state_clone, &sender_id, &payload, &sender_id, "fb",
                        ).await;
                    });
                }
            }
        }
    }

    (StatusCode::OK, "EVENT_RECEIVED").into_response()
}
