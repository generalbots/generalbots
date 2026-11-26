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

    let SessionIdentity {
        session_id,
        user_id,
        ..
    } = *identity;
    let Some(vm_name) = crate::core::bot::agent_vm_hook::user_vm_name(user_id).await else {
        send(
            identity,
            "No running workspace VM. Open the workspace first.",
        )
        .await;
        return;
    };
    let thread_id = session_id.to_string();
    let run_id = crate::core::bot::runs::bus::bus()
        .current_run_id(&thread_id)
        .map_or_else(Default::default, |v| v);
    let vm_for_task = vm_name.clone();
    let thread_for_task = thread_id.clone();
    let run_for_task = run_id.clone();

    let result = tokio::task::spawn_blocking(move || -> Result<String, String> {
        let pty_system = portable_pty::native_pty_system();
        let pair = pty_system
            .openpty(portable_pty::PtySize {
                rows: 24,
                cols: 120,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|error| format!("openpty: {error}"))?;
        let mut builder = portable_pty::CommandBuilder::new("incus");
        builder.args([
            "exec",
            &vm_for_task,
            "--",
            "timeout",
            &format!("{timeout_secs}s"),
            "bash",
            "-lc",
            &command,
        ]);
        let mut child = pair
            .slave
            .spawn_command(builder)
            .map_err(|error| format!("spawn: {error}"))?;
        drop(pair.slave);
        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|error| format!("reader: {error}"))?;
        let bus = crate::core::bot::runs::bus::bus();
        let mut output = String::new();
        let mut buffer = [0u8; 2048];
        loop {
            let read = match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(read) => read,
                Err(_) => break,
            };
            let chunk = String::from_utf8_lossy(&buffer[..read]).to_string();
            output.push_str(&chunk);
            for line in chunk.lines() {
                let line = line.trim_end();
                if line.is_empty() {
                    continue;
                }
                if !run_for_task.is_empty() {
                    bus.publish(
                        &thread_for_task,
                        serde_json::json!({
                            "type": "terminal_output",
                            "run_id": run_for_task,
                            "data": { "line": line },
                        }),
                    );
                }
                bus.publish(
                    &thread_for_task,
                    serde_json::json!({
                        "type": "CUSTOM",
                        "run_id": run_for_task,
                        "data": {
                            "name": "gb.command_output",
                            "payload": { "line": line, "stream": "stdout" },
                        },
                    }),
                );
            }
        }
        if let Err(e) = child.wait() {
            log::warn!("terminal: child wait failed: {e}");
        }
        Ok(output)
    })
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
