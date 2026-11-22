//! `terminal_run`: a live PTY inside the user's agent VM, streaming
//! `terminal_output` / `gb.command_output` frames to the chat run so the run
//! card can expose Take over.

use super::super::types::SessionIdentity;
use super::send;

/// A live terminal run: spawns a real PTY inside the user's agent VM and
/// streams `terminal_output` / `gb.command_output` frames to the chat run so
/// the run card can expose Take over. This is the PTY equivalent of the
/// AutoTask agent executor (which is not wired in this build).
///
/// Returns the program's exit code when the run reached the PTY.
pub(super) async fn run_terminal_run(
    identity: &SessionIdentity<'_>,
    args: &str,
    ticket: crate::core::bot::pipeline::vm_consent::VmConsent,
) -> Option<i32> {
    let parsed = match super::super::native_args::parse_args(args) {
        Ok(value) => value,
        Err(error) => {
            send(identity, &format!("Could not read the terminal request: {error}")).await;
            return None;
        }
    };
    let command = parsed
        .get("command")
        .and_then(|value| value.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if command.is_empty() {
        send(identity, "terminal_run requires a `command`.").await;
        return None;
    }
    let timeout_secs = parsed
        .get("timeout_ms")
        .and_then(|value| value.as_u64())
        .unwrap_or(60_000)
        .clamp(5_000, 120_000)
        / 1000;

    let run = crate::core::bot::agent_vm_hook::run_in_terminal(
        ticket,
        identity.user_id,
        identity.session_id,
        command,
        timeout_secs,
    )
    .await;
    let (text, exit_code) = match run {
        Ok(run) => (run.output, run.exit_code),
        Err(error) => (format!("terminal_run failed: {error}"), None),
    };
    let trimmed = text.trim();
    let summary: String = if trimmed.is_empty() {
        "The command finished with no output.".to_string()
    } else {
        trimmed.chars().take(8_000).collect()
    };
    send(identity, &summary).await;
    exit_code
}
