//! Lean prompt for an engine on this machine.
//!
//! A local engine reads the whole prompt before the first word of the answer,
//! at a few hundred to a few thousand tokens per second. The text
//! description of the structured tools repeats what the tool definitions sent
//! with the same request already say, so a local turn sends only the
//! definitions. Nothing else is removed: the bot's prompt, the app command
//! catalogue, the language rule and the turn context stay as they are.

/// Remove `block` (and the blank line before it) from the first system
/// message. Returns the bytes saved; 0 when the block is not there.
pub fn strip_from_system(messages: &mut serde_json::Value, block: &str) -> usize {
    if block.is_empty() {
        return 0;
    }
    let Some(content) = messages
        .as_array_mut()
        .and_then(|list| list.iter_mut().find(|m| m.get("role").and_then(serde_json::Value::as_str) == Some("system")))
        .and_then(|system| system.get_mut("content"))
    else {
        return 0;
    };
    let Some(text) = content.as_str() else { return 0 };
    let separated = format!("\n\n{block}");
    let target = if text.contains(&separated) { separated.as_str() } else { block };
    if !text.contains(target) {
        return 0;
    }
    let lean = text.replacen(target, "", 1);
    let saved = text.len() - lean.len();
    *content = serde_json::Value::String(lean);
    saved
}

/// Apply the lean profile when the tools travel as definitions.
pub fn apply(messages: &mut serde_json::Value, tools_attached: bool) {
    if !tools_attached {
        return;
    }
    let saved = strip_from_system(messages, super::super::surface_tools::instructions());
    if saved > 0 {
        log::info!("[local_profile] local engine: structured-tool text left to the tool definitions ({saved} bytes saved)");
    }
}

#[cfg(test)]
mod local_profile_tests {
    use super::strip_from_system;

    #[test]
    fn the_block_is_removed_from_the_first_system_message_only() {
        let mut messages = serde_json::json!([
            {"role": "system", "content": "persona\n\nTOOLS BLOCK"},
            {"role": "system", "content": "TOOLS BLOCK"},
            {"role": "user", "content": "Qual a capital do Brasil?"},
        ]);
        assert_eq!(strip_from_system(&mut messages, "TOOLS BLOCK"), 13);
        assert_eq!(messages[0]["content"], "persona");
        assert_eq!(messages[1]["content"], "TOOLS BLOCK");
        assert_eq!(messages[2]["content"], "Qual a capital do Brasil?");
    }

    #[test]
    fn nothing_changes_when_the_block_is_absent() {
        let mut messages = serde_json::json!([
            {"role": "system", "content": "persona"},
            {"role": "user", "content": "oi"},
        ]);
        let before = messages.clone();
        assert_eq!(strip_from_system(&mut messages, "TOOLS BLOCK"), 0);
        assert_eq!(strip_from_system(&mut messages, ""), 0);
        assert_eq!(messages, before);
        assert_eq!(strip_from_system(&mut serde_json::json!([{"role": "user", "content": "x"}]), "x"), 0);
    }
}
