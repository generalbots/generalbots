//! What a resumed engine session receives for one turn.
//!
//! Codex and Claude Code keep the conversation in their own thread, so a
//! resumed turn sends only the person's latest message. The turn's system
//! notes (attachment paths and extracted text, pages the person named,
//! workspace and mode notes) are not part of that thread: sent beside the
//! message (as developer instructions, or not at all) they did not reach the
//! model, which then said it never received the PDF or the pasted text. They
//! now travel inside the message, delimited as context.

use serde_json::Value;

const CONTEXT_START: &str = "[Context for this message. These notes come from the assistant's own system \
(files the person attached and their text, pages the person named, the workspace), not from the person. \
Use them to answer; when a note gives a file path, you may open and read that file.]";
const CONTEXT_END: &str = "[End of context]";
const MESSAGE_HEADER: &str = "[The person's message]";
const NOTE_SEPARATOR: &str = "\n\n---\n\n";

fn role(message: &Value) -> &str {
    message.get("role").and_then(Value::as_str).unwrap_or("user")
}

fn content(message: &Value) -> Option<&str> {
    message
        .get("content")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
}

/// The person's latest message: what a resumed thread still lacks.
pub(super) fn latest_message(prompt: &str, config: &Value) -> String {
    config
        .as_array()
        .and_then(|messages| messages.iter().rev().find(|message| role(message) == "user"))
        .and_then(content)
        .unwrap_or(prompt)
        .to_string()
}

/// The persona prompt: the system message the turn starts with.
pub(super) fn persona(config: &Value) -> Option<String> {
    let first = config.as_array()?.first()?;
    (role(first) == "system").then(|| content(first)).flatten().map(str::to_string)
}

/// The system notes of this turn: every system message except the leading
/// persona that is not part of the replayed history. The pipeline adds this
/// turn's notes before the history (persona, mode, workspace), right after it
/// (the attachment text, Drive and page context, which come after the
/// history and before the person's message) and after the message; only
/// system rows inside the replayed history are left out.
pub(super) fn current_notes(config: &Value) -> Vec<&str> {
    let Some(messages) = config.as_array() else {
        return Vec::new();
    };
    let history_start = messages.iter().position(|message| role(message) != "system");
    let latest_user = messages.iter().rposition(|message| role(message) == "user");
    let notes_before_message = latest_user.map(|latest| {
        messages[..latest]
            .iter()
            .rposition(|message| role(message) != "system")
            .map_or(0, |last_history| last_history + 1)
    });
    let skip_persona = messages.first().is_some_and(|message| role(message) == "system");
    messages
        .iter()
        .enumerate()
        .filter(|(index, message)| {
            let before_history = history_start.map_or(true, |start| *index < start);
            let next_to_latest = notes_before_message.is_some_and(|start| *index >= start);
            role(message) == "system" && !(skip_persona && *index == 0) && (before_history || next_to_latest)
        })
        .filter_map(|(_, message)| content(message))
        .collect()
}

/// `latest` preceded by `notes` in one delimited context block; `latest`
/// unchanged when there are none.
pub(super) fn with_notes(latest: &str, notes: &[&str]) -> String {
    if notes.is_empty() {
        return latest.to_string();
    }
    format!(
        "{CONTEXT_START}\n\n{}\n\n{CONTEXT_END}\n\n{MESSAGE_HEADER}\n{latest}",
        notes.join(NOTE_SEPARATOR)
    )
}

/// The message a resumed session receives: the latest message with all of
/// this turn's system notes.
pub(super) fn for_resumed(prompt: &str, config: &Value) -> String {
    with_notes(&latest_message(prompt, config), &current_notes(config))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn turn() -> Value {
        json!([
            {"role": "system", "content": "You are Ana."},
            {"role": "system", "content": "THIS TURN: non-agentic."},
            {"role": "system", "content": "Attached: /opt/vibe/conversations/c/data/r.pdf"},
            {"role": "user", "content": "first"},
            {"role": "system", "content": "an old stored note"},
            {"role": "assistant", "content": "ok"},
            {"role": "system", "content": "[Attachment text] Total: 42"},
            {"role": "user", "content": "what does the report say?"},
            {"role": "system", "content": "[Página lida: example.org] Hello"},
        ])
    }

    /// A long paste: the pipeline's order (persona and workspace notes,
    /// history, the attachment note, the person's message).
    fn long_paste_turn(pasted: &str) -> Value {
        json!([
            {"role": "system", "content": "You are Ana."},
            {"role": "system", "content": "The person attached these files: /opt/vibe/conversations/c/data/texto-colado.txt"},
            {"role": "user", "content": "earlier question"},
            {"role": "assistant", "content": "earlier answer"},
            {"role": "system", "content": format!(
                "The user attached 'texto-colado.txt'.\n\n--- BEGIN ATTACHMENT texto-colado.txt ---\n{pasted}\n--- END ATTACHMENT texto-colado.txt ---"
            )},
            {"role": "user", "content": "Qual é o CODIGO-FINAL no texto colado?"},
        ])
    }

    #[test]
    fn a_long_paste_after_the_history_reaches_the_resumed_message_whole() {
        let mut pasted = String::new();
        let mut line = 0;
        while pasted.chars().count() < 30_000 {
            line += 1;
            pasted.push_str(&format!("Linha {line}: texto de preenchimento para o teste de colagem longa.\n"));
        }
        pasted.push_str("CODIGO-FINAL: PITANGA-5512");
        assert!(pasted.chars().count() > 30_000);
        let message = for_resumed("fallback", &long_paste_turn(&pasted));
        assert!(message.contains("Linha 1: texto"));
        assert!(message.contains("CODIGO-FINAL: PITANGA-5512"));
        assert!(message.contains(&pasted));
        assert!(message.contains("/opt/vibe/conversations/c/data/texto-colado.txt"));
        assert!(!message.contains("earlier answer"));
        assert!(message.ends_with("[The person's message]\nQual é o CODIGO-FINAL no texto colado?"));
        let sent = botagent::codex::app_server::turn_start_with_images("thread", &message, &[], "default");
        let text = sent["input"][0]["text"].as_str().unwrap_or_default();
        assert!(text.contains("CODIGO-FINAL: PITANGA-5512"));
        assert_eq!(text, message);
    }

    #[test]
    fn the_notes_of_this_turn_are_found_on_both_sides_of_the_history() {
        assert_eq!(
            current_notes(&turn()),
            vec![
                "THIS TURN: non-agentic.",
                "Attached: /opt/vibe/conversations/c/data/r.pdf",
                "[Attachment text] Total: 42",
                "[Página lida: example.org] Hello",
            ]
        );
    }

    #[test]
    fn the_resumed_message_carries_every_note_before_the_question() {
        let message = for_resumed("fallback", &turn());
        assert!(message.starts_with(CONTEXT_START));
        assert!(message.contains("/opt/vibe/conversations/c/data/r.pdf"));
        assert!(message.contains("Total: 42"));
        assert!(message.contains("[Página lida: example.org] Hello"));
        assert!(!message.contains("You are Ana."));
        assert!(!message.contains("an old stored note"));
        assert!(message.ends_with("[The person's message]\nwhat does the report say?"));
        let context_end = message.find(CONTEXT_END).unwrap_or(0);
        assert!(message.find("Total: 42").unwrap_or(usize::MAX) < context_end);
    }

    #[test]
    fn a_message_without_notes_is_sent_as_it_is() {
        let config = json!([{"role": "system", "content": "You are Ana."}, {"role": "user", "content": "hi"}]);
        assert_eq!(for_resumed("fallback", &config), "hi");
        assert_eq!(for_resumed("only", &json!(null)), "only");
    }

    #[test]
    fn the_persona_is_the_leading_system_message() {
        assert_eq!(persona(&turn()).as_deref(), Some("You are Ana."));
        assert_eq!(persona(&json!([{"role": "user", "content": "hi"}])), None);
        assert_eq!(persona(&json!(null)), None);
    }

    #[test]
    fn the_latest_user_message_is_picked() {
        assert_eq!(latest_message("fallback", &turn()), "what does the report say?");
        assert_eq!(latest_message("only", &json!(null)), "only");
    }
}
