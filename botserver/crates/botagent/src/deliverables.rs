//! Where the subscription agents (Codex, Claude Code, Antigravity) put the
//! files they make for the person: the conversation's `output/` folder, the
//! one published to the person's Drive after the turn.
//!
//! The conversation folders belong to `runner` (the identity that runs
//! `vm_exec` programs); each agent CLI runs as its own user. Before the CLI
//! starts, root creates `output/` and lets every agent identity of this
//! container write into it, and the agent is told to save there instead of
//! its home folder.

use serde_json::{json, Value};

const CONVERSATIONS: &str = "/opt/vibe/conversations";

/// The conversation's `output/` folder (`conversation` already sanitized).
#[must_use]
pub fn output_dir(conversation: &str) -> String {
    format!("{CONVERSATIONS}/{conversation}/output")
}

/// Root shell step that creates `output/` and makes it writable by the agent
/// users. It never fails the script it is part of.
#[must_use]
pub fn writable_output(conversation: &str) -> String {
    let dir = output_dir(conversation);
    format!("{{ mkdir -p {dir} && chmod 0777 {dir}; }} 2>/dev/null || true")
}

/// The note that tells an agent where files for the person go.
#[must_use]
pub fn note(conversation: &str) -> String {
    let dir = output_dir(conversation);
    format!(
        "[Files for the person] Save every file you make for the person (documents, slides, \
         spreadsheets, images, code they asked for) in {dir}/, with its final name. Only files in \
         that folder reach the person and their Drive, and only links to them work: do not save \
         deliverables in your home folder, in /mnt/data or anywhere else."
    )
}

/// `config` (the turn's messages) with [`note`] as a system message right
/// after the persona, so a new session reads it in the transcript and a
/// resumed one with this turn's notes.
#[must_use]
pub fn with_note(config: &Value, conversation: &str) -> Value {
    let Some(messages) = config.as_array() else {
        return config.clone();
    };
    let mut kept = messages.clone();
    let after_persona = usize::from(
        kept.first()
            .is_some_and(|first| first.get("role").and_then(Value::as_str) == Some("system")),
    );
    kept.insert(after_persona, json!({"role": "system", "content": note(conversation)}));
    Value::Array(kept)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_output_folder_is_made_writable_without_failing_the_script() {
        let step = writable_output("c1");
        assert_eq!(
            step,
            "{ mkdir -p /opt/vibe/conversations/c1/output && chmod 0777 /opt/vibe/conversations/c1/output; } 2>/dev/null || true"
        );
    }

    #[test]
    fn the_note_follows_the_persona_and_names_the_folder() {
        let config = json!([
            {"role": "system", "content": "You are Ana."},
            {"role": "user", "content": "Faça um memorando em Word"},
        ]);
        let noted = with_note(&config, "c1");
        let messages = noted.as_array().cloned().unwrap_or_default();
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0]["content"], "You are Ana.");
        let text = messages[1]["content"].as_str().unwrap_or_default();
        assert!(text.contains("/opt/vibe/conversations/c1/output/"), "got: {text}");
        assert_eq!(messages[2]["role"], "user");
        let bare = with_note(&json!([{"role": "user", "content": "oi"}]), "c1");
        assert_eq!(bare[0]["role"], "system");
    }
}
