//! One Codex turn over `codex app-server`, where every command and file
//! change waits for the server's gate.
//!
//! Sequence: `initialize` + `initialized`, `account/read`, `thread/resume`
//! (the conversation's stored thread) or `thread/start`, then `turn/start`
//! and the turn's messages until `turn/completed`:
//! - `item/commandExecution/requestApproval` → [`Approvals::command`];
//! - `item/fileChange/requestApproval` → [`Approvals::file`] per path (the
//!   paths come from the item's `item/started`);
//! - `item/completed` of an approved action → [`Approvals::finished`];
//! - agent message deltas and items → [`Approvals::text`] and the answer.
//!
//! Any other server request is refused by the transport. An action that
//! finished without having asked ends the turn as unsafe. A request still
//! waiting when the session ends is dropped, which withdraws its card.

use std::future::Future;
use std::time::Duration;

use axum::http::StatusCode;
use serde_json::{json, Value};

use crate::duplex::{RpcClient, RpcError};

use super::app_server::{self as proto, ActionOutcome};

pub use super::turn_loop::{run_turn, run_turn_with_images};

pub(super) const REQUEST_WAIT: Duration = Duration::from_secs(60);

/// What the gate said about one action.
#[derive(Debug)]
pub enum Verdict<T> {
    /// Allowed; the ticket comes back in [`Approvals::finished`].
    Allow(T),
    /// The person denied it: Codex is told and the turn goes on.
    Decline,
    /// No answer in time, turn stopped, gate unavailable: the turn ends.
    Cancel,
}

/// The server side of the bridge: the permission gate and the answer.
pub trait Approvals: Send + Sync {
    type Ticket: Send;

    fn command(&self, command: &str, cwd: Option<&str>) -> impl Future<Output = Verdict<Self::Ticket>> + Send;
    fn file(&self, path: &str, delete: bool) -> impl Future<Output = Verdict<Self::Ticket>> + Send;
    fn finished(&self, ticket: Self::Ticket, outcome: ActionOutcome) -> impl Future<Output = ()> + Send;
    /// A piece of the agent's answer, in order.
    fn text(&self, chunk: &str) -> impl Future<Output = ()> + Send;
    /// A step Codex took without asking (`web_search`, `web_fetch`, `mcp_tool`), shown
    /// in the steps block.
    fn noted(&self, kind: &str, text: &str, ok: bool) -> impl Future<Output = ()> + Send;
    /// The person stopped the turn (or nobody reads the answer any more).
    fn stopped(&self) -> bool;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BridgeError {
    /// The CLI has no signed-in account.
    NotSignedIn,
    /// An action ran without asking (behaviour drift in this CLI version).
    Unasked,
    /// The installed CLI failed the approval self-check.
    Unsafe { version: String },
    /// Any other failure; a spent plan carries `plan_limit::MARKER`.
    Failed(StatusCode, String),
}

impl BridgeError {
    pub(super) fn failed(message: impl Into<String>) -> Self {
        Self::Failed(StatusCode::INTERNAL_SERVER_ERROR, message.into())
    }
}

pub(super) fn rpc_failure(step: &str, error: &RpcError) -> BridgeError {
    BridgeError::failed(format!("codex app-server {step}: {error}"))
}

/// `initialize` + `initialized`; returns the CLI version.
pub async fn handshake(client: &RpcClient) -> Result<String, BridgeError> {
    let init = client
        .request("initialize", proto::initialize_params(), REQUEST_WAIT)
        .await
        .map_err(|e| rpc_failure("initialize", &e))?;
    client
        .notify("initialized", Value::Null)
        .map_err(|e| rpc_failure("initialized", &e))?;
    Ok(proto::version_of(&init))
}

pub async fn check_account(client: &RpcClient) -> Result<(), BridgeError> {
    let account = client
        .request("account/read", json!({}), REQUEST_WAIT)
        .await
        .map_err(|e| rpc_failure("account/read", &e))?;
    if proto::signed_in(&account) {
        Ok(())
    } else {
        Err(BridgeError::NotSignedIn)
    }
}

/// Resumes `stored` when Codex still has it, else starts a thread.
/// Returns the thread id and whether it is new.
pub async fn open_thread(
    client: &RpcClient,
    stored: Option<&str>,
    cwd: &str,
    model: &str,
    instructions: Option<&str>,
) -> Result<(String, bool), BridgeError> {
    if let Some(id) = stored.filter(|id| proto::safe_id(id)) {
        match client
            .request("thread/resume", proto::thread_resume_params(id, cwd, model, instructions), REQUEST_WAIT)
            .await
        {
            Ok(result) => {
                if let Some(id) = proto::thread_id_of(&result) {
                    return Ok((id, false));
                }
            }
            Err(error) => tracing::info!("codex bridge: stored thread not resumed ({error}); starting a new one"),
        }
    }
    let result = client
        .request("thread/start", proto::thread_start_params(cwd, model, false), REQUEST_WAIT)
        .await
        .map_err(|e| rpc_failure("thread/start", &e))?;
    let id = proto::thread_id_of(&result).ok_or_else(|| BridgeError::failed("thread/start returned no thread id"))?;
    Ok((id, true))
}
