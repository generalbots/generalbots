use std::sync::Arc;

use diesel::prelude::*;

use crate::models::WhatsAppMessage;
use crate::state::WhatsAppState;
use crate::session_management::find_or_create_session;
use crate::utils::{format_phone_number, is_list_message, split_long_message};

pub async fn process_incoming_message(
    state: &Arc<WhatsAppState>,
    phone_number: &str,
    content: &str,
    _message: &WhatsAppMessage,
) -> Result<(), String> {
    let formatted_phone = format_phone_number(phone_number);

    log::info!("Processing message from {}: {}", formatted_phone, content);

    let mut conn = state
        .pool
        .get()
        .map_err(|e| format!("Pool error: {}", e))?;

    let (bot_id, _bot_name) = (state.get_default_bot)(&mut conn);

    drop(conn);

    let session_id = find_or_create_session(state, &formatted_phone, &bot_id).await?;

    save_incoming_message(state, &bot_id, &formatted_phone, &session_id, content)?;

    if is_list_message(content) {
        log::info!("List message detected from {}", formatted_phone);
    }

    match (state.process_message)(
        bot_id.to_string(),
        formatted_phone.clone(),
        content.to_string(),
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

fn save_incoming_message(
    state: &Arc<WhatsAppState>,
    msg_bot_id: &uuid::Uuid,
    msg_phone: &str,
    msg_session_id: &uuid::Uuid,
    msg_content: &str,
) -> Result<(), String> {
    use crate::models::NewMessage;

    let mut conn = state
        .pool
        .get()
        .map_err(|e| format!("Pool error: {}", e))?;

    let new_msg = NewMessage {
        id: uuid::Uuid::new_v4(),
        bot_id: *msg_bot_id,
        user_id: None,
        session_id: Some(*msg_session_id),
        phone_number: Some(msg_phone.to_string()),
        direction: "inbound".to_string(),
        content: msg_content.to_string(),
        message_type: Some("text".to_string()),
    };

    diesel::insert_into(crate::schema::message_history::table)
        .values(&new_msg)
        .execute(&mut conn)
        .map_err(|e| format!("Insert error: {}", e))?;

    Ok(())
}

pub fn process_outbound_message(
    state: &Arc<WhatsAppState>,
    msg_bot_id: &uuid::Uuid,
    msg_phone: &str,
    msg_session_id: &uuid::Uuid,
    msg_content: &str,
) -> Result<(), String> {
    use crate::models::NewMessage;

    let mut conn = state
        .pool
        .get()
        .map_err(|e| format!("Pool error: {}", e))?;

    let new_msg = NewMessage {
        id: uuid::Uuid::new_v4(),
        bot_id: *msg_bot_id,
        user_id: None,
        session_id: Some(*msg_session_id),
        phone_number: Some(msg_phone.to_string()),
        direction: "outbound".to_string(),
        content: msg_content.to_string(),
        message_type: Some("text".to_string()),
    };

    diesel::insert_into(crate::schema::message_history::table)
        .values(&new_msg)
        .execute(&mut conn)
        .map_err(|e| format!("Insert error: {}", e))?;

    Ok(())
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

pub async fn process_audio_message(
    state: &Arc<WhatsAppState>,
    phone_number: &str,
    audio_id: &str,
) -> Result<String, String> {
    let audio_data = download_media(state, audio_id).await?;

    let transcription = (state.transcribe_audio)(&audio_data)
        .await
        .map_err(|e| format!("Transcription error: {}", e))?;

    log::info!("Audio transcribed for {}: {} chars", phone_number, transcription.len());
    Ok(transcription)
}

async fn download_media(
    state: &Arc<WhatsAppState>,
    media_id: &str,
) -> Result<Vec<u8>, String> {
    let _api_url = (state.get_config)("whatsapp_api_url").unwrap_or_else(|_| "https://graph.facebook.com/v18.0".to_string());
    let _token = (state.secrets)("whatsapp_api_key").unwrap_or_default();

    log::info!("Media download requested for id: {}", media_id);

    Err("Media download not implemented in standalone crate".to_string())
}
