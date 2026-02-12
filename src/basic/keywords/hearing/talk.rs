use crate::core::shared::message_types::MessageType;
use crate::core::shared::models::{BotResponse, UserSession};
use crate::core::shared::state::AppState;
use log::{error, trace};
use rhai::{Dynamic, Engine};
use std::sync::Arc;

use super::super::universal_messaging::send_message_to_recipient;

pub async fn execute_talk(
    state: Arc<AppState>,
    user_session: UserSession,
    message: String,
) -> Result<BotResponse, Box<dyn std::error::Error + Send + Sync>> {
    let mut suggestions = Vec::new();

    if let Some(redis_client) = &state.cache {
        if let Ok(mut conn) = redis_client.get_multiplexed_async_connection().await {
            let redis_key = format!("suggestions:{}:{}", user_session.user_id, user_session.id);

            let suggestions_json: Result<Vec<String>, _> = redis::cmd("LRANGE")
                .arg(redis_key.as_str())
                .arg(0)
                .arg(-1)
                .query_async(&mut conn)
                .await;

            if let Ok(suggestions_list) = suggestions_json {
                suggestions = suggestions_list
                    .into_iter()
                    .filter_map(|s| serde_json::from_str(&s).ok())
                    .collect();
            }
        }
    }

    let response = BotResponse {
        bot_id: user_session.bot_id.to_string(),
        user_id: user_session.user_id.to_string(),
        session_id: user_session.id.to_string(),
        channel: "web".to_string(),
        content: message,
        message_type: MessageType::BOT_RESPONSE,
        stream_token: None,
        is_complete: true,
        suggestions,
        context_name: None,
        context_length: 0,
        context_max_length: 0,
    };

    let user_id = user_session.id.to_string();
    let response_clone = response.clone();

    let web_adapter = Arc::clone(&state.web_adapter);
    tokio::spawn(async move {
        if let Err(e) = web_adapter
            .send_message_to_session(&user_id, response_clone)
            .await
        {
            error!("Failed to send TALK message via web adapter: {}", e);
        } else {
            trace!("TALK message sent via web adapter");
        }
    });

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
