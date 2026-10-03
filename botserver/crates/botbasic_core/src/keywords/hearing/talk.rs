use botlib::message_types::MessageType;
use botlib::models::BotResponse;
use botbasic_types::BasicRuntime;
use botbasic_types::UserSession;
use log::{info, trace};
use rhai::{Dynamic, Engine};
use std::sync::Arc;

pub fn execute_talk(
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

    let target_user_id = reply_recipient(&user_session, &channel);

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
        reasoning: String::new(),
    };

    if let Err(e) = state.send_message(&response) {
        log::error!("Failed to send TALK message: {}", e);
    } else {
        trace!("TALK message sent via runtime adapter");
    }

    Ok(response)
}

/// Address a reply must carry for the channel adapter to reach the sender.
///
/// `channel_user_id` is the sender as the channel knows it (a Telegram chat id,
/// a WhatsApp phone number) and is what the adapters address replies with.
/// Background runs — a tool triggered by `ON EVENT`, which has no live response
/// channel — depend on it to reach the conversation; sessions built by the
/// message pipeline carry no such value and keep their internal user id, and
/// WhatsApp still falls back to the phone stored in the session context.
fn reply_recipient(user_session: &UserSession, channel: &str) -> String {
    let channel_sender = user_session
        .context_data
        .get("channel_user_id")
        .and_then(|value| value.as_str())
        .filter(|value| !value.is_empty());

    if let Some(sender) = channel_sender {
        return sender.to_string();
    }
    if channel == "whatsapp" {
        return user_session
            .context_data
            .get("phone")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string();
    }
    user_session.user_id.to_string()
}

pub fn talk_keyword(state: &Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(state);
    let user_clone = user.clone();

    let state_clone2 = Arc::clone(state);
    let user_clone2 = user.clone();

    if let Err(e) = engine
        .register_custom_syntax(
            ["TALK", "TO", "$expr$", ",", "$expr$"],
            false,
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
        content: message.replace('\n', "<br>"),
                    message_type: MessageType::BOT_RESPONSE,
                    stream_token: None,
                    is_complete: true,
                    suggestions: Vec::new(),
                    switchers: Vec::new(),
                    context_name: None,
                    context_length: 0,
                    context_max_length: 0,
                    reasoning: String::new(),
                };

                if let Err(e) = state_for_send.send_message(&response) {
                    log::error!("Failed to send TALK TO message: {}", e);
                }

                Ok(Dynamic::UNIT)
            },
        )
    {
        log::error!("Failed to register the custom syntax: {e}");
    }

    if let Err(e) = engine
        .register_custom_syntax(["TALK", "$expr$"], false, move |context, inputs| {
            let message = context.eval_expression_tree(&inputs[0])?.to_string();
            let state_for_talk = Arc::clone(&state_clone);
            let user_for_talk = user_clone.clone();

            if let Err(e) = execute_talk(&state_for_talk, user_for_talk, message) {
                log::error!("Error executing TALK command: {}", e);
            }

            Ok(Dynamic::UNIT)
        })
        {
            log::error!("Failed to register the custom syntax: {e}");
        }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn session(context: serde_json::Value, user_id: Uuid) -> UserSession {
        UserSession {
            id: Uuid::new_v4(),
            user_id,
            branch_id: Uuid::nil(),
            bot_id: Uuid::new_v4(),
            title: String::new(),
            context_data: context,
            current_tool: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn a_channel_event_replies_to_the_sender_the_channel_knows() {
        let user = session(
            serde_json::json!({"channel": "telegram", "channel_user_id": "6676512312"}),
            Uuid::nil(),
        );
        assert_eq!(reply_recipient(&user, "telegram"), "6676512312");
    }

    #[test]
    fn a_pipeline_session_keeps_the_internal_user_id() {
        let user_id = Uuid::new_v4();
        let user = session(serde_json::json!({"channel": "web"}), user_id);
        assert_eq!(reply_recipient(&user, "web"), user_id.to_string());
    }

    #[test]
    fn whatsapp_falls_back_to_the_session_phone() {
        let user = session(
            serde_json::json!({"channel": "whatsapp", "phone": "5511999998888"}),
            Uuid::new_v4(),
        );
        assert_eq!(reply_recipient(&user, "whatsapp"), "5511999998888");
    }

    #[test]
    fn an_empty_sender_never_becomes_the_recipient() {
        let user_id = Uuid::new_v4();
        let user = session(
            serde_json::json!({"channel": "telegram", "channel_user_id": ""}),
            user_id,
        );
        assert_eq!(reply_recipient(&user, "telegram"), user_id.to_string());
    }
}
