use crate::state::ChannelState;
use crate::session::{find_or_create_session, route_to_attendant, route_to_bot};

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use log::{debug, error, info};
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

fn extract_message_content(message: &TelegramMessage) -> String {
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

pub async fn handle_webhook(
    State(state): State<Arc<ChannelState>>,
    Json(update): Json<TelegramUpdate>,
) -> impl IntoResponse {
    info!("Telegram webhook received: update_id={}", update.update_id);

    if let Some(message) = update.message.or(update.edited_message) {
        if let Err(e) = process_message(state.clone(), &message).await {
            error!("Failed to process Telegram message: {}", e);
        }
    }

    if let Some(callback) = update.callback_query {
        if let Err(e) = process_callback(state.clone(), &callback).await {
            error!("Failed to process Telegram callback: {}", e);
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

    let content = extract_message_content(message);

    if content.is_empty() {
        debug!("Empty message content, skipping");
        return Ok(());
    }

    info!(
        "Processing Telegram message from {} (chat_id={}): {}",
        user_name,
        chat_id,
        if content.len() > 50 { &content[..50] } else { &content }
    );

    let session = find_or_create_session(&state, &chat_id, &user_name)?;

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
