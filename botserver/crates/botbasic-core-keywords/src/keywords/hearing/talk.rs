use botlib::message_types::MessageType;
use botlib::models::BotResponse;
use botbasic_types::BasicRuntime;
use botbasic_types::UserSession;
use log::{error, info, trace};
use rhai::{Dynamic, Engine};
use std::sync::Arc;

pub async fn execute_talk(
    state: &Arc<dyn BasicRuntime>,
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

    if let Err(e) = state.send_message(&response) {
        error!("Failed to send TALK message: {}", e);
    } else {
        trace!("TALK message sent via runtime adapter");
    }

    Ok(response)
}

pub fn talk_keyword(state: &Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(state);
    let user_clone = user.clone();

    let state_clone2 = Arc::clone(state);
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

                let response = BotResponse {
                    bot_id: user_for_send.bot_id.to_string(),
                    user_id: recipient.clone(),
                    session_id: user_for_send.id.to_string(),
                    channel: "direct".to_string(),
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

                if let Err(e) = state_for_send.send_message(&response) {
                    error!("Failed to send TALK TO message: {}", e);
                }

                Ok(Dynamic::UNIT)
            },
        )
        .expect("valid syntax registration");

    engine
        .register_custom_syntax(["TALK", "$expr$"], true, move |context, inputs| {
            let message = context.eval_expression_tree(&inputs[0])?.to_string();
            let state_for_talk = Arc::clone(&state_clone);
            let user_for_talk = user_clone.clone();

            tokio::spawn(async move {
                if let Err(e) = execute_talk(&state_for_talk, user_for_talk, message).await {
                    error!("Error executing TALK command: {}", e);
                }
            });

            Ok(Dynamic::UNIT)
        })
        .expect("valid syntax registration");
}
