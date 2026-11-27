//! Message order for the engine on this machine.
//!
//! The engine keeps the prompt of the previous request and only reads again
//! what changed after the longest common start. The pipeline sends one stable
//! system prompt followed by system blocks that change every turn (the turn
//! instruction, the memory note, web and knowledge results). Merging all of
//! them into the leading system message put that changing text ahead of the
//! history, so the whole history was read again on every turn.
//!
//! The chat templates of recent local models (Qwen3.5 among them) refuse a
//! system message that is not the first one ("System message must be at the
//! beginning."), so the changing blocks go at the start of the last user
//! message instead: the system prompt and the history before it stay
//! identical from one turn to the next.

use serde_json::Value;

fn role(message: &Value) -> Option<&str> {
    message.get("role").and_then(Value::as_str)
}

fn text_of(message: &Value) -> Option<String> {
    match message.get("content") {
        Some(Value::String(text)) => Some(text.trim().to_string()),
        Some(Value::Null) | None => None,
        Some(other) => Some(other.to_string()),
    }
    .filter(|text| !text.is_empty())
}

/// Every system message merged into one, placed first; the rest keeps its
/// order. Used when the turn blocks cannot ride with the last user message.
pub fn merge_system_first(messages: &Value) -> Value {
    let Some(array) = messages.as_array() else {
        return messages.clone();
    };
    let parts: Vec<String> = array
        .iter()
        .filter(|m| role(m) == Some("system"))
        .filter_map(text_of)
        .collect();
    let mut result = Vec::with_capacity(array.len());
    if !parts.is_empty() {
        result.push(serde_json::json!({ "role": "system", "content": parts.join("\n\n") }));
    }
    result.extend(array.iter().filter(|m| role(m) != Some("system")).cloned());
    Value::Array(result)
}

/// Whether the turn blocks can move into the last user message: the request
/// starts with a system prompt, ends with a plain-text user message, and no
/// later system block carries files.
fn can_move_turn_blocks(array: &[Value]) -> bool {
    let starts_with_prompt = array.first().is_some_and(|m| role(m) == Some("system") && text_of(m).is_some());
    let ends_with_user_text = array
        .last()
        .is_some_and(|m| role(m) == Some("user") && m.get("content").is_some_and(Value::is_string));
    let no_media = array
        .iter()
        .skip(1)
        .filter(|m| role(m) == Some("system"))
        .all(|m| m.get("media").is_none());
    starts_with_prompt && ends_with_user_text && no_media
}

/// One leading system message for the local engine: the first system prompt
/// stays first, every later system block is placed, in order, at the start
/// of the last user message. Falls back to [`merge_system_first`].
pub fn order_for_prompt_cache(messages: &Value) -> Value {
    let Some(array) = messages.as_array() else {
        return messages.clone();
    };
    if !can_move_turn_blocks(array) {
        return merge_system_first(messages);
    }
    let blocks: Vec<String> = array
        .iter()
        .skip(1)
        .filter(|m| role(m) == Some("system"))
        .filter_map(text_of)
        .collect();
    let mut result: Vec<Value> = Vec::with_capacity(array.len());
    let first_text = text_of(&array[0]).unwrap_or_else(|| {
        log::debug!("local prompt cache: the leading system message has no text");
        String::new()
    });
    result.push(serde_json::json!({ "role": "system", "content": first_text }));
    result.extend(array.iter().skip(1).filter(|m| role(m) != Some("system")).cloned());
    if let (false, Some(last)) = (blocks.is_empty(), result.last_mut()) {
        let question = last.get("content").and_then(Value::as_str).unwrap_or_else(|| {
            log::debug!("local prompt cache: the last message has no text content");
            ""
        });
        last["content"] = Value::String(format!("{}\n\n{question}", blocks.join("\n\n")));
    }
    Value::Array(result)
}

#[cfg(test)]
#[path = "turn_order_tests.rs"]
mod turn_order_tests;
