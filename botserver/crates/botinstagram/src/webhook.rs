use crate::adapter::InstagramAdapter;
use crate::auth::{self, DeliveryAuth};
use crate::state::ChannelState;
use crate::types::InstagramWebhookPayload;

use axum::{
    body::Bytes,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use log::{info, warn};
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

/// Reads one bot configuration value for the deployment's bot.
///
/// Instagram is addressed by handle, and the webhook carries no bot identity, so
/// the empty handle resolves to the deployment's default bot. A configuration
/// read failure is treated as "not configured": the gate fails closed rather
/// than accepting a delivery it cannot check.
fn config_value(state: &Arc<ChannelState>, key: &str) -> String {
    match (state.get_config)("", key, None) {
        Ok(value) => value,
        Err(e) => {
            warn!("Instagram webhook config read failed for '{key}': {e}");
            String::new()
        }
    }
}

/// Meta subscription handshake. The verify token is mandatory: the handler used
/// to fall back to the published string `webhook_verify`, which meant any caller
/// could complete the handshake for any bot (#2142).
async fn verify_webhook(
    State(state): State<Arc<ChannelState>>,
    Query(query): Query<WebhookVerifyQuery>,
) -> Response {
    let expected = config_value(&state, auth::VERIFY_TOKEN_KEY);

    match auth::authorize_handshake(
        &expected,
        query.mode.as_deref(),
        query.verify_token.as_deref(),
        query.challenge.as_deref(),
    ) {
        Ok(challenge) => (StatusCode::OK, challenge).into_response(),
        Err(error) => {
            warn!("Instagram subscription handshake refused: {}", error.reason());
            (error.status(), error.reason()).into_response()
        }
    }
}

/// Inbound delivery. The signature is checked before the body is parsed: an
/// unsigned POST must not reach the dispatcher at all (#2142).
pub async fn handle_webhook(
    State(state): State<Arc<ChannelState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let app_secret = config_value(&state, auth::APP_SECRET_KEY);
    let signature = headers
        .get(auth::SIGNATURE_HEADER)
        .and_then(|value| value.to_str().ok());

    match auth::authorize_delivery(&app_secret, signature, &body) {
        DeliveryAuth::Verified => {}
        DeliveryAuth::Mismatch => {
            warn!(
                "Instagram webhook delivery rejected: {} missing or mismatched",
                auth::SIGNATURE_HEADER
            );
            return (StatusCode::FORBIDDEN, "Invalid signature").into_response();
        }
        DeliveryAuth::Unconfigured => {
            auth::warn_unconfigured_once();
            return (StatusCode::SERVICE_UNAVAILABLE, "Webhook signature not configured")
                .into_response();
        }
    }

    let payload: InstagramWebhookPayload = match serde_json::from_slice(&body) {
        Ok(payload) => payload,
        Err(e) => {
            warn!("Instagram webhook payload is not valid JSON: {e}");
            return (StatusCode::BAD_REQUEST, "Invalid payload").into_response();
        }
    };

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
                        // The pipeline runs detached; dropping the JoinHandle
                        // keeps it alive (it does not cancel the task).
                        drop((state.stream_response)(user_message, tx));

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
                                if response.is_complete && !accumulated.is_empty() {
                                    let _ = adapter
                                        .send_instagram_message(&recipient, &accumulated)
                                        .await;
                                    accumulated.clear();
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

    (StatusCode::OK, "ok").into_response()
}
