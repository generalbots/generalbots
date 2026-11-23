//! Antigravity chat turns over the Agent Client Protocol (`agy acp`): one
//! ACP process per turn over the duplex transport, the conversation's
//! session resumed when the CLI still has it.
//!
//! Sequence: `initialize`, then `session/resume` or `session/load` of the
//! stored session (`session/new` otherwise, its id kept next to the CLI's
//! state in `~antigravity/.gemini/gb-sessions/<conversation>`), then one
//! `session/prompt` with the text, pictures and text files (see
//! [`turn`]). Every tool call the agent wants is a
//! `session/request_permission` that waits for the server's gate.
//!
//! [`AcpError::NoAcp`] means the CLI did not speak ACP (old version, failed
//! `initialize`): the caller may fall back to print mode.

mod check;
mod proto;
mod turn;

#[cfg(test)]
mod tests;

use std::time::Duration;

use axum::http::StatusCode;

use crate::cli_turn::cli_model;
use crate::duplex::{DuplexOptions, Incoming, RpcClient, RpcError, RpcInbox, RpcOptions};
use crate::{vm, AgentService};

use super::paths::{ANTIGRAVITY_HOME, ANTIGRAVITY_USER};

pub use proto::{actions_of, agent_info, safe_id, AcpAction, AgentInfo, ImagePart, TextResource};
pub use turn::{AcpGate, QUIET_LIMIT};

/// Whole-turn limit: the person's thinking time on cards counts too.
pub const BRIDGE_DEADLINE: Duration = Duration::from_secs(30 * 60);
const REQUEST_WAIT: Duration = Duration::from_secs(60);
/// `initialize` of a CLI that does not speak ACP never answers: give up soon.
const INITIALIZE_WAIT: Duration = Duration::from_secs(30);
const STORE_TIMEOUT: Duration = Duration::from_secs(60);
const SESSIONS_DIR: &str = "/home/antigravity/.gemini/gb-sessions";
const SESSION_LINE: &str = "GB-SESSION ";
/// The conversation folders, where attachments are copied: readable by the
/// agent next to its home.
const CONVERSATIONS: &str = "/opt/vibe/conversations";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcpError {
    /// `agy acp` did not start or did not answer `initialize`.
    NoAcp(String),
    /// The CLI has no signed-in account.
    NotSignedIn,
    /// An action ran without asking (behaviour drift in this CLI version).
    Unasked,
    /// This CLI version was refused for acting without asking.
    Unsafe { version: String },
    /// Any other failure; a spent plan carries the plan-limit marker.
    Failed(StatusCode, String),
}

/// What one chat turn sends to Antigravity.
#[derive(Debug, Clone)]
pub struct AcpTurn<'a> {
    /// The conversation (session) id: the session store key.
    pub conversation: &'a str,
    /// The whole transcript, for a new session.
    pub transcript: &'a str,
    /// The person's latest message with this turn's notes, for a resumed one.
    pub latest: &'a str,
    pub model: &'a str,
    pub images: &'a [ImagePart],
    pub resources: &'a [TextResource],
}

/// The session's shell script: print the stored session id on the first
/// line, then become `agy acp` as the `antigravity` user (`su` without `-`,
/// so the session tag survives), with the conversation folders in its
/// workspace.
pub(crate) fn launch_script(conversation: &str) -> String {
    format!(
        "printf '{SESSION_LINE}'; head -c 130 {SESSIONS_DIR}/{conversation} 2>/dev/null; echo; \
         exec su {ANTIGRAVITY_USER} -s /bin/sh -c 'cd {ANTIGRAVITY_HOME} && export HOME={ANTIGRAVITY_HOME} && exec agy acp --add-dir {CONVERSATIONS}'"
    )
}

pub(crate) fn store_script(conversation: &str, session_id: &str) -> String {
    format!(
        "umask 077; mkdir -p {SESSIONS_DIR} && printf '%s' {session_id} > {SESSIONS_DIR}/{conversation} && \
         chown -R {ANTIGRAVITY_USER}:{ANTIGRAVITY_USER} {SESSIONS_DIR}"
    )
}

/// The stored session id announced on the session's first line.
pub(crate) fn stored_session(inbox: &mut RpcInbox) -> Option<String> {
    while let Some(message) = inbox.try_next() {
        if let Incoming::Unparsed(line) = message {
            if let Some(id) = line.strip_prefix(SESSION_LINE.trim_end()) {
                let id = id.trim();
                return safe_id(id).then(|| id.to_string());
            }
        }
    }
    None
}

async fn store_session(state: &AgentService, vm_name: &str, conversation: &str, session_id: &str) {
    if !safe_id(session_id) {
        return;
    }
    let script = store_script(conversation, session_id);
    match vm::exec_in_vm_within(state, vm_name, &["sh".into(), "-c".into(), script], STORE_TIMEOUT).await {
        Ok((true, _, _)) => {}
        Ok(_) => tracing::warn!("antigravity bridge: session id not stored; the next turn starts a new session"),
        Err((code, _)) => tracing::warn!("antigravity bridge: session id not stored ({})", code.as_u16()),
    }
}

fn rpc_failure(step: &str, error: &RpcError) -> AcpError {
    match error {
        RpcError::Remote { message, .. } if proto::needs_sign_in(message) => AcpError::NotSignedIn,
        _ => AcpError::Failed(StatusCode::INTERNAL_SERVER_ERROR, format!("agy acp {step}: {error}")),
    }
}

/// Runs one chat turn through `agy acp` and returns the agent's answer.
pub async fn run_acp_turn<H: AcpGate>(
    state: &AgentService,
    vm_name: &str,
    turn: AcpTurn<'_>,
    handler: &H,
) -> Result<String, AcpError> {
    if turn.transcript.trim().is_empty() {
        return Err(AcpError::Failed(StatusCode::BAD_REQUEST, "Empty prompt".to_string()));
    }
    let conversation = vm::sanitize_id(turn.conversation).map_err(|(code, message)| AcpError::Failed(code, message))?;
    let options = DuplexOptions { deadline: BRIDGE_DEADLINE, ..DuplexOptions::default() };
    let session = vm::exec_in_vm_duplex(state, vm_name, &["sh".into(), "-c".into(), launch_script(&conversation)], options)
        .await
        .map_err(|(_, message)| AcpError::NoAcp(message))?;
    let (client, mut inbox) = RpcClient::start(session, RpcOptions::accepting(proto::HANDLED_REQUESTS));
    let outcome = drive(state, vm_name, &conversation, &turn, &client, &mut inbox, handler).await;
    let end = client.close().await;
    tracing::debug!("antigravity bridge: session ended ({:?}) after {} ms", end.reason, end.elapsed.as_millis());
    outcome
}

async fn drive<H: AcpGate>(
    state: &AgentService,
    vm_name: &str,
    conversation: &str,
    turn: &AcpTurn<'_>,
    client: &RpcClient,
    inbox: &mut RpcInbox,
    handler: &H,
) -> Result<String, AcpError> {
    let init = client
        .request("initialize", proto::initialize_params(), INITIALIZE_WAIT)
        .await
        .map_err(|error| AcpError::NoAcp(format!("initialize: {error}")))?;
    let info = agent_info(&init);
    check::ensure(vm_name, &info)?;
    let stored = stored_session(inbox);
    let (session_id, opened, new_session) = open_session(client, stored.as_deref(), &info).await?;
    // A loaded session replays its history as updates: none of it is this
    // turn's answer.
    while inbox.try_next().is_some() {}
    check::ensure_asking_mode(client, &session_id, &opened, &info).await?;
    select_model(client, &session_id, &opened, turn.model).await;
    if new_session {
        store_session(state, vm_name, conversation, &session_id).await;
    }
    let text = if new_session || turn.latest.trim().is_empty() { turn.transcript } else { turn.latest };
    tracing::info!(
        "antigravity bridge: acp {} session, {} images, {} files, agy {}",
        if new_session { "new" } else { "resumed" },
        if info.images { turn.images.len() } else { 0 },
        if info.embedded_context { turn.resources.len() } else { 0 },
        info.version
    );
    let input = turn::PromptInput {
        session_id: &session_id,
        text,
        images: turn.images,
        resources: turn.resources,
        info: &info,
    };
    match turn::run_prompt(client, inbox, input, handler).await {
        Err(AcpError::Unasked) => {
            check::mark_unsafe(vm_name, &info.version);
            Err(AcpError::Unsafe { version: info.version })
        }
        other => other,
    }
}

/// Resumes or loads `stored` when the CLI still has it, else starts a
/// session. Returns its id, the open result, and whether it is new.
async fn open_session(
    client: &RpcClient,
    stored: Option<&str>,
    info: &AgentInfo,
) -> Result<(String, serde_json::Value, bool), AcpError> {
    if let Some(id) = stored.filter(|id| safe_id(id)) {
        let method = if info.resume_session {
            Some("session/resume")
        } else if info.load_session {
            Some("session/load")
        } else {
            None
        };
        if let Some(method) = method {
            match client.request(method, proto::session_open_params(id, ANTIGRAVITY_HOME), REQUEST_WAIT).await {
                Ok(result) => return Ok((id.to_string(), result, false)),
                Err(RpcError::Remote { message, .. }) if proto::needs_sign_in(&message) => {
                    return Err(AcpError::NotSignedIn)
                }
                Err(error) => tracing::info!("antigravity bridge: stored session not resumed ({error}); starting a new one"),
            }
        }
    }
    let result = client
        .request("session/new", proto::session_new_params(ANTIGRAVITY_HOME), REQUEST_WAIT)
        .await
        .map_err(|error| rpc_failure("session/new", &error))?;
    let id = proto::session_id_of(&result)
        .ok_or_else(|| AcpError::Failed(StatusCode::INTERNAL_SERVER_ERROR, "session/new returned no session id".to_string()))?;
    Ok((id, result, true))
}

/// Picks the chosen model when the session offers it; the CLI's default
/// answers otherwise.
async fn select_model(client: &RpcClient, session_id: &str, opened: &serde_json::Value, model: &str) {
    let model = cli_model(model);
    if model.is_empty() {
        return;
    }
    let offered = opened
        .get("models")
        .and_then(|models| models.get("availableModels"))
        .and_then(serde_json::Value::as_array)
        .is_some_and(|models| {
            models.iter().any(|entry| entry.get("modelId").and_then(serde_json::Value::as_str) == Some(model.as_str()))
        });
    if !offered {
        return;
    }
    let params = serde_json::json!({"sessionId": session_id, "modelId": model});
    if let Err(error) = client.request("session/set_model", params, REQUEST_WAIT).await {
        tracing::info!("antigravity bridge: model not selected ({error}); the CLI default answers");
    }
}
