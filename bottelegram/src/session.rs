use crate::adapter::TelegramAdapter;
use crate::channel::ChannelAdapter;
use crate::schema::user_sessions::dsl::*;
use crate::state::{AttendantNotification, ChannelState};
use botlib::models::BotResponse;
use chrono::Utc;
use diesel::prelude::*;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Selectable)]
#[diesel(table_name = crate::schema::user_sessions)]
pub struct UserSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub bot_id: Uuid,
    pub title: String,
    pub context_data: serde_json::Value,
    pub current_tool: Option<String>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

pub fn find_or_create_session(
    state: &Arc<ChannelState>,
    chat_id: &str,
    user_name: &str,
) -> Result<UserSession, Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = state.conn.get()?;

    let telegram_user_uuid =
        Uuid::new_v5(&Uuid::NAMESPACE_OID, format!("telegram:{}", chat_id).as_bytes());

    let existing: Option<UserSession> = user_sessions
        .filter(user_id.eq(telegram_user_uuid))
        .order(crate::schema::user_sessions::updated_at.desc())
        .first(&mut conn)
        .optional()?;

    if let Some(session) = existing {
        diesel::update(user_sessions.filter(crate::schema::user_sessions::id.eq(session.id)))
            .set(crate::schema::user_sessions::updated_at.eq(Utc::now()))
            .execute(&mut conn)?;
        return Ok(session);
    }

    let bot_uuid = (state.get_default_bot)(&mut conn).0;
    let session_uuid = Uuid::new_v4();

    let context = serde_json::json!({
        "channel": "telegram",
        "chat_id": chat_id,
        "name": user_name,
    });

    let now = Utc::now();

    diesel::insert_into(user_sessions)
        .values((
            crate::schema::user_sessions::id.eq(session_uuid),
            user_id.eq(telegram_user_uuid),
            bot_id.eq(bot_uuid),
            title.eq(format!("Telegram: {}", user_name)),
            context_data.eq(&context),
            crate::schema::user_sessions::created_at.eq(now),
            crate::schema::user_sessions::updated_at.eq(now),
        ))
        .execute(&mut conn)?;

    info!(
        "Created new Telegram session {} for chat_id {}",
        session_uuid, chat_id
    );

    let new_session = user_sessions
        .filter(crate::schema::user_sessions::id.eq(session_uuid))
        .first(&mut conn)?;

    Ok(new_session)
}

pub async fn route_to_bot(
    state: Arc<ChannelState>,
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
    let handle = (state.stream_response)(user_message, tx);

    let adapter = TelegramAdapter::new(
        state.conn.clone(),
        session.bot_id,
        state.get_config.clone(),
    );
    let chat_id_clone = chat_id.to_string();

    tokio::spawn(async move {
        let mut accumulated_content = String::new();
        let mut chunk_count = 0u32;

        while let Some(response) = rx.recv().await {
            if !response.content.is_empty() {
                accumulated_content.push_str(&response.content);
                chunk_count += 1;
            }

            if response.is_complete || chunk_count >= 5 {
                if !accumulated_content.is_empty() {
                    let tg_response = BotResponse::new(
                        response.bot_id,
                        response.session_id,
                        chat_id_clone.clone(),
                        accumulated_content.clone(),
                        "telegram",
                    );

                    if let Err(e) = adapter.send_message(tg_response).await {
                        error!("Failed to send Telegram response: {}", e);
                    }

                    accumulated_content.clear();
                    chunk_count = 0;
                }
            }
        }
    });

    if let Err(e) = handle.await {
        error!("Bot processing error: {:?}", e);

        let adapter = TelegramAdapter::new(
            state.conn.clone(),
            session.bot_id,
            state.get_config.clone(),
        );
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

pub fn route_to_attendant(
    state: Arc<ChannelState>,
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
