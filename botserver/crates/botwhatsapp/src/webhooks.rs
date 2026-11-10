//! WhatsApp Cloud API webhook: verification handshake, signature gate and the
//! inbound normalization that turns a delivery into bot turns.
//!
//! The dispatch used to be one long loop inside the `async` handler, which made
//! every decision (media vs. text, skip vs. empty marker, dedup) unreachable
//! offline. The pure pieces live here so the tests can drive them directly.

use axum::{
    body::Bytes,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use std::collections::HashSet;
use std::sync::{Arc, LazyLock};
use tokio::sync::Mutex;

use crate::models::{WhatsAppMessage, WhatsAppWebhookPayload};
use crate::state::WhatsAppState;
use crate::message_processing::process_incoming_message;
use crate::utils::{extract_message_text, verify_webhook_signature};

/// Meta sends `X-Hub-Signature-256: sha256=<hex>` on every delivery once the
/// app secret is configured.
const SIGNATURE_HEADER: &str = "x-hub-signature-256";

/// Bot config key holding the Meta app secret. When it is absent or empty the
/// endpoint accepts unsigned deliveries — the behaviour of every installation
/// that has not configured the secret yet.
const APP_SECRET_KEY: &str = "whatsapp_app_secret";

/// Message ids remembered for redelivery suppression.
const DEDUP_LIMIT: usize = 1000;

static DEDUP_CACHE: LazyLock<Mutex<DedupCache>> =
    LazyLock::new(|| Mutex::new(DedupCache::new(DEDUP_LIMIT)));

#[derive(Deserialize)]
pub struct VerifyParams {
    #[serde(rename = "hub.mode")]
    pub hub_mode: Option<String>,
    #[serde(rename = "hub.challenge")]
    pub hub_challenge: Option<String>,
    #[serde(rename = "hub.verify_token")]
    pub hub_verify_token: Option<String>,
}

/// One inbound turn: everything the processing step needs, already normalized.
#[derive(Debug)]
pub(crate) struct InboundTurn {
    pub(crate) phone_number: String,
    pub(crate) content: String,
    pub(crate) phone_number_id: Option<String>,
    pub(crate) message: WhatsAppMessage,
}

/// Redelivery guard. Meta retries a delivery whenever the 200 takes too long,
/// and media ingestion routinely does; a message id seen before is dropped.
///
/// The cache is bounded: when it is full it is emptied *before* the new id is
/// recorded, so the id that triggered the reset is still remembered and its
/// redelivery is dropped (the previous code cleared after inserting, which
/// forgot exactly that one).
pub(crate) struct DedupCache {
    limit: usize,
    seen: HashSet<String>,
}

impl DedupCache {
    pub(crate) fn new(limit: usize) -> Self {
        Self {
            limit,
            seen: HashSet::new(),
        }
    }

    /// Records `message_id` and reports whether the turn must be processed.
    pub(crate) fn accept(&mut self, message_id: &str) -> bool {
        if self.seen.contains(message_id) {
            return false;
        }
        if self.seen.len() >= self.limit {
            self.seen.clear();
        }
        self.seen.insert(message_id.to_string());
        true
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.seen.len()
    }
}

/// Answers the Meta subscription handshake. `Ok(challenge)` is echoed back
/// verbatim (Meta compares it byte for byte); every failure carries the status
/// and message the handler returns.
pub(crate) fn verify_challenge(
    params: &VerifyParams,
    verify_token: &str,
) -> Result<String, (StatusCode, &'static str)> {
    if params.hub_mode.as_deref() != Some("subscribe") {
        return Err((StatusCode::FORBIDDEN, "Invalid mode"));
    }

    if params.hub_verify_token.as_deref() != Some(verify_token) {
        return Err((StatusCode::FORBIDDEN, "Invalid verify token"));
    }

    match &params.hub_challenge {
        Some(challenge) => Ok(challenge.clone()),
        None => Err((StatusCode::BAD_REQUEST, "Missing challenge")),
    }
}

/// Decides whether a delivery is authentic. An unconfigured secret keeps the
/// documented "accept unsigned" default; a configured one requires a valid
/// `X-Hub-Signature-256` over the raw body.
pub(crate) fn signature_gate(app_secret: &str, signature: Option<&str>, body: &str) -> bool {
    let secret = app_secret.trim();
    if secret.is_empty() {
        return true;
    }
    verify_webhook_signature(body, signature.unwrap_or_default(), secret)
}

/// Flattens the nested webhook envelope into one turn per processable message.
///
/// Rules, all pinned by tests:
/// - a message without `from` is skipped (nothing to answer);
/// - media-only messages produce a turn with empty content — the media
///   ingestion step fills it in, so dropping them here would lose the file;
/// - an unsupported type is skipped;
/// - `statuses` are not turns and are not returned.
pub(crate) fn normalize_payload(payload: &WhatsAppWebhookPayload) -> Vec<InboundTurn> {
    let mut turns = Vec::new();

    for entry in &payload.entry {
        for change in &entry.changes {
            let phone_number_id = change
                .value
                .metadata
                .as_ref()
                .and_then(|metadata| metadata.phone_number_id.clone());

            let Some(messages) = change.value.messages.as_ref() else {
                continue;
            };

            for message in messages {
                let Some(phone_number) = message.from.clone() else {
                    continue;
                };

                let is_media = matches!(
                    message.message_type.as_deref(),
                    Some("image" | "video" | "audio" | "document" | "sticker")
                );

                let content = match extract_message_text(message) {
                    Some(text) => text,
                    None if is_media => String::new(),
                    None => {
                        log::info!("Unsupported message type: {:?}", message.message_type);
                        continue;
                    }
                };

                turns.push(InboundTurn {
                    phone_number,
                    content,
                    phone_number_id: phone_number_id.clone(),
                    message: message.clone(),
                });
            }
        }
    }

    turns
}

pub async fn handle_webhook_verify(
    State(state): State<Arc<WhatsAppState>>,
    Query(params): Query<VerifyParams>,
) -> Response {
    let verify_token = match (state.secrets)("whatsapp_verify_token") {
        Ok(token) => token,
        Err(_) => {
            return (StatusCode::FORBIDDEN, "Verification token not configured").into_response()
        }
    };

    match verify_challenge(&params, &verify_token) {
        Ok(challenge) => (StatusCode::OK, challenge).into_response(),
        Err((status, message)) => (status, message).into_response(),
    }
}

pub async fn handle_webhook(
    State(state): State<Arc<WhatsAppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    // The signature binds the exact bytes Meta sent, so the body is read raw
    // and parsed only after the gate: `Json` would discard the original
    // serialization and no digest would ever match.
    let app_secret = (state.secrets)(APP_SECRET_KEY).unwrap_or_default();
    let signature = headers
        .get(SIGNATURE_HEADER)
        .and_then(|value| value.to_str().ok());

    let raw_body = String::from_utf8_lossy(&body);
    if !signature_gate(&app_secret, signature, &raw_body) {
        log::warn!("WhatsApp webhook delivery rejected: {SIGNATURE_HEADER} missing or mismatched");
        return (StatusCode::UNAUTHORIZED, "Invalid signature").into_response();
    }

    let payload: WhatsAppWebhookPayload = match serde_json::from_slice(&body) {
        Ok(payload) => payload,
        Err(e) => {
            log::warn!("WhatsApp webhook payload is not valid JSON: {e}");
            return (StatusCode::BAD_REQUEST, "Invalid payload").into_response();
        }
    };

    if payload.object.as_deref() != Some("whatsapp_business_account") {
        return (StatusCode::OK, "ignored").into_response();
    }

    for entry in &payload.entry {
        for change in &entry.changes {
            if let Some(statuses) = change.value.statuses.as_ref() {
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

    for turn in normalize_payload(&payload) {
        if let Some(message_id) = turn.message.id.as_deref() {
            let mut dedup = DEDUP_CACHE.lock().await;
            if !dedup.accept(message_id) {
                log::info!("Dedup: skipping already processed message {message_id}");
                continue;
            }
        }

        let state_clone = state.clone();
        let phone_number = turn.phone_number;
        let content = turn.content;
        let phone_number_id = turn.phone_number_id;
        let message = turn.message;

        tokio::spawn(async move {
            if let Err(e) =
                process_incoming_message(&state_clone, &phone_number, &content, &message, phone_number_id)
                    .await
            {
                log::error!("Failed to process message from {}: {}", phone_number, e);
            }
        });
    }

    (StatusCode::OK, "ok").into_response()
}

#[cfg(test)]
#[path = "webhooks_tests.rs"]
mod webhooks_tests;
