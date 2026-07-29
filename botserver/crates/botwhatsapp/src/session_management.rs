use std::sync::Arc;

use diesel::prelude::*;
use log::info;
use uuid::Uuid;

use crate::models::{User, UserSession};
use crate::schema::{user_sessions, users};
use crate::state::WhatsAppState;

pub async fn find_or_create_session(
    state: &Arc<WhatsAppState>,
    phone_number: &str,
    bot_id: &Uuid,
) -> Result<(Uuid, Uuid), String> {
    let user_id = find_or_create_user(state, phone_number).await?;

    let mut conn = state
        .pool
        .get()
        .map_err(|e| format!("Pool error: {}", e))?;

    let user_id_str = user_id.to_string();
    let existing_session: Option<Uuid> = user_sessions::table
        .filter(user_sessions::user_id.eq(&user_id_str))
        .filter(user_sessions::bot_id.eq(*bot_id))
        .order(user_sessions::id.desc())
        .select(user_sessions::id)
        .first::<Uuid>(&mut conn)
        .ok();

    if let Some(session_id) = existing_session {
        return Ok((session_id, user_id));
    }

    let new_session_id = Uuid::new_v4();
    let now = chrono::Utc::now();
    diesel::insert_into(user_sessions::table)
        .values((
            user_sessions::id.eq(new_session_id),
            user_sessions::user_id.eq(user_id_str),
            user_sessions::bot_id.eq(*bot_id),
            user_sessions::title.eq(""),
            user_sessions::answer_mode.eq(0),
            user_sessions::context_data.eq(serde_json::json!({"channel": "whatsapp"})),
            user_sessions::message_count.eq(0),
            user_sessions::total_tokens.eq(0),
            user_sessions::created_at.eq(now),
            user_sessions::updated_at.eq(now),
            user_sessions::last_activity.eq(now),
            user_sessions::session_id.eq(new_session_id.to_string()),
        ))
        .execute(&mut conn)
        .map_err(|e| format!("Insert session error: {}", e))?;

    info!(
        "Created new session {} for user {} on bot {}",
        new_session_id, user_id, bot_id
    );

    Ok((new_session_id, user_id))
}

async fn find_or_create_user(
    state: &Arc<WhatsAppState>,
    phone_number: &str,
) -> Result<Uuid, String> {
    if let Ok(Some(zitadel_user_id)) = (state.user_lookup)(phone_number).await {
        let parsed = Uuid::parse_str(&zitadel_user_id)
            .map_err(|e| format!("Invalid Zitadel user ID format: {}", e))?;
        return Ok(parsed);
    }

    let zitadel_id = match (state.user_create)(phone_number, "WhatsApp", "User", Some(phone_number)).await {
        Ok(zitadel_user_id) => {
            let parsed = Uuid::parse_str(&zitadel_user_id)
                .map_err(|e| format!("Invalid Zitadel user ID format: {}", e))?;
            info!("Created Zitadel user {} for phone {}", parsed, phone_number);
            parsed
        }
        Err(e) => {
            log::warn!("Failed to create Zitadel user for phone {}: {}. Falling back to local database.", phone_number, e);
            Uuid::nil()
        }
    };

    let mut conn = state
        .pool
        .get()
        .map_err(|e| format!("Pool error: {}", e))?;

    let existing_user: Option<Uuid> = users::table
        .filter(users::phone_number.eq(phone_number))
        .select(users::id)
        .first::<Uuid>(&mut conn)
        .ok();

    if let Some(user_id) = existing_user {
        return Ok(user_id);
    }

    let new_user_id = if zitadel_id.is_nil() { Uuid::new_v4() } else { zitadel_id };
    let now = chrono::Utc::now();
    diesel::insert_into(users::table)
        .values((
            users::id.eq(new_user_id),
            users::username.eq(phone_number),
            users::phone_number.eq(Some(phone_number)),
            users::email.eq(""),
            users::password_hash.eq(""),
            users::is_active.eq(true),
            users::created_at.eq(now),
            users::updated_at.eq(now),
        ))
        .execute(&mut conn)
        .map_err(|e| format!("Insert user error: {}", e))?;

    info!("Created local user {} for phone {}", new_user_id, phone_number);

    Ok(new_user_id)
}

pub fn get_bot_for_phone(
    state: &Arc<WhatsAppState>,
    _phone_number: &str,
) -> Result<(Uuid, String), String> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| format!("Pool error: {}", e))?;

    let (bot_id, bot_name) = (state.get_default_bot)(&mut conn);
    Ok((bot_id, bot_name))
}

pub fn get_user_session(
    state: &Arc<WhatsAppState>,
    session_id: Uuid,
) -> Result<UserSession, String> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| format!("Pool error: {}", e))?;

    user_sessions::table
        .find(session_id)
        .first::<UserSession>(&mut conn)
        .map_err(|e| format!("Session not found: {}", e))
}

pub fn get_user_by_phone(
    state: &Arc<WhatsAppState>,
    phone_number: &str,
) -> Result<Option<User>, String> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| format!("Pool error: {}", e))?;

    users::table
        .filter(users::phone_number.eq(phone_number))
        .first::<User>(&mut conn)
        .optional()
        .map_err(|e| format!("Query error: {}", e))
}

pub fn end_session(
    state: &Arc<WhatsAppState>,
    session_id: Uuid,
) -> Result<(), String> {
    let mut conn = state
        .pool
        .get()
        .map_err(|e| format!("Pool error: {}", e))?;

    diesel::delete(user_sessions::table.find(session_id))
        .execute(&mut conn)
        .map_err(|e| format!("Delete session error: {}", e))?;

    info!("Ended session {}", session_id);
    Ok(())
}
