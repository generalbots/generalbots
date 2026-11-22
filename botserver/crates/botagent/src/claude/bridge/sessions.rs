//! Claude's session id per conversation, kept inside the workspace next to
//! Claude's own session store (`~claude/.claude/gb-sessions/<conversation>`),
//! so a server restart still resumes the conversation and a recreated
//! workspace loses both together. The ids are read and written by short
//! shell sessions of their own; the turn's session only receives `--resume`.

use std::time::Duration;

use super::command::clean_session_id;
use crate::claude::{CLAUDE_HOME, CLAUDE_USER};
use crate::{vm, AgentService};

const STORE_TIMEOUT: Duration = Duration::from_secs(60);

fn sessions_dir() -> String {
    format!("{CLAUDE_HOME}/.claude/gb-sessions")
}

pub(super) fn read_script(conversation: &str) -> String {
    format!("head -c 80 {}/{conversation} 2>/dev/null; true", sessions_dir())
}

pub(super) fn store_script(conversation: &str, session_id: &str) -> String {
    let dir = sessions_dir();
    format!(
        "umask 077; mkdir -p {dir} && printf '%s' {session_id} > {dir}/{conversation} && \
         chown -R {CLAUDE_USER}:{CLAUDE_USER} {dir}"
    )
}

/// The session id stored for `conversation`, if the workspace has one.
pub async fn stored_session(state: &AgentService, vm_name: &str, conversation: &str) -> Option<String> {
    let conversation = vm::sanitize_id(conversation).ok()?;
    let script = read_script(&conversation);
    match vm::exec_in_vm_within(state, vm_name, &["sh".into(), "-c".into(), script], STORE_TIMEOUT).await {
        Ok((true, stdout, _)) => clean_session_id(stdout.trim()).map(str::to_string),
        Ok(_) => None,
        Err((code, _)) => {
            tracing::info!("claude bridge: stored session not read ({})", code.as_u16());
            None
        }
    }
}

/// Keeps `session_id` as the one to resume for `conversation`.
pub async fn store_session(state: &AgentService, vm_name: &str, conversation: &str, session_id: &str) {
    let (Ok(conversation), Some(id)) = (vm::sanitize_id(conversation), clean_session_id(session_id)) else {
        return;
    };
    let script = store_script(&conversation, id);
    match vm::exec_in_vm_within(state, vm_name, &["sh".into(), "-c".into(), script], STORE_TIMEOUT).await {
        Ok((true, _, _)) => {}
        Ok(_) => tracing::warn!("claude bridge: session id not stored; a restart starts a fresh session"),
        Err((code, _)) => tracing::warn!("claude bridge: session id not stored ({})", code.as_u16()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_session_store_lives_in_claudes_home_and_holds_only_the_id() {
        let read = read_script("abc-123");
        assert_eq!(read, "head -c 80 /home/claude/.claude/gb-sessions/abc-123 2>/dev/null; true");
        let store = store_script("abc-123", "2f25aa59-31f0-40d5-82a5-b2e43226ab98");
        assert!(store.starts_with("umask 077; mkdir -p /home/claude/.claude/gb-sessions && "));
        assert!(store.contains("printf '%s' 2f25aa59-31f0-40d5-82a5-b2e43226ab98 > /home/claude/.claude/gb-sessions/abc-123"));
        assert!(store.contains("chown -R claude:claude /home/claude/.claude/gb-sessions"));
        assert!(!store.contains("/tmp") && !read.contains("/tmp"));
    }

    #[test]
    fn only_a_clean_id_is_resumed_from_the_store() {
        assert_eq!(clean_session_id("2f25aa59-31f0-40d5-82a5-b2e43226ab98"), Some("2f25aa59-31f0-40d5-82a5-b2e43226ab98"));
        assert_eq!(clean_session_id("x; rm -rf /"), None);
    }
}
