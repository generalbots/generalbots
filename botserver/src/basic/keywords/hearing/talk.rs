use crate::core::shared::message_types::MessageType;
use crate::core::shared::models::{BotResponse, UserSession};
use crate::core::shared::state::AppState;
use log::{error, info, trace};
use rhai::{Dynamic, Engine};
use std::sync::Arc;

use super::super::universal_messaging::send_message_to_recipient;

pub async fn execute_talk(
    state: Arc<AppState>,
    user_session: UserSession,
    message: String,
) -> Result<BotResponse, Box<dyn std::error::Error + Send + Sync>> {
    info!("TALK called with message: {}", message);

    let channel = user_session
        .context_data
        .get("channel")
        .and_then(|v| v.as_str())
        .unwrap_or("web")
        .to_string();

    let target_user_id = if channel == "whatsapp" {
        user_session
            .context_data
            .get("phone")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    } else {
        user_session.user_id.to_string()
    };

    let response = BotResponse {
        bot_id: user_session.bot_id.to_string(),
        user_id: target_user_id.clone(),
        session_id: user_session.id.to_string(),
        channel: channel.clone(),
        content: message,
        message_type: MessageType::BOT_RESPONSE,
        stream_token: None,
        is_complete: true,
        suggestions: Vec::new(),
        switchers: Vec::new(),
        context_name: None,
        context_length: 0,
        context_max_length: 0,
    };

    let response_clone = response.clone();

    if channel == "whatsapp" {
        use crate::core::bot::channels::ChannelAdapter;
        use crate::core::bot::channels::whatsapp::WhatsAppAdapter;
        
        // WhatsApp expects the phone number as the target
        let mut wa_response = response_clone;
        wa_response.user_id = target_user_id;

        let bot_id = user_session.bot_id;
        
        tokio::spawn(async move {
            let adapter = WhatsAppAdapter::new(&state, bot_id);
            if let Err(e) = adapter.send_message(wa_response).await {
                error!("Failed to send TALK message via whatsapp adapter: {}", e);
            } else {
                trace!("TALK message sent via whatsapp adapter");
            }
        });
    } else {
        // Use WebSocket session_id from context if available, otherwise fall back to session.id
        let target_session_id = user_session
            .context_data
            .get("websocket_session_id")
            .and_then(|v| v.as_str())
            .unwrap_or(&user_session.id.to_string())
            .to_string();
        
        let web_adapter = Arc::clone(&state.web_adapter);
        
        tokio::spawn(async move {
            if let Err(e) = web_adapter
                .send_message_to_session(&target_session_id, response_clone)
                .await
            {
                trace!("No WebSocket connection for session {}: {}", target_session_id, e);
            } else {
                trace!("TALK message sent via web adapter to session {}", target_session_id);
            }
        });
    }

    Ok(response)
}

pub fn talk_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    // Register TALK TO "recipient", "message" syntax FIRST (more specific pattern)
    let state_clone2 = Arc::clone(&state);
    let user_clone2 = user.clone();

    engine
        .register_custom_syntax(
            ["TALK", "TO", "$expr$", ",", "$expr$"],
            true,
            move |context, inputs| {
                let recipient = context.eval_expression_tree(&inputs[0])?.to_string();
                let message = context.eval_expression_tree(&inputs[1])?.to_string();

                trace!("TALK TO: Sending message to {}", recipient);

                let state_for_send = Arc::clone(&state_clone2);
                let user_for_send = user_clone2.clone();

                tokio::spawn(async move {
                    if let Err(e) =
                        send_message_to_recipient(state_for_send, &user_for_send, &recipient, &message)
                            .await
                    {
                        error!("Failed to send TALK TO message: {}", e);
                    }
                });

                Ok(Dynamic::UNIT)
            },
        )
        .expect("valid syntax registration");

    // Register simple TALK "message" syntax SECOND (fallback pattern)
    engine
        .register_custom_syntax(["TALK", "$expr$"], true, move |context, inputs| {
            let message = context.eval_expression_tree(&inputs[0])?.to_string();
            let state_for_talk = Arc::clone(&state_clone);
            let user_for_talk = user_clone.clone();

            tokio::spawn(async move {
                if let Err(e) = execute_talk(state_for_talk, user_for_talk, message).await {
                    error!("Error executing TALK command: {}", e);
                }
            });

            Ok(Dynamic::UNIT)
        })
        .expect("valid syntax registration");
}
