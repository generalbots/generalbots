use std::sync::Arc;

use diesel::prelude::*;
use uuid::Uuid;
use crate::models::WhatsAppMessage;
use crate::state::WhatsAppState;
use crate::utils::{format_phone_number, is_list_message, split_long_message};

pub async fn process_incoming_message(
    state: &Arc<WhatsAppState>,
    phone_number: &str,
    content: &str,
    message: &WhatsAppMessage,
    phone_number_id: Option<String>,
) -> Result<(), String> {
    let formatted_phone = format_phone_number(phone_number);

    log::info!("Processing message from {}: {}", formatted_phone, content);

    let (bot_id, bot_name) = if let Some(ref pni) = phone_number_id {
        (state.find_bot)(pni)
    } else {
        let mut conn = state
            .pool
            .get()
            .map_err(|e| format!("Pool error: {}", e))?;
        let result = (state.get_default_bot)(&mut conn);
        drop(conn);
        result
    };

    let content = if crate::media::select_media(message).is_some() {
        crate::media::message_content(state, bot_id, message).await
    } else {
        content.to_string()
    };

    if content.trim().is_empty() {
        log::info!("Empty message content from {}", formatted_phone);
        return Ok(());
    }

    if is_list_message(&content) {
        log::info!("List message detected from {}", formatted_phone);
    }

    let session_id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, format!("wa-session:{}", formatted_phone).as_bytes());

    match (state.process_message)(
        bot_id.to_string(),
        formatted_phone.clone(),
        content.to_string(),
        session_id.to_string(),
        bot_name,
    )
    .await
    {
        Ok(()) => Ok(()),
        Err(e) => {
            log::error!("Message processing failed for {}: {}", formatted_phone, e);

            let error_msg = "Desculpe, ocorreu um erro ao processar sua mensagem. Tente novamente.";
            (state.send_message)(&formatted_phone, error_msg, &bot_id.to_string())
                .await
                .map_err(|e| format!("Failed to send error message: {}", e))?;

            Err(e)
        }
    }
}

pub fn process_outbound_message(
    state: &Arc<WhatsAppState>,
    msg_user_id: &uuid::Uuid,
    msg_session_id: &uuid::Uuid,
    msg_content: &str,
) {
    use crate::models::NewMessage;

    let mut conn = match state.pool.get() {
        Ok(c) => c,
        Err(e) => {
            log::warn!("Could not save outbound message (pool): {}", e);
            return;
        }
    };

    let new_msg = NewMessage {
        id: uuid::Uuid::new_v4(),
        session_id: *msg_session_id,
        user_id: *msg_user_id,
        role: 2,
        content_encrypted: msg_content.to_string(),
        message_type: 0,
        media_url: None,
        token_count: 0,
        processing_time_ms: None,
        llm_model: None,
        created_at: chrono::Utc::now(),
        message_index: 0,
    };

    if let Err(e) = diesel::insert_into(crate::schema::message_history::table)
        .values(&new_msg)
        .execute(&mut conn)
    {
        log::warn!("Could not save outbound message (non-fatal): {}", e);
    }
}

pub async fn send_outbound_message(
    state: &Arc<WhatsAppState>,
    msg_bot_id: &uuid::Uuid,
    msg_phone: &str,
    msg_content: &str,
) -> Result<(), String> {
    let parts = split_long_message(msg_content);
    for part in parts {
        (state.send_message)(msg_phone, &part, &msg_bot_id.to_string())
            .await
            .map_err(|e| format!("Send error: {}", e))?;
    }

    Ok(())
}
