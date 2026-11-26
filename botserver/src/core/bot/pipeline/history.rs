use std::sync::Arc;

use botcore::shared::state::AppState;
use uuid::Uuid;

use super::sink::ChannelSink;

/// Replays the session's conversation history as LLM messages, limited by the
/// bot's `history-limit` botconfig (default 10 turns). Each message carries
/// the record of what its turn received or did (`turn_record`).
pub(super) async fn append_conversation_history(
    state: &Arc<AppState>,
    bot_uuid: Uuid,
    session_id: Uuid,
    user_id: Uuid,
    messages: &mut Vec<serde_json::Value>,
) {
    use botcore::config::ConfigManager;
    let cfg = ConfigManager::new(state.conn.clone());
    let history_limit: i64 = cfg
        .get_config(&bot_uuid, "history-limit", Some("10"))
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10);

    let mut sm = state.session_manager.lock().await;
    if let Ok(history) =
        sm.get_conversation_history_raw(session_id, user_id, Some(history_limit), None)
    {
        for (role, stored, _) in history.iter() {
            let content = super::turn_record::replay(role, stored);
            let api_role = match role.as_str() {
                "user" => "user",
                "assistant" | "bot" => "assistant",
                _ => "system",
            };
            messages.push(serde_json::json!({
                "role": api_role,
                "content": content
            }));
        }
    }
}

/// Empty streaming frame that flips the web composer into "streaming" mode
/// before the first LLM chunk arrives.
pub(super) async fn send_stream_start_marker(
    sink: &dyn ChannelSink,
    bot_uuid: Uuid,
    session_id: Uuid,
    user_id: Uuid,
) {
    if let Err(e) = sink
        .send_bot_response(&botlib::models::BotResponse {
            bot_id: bot_uuid.to_string(),
            user_id: user_id.to_string(),
            session_id: session_id.to_string(),
            channel: "web".to_string(),
            content: String::new(),
            message_type: botlib::message_types::MessageType::BOT_RESPONSE,
            stream_token: None,
            is_complete: false,
            suggestions: Vec::new(),
            switchers: Vec::new(),
            context_name: None,
            context_length: 0,
            context_max_length: 0,
            reasoning: String::new(),
        })
        .await
    {
        log::warn!("history: stream-start marker not delivered: {e}");
    }
}
