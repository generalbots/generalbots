use crate::adapter::TelegramAdapter;
use crate::media::message_content;
use crate::state::ChannelState;
use crate::session::{find_or_create_session, resolve_bot_scope, route_to_attendant, route_to_bot};

use axum::{
    extract::State,
    http::HeaderMap,
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramUpdate {
    pub update_id: i64,
    #[serde(default)]
    pub message: Option<TelegramMessage>,
    #[serde(default)]
    pub edited_message: Option<TelegramMessage>,
    #[serde(default)]
    pub callback_query: Option<TelegramCallbackQuery>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramMessage {
    pub message_id: i64,
    pub from: Option<TelegramUser>,
    pub chat: TelegramChat,
    pub date: i64,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub photo: Option<Vec<TelegramPhotoSize>>,
    #[serde(default)]
    pub document: Option<TelegramDocument>,
    #[serde(default)]
    pub voice: Option<TelegramVoice>,
    #[serde(default)]
    pub audio: Option<TelegramAudio>,
    #[serde(default)]
    pub video: Option<TelegramVideo>,
    #[serde(default)]
    pub location: Option<TelegramLocation>,
    #[serde(default)]
    pub contact: Option<TelegramContact>,
    #[serde(default)]
    pub caption: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramUser {
    pub id: i64,
    pub is_bot: bool,
    pub first_name: String,
    #[serde(default)]
    pub last_name: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub language_code: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramChat {
    pub id: i64,
    #[serde(rename = "type")]
    pub chat_type: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramPhotoSize {
    pub file_id: String,
    pub file_unique_id: String,
    pub width: i32,
    pub height: i32,
    #[serde(default)]
    pub file_size: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramDocument {
    pub file_id: String,
    pub file_unique_id: String,
    #[serde(default)]
    pub file_name: Option<String>,
    #[serde(default)]
    pub mime_type: Option<String>,
    #[serde(default)]
    pub file_size: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramVoice {
    pub file_id: String,
    pub file_unique_id: String,
    pub duration: i32,
    #[serde(default)]
    pub mime_type: Option<String>,
    #[serde(default)]
    pub file_size: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramAudio {
    pub file_id: String,
    pub file_unique_id: String,
    pub duration: i32,
    #[serde(default)]
    pub performer: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub mime_type: Option<String>,
    #[serde(default)]
    pub file_size: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramVideo {
    pub file_id: String,
    pub file_unique_id: String,
    pub width: i32,
    pub height: i32,
    pub duration: i32,
    #[serde(default)]
    pub mime_type: Option<String>,
    #[serde(default)]
    pub file_size: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramLocation {
    pub longitude: f64,
    pub latitude: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramContact {
    pub phone_number: String,
    pub first_name: String,
    #[serde(default)]
    pub last_name: Option<String>,
    #[serde(default)]
    pub user_id: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramCallbackQuery {
    pub id: String,
    pub from: TelegramUser,
    #[serde(default)]
    pub message: Option<TelegramMessage>,
    #[serde(default)]
    pub data: Option<String>,
}

pub(crate) fn extract_message_content(message: &TelegramMessage) -> String {
    if let Some(text) = &message.text {
        return text.clone();
    }
    if let Some(caption) = &message.caption {
        return caption.clone();
    }
    if message.photo.is_some() {
        return "[Photo received]".to_string();
    }
    if message.document.is_some() {
        return "[Document received]".to_string();
    }
    if message.voice.is_some() {
        return "[Voice message received]".to_string();
    }
    if message.audio.is_some() {
        return "[Audio received]".to_string();
    }
    if message.video.is_some() {
        return "[Video received]".to_string();
    }
    if let Some(location) = &message.location {
        return format!("[Location: {}, {}]", location.latitude, location.longitude);
    }
    if let Some(contact) = &message.contact {
        return format!("[Contact: {} {}]", contact.first_name, contact.phone_number);
    }
    String::new()
}

pub fn configure() -> Router<Arc<ChannelState>> {
    Router::new()
        .route("/webhook/telegram", post(handle_webhook))
        .route("/api/telegram/send", post(crate::handlers::send_message))
}

/// Header Telegram sends on every delivery once the webhook is registered with
/// `secret_token` (Bot API `setWebhook`).
const SECRET_HEADER: &str = "x-telegram-bot-api-secret-token";

/// Bot config key holding the expected secret token. When it is absent or
/// empty the endpoint accepts unsigned deliveries, which is the behaviour for
/// every existing installation and the documented default.
const SECRET_CONFIG_KEY: &str = "telegram-webhook-secret";

/// Outcome of the webhook authenticity gate.
enum WebhookGate {
    Allowed,
    Rejected(StatusCode),
}

/// A webhook is accepted when no secret is configured, or when the delivered
/// header matches the configured secret exactly. An empty delivery never
/// satisfies a configured secret.
fn secret_matches(expected: &str, provided: &str) -> bool {
    let expected = expected.trim();
    if expected.is_empty() {
        return true;
    }
    !provided.is_empty() && provided.trim() == expected
}

/// Reads the configured secret from the default bot and compares it with the
/// delivered header. Returns `500` when the configuration cannot be read: the
/// message would fail later anyway (the session needs the same database), and
/// failing closed keeps a spoofed delivery out during a database outage.
fn verify_webhook_secret(state: &Arc<ChannelState>, headers: &HeaderMap) -> WebhookGate {
    let mut conn = match state.conn.get() {
        Ok(conn) => conn,
        Err(e) => {
            log::error!("Telegram webhook secret check failed, no database connection: {e}");
            return WebhookGate::Rejected(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let (bot_id, _) = resolve_bot_scope(state, &mut conn);
    let expected = (state.get_config)(&bot_id, SECRET_CONFIG_KEY, None).unwrap_or_default();

    let provided = headers
        .get(SECRET_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();

    if secret_matches(&expected, provided) {
        WebhookGate::Allowed
    } else {
        warn!(
            "Telegram webhook delivery for bot {bot_id} rejected: {SECRET_HEADER} missing or mismatched"
        );
        WebhookGate::Rejected(StatusCode::UNAUTHORIZED)
    }
}

pub async fn handle_webhook(
    State(state): State<Arc<ChannelState>>,
    headers: HeaderMap,
    Json(update): Json<TelegramUpdate>,
) -> impl IntoResponse {
    match verify_webhook_secret(&state, &headers) {
        WebhookGate::Allowed => {}
        WebhookGate::Rejected(status) => return status,
    }

    info!("Telegram webhook received: update_id={}", update.update_id);

    if let Some(message) = update.message.or(update.edited_message) {
        if let Err(e) = process_message(state.clone(), &message).await {
            log::error!("Failed to process Telegram message: {}", e);
        }
    }

    if let Some(callback) = update.callback_query {
        if let Err(e) = process_callback(state.clone(), &callback).await {
            log::error!("Failed to process Telegram callback: {}", e);
        }
    }

    StatusCode::OK
}

async fn process_message(
    state: Arc<ChannelState>,
    message: &TelegramMessage,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let chat_id = message.chat.id.to_string();
    let user = message.from.as_ref();

    let user_name = user
        .map(|u| {
            let mut name = u.first_name.clone();
            if let Some(last) = &u.last_name {
                name.push(' ');
                name.push_str(last);
            }
            name
        })
        .unwrap_or_else(|| "Unknown".to_string());

    let session = find_or_create_session(&state, &chat_id, &user_name)?;

    let adapter = TelegramAdapter::new(
        state.conn.clone(),
        session.bot_id,
        state.get_config.clone(),
    );

    // Media is fetched into Drive before routing, so the bot receives the
    // stored path instead of a placeholder.
    let content = message_content(&state, &adapter, message, session.bot_id).await;

    if content.is_empty() {
        debug!("Empty message content, skipping");
        return Ok(());
    }

    let preview: String = content.chars().take(50).collect();

    info!(
        "Processing Telegram message from {} (chat_id={}): {}",
        user_name, chat_id, preview
    );

    let assigned_to = session
        .context_data
        .get("assigned_to")
        .and_then(|v| v.as_str());

    if assigned_to.is_some() {
        route_to_attendant(state, &session, &content, &chat_id, &user_name)?;
    } else {
        route_to_bot(state, &session, &content, &chat_id).await?;
    }

    Ok(())
}

async fn process_callback(
    state: Arc<ChannelState>,
    callback: &TelegramCallbackQuery,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let chat_id = callback
        .message
        .as_ref()
        .map(|m| m.chat.id.to_string())
        .unwrap_or_default();

    let user_name = {
        let mut name = callback.from.first_name.clone();
        if let Some(last) = &callback.from.last_name {
            name.push(' ');
            name.push_str(last);
        }
        name
    };

    let data = callback.data.clone().unwrap_or_default();

    if data.is_empty() || chat_id.is_empty() {
        return Ok(());
    }

    info!(
        "Processing Telegram callback from {} (chat_id={}): {}",
        user_name, chat_id, data
    );

    let session = find_or_create_session(&state, &chat_id, &user_name)?;

    route_to_bot(state, &session, &data, &chat_id).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{secret_matches, SECRET_HEADER};

    #[test]
    fn unsigned_delivery_is_accepted_when_no_secret_is_configured() {
        assert!(secret_matches("", ""));
        assert!(secret_matches("   ", ""));
        assert!(secret_matches("", "anything"));
    }

    #[test]
    fn configured_secret_requires_the_exact_header_value() {
        assert!(secret_matches("s3cret", "s3cret"));
        assert!(secret_matches(" s3cret ", "s3cret"));
        assert!(!secret_matches("s3cret", ""));
        assert!(!secret_matches("s3cret", "s3cre"));
        assert!(!secret_matches("s3cret", "S3CRET"));
    }

    #[test]
    fn secret_header_name_matches_the_bot_api() {
        assert_eq!(SECRET_HEADER, "x-telegram-bot-api-secret-token");
    }
}
