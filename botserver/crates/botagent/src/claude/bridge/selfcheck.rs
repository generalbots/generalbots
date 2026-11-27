//! Before the first bridged turn on a workspace: proof that its Claude Code
//! asks before it runs a harmless command.
//!
//! The check runs the real session command against a loopback stand-in for
//! the model (`selfcheck_model.py`, started inside the VM as `claude`), which
//! asks for `Bash` `true` once. Safe means that request reached the bridge
//! as `can_use_tool` (answered "deny", so nothing runs) and no tool result
//! came back without one. No account, quota or network is used. A workspace
//! that fails is refused with [`UNSAFE_VERSION_MARKER`] for a while.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use botsecurity_core::error_sanitizer::SafeErrorResponse;

use crate::duplex::DuplexOptions;
use crate::AgentService;

use super::actions::ClaudeAction;
use super::command::{claude_invocation, session_script, write_root_file};
use super::drive::{drive, BridgeGate, GateAnswer, ToolOutcome};
use super::protocol::ToolRequest;

/// Starts the error of a refused turn: the CLI in this workspace runs tools
/// without asking. `llm::subscription` turns it into `agent_unsafe_version`.
pub const UNSAFE_VERSION_MARKER: &str = "[agent-unsafe-version]";

const MODEL_SCRIPT: &str = include_str!("selfcheck_model.py");
const MODEL_PATH: &str = "/usr/local/lib/gb/claude-selfcheck.py";
const CHECK_DEADLINE: Duration = Duration::from_secs(150);
/// A failed workspace is checked again after this long.
const RETRY_AFTER: Duration = Duration::from_secs(300);

static CHECKS: OnceLock<Mutex<HashMap<String, (bool, Instant)>>> = OnceLock::new();

fn checks() -> std::sync::MutexGuard<'static, HashMap<String, (bool, Instant)>> {
    CHECKS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// A refused turn keeps `agent_unsafe_version` as its machine-readable code
/// and the marker as the first words of the message (the chat pipeline reads
/// both). The sanitised envelope has no 412 variant, so the closest client
/// status is `conflict`.
pub(super) fn unsafe_error() -> SafeErrorResponse {
    SafeErrorResponse::new(
        "conflict",
        format!(
            "{UNSAFE_VERSION_MARKER} Claude Code in this workspace did not ask before running a \
             command, so it is not used."
        ),
    )
    .with_code("agent_unsafe_version")
}

#[derive(Default)]
pub(super) struct CheckGate {
    asked: AtomicBool,
}

impl CheckGate {
    pub(super) fn saw_the_request(&self) -> bool {
        self.asked.load(Ordering::SeqCst)
    }
}

impl BridgeGate for CheckGate {
    async fn decide(&self, _request: &ToolRequest, action: &ClaudeAction) -> GateAnswer {
        if matches!(action, ClaudeAction::Command { command } if command == "true") {
            self.asked.store(true, Ordering::SeqCst);
        }
        GateAnswer::Deny("permission self-check: not run".to_string())
    }

    async fn finished(&self, _tool_use_id: &str, _outcome: &ToolOutcome) {}

    async fn text(&self, _chunk: &str) -> bool {
        true
    }

    fn stopped(&self) -> bool {
        false
    }
}

/// The verdict from one check session.
pub(super) fn verdict(asked: bool, unasked: &[String]) -> bool {
    asked && unasked.is_empty()
}

pub(super) fn check_script(port: u16) -> String {
    let env = format!(
        " ANTHROPIC_BASE_URL=http://127.0.0.1:{port} ANTHROPIC_API_KEY=gb-self-check \
         CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1 DISABLE_TELEMETRY=1"
    );
    let before = format!("{{ python3 {MODEL_PATH} {port} & }} && sleep 1 && ");
    session_script(&write_root_file(MODEL_PATH, MODEL_SCRIPT), &env, &before, &claude_invocation("", None))
}

async fn run_check(state: &AgentService, vm_name: &str) -> Result<bool, SafeErrorResponse> {
    let port = 20_000 + u16::try_from(uuid::Uuid::new_v4().as_u128() % 30_000).unwrap_or(0);
    let args = ["sh".to_string(), "-c".to_string(), check_script(port)];
    let options = DuplexOptions { deadline: CHECK_DEADLINE, ..DuplexOptions::default() };
    let session = crate::vm::exec_in_vm_duplex(state, vm_name, &args, options).await?;
    let gate = CheckGate::default();
    let report = drive(session, "Run the workspace permission self-check.", &gate).await;
    let asked = gate.saw_the_request();
    tracing::info!(
        "claude bridge self-check {vm_name}: asked={asked} unasked={} result={} end={:?}",
        report.unasked.len(),
        report.result.as_ref().map_or("none", |r| r.subtype.as_str()),
        report.end.as_ref().map(|e| e.reason)
    );
    Ok(verdict(asked, &report.unasked))
}

/// Ok when this workspace's Claude Code was seen asking; the check runs
/// once per workspace and server run (again after [`RETRY_AFTER`] when it
/// failed).
pub(super) async fn ensure(state: &AgentService, vm_name: &str) -> Result<(), SafeErrorResponse> {
    if let Some(known) = known_verdict(vm_name) {
        return known;
    }
    let _one_at_a_time = running().lock().await;
    if let Some(known) = known_verdict(vm_name) {
        return known;
    }
    let safe = run_check(state, vm_name).await?;
    checks().insert(vm_name.to_string(), (safe, Instant::now()));
    if safe {
        Ok(())
    } else {
        tracing::warn!("claude bridge: {vm_name} failed the permission self-check; Claude Code turns refused");
        Err(unsafe_error())
    }
}

/// The kept verdict for `vm_name`, None when it must be checked (never, or a
/// failure older than [`RETRY_AFTER`]).
fn known_verdict(vm_name: &str) -> Option<Result<(), SafeErrorResponse>> {
    let (safe, at) = checks().get(vm_name).copied()?;
    if safe {
        Some(Ok(()))
    } else {
        (at.elapsed() < RETRY_AFTER).then(|| Err(unsafe_error()))
    }
}

fn running() -> &'static tokio::sync::Mutex<()> {
    static RUNNING: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();
    RUNNING.get_or_init(|| tokio::sync::Mutex::new(()))
}

/// Runs the check ahead of the first turn (after a server start or a
/// sign-in), so that turn finds the verdict kept. Does nothing when this
/// workspace already has one.
pub async fn precheck(state: &AgentService, vm_name: &str) -> Result<(), SafeErrorResponse> {
    if checks().contains_key(vm_name) {
        return Ok(());
    }
    ensure(state, vm_name).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_an_asked_check_with_nothing_run_unasked_is_safe() {
        assert!(verdict(true, &[]));
        assert!(!verdict(false, &[]));
        assert!(!verdict(true, &["Bash".to_string()]));
    }

    #[test]
    fn a_kept_verdict_spares_the_next_check() {
        assert!(known_verdict("vm-precheck-none").is_none());
        checks().insert("vm-precheck-safe".to_string(), (true, Instant::now()));
        checks().insert("vm-precheck-unsafe".to_string(), (false, Instant::now()));
        assert!(matches!(known_verdict("vm-precheck-safe"), Some(Ok(()))));
        assert!(matches!(known_verdict("vm-precheck-unsafe"), Some(Err(_))));
    }

    #[test]
    fn check_script_points_the_cli_at_the_loopback_model_only() {
        let script = check_script(23_456);
        assert!(script.contains("ANTHROPIC_BASE_URL=http://127.0.0.1:23456"));
        assert!(script.contains("{ python3 /usr/local/lib/gb/claude-selfcheck.py 23456 & } && sleep 1 && exec claude -p"));
        assert!(script.len() < 16_384, "the command guard takes trusted scripts up to 16 KiB");
        assert!(unsafe_error().message.starts_with(UNSAFE_VERSION_MARKER));
    }
}
