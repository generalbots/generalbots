//! One VM shell script fed from bytes on its standard input.
//!
//! Files and prompts used to reach the container as base64 pieces inside
//! `sh -c` arguments, one Incus call per piece: the command guard caps a
//! script at 16 KB, so a long paste failed with "Script too long", and each
//! piece cost a full round trip through the WSL wrapper. Here the bytes go
//! through the pipe of a single `incus exec -T`, whatever their size.

use std::time::Duration;

use axum::http::StatusCode;
use botlib::security::command_guard::{CommandGuardError, SafeCommand};

use crate::AgentService;

use super::incus_ready::ready_command;
use super::incus_errors::incus_setup_error;
use super::incus_run::{capped, output_with_input, timeout_error, RunFailure};
use super::internal;

/// `<incus> exec <vm> -T -- sh -c <script>`; `-T` keeps the pipe plain, so
/// the bytes pass unchanged.
fn stdin_command(base: SafeCommand, vm_name: &str, script: &str) -> Result<SafeCommand, CommandGuardError> {
    base.arg("exec")?
        .arg(vm_name)?
        .arg("-T")?
        .arg("--")?
        .arg("sh")?
        .arg("-c")?
        .trusted_shell_script_arg(script)
}

/// Runs `script` in `vm_name` as root with `input` on its standard input.
/// Returns `(success, stdout, stderr)`; a run still going at `limit` is
/// killed and reported as `incus_timeout`.
pub async fn exec_in_vm_with_input(
    state: &AgentService,
    vm_name: &str,
    script: &str,
    input: Vec<u8>,
    limit: Duration,
) -> Result<(bool, String, String), (StatusCode, String)> {
    let limit = capped(limit);
    let cmd = stdin_command(ready_command(state).await?, vm_name, script)
        .map_err(|error| internal(format!("incus script: {error}")))?;
    let finished = tokio::task::spawn_blocking(move || output_with_input(&cmd, input, limit))
        .await
        .map_err(|error| internal(format!("incus wait join: {error}")))?;
    match finished {
        Ok(output) => Ok((
            output.status.success(),
            String::from_utf8_lossy(&output.stdout).into_owned(),
            String::from_utf8_lossy(&output.stderr).into_owned(),
        )),
        Err(RunFailure::Spawn(detail)) => {
            Err(incus_setup_error(&detail).unwrap_or_else(|| internal(format!("incus spawn: {detail}"))))
        }
        Err(RunFailure::TimedOut { orphaned }) => Err(timeout_error("exec", limit, orphaned)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bytes_never_travel_in_the_command_line() {
        let base = SafeCommand::new("sh").expect("sh is an allowed command");
        let script = "cat > '/opt/vibe/conversations/c/data/a b.txt'";
        let cmd = stdin_command(base, "ag-test", script).expect("a short script passes the guard");
        let args: Vec<String> = cmd
            .into_std_command()
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        assert_eq!(&args[..6], ["exec", "ag-test", "-T", "--", "sh", "-c"]);
        assert_eq!(args.len(), 7, "{args:?}");
    }
}
