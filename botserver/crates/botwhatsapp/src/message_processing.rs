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
    _message: &WhatsAppMessage,
    phone_number_id: Option<String>,
) -> Result<(), String> {
    let formatted_phone = format_phone_number(phone_number);

    log::info!("Processing message from {}: {}", formatted_phone, content);

    if is_list_message(content) {
        log::info!("List message detected from {}", formatted_phone);
    }

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
