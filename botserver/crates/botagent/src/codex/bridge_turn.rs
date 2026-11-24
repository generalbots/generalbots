//! A bridged Codex chat turn inside the person's workspace: a
//! `codex app-server` process over the duplex transport, kept warm for the
//! conversation's next turn ([`super::warm`]), the conversation's Codex
//! thread resumed when Codex still has it.
//!
//! The thread id of each conversation is kept next to Codex's own thread
//! store (`~codex/.codex/gb-threads/<conversation>`), so a recreated
//! workspace loses both together. The prompt travels over the session
//! itself, never through a file in `/tmp`.

use std::time::{Duration, Instant};

use crate::cli_turn::cli_model;
use crate::duplex::{DuplexOptions, Incoming, RpcClient, RpcInbox, RpcOptions};
use crate::warm::observe::TurnClock;
use crate::warm::{self, Warm, WarmKey, WarmProcess};
use crate::{vm, AgentService};

use super::app_server::{safe_id, HANDLED_REQUESTS};
use super::bridge::{check_account, handshake, open_thread, Approvals, BridgeError};
use super::turn_loop::{run_turn_tracked, TurnEnd};
use super::warm::{refuse_idle, warm_pool, CodexWarm, Observed, LIFETIME};
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
    /// Absolute in-VM paths of pictures attached to this turn.
    pub images: &'a [String],
}

/// The session's shell script: print the stored thread id on the first
/// line, then become the Codex app server as the `codex` user (`su`
/// without `-`, so the session tag survives).
pub(crate) fn launch_script(conversation: &str) -> String {
    let output = crate::deliverables::writable_output(conversation);
    format!(
        "printf '{THREAD_LINE}'; head -c 80 {THREADS_DIR}/{conversation} 2>/dev/null; echo; {output}; \
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
/// The conversation's warm app server takes the turn when there is one;
/// when it fails before Codex accepted the prompt, a fresh one does.
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
    let model = cli_model(turn.model);
    let key = WarmKey::new(vm_name, &conversation);
    let fingerprint = warm::fingerprint(&[&model, turn.instructions.unwrap_or("")]);
    if let Some(mut process) = warm_pool().take(&key, &fingerprint, Instant::now()) {
        if refuse_idle(&mut process.process.inbox) {
            let clock = TurnClock::start("codex", true);
            selfcheck::ensure(state, vm_name, &process.process.version).await?;
            let prompt = if turn.latest.trim().is_empty() { turn.transcript } else { turn.latest };
            let thread_id = process.process.thread_id.clone();
            let version = process.process.version.clone();
            let end = timed_turn(&mut process.process, &thread_id, prompt, &turn, &model, handler, &clock).await;
            if end.started || clock.touched() {
                return settle(end, process, key, fingerprint, vm_name, &version);
            }
            tracing::info!("codex bridge: the warm process failed before the turn began; starting a fresh one");
        }
        process.process.shut_down();
    }
    let clock = TurnClock::start("codex", false);
    let options = DuplexOptions { deadline: LIFETIME, ..DuplexOptions::default() };
    let session = vm::exec_in_vm_duplex(
        state,
        vm_name,
        &["sh".into(), "-c".into(), launch_script(&conversation)],
        options,
    )
    .await
    .map_err(|(code, message)| BridgeError::Failed(code, message))?;
    let born = Instant::now();
    let (client, inbox) = RpcClient::start(session, RpcOptions::accepting(HANDLED_REQUESTS));
    let mut process = CodexWarm { client, inbox, thread_id: String::new(), version: String::new() };
    let end = match open(state, vm_name, &conversation, &turn, &model, &mut process).await {
        Ok(new_thread) => {
            clock.ready();
            let prompt = if new_thread || turn.latest.trim().is_empty() { turn.transcript } else { turn.latest };
            let thread_id = process.thread_id.clone();
            timed_turn(&mut process, &thread_id, prompt, &turn, &model, handler, &clock).await
        }
        Err(error) => TurnEnd { result: Err(error), started: false, completed: false },
    };
    let version = process.version.clone();
    settle(end, Warm { process, born }, key, fingerprint, vm_name, &version)
}

/// Keeps the process for the next turn when this one completed well, ends
/// it otherwise; an action that ran unasked marks the version unsafe.
fn settle(
    end: TurnEnd,
    process: Warm<CodexWarm>,
    key: WarmKey,
    fingerprint: String,
    vm_name: &str,
    version: &str,
) -> Result<String, BridgeError> {
    match end.result {
        Ok(text) if end.completed => {
            warm_pool().put(key, fingerprint, process, Instant::now());
            warm::keep_reaped(warm_pool());
            Ok(text)
        }
        Err(BridgeError::Unasked) => {
            process.process.shut_down();
            selfcheck::mark_unsafe(vm_name, version);
            Err(BridgeError::Unsafe { version: version.to_string() })
        }
        other => {
            process.process.shut_down();
            other
        }
    }
}

/// One turn on an open thread, within [`BRIDGE_DEADLINE`].
async fn timed_turn<H: Approvals>(
    process: &mut CodexWarm,
    thread_id: &str,
    prompt: &str,
    turn: &CodexTurn<'_>,
    model: &str,
    handler: &H,
    clock: &TurnClock,
) -> TurnEnd {
    let observed = Observed { inner: handler, clock };
    clock.prompted();
    let run = run_turn_tracked(&process.client, &mut process.inbox, thread_id, prompt, turn.images, model, &observed);
    match tokio::time::timeout(BRIDGE_DEADLINE, run).await {
        Ok(end) => end,
        Err(_) => TurnEnd {
            result: Err(BridgeError::failed("codex turn passed its time limit")),
            started: true,
            completed: false,
        },
    }
}

/// Handshake, account, self-check and the conversation's thread on a fresh
/// process. Returns whether the thread is new.
async fn open(
    state: &AgentService,
    vm_name: &str,
    conversation: &str,
    turn: &CodexTurn<'_>,
    model: &str,
    process: &mut CodexWarm,
) -> Result<bool, BridgeError> {
    let version = handshake(&process.client).await?;
    process.version.clone_from(&version);
    let stored = stored_thread(&mut process.inbox);
    check_account(&process.client).await?;
    selfcheck::ensure(state, vm_name, &version).await?;
    // Codex works in the conversation's output/ folder: its sandbox may write
    // there, and what it saves there is published to the person's Drive.
    let cwd = crate::deliverables::output_dir(conversation);
    let (thread_id, new_thread) =
        open_thread(&process.client, stored.as_deref(), &cwd, model, turn.instructions).await?;
    if new_thread {
        store_thread(state, vm_name, conversation, &thread_id).await;
    }
    process.thread_id = thread_id;
    Ok(new_thread)
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
        assert!(script.contains("chmod 0777 /opt/vibe/conversations/abc-123/output; } 2>/dev/null || true; exec su codex"));
    }

    #[test]
    fn the_store_script_writes_only_the_id() {
        let script = store_script("abc", "01a2-ff");
        assert!(script.contains("printf '%s' 01a2-ff > /home/codex/.codex/gb-threads/abc"));
        assert!(script.contains("chown -R codex:codex"));
    }
}
