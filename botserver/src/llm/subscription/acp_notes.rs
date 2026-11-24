//! The system notes an Antigravity (ACP) turn receives.
//!
//! The pipeline's workspace contract tells API models to run code by writing
//! a `vm_exec` call as JSON text. Antigravity has its own tools, and each of
//! them asks through ACP (`session/request_permission`, shown as the card).
//! Given both, it wavered for minutes and then handed the command back as
//! text. Its turns therefore get this note in place of that contract: the
//! conversation folder, "use your own tools, each one asks", and the
//! language of its reasoning and answer, which otherwise is often English.
//! The note sits right after the persona, so a resumed session gets it with
//! the turn's other notes every time.

use serde_json::{json, Value};

/// How the pipeline's workspace contract starts (`agent_vm_context`).
const WORKSPACE_CONTRACT: &str = "You have a running VM workspace '";

fn is_contract(message: &Value) -> bool {
    message.get("role").and_then(Value::as_str) == Some("system")
        && message
            .get("content")
            .and_then(Value::as_str)
            .is_some_and(|text| text.trim_start().starts_with(WORKSPACE_CONTRACT))
}

#[must_use]
pub(super) fn note(conversation: &str) -> String {
    format!(
        "[Antigravity in this workspace] This conversation's folder is /opt/vibe/conversations/{conversation} \
         (data/ holds the person's files, output/ is for files you make). Run commands, read and change \
         files with your own tools: each one asks the person for permission first, the person sees every \
         step, and a refusal means do not do it. Never write a tool call as JSON text (`__tool_call__`) \
         in the answer. Language: write your reasoning (your thoughts) and your answer in the language \
         the person writes in; for Portuguese, Brazilian Portuguese. Do not switch to English on your own."
    )
}

/// `config` (the turn's messages) for Antigravity: the workspace contract
/// replaced by [`note`], placed after the leading persona.
#[must_use]
pub(super) fn for_acp(config: &Value, conversation: &str) -> Value {
    let Some(messages) = config.as_array() else {
        return config.clone();
    };
    let mut kept: Vec<Value> = messages.iter().filter(|message| !is_contract(message)).cloned().collect();
    let after_persona = usize::from(kept.first().is_some_and(|first| {
        first.get("role").and_then(Value::as_str) == Some("system")
    }));
    kept.insert(after_persona, json!({"role": "system", "content": note(conversation)}));
    Value::Array(kept)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn turn() -> Value {
        json!([
            {"role": "system", "content": "You are Ana."},
            {"role": "system", "content": "You have a running VM workspace 'ag-u1' (status: running). Emit {\"__tool_call__\": true}"},
            {"role": "user", "content": "Rode uname -a"},
        ])
    }

    #[test]
    fn antigravity_gets_its_own_tools_note_instead_of_the_vm_exec_contract() {
        let config = for_acp(&turn(), "c1");
        let messages = config.as_array().cloned().unwrap_or_default();
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0]["content"], "You are Ana.");
        let note = messages[1]["content"].as_str().unwrap_or("");
        assert!(note.contains("/opt/vibe/conversations/c1") && note.contains("asks the person for permission"));
        assert!(!config.to_string().contains("running VM workspace"));
        assert_eq!(messages[2]["content"], "Rode uname -a");
    }

    #[test]
    fn the_note_asks_for_reasoning_in_the_persons_language_on_every_turn() {
        let note = note("c1");
        assert!(note.contains("reasoning") && note.contains("Brazilian Portuguese"));
        let resumed = super::super::turn_message::for_resumed("Rode uname -a", &for_acp(&turn(), "c1"));
        assert!(resumed.contains("Brazilian Portuguese") && resumed.ends_with("Rode uname -a"));
        let bare = for_acp(&json!([{"role": "user", "content": "oi"}]), "c1");
        assert_eq!(bare[0]["role"], "system");
        assert_eq!(for_acp(&json!(null), "c1"), json!(null));
    }
}
