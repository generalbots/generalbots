//! `terminal_run`: a live PTY inside the user's agent VM, streaming
//! `terminal_output` / `gb.command_output` frames to the chat run so the run
//! card can expose Take over.

use super::super::types::SessionIdentity;
use super::super::vm_consent::{reason, VmOutcome};
use super::tell_model;

/// Longest program output kept for the model and the step.
const OUTPUT_LIMIT: usize = 8_000;

/// A live terminal run: spawns a real PTY inside the user's agent VM and
/// streams `terminal_output` / `gb.command_output` frames to the chat run so
/// the run card can expose Take over. This is the PTY equivalent of the
/// AutoTask agent executor (which is not wired in this build).
///
/// The model reads the program's output, or an English sentence when there
/// is none; the returned outcome (the step the person sees) carries only the
/// program's own output and, when the run did not reach the PTY, a reason
/// code the shell words in the person's language.
pub(super) async fn run_terminal_run(
    identity: &SessionIdentity<'_>,
    args: &str,
    ticket: crate::core::bot::pipeline::vm_consent::VmConsent,
) -> VmOutcome {
    let parsed = match super::super::native_args::parse_args(args) {
        Ok(value) => value,
        Err(error) => {
            tell_model(identity.session_id, &format!("Could not read the terminal request: {error}"));
            return VmOutcome::default();
        }
    };
    let command = parsed
        .get("command")
        .and_then(|value| value.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if command.is_empty() {
        tell_model(identity.session_id, "terminal_run requires a `command`.");
        return VmOutcome::default();
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
        command.clone(),
        timeout_secs,
    )
    .await;
    let outcome = match run {
        Ok(run) => finished(&run.output, run.exit_code),
        Err(error) => {
            tell_model(identity.session_id, &format!("terminal_run failed: {error}"));
            VmOutcome { reason: Some(reason::WORKSPACE_UNAVAILABLE), ..VmOutcome::default() }
        }
    };
    super::super::turn_record::note_command(identity.session_id, &command, outcome.exit_code);
    if outcome.reason.is_none() {
        let for_model = if outcome.output.is_empty() {
            "The command finished with no output."
        } else {
            outcome.output.as_str()
        };
        tell_model(identity.session_id, for_model);
    }
    outcome
}

/// The step of a run that reached the PTY: its trimmed, bounded output.
fn finished(output: &str, exit_code: Option<i32>) -> VmOutcome {
    VmOutcome {
        ok: exit_code == Some(0),
        exit_code,
        output: output.trim().chars().take(OUTPUT_LIMIT).collect(),
        ..VmOutcome::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_silent_run_shows_no_wrapper_sentence() {
        let outcome = finished("  \n ", Some(0));
        assert!(outcome.ok);
        assert!(outcome.output.is_empty());
        assert!(outcome.reason.is_none());
    }

    #[test]
    fn the_program_output_is_kept_as_is_and_bounded() {
        let outcome = finished(" total 0\n", Some(2));
        assert!(!outcome.ok);
        assert_eq!(outcome.exit_code, Some(2));
        assert_eq!(outcome.output, "total 0");
        let long = "x".repeat(OUTPUT_LIMIT + 10);
        assert_eq!(finished(&long, Some(0)).output.len(), OUTPUT_LIMIT);
    }
}
