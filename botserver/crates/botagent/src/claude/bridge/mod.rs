//! The Claude Code permission bridge: one chat turn as a stream-json session
//! with the CLI inside the person's workspace, in which every tool use asks
//! the chat (the permission card) before it runs.
//!
//! Mechanism, proven live on Claude Code 2.1.330 with a loopback model:
//! `--permission-prompt-tool stdio` makes the CLI write
//! `{"type":"control_request","request_id":…,"request":{"subtype":"can_use_tool",
//! "tool_name":"Bash","input":{"command":…},"tool_use_id":…}}` and wait for
//! `{"type":"control_response","response":{"subtype":"success","request_id":…,
//! "response":{"behavior":"allow","updatedInput":{…}}}}` (or
//! `{"behavior":"deny","message":…}`, which the model reads as the tool
//! result). `{"subtype":"interrupt"}` stops the turn; the CLI then cancels
//! its open request (`control_cancel_request`) and ends with a `result`.
//! See [`command`] for the flags that make every tool ask.
//!
//! The process is kept warm after a turn that ended well with its `result`,
//! for the conversation's next message ([`warm`]); it ends on Stop, on any
//! failure, at the 30 minute turn deadline, after 10 idle minutes, or when
//! the turn's future is dropped.

mod actions;
mod command;
mod drive;
mod protocol;
mod selfcheck;
mod sessions;
mod warm;

use botsecurity_core::error_sanitizer::SafeErrorResponse;

use crate::cli_turn::{plan_limit, spent_answer};
use crate::AgentService;

pub use actions::{exit_code, output_text, ClaudeAction};
pub use command::CLAUDE_CODE_VERSION;
pub use drive::{drive, drive_with, BridgeGate, DriveReport, GateAnswer, ToolOutcome, STOPPED, UNKNOWN_TOOL};
pub use protocol::{Attachment, ToolRequest, TurnResult};
pub use selfcheck::{precheck, UNSAFE_VERSION_MARKER};
pub use sessions::{store_session, stored_session};
pub use warm::{warm_pool, ClaudeWarm};

/// One chat turn.
#[derive(Debug, Clone, Copy)]
pub struct BridgedTurn<'a> {
    /// The newest message only, for a resumed session.
    pub message: &'a str,
    /// The whole conversation, for a fresh session.
    pub transcript: &'a str,
    pub model: &'a str,
    /// Claude's session id from this conversation's previous turn.
    pub resume: Option<&'a str>,
    /// The conversation whose workspace folder the file tools may reach.
    pub conversation: Option<&'a str>,
    /// The files the person attached to this message, sent with it.
    pub attachments: &'a [Attachment],
}

/// How a turn ended well.
#[derive(Debug, Clone, Default)]
pub struct BridgedAnswer {
    /// Claude's session id, to resume on the next turn.
    pub session_id: Option<String>,
    /// The model text, as sent to the gate.
    pub answer: String,
    pub stopped: bool,
}

/// #1939 — delivery policy for VM agent turns.
pub(crate) const DELIVERY_DIRECTIVE: &str = "Deliver the exact artifact the user asked for, in the \
requested format. Prefer a maintained open-source library or command-line tool that already produces \
it, and search the web first when unsure which one is current. Install what the task needs, but stay \
inside the sandbox limits. Never substitute another format silently: if the exact format cannot be \
produced reliably, say so and offer the closest alternative you can verify.";

const NOT_SIGNED_IN: &str = "Claude Code is not signed in in your workspace. Connect it again in \
Settings, AI accounts.";

fn failed(detail: &str) -> SafeErrorResponse {
    tracing::error!("Claude Code bridged turn failed: {}", detail.chars().take(300).collect::<String>());
    SafeErrorResponse::new("internal_error", "Claude Code failed to answer.")
}

fn is_missing_session(report: &DriveReport) -> bool {
    let said = format!(
        "{} {} {}",
        report.result.as_ref().map(|r| r.text.as_str()).unwrap_or(""),
        report.cli_text,
        report.end.as_ref().map(|e| e.stderr_tail.as_str()).unwrap_or("")
    );
    report.answer.is_empty()
        && report.asked.is_empty()
        && report.result.as_ref().is_none_or(|r| r.is_error)
        && said.contains("No conversation found")
}

/// The turn's verdict from what the session did.
pub fn interpret(report: DriveReport) -> Result<BridgedAnswer, SafeErrorResponse> {
    let stderr = report.end.as_ref().map(|e| e.stderr_tail.clone()).unwrap_or_else(|| {
        tracing::debug!("Claude session ended without a stderr tail");
        String::new()
    });
    let cli = format!("{}\n{}", report.cli_text, report.result.as_ref().map(|r| r.text.as_str()).unwrap_or(""));
    let answer = BridgedAnswer {
        session_id: report.session_id.clone(),
        answer: report.answer.clone(),
        stopped: report.stopped,
    };
    if report.stopped {
        return Ok(answer);
    }
    let Some(result) = report.result.as_ref() else {
        let reason = report.end.as_ref().map(|e| format!("{:?}", e.reason)).unwrap_or_else(|| {
            tracing::debug!("Claude session ended without a recorded reason");
            String::new()
        });
        return Err(failed(&format!("the session ended without a result ({reason})")));
    };
    if result.is_error || report.answer.trim().is_empty() {
        if let Some(limit) = plan_limit(&cli, &stderr).or_else(|| spent_answer(cli.trim())) {
            return Err(limit);
        }
        if cli.contains("Not logged in") || cli.contains("/login") {
            return Err(SafeErrorResponse::new("unauthorized", NOT_SIGNED_IN));
        }
    }
    if result.is_error {
        return Err(failed(&format!("{}: {}", result.subtype, cli.trim())));
    }
    if report.answer.trim().is_empty() {
        return Err(failed("empty response"));
    }
    Ok(answer)
}

/// Runs one chat turn in `vm_name` through the bridge. The workspace must
/// first pass the permission self-check. A stored session that the CLI no
/// longer has is replaced by a fresh one with the whole transcript.
pub async fn run_bridged_turn(
    state: &AgentService,
    vm_name: &str,
    turn: BridgedTurn<'_>,
    gate: &impl BridgeGate,
) -> Result<BridgedAnswer, SafeErrorResponse> {
    if turn.transcript.trim().is_empty() {
        return Err(SafeErrorResponse::bad_request("Empty prompt"));
    }
    selfcheck::ensure(state, vm_name).await?;
    let fresh = format!("{DELIVERY_DIRECTIVE}\n\n{}", turn.transcript);
    if let Some(id) = turn.resume.and_then(command::clean_session_id) {
        let message = if turn.message.trim().is_empty() { turn.transcript } else { turn.message };
        if let Some(report) = warm::run_warm(vm_name, &turn, message, gate).await {
            return interpret(report);
        }
        let report = warm::run_fresh(state, vm_name, &turn, Some(id), message, gate).await?;
        if !is_missing_session(&report) {
            return interpret(report);
        }
        tracing::info!("claude bridge: session to resume is gone; starting a fresh one");
    }
    let report = warm::run_fresh(state, vm_name, &turn, None, &fresh, gate).await?;
    interpret(report)
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests_drive;

#[cfg(test)]
mod live_tests;

#[cfg(test)]
mod tests_warm;
