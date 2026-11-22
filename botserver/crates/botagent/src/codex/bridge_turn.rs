//! A bridged Codex chat turn inside the person's workspace: one
//! `codex app-server` process per turn over the duplex transport, the
//! conversation's Codex thread resumed when Codex still has it.
//!
//! The thread id of each conversation is kept next to Codex's own thread
//! store (`~codex/.codex/gb-threads/<conversation>`), so a recreated
//! workspace loses both together. The prompt travels over the session
//! itself, never through a file in `/tmp`.

use std::time::Duration;

use crate::cli_turn::cli_model;
use crate::duplex::{DuplexOptions, Incoming, RpcClient, RpcInbox, RpcOptions};
use crate::{vm, AgentService};

use super::app_server::{safe_id, HANDLED_REQUESTS};
use super::bridge::{check_account, handshake, open_thread, run_turn, Approvals, BridgeError};
use super::paths::{CODEX_HOME, CODEX_USER};
use super::selfcheck;

/// Whole-turn limit: the person's thinking time on cards counts too.
pub const BRIDGE_DEADLINE: Duration = Duration::from_secs(30 * 60);
const THREADS_DIR: &str = "/home/codex/.codex/gb-threads";
const THREAD_LINE: &str = "GB-THREAD ";
const STORE_TIMEOUT: Duration = Duration::from_secs(60);

/// What one chat turn sends to Codex.
#[derive(Debug, Clone)]
pub struct CodexTurn<'a> {
    /// The conversation (session) id: the thread store key.
    pub conversation: &'a str,
    /// The whole transcript, for a new thread.
    pub transcript: &'a str,
    /// The person's latest message, for a resumed thread.
    pub latest: &'a str,
    pub model: &'a str,
    /// The system prompt, passed again when the thread is resumed.
    pub instructions: Option<&'a str>,
}

/// The session's shell script: print the stored thread id on the first
/// line, then become the Codex app server as the `codex` user (`su`
/// without `-`, so the session tag survives).
pub(crate) fn launch_script(conversation: &str) -> String {
    format!(
        "printf '{THREAD_LINE}'; head -c 80 {THREADS_DIR}/{conversation} 2>/dev/null; echo; \
         exec su {CODEX_USER} -s /bin/sh -c 'cd {CODEX_HOME} && export HOME={CODEX_HOME} && exec codex app-server'"
    )
}

pub(crate) fn store_script(conversation: &str, thread_id: &str) -> String {
    format!(
        "umask 077; mkdir -p {THREADS_DIR} && printf '%s' {thread_id} > {THREADS_DIR}/{conversation} && \
         chown -R {CODEX_USER}:{CODEX_USER} {THREADS_DIR}"
    )
}

/// The stored thread id announced on the session's first line. Anything
/// queued before it (configuration warnings) is not needed by the turn.
pub(crate) fn stored_thread(inbox: &mut RpcInbox) -> Option<String> {
    while let Some(message) = inbox.try_next() {
        if let Incoming::Unparsed(line) = message {
            if let Some(id) = line.strip_prefix(THREAD_LINE.trim_end()) {
                let id = id.trim();
                return safe_id(id).then(|| id.to_string());
            }
        }
    }
    None
}

async fn store_thread(state: &AgentService, vm_name: &str, conversation: &str, thread_id: &str) {
    if !safe_id(thread_id) {
        return;
    }
    let script = store_script(conversation, thread_id);
    match vm::exec_in_vm_within(state, vm_name, &["sh".into(), "-c".into(), script], STORE_TIMEOUT).await {
        Ok((true, _, _)) => {}
        Ok(_) => tracing::warn!("codex bridge: thread id not stored; the next turn starts a new thread"),
        Err((code, _)) => tracing::warn!("codex bridge: thread id not stored ({})", code.as_u16()),
    }
}

/// Runs one chat turn through the bridge and returns the agent's answer.
pub async fn run_bridged<H: Approvals>(
    state: &AgentService,
    vm_name: &str,
    turn: CodexTurn<'_>,
    handler: &H,
) -> Result<String, BridgeError> {
    if turn.transcript.trim().is_empty() {
        return Err(BridgeError::Failed(axum::http::StatusCode::BAD_REQUEST, "Empty prompt".to_string()));
    }
    let conversation = vm::sanitize_id(turn.conversation).map_err(|(code, message)| BridgeError::Failed(code, message))?;
    let options = DuplexOptions { deadline: BRIDGE_DEADLINE, ..DuplexOptions::default() };
    let session = vm::exec_in_vm_duplex(
        state,
        vm_name,
        &["sh".into(), "-c".into(), launch_script(&conversation)],
        options,
    )
    .await
    .map_err(|(code, message)| BridgeError::Failed(code, message))?;
    let (client, mut inbox) = RpcClient::start(session, RpcOptions::accepting(HANDLED_REQUESTS));
    let outcome = drive(state, vm_name, &conversation, &turn, &client, &mut inbox, handler).await;
    let end = client.close().await;
    tracing::debug!("codex bridge: session ended ({:?}) after {} ms", end.reason, end.elapsed.as_millis());
    outcome
}

async fn drive<H: Approvals>(
    state: &AgentService,
    vm_name: &str,
    conversation: &str,
    turn: &CodexTurn<'_>,
    client: &RpcClient,
    inbox: &mut RpcInbox,
    handler: &H,
) -> Result<String, BridgeError> {
    let version = handshake(client).await?;
    let stored = stored_thread(inbox);
    check_account(client).await?;
    selfcheck::ensure(state, vm_name, &version).await?;
    let model = cli_model(turn.model);
    let (thread_id, new_thread) = open_thread(client, stored.as_deref(), CODEX_HOME, &model, turn.instructions).await?;
    if new_thread {
        store_thread(state, vm_name, conversation, &thread_id).await;
    }
    let prompt = if new_thread || turn.latest.trim().is_empty() { turn.transcript } else { turn.latest };
    match run_turn(client, inbox, &thread_id, prompt, &model, handler).await {
        Err(BridgeError::Unasked) => {
            selfcheck::mark_unsafe(vm_name, &version);
            Err(BridgeError::Unsafe { version })
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_launch_script_announces_the_thread_and_keeps_the_session_tag() {
        let script = launch_script("abc-123");
        assert!(script.starts_with("printf 'GB-THREAD '; head -c 80 /home/codex/.codex/gb-threads/abc-123"));
        assert!(script.contains("exec su codex -s /bin/sh -c '"));
        assert!(!script.contains("su -"));
        assert!(script.ends_with("exec codex app-server'"));
        assert!(!script.contains("/tmp"));
    }

    #[test]
    fn the_store_script_writes_only_the_id() {
        let script = store_script("abc", "01a2-ff");
        assert!(script.contains("printf '%s' 01a2-ff > /home/codex/.codex/gb-threads/abc"));
        assert!(script.contains("chown -R codex:codex"));
    }
}
