use crate::core::bot::BotOrchestrator;
use crate::core::bot::channels::telegram::TelegramAdapter;
use crate::core::bot::channels::ChannelAdapter;
use crate::core::shared::models::{BotResponse, UserSession};
use crate::core::shared::state::{AppState, AttendantNotification};
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};

use chrono::Utc;
use diesel::prelude::*;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

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

pub fn configure() -> Router<Arc<AppState>> {
    Router::new()
        .route("/webhook/telegram", post(handle_webhook))
        .route("/api/telegram/send", post(send_message))
}

pub async fn handle_webhook(
    State(state): State<Arc<AppState>>,
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
    state: Arc<AppState>,
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

    let session = find_or_create_session(&state, &chat_id, &user_name).await?;

    let assigned_to = session
        .context_data
        .get("assigned_to")
        .and_then(|v| v.as_str());

    if assigned_to.is_some() {
        route_to_attendant(state.clone(), &session, &content, &chat_id, &user_name).await?;
    } else {
        route_to_bot(state.clone(), &session, &content, &chat_id).await?;
    }

    Ok(())
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

async fn process_callback(
    state: Arc<AppState>,
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

    let session = find_or_create_session(&state, &chat_id, &user_name).await?;

    route_to_bot(state, &session, &data, &chat_id).await?;

    Ok(())
}

async fn find_or_create_session(
    state: &Arc<AppState>,
    chat_id: &str,
    user_name: &str,
) -> Result<UserSession, Box<dyn std::error::Error + Send + Sync>> {
    use crate::core::shared::models::schema::user_sessions::dsl::*;

    let mut conn = state.conn.get()?;

    let telegram_user_uuid = Uuid::new_v5(&Uuid::NAMESPACE_OID, format!("telegram:{}", chat_id).as_bytes());

    let existing: Option<UserSession> = user_sessions
        .filter(user_id.eq(telegram_user_uuid))
        .order(updated_at.desc())
        .first(&mut conn)
        .optional()?;

    if let Some(session) = existing {
        diesel::update(user_sessions.filter(id.eq(session.id)))
            .set(updated_at.eq(Utc::now()))
            .execute(&mut conn)?;
        return Ok(session);
    }

    let bot_uuid = get_default_bot_id(state).await;
    let session_uuid = Uuid::new_v4();

    let context = serde_json::json!({
        "channel": "telegram",
        "chat_id": chat_id,
        "name": user_name,
    });

    let now = Utc::now();

    diesel::insert_into(user_sessions)
        .values((
            id.eq(session_uuid),
            user_id.eq(telegram_user_uuid),
            bot_id.eq(bot_uuid),
            title.eq(format!("Telegram: {}", user_name)),
            context_data.eq(&context),
            created_at.eq(now),
            updated_at.eq(now),
        ))
        .execute(&mut conn)?;

    info!("Created new Telegram session {} for chat_id {}", session_uuid, chat_id);

    let new_session = user_sessions
        .filter(id.eq(session_uuid))
        .first(&mut conn)?;

    Ok(new_session)
}

async fn route_to_bot(
    state: Arc<AppState>,
    session: &UserSession,
    content: &str,
    chat_id: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Routing Telegram message to bot for session {}", session.id);

    let user_message = botlib::models::UserMessage::text(
        session.bot_id.to_string(),
        chat_id.to_string(),
        session.id.to_string(),
        "telegram".to_string(),
        content.to_string(),
    );

    let (tx, mut rx) = tokio::sync::mpsc::channel::<BotResponse>(10);
    let orchestrator = BotOrchestrator::new(state.clone());

    let adapter = TelegramAdapter::new(state.conn.clone(), session.bot_id);
    let chat_id_clone = chat_id.to_string();

    tokio::spawn(async move {
        while let Some(response) = rx.recv().await {
            let tg_response = BotResponse::new(
                response.bot_id,
                response.session_id,
                chat_id_clone.clone(),
                response.content,
                "telegram",
            );

            if let Err(e) = adapter.send_message(tg_response).await {
                error!("Failed to send Telegram response: {}", e);
            }
        }
    });

    if let Err(e) = orchestrator.stream_response(user_message, tx).await {
        error!("Bot processing error: {}", e);

        let adapter = TelegramAdapter::new(state.conn.clone(), session.bot_id);
        let error_response = BotResponse::new(
            session.bot_id.to_string(),
            session.id.to_string(),
            chat_id.to_string(),
            "Sorry, I encountered an error processing your message. Please try again.",
            "telegram",
        );

        if let Err(e) = adapter.send_message(error_response).await {
            error!("Failed to send error response: {}", e);
        }
    }

    Ok(())
}

async fn route_to_attendant(
    state: Arc<AppState>,
    session: &UserSession,
    content: &str,
    chat_id: &str,
    user_name: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!(
        "Routing Telegram message to attendant for session {}",
        session.id
    );

    let assigned_to = session
        .context_data
        .get("assigned_to")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let notification = AttendantNotification {
        notification_type: "message".to_string(),
        session_id: session.id.to_string(),
        user_id: chat_id.to_string(),
        user_name: Some(user_name.to_string()),
        user_phone: Some(chat_id.to_string()),
        channel: "telegram".to_string(),
        content: content.to_string(),
        timestamp: Utc::now().to_rfc3339(),
        assigned_to,
        priority: 1,
    };

    if let Some(broadcast_tx) = state.attendant_broadcast.as_ref() {
        if let Err(e) = broadcast_tx.send(notification.clone()) {
            debug!("No attendants listening: {}", e);
        } else {
            info!("Notification sent to attendants");
        }
    }

    Ok(())
}

async fn get_default_bot_id(state: &Arc<AppState>) -> Uuid {
    use crate::core::shared::models::schema::bots::dsl::*;

    if let Ok(mut conn) = state.conn.get() {
        if let Ok(bot_uuid) = bots
            .filter(is_active.eq(true))
            .select(id)
            .first::<Uuid>(&mut conn)
        {
            return bot_uuid;
        }
    }

    Uuid::parse_str("f47ac10b-58cc-4372-a567-0e02b2c3d480").unwrap_or_else(|_| Uuid::new_v4())
}

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub to: String,
    pub message: String,
}

pub async fn send_message(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SendMessageRequest>,
) -> impl IntoResponse {
    info!("Sending Telegram message to {}", request.to);

    let bot_id = get_default_bot_id(&state).await;
    let adapter = TelegramAdapter::new(state.conn.clone(), bot_id);

    let response = BotResponse::new(
        bot_id.to_string(),
        Uuid::new_v4().to_string(),
        request.to.clone(),
        request.message.clone(),
        "telegram",
    );

    match adapter.send_message(response).await {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "message": "Message sent successfully"
            })),
        ),
        Err(e) => {
            error!("Failed to send Telegram message: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": "Failed to send message"
                })),
            )
        }
    }
}
