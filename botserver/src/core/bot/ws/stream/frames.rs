use serde_json::{json, Value};

use super::LlmTurn;

/// Error frame used when the turn cannot start.
pub(super) fn error(turn: &LlmTurn<'_>, content: String) -> Value {
    json!({
        "bot_id": turn.bot_uuid.to_string(),
        "user_id": turn.user_id.to_string(),
        "session_id": turn.session_id.to_string(),
        "channel": "web",
        "content": content,
        "message_type": 2, "is_complete": true,
        "suggestions": [], "switchers": [],
        "context_length": 0, "context_max_length": 0,
    })
}

/// Opening frame of a streaming turn, carrying the bot stylesheet when there
/// is one.
pub(super) fn init(turn: &LlmTurn<'_>, style_css: &str) -> Value {
    let mut init_msg = json!({
        "bot_id": turn.bot_uuid.to_string(),
        "user_id": turn.user_id.to_string(),
        "session_id": turn.session_id.to_string(),
        "channel": "web",
        "content": "",
        "message_type": 2,
        "is_complete": false,
        "thinking": true,
        "suggestions": [],
        "switchers": [],
        "context_length": 0,
        "context_max_length": 0,
    });
    if !style_css.is_empty() {
        init_msg["css"] = serde_json::Value::String(style_css.to_string());
    }
    init_msg
}

/// Keeps the socket warm while the provider has produced nothing yet.
pub(super) fn keepalive(turn: &LlmTurn<'_>) -> Value {
    json!({
        "bot_id": turn.bot_uuid.to_string(),
        "user_id": turn.user_id.to_string(),
        "session_id": turn.session_id.to_string(),
        "channel": "web",
        "content": "",
        "message_type": 2,
        "is_complete": false,
        "thinking": true,
        "suggestions": [],
        "switchers": [],
        "context_length": 0,
        "context_max_length": 0,
    })
}

/// One content chunk of the answer.
pub(super) fn chunk(turn: &LlmTurn<'_>, content: String) -> Value {
    json!({
        "bot_id": turn.bot_uuid.to_string(),
        "user_id": turn.user_id.to_string(),
        "session_id": turn.session_id.to_string(),
        "channel": "web",
        "content": content,
        "message_type": 2,
        "is_complete": false,
        "suggestions": [],
        "switchers": [],
        "citations": [],
        "context_length": 0,
        "context_max_length": 0,
    })
}

/// Closing frame: the renderer reads `citations` only from here.
pub(super) fn final_frame(
    turn: &LlmTurn<'_>,
    content: String,
    reasoning: &str,
    citations: Vec<Value>,
) -> Value {
    json!({
        "bot_id": turn.bot_uuid.to_string(),
        "user_id": turn.user_id.to_string(),
        "session_id": turn.session_id.to_string(),
        "channel": "web",
        "content": content,
        "message_type": 2,
        "is_complete": true,
        "reasoning": reasoning,
        "suggestions": [],
        "switchers": [],
        "citations": citations,
        "context_length": 0,
        "context_max_length": 0,
    })
}

/// Answer emitted when no LLM provider is configured.
pub(super) fn fallback(turn: &LlmTurn<'_>, content: String) -> Value {
    json!({
        "bot_id": turn.bot_uuid.to_string(),
        "user_id": turn.user_id.to_string(),
        "session_id": turn.session_id.to_string(),
        "channel": "web",
        "content": content,
        "message_type": 2,
        "is_complete": true,
        "suggestions": [],
        "switchers": [],
        "context_length": 0,
        "context_max_length": 0,
    })
}
