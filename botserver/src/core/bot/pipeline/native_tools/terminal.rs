//! `terminal_run`: a live PTY inside the user's agent VM, streaming
//! `terminal_output` / `gb.command_output` frames to the chat run so the run
//! card can expose Take over.

use super::super::types::SessionIdentity;
use super::send;

/// A live terminal run: spawns a real PTY inside the user's agent VM and
/// streams `terminal_output` / `gb.command_output` frames to the chat run so
/// the run card can expose Take over. This is the PTY equivalent of the
/// AutoTask agent executor (which is not wired in this build).
pub(super) async fn run_terminal_run(identity: &SessionIdentity<'_>, args: &str) {
    use std::io::Read;

    let parsed = match super::super::native_args::parse_args(args) {
        Ok(value) => value,
        Err(error) => {
            send(
                identity,
                &format!("Could not read the terminal request: {error}"),
            )
            .await;
            return;
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
        return;
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
        .unwrap_or_default();
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
        let _ = child.wait();
        Ok(output)
    })
    .await;

    let text = match result {
        Ok(Ok(output)) => output,
        Ok(Err(error)) => format!("terminal_run failed: {error}"),
        Err(error) => format!("terminal_run task failed: {error}"),
    };
    let trimmed = text.trim();
    let summary: String = if trimmed.is_empty() {
        "The command finished with no output.".to_string()
    } else {
        trimmed.chars().take(8_000).collect()
    };
    send(identity, &summary).await;
}
