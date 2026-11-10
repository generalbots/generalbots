//! Minimal `BotResponse` builder for the generated answers.

use botlib::models::BotResponse;
use uuid::Uuid;

pub(super) fn make_bot_response(
    bot_uuid: Uuid,
    user_id: Uuid,
    session_id: Uuid,
    content: String,
) -> BotResponse {
    BotResponse {
        bot_id: bot_uuid.to_string(),
        user_id: user_id.to_string(),
        session_id: session_id.to_string(),
        channel: "web".to_string(),
        content,
        message_type: botlib::message_types::MessageType(2),
        stream_token: None,
        is_complete: true,
        suggestions: vec![],
        switchers: vec![],
        context_name: None,
        context_length: 0,
        context_max_length: 0,
        reasoning: String::new(),
    }
}
