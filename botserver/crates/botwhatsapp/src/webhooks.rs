use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::models::WhatsAppWebhookPayload;
use crate::state::WhatsAppState;
use crate::message_processing::process_incoming_message;

#[derive(Deserialize)]
pub struct VerifyParams {
    #[serde(rename = "hub.mode")]
    pub hub_mode: Option<String>,
    #[serde(rename = "hub.challenge")]
    pub hub_challenge: Option<String>,
    #[serde(rename = "hub.verify_token")]
    pub hub_verify_token: Option<String>,
}

pub async fn handle_webhook_verify(
    State(state): State<Arc<WhatsAppState>>,
    Query(params): Query<VerifyParams>,
) -> Response {
    let verify_token = match (state.secrets)("whatsapp_verify_token") {
        Ok(t) => t,
        Err(_) => return (StatusCode::FORBIDDEN, "Verification token not configured").into_response(),
    };

    if params.hub_mode.as_deref() != Some("subscribe") {
        return (StatusCode::FORBIDDEN, "Invalid mode").into_response();
    }

    if params.hub_verify_token.as_deref() != Some(&verify_token) {
        return (StatusCode::FORBIDDEN, "Invalid verify token").into_response();
    }

    match params.hub_challenge {
        Some(challenge) => (StatusCode::OK, challenge).into_response(),
        None => (StatusCode::BAD_REQUEST, "Missing challenge").into_response(),
    }
}

pub async fn handle_webhook(
    State(state): State<Arc<WhatsAppState>>,
    _headers: HeaderMap,
    Json(payload): Json<WhatsAppWebhookPayload>,
) -> Response {
    if payload.object.as_deref() != Some("whatsapp_business_account") {
        return (StatusCode::OK, "ignored").into_response();
    }

    for entry in &payload.entry {
        for change in &entry.changes {
            let phone_number_id = change.value.metadata.as_ref()
                .and_then(|m| m.phone_number_id.clone());

            if let Some(ref messages) = change.value.messages {
                for message in messages {
                    let phone_number = match &message.from {
                        Some(p) => p.clone(),
                        None => continue,
                    };

                    let message_content = match message.message_type.as_deref() {
                        Some("text") => message.text.as_ref().and_then(|t| t.body.clone()),
                        Some("interactive") => {
                            message.interactive.as_ref().and_then(|i| {
                                i.button_reply
                                    .as_ref()
                                    .map(|b| b.title.clone().unwrap_or_default())
                                    .or_else(|| {
                                        i.list_reply
                                            .as_ref()
                                            .map(|l| l.title.clone().unwrap_or_default())
                                    })
                            })
                        }
                        Some("button") => message
                            .button
                            .as_ref()
                            .and_then(|b| b.text.clone()),
                        _ => {
                            log::info!(
                                "Unsupported message type: {:?}",
                                message.message_type
                            );
                            continue;
                        }
                    };

                    let content = match message_content {
                        Some(c) => c,
                        None => {
                            log::warn!("Empty message content from {}", phone_number);
                            continue;
                        }
                    };

                    let state_clone = state.clone();
                    let pn = phone_number.clone();
                    let ct = content.clone();
                    let pni = phone_number_id.clone();
                    let msg = message.clone();
                    tokio::spawn(async move {
                        if let Err(e) =
                            process_incoming_message(&state_clone, &pn, &ct, &msg, pni).await
                        {
                            log::error!("Failed to process message from {}: {}", pn, e);
                        }
                    });
                }
            }

            if let Some(ref statuses) = change.value.statuses {
                for status in statuses {
                    log::info!(
                        "Message {} status: {:?}",
                        status.id.as_deref().unwrap_or("unknown"),
                        status.status
                    );
                }
            }
        }
    }

    (StatusCode::OK, "ok").into_response()
}
