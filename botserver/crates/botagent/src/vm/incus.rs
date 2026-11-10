//! Incus CLI access: guarded command construction, instance state polling and
//! command execution (buffered and streaming).

use axum::http::StatusCode;
use botlib::security::command_guard::SafeCommand;
use std::process::Stdio;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::{Duration as StdDuration, Instant};
use tokio::sync::mpsc;

use crate::stream::{pump_lines, wait_for_child, ExecEvent};
use crate::AgentService;

use super::internal;
use super::retry::{cold_start_timeout_message, retry_incus};

const LAUNCH_WAIT_SECS: u64 = 60;
/// First launch may download and unpack the image; the 60 s restart budget is
/// not enough on a cold host (issue #1744). Override with
/// AGENT_VM_LAUNCH_WAIT_SECS when a slower mirror needs longer.
const COLD_LAUNCH_WAIT_SECS: u64 = 900;
const POLL_INTERVAL_SECS: u64 = 3;

/// Remediation returned when the host has no usable Incus daemon. The text is
/// fixed and free of host paths/identities, so it is safe to send to API
/// callers (the raw command output stays in the log).
const INCUS_MISSING_MESSAGE: &str = "Incus is not available on this host: install the incus package (Debian/Ubuntu: apt install incus), run `incus admin init --minimal`, add the botserver user to the incus-admin group, then restart the botserver service";

/// Remediation for the socket-permission failure: `/var/lib/incus/unix.socket`
/// is `root:incus-admin 0660` and the daemon checks the peer credentials, so
/// the group must be present in the running process, not only in `/etc/group`.
const INCUS_PERMISSION_MESSAGE: &str = "The botserver service cannot access the Incus daemon: add the botserver user to the incus-admin group (sudo usermod -aG incus-admin botserver) and restart the botserver service";

/// Maps an Incus failure to an actionable capability error when it is caused
/// by host setup (missing binary/daemon or socket permission). Returns `None`
/// for everything else so callers keep the generic sanitized message and its
/// detail stays in the log.
fn incus_setup_error(detail: &str) -> Option<(StatusCode, String)> {
    let lower = detail.to_ascii_lowercase();
    let permission_denied = lower.contains("permission denied")
        || lower.contains("needed permissions")
        || lower.contains("access denied")
        || lower.contains("os error 13");
    if permission_denied {
        tracing::error!("incus socket permission denied: {detail}");
        return Some((
            StatusCode::SERVICE_UNAVAILABLE,
            INCUS_PERMISSION_MESSAGE.to_string(),
        ));
    }
    let daemon_missing = lower.contains("no such file or directory")
        || lower.contains("os error 2")
        // #2086 — Windows reports a missing binary differently. Without these
        // phrases the chat pipeline retried `incus list` five times (≈30 s)
        // before every LLM call on a host that has no Incus at all.
        || lower.contains("cannot find the file specified")
        || lower.contains("not recognized as an internal or external command")
        || lower.contains("os error 3")
        || lower.contains("failed to connect to local daemon")
        || lower.contains("daemon is not running")
        || lower.contains("cannot connect to the incus daemon");
    if daemon_missing {
        tracing::error!("incus unavailable: {detail}");
        return Some((
            StatusCode::SERVICE_UNAVAILABLE,
            INCUS_MISSING_MESSAGE.to_string(),
        ));
    }
    None
}

fn incus_command(state: &AgentService) -> Result<SafeCommand, (StatusCode, String)> {
    // #2086 — there is no Incus daemon on Windows: the agent-VM path must fail
    // fast as a capability error instead of retrying `incus list` five times
    // (≈30 s) before every LLM call.
    if cfg!(windows) {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            INCUS_MISSING_MESSAGE.to_string(),
        ));
    }
    SafeCommand::new(state.incus_bin()).map_err(|e| {
        tracing::error!("incus command rejected by guard: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "VM subsystem unavailable".to_string(),
        )
    })
}

pub(super) async fn run_incus(
    state: &AgentService,
    args: Vec<String>,
) -> Result<std::process::Output, (StatusCode, String)> {
    let mut cmd = incus_command(state)?;
    for arg in &args {
        cmd = cmd
            .arg(arg)
            .map_err(|e| internal(format!("incus arg guard: {e}")))?;
    }
    let output = tokio::task::spawn_blocking(move || cmd.execute())
        .await
        .map_err(|e| internal(format!("incus join: {e}")))?
        .map_err(|e| match incus_setup_error(&e.to_string()) {
            Some(err) => err,
            None => internal(format!("incus execution: {e}")),
        })?;
    if !output.status.success() {
        // `incus` exits non-zero for a missing daemon and for the socket
        // permission check. Turn those two into capability errors; every
        // other failure keeps its previous best-effort behavior (callers such
        // as the NIC attachment intentionally ignore failures).
        let stderr = String::from_utf8_lossy(&output.stderr);
        if let Some(err) = incus_setup_error(&stderr) {
            return Err(err);
        }
    }
    Ok(output)
}

fn running_from_list(stdout: &str, vm_name: &str) -> bool {
    match serde_json::from_str::<serde_json::Value>(stdout) {
        Ok(serde_json::Value::Array(items)) => items.iter().any(|item| {
            item.get("name").and_then(|n| n.as_str()) == Some(vm_name)
                && item
                    .get("status")
                    .and_then(|s| s.as_str())
                    .map(|s| s.eq_ignore_ascii_case("running"))
                    .unwrap_or(false)
        }),
        _ => false,
    }
}

fn exists_from_list(stdout: &str, vm_name: &str) -> bool {
    match serde_json::from_str::<serde_json::Value>(stdout) {
        Ok(serde_json::Value::Array(items)) => items
            .iter()
            .any(|item| item.get("name").and_then(|n| n.as_str()) == Some(vm_name)),
        _ => false,
    }
}

async fn list_instances(state: &AgentService) -> Result<String, (StatusCode, String)> {
    let output = retry_incus(
        state,
        &[
            "list".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ],
        "list",
    )
    .await?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub(crate) async fn is_running(
    state: &AgentService,
    vm_name: &str,
) -> Result<bool, (StatusCode, String)> {
    let stdout = list_instances(state).await?;
    Ok(running_from_list(&stdout, vm_name))
}

/// True when the instance exists at all (RUNNING or STOPPED). A container that
/// is still being created by a cold `incus launch` is absent here, which is why
/// callers must wait rather than launch a second copy (issue #1798).
pub(crate) async fn container_exists(
    state: &AgentService,
    vm_name: &str,
) -> Result<bool, (StatusCode, String)> {
    let stdout = list_instances(state).await?;
    Ok(exists_from_list(&stdout, vm_name))
}

pub(super) fn cold_launch_wait_secs() -> u64 {
    std::env::var("AGENT_VM_LAUNCH_WAIT_SECS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value >= LAUNCH_WAIT_SECS)
        .unwrap_or(COLD_LAUNCH_WAIT_SECS)
}

pub(crate) async fn wait_for_running(
    state: &AgentService,
    vm_name: &str,
) -> Result<(), (StatusCode, String)> {
    wait_for_running_within(state, vm_name, LAUNCH_WAIT_SECS).await
}

pub(super) async fn wait_for_running_within(
    state: &AgentService,
    vm_name: &str,
    wait_secs: u64,
) -> Result<(), (StatusCode, String)> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(wait_secs);
    let mut last_error: Option<String> = None;
    while std::time::Instant::now() < deadline {
        // A cold start makes the daemon busy: `incus list` can fail briefly
        // while the image is unpacked or the instance is created. Keep polling
        // until the deadline instead of failing the whole workspace on the
        // first transient error (issue #1798).
        match is_running(state, vm_name).await {
            Ok(true) => return Ok(()),
            Ok(false) => last_error = None,
            Err((code, detail)) => {
                tracing::warn!(
                    "incus state check for {vm_name} failed ({code}): {detail}; retrying"
                );
                last_error = Some(detail);
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(POLL_INTERVAL_SECS)).await;
    }
    let message = match &last_error {
        Some(detail) => format!(
            "{}; the last Incus state check failed with: {detail}",
            cold_start_timeout_message(vm_name, wait_secs)
        ),
        None => cold_start_timeout_message(vm_name, wait_secs),
    };
    tracing::error!(
        "container {vm_name} did not reach RUNNING within {wait_secs}s{}",
        last_error
            .as_deref()
            .map(|detail| format!("; last error: {detail}"))
            .unwrap_or_default()
    );
    Err((StatusCode::SERVICE_UNAVAILABLE, message))
}

/// Last `ERROR` entries of the container's own start log. The host-side
/// reason a container refuses to run lives here, not in the start exit code
/// (the daemon reports exit 0 while the instance stays STOPPED).
pub(super) async fn instance_log_tail(state: &AgentService, vm_name: &str) -> Option<String> {
    let output = run_incus(
        state,
        vec![
            "info".to_string(),
            vm_name.to_string(),
            "--show-log".to_string(),
        ],
    )
    .await
    .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    let errors: Vec<String> = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && line.contains("ERROR"))
        .rev()
        .take(2)
        .map(|line| line.chars().take(160).collect::<String>())
        .collect();
    if errors.is_empty() {
        return None;
    }
    Some(errors.join(" | "))
}

pub async fn exec_in_vm(
    state: &AgentService,
    vm_name: &str,
    cmd_args: &[String],
) -> Result<(bool, String, String), (StatusCode, String)> {
    let outcome = exec_in_vm_streaming(state, vm_name, cmd_args, None, None, None).await?;
    Ok((outcome.success, outcome.stdout, outcome.stderr))
}

/// The result of one VM command. `success` is kept separate from `exit_code`
/// because a cancelled or timed-out process has no exit code at all.
#[derive(Debug, Clone, Default)]
pub struct ExecOutcome {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub cancelled: bool,
    pub timed_out: bool,
}

/// Runs a command and streams stdout/stderr as line events while it runs.
///
/// The guarded `SafeCommand` still builds the invocation; this path only adds
/// piped readers, a real exit code, a timeout and cooperative cancellation.
pub async fn exec_in_vm_streaming(
    state: &AgentService,
    vm_name: &str,
    cmd_args: &[String],
    timeout: Option<StdDuration>,
    cancel: Option<Arc<AtomicBool>>,
    events: Option<mpsc::UnboundedSender<ExecEvent>>,
) -> Result<ExecOutcome, (StatusCode, String)> {
    let mut cmd = incus_command(state)?;
    cmd = cmd
        .arg("exec")
        .map_err(|error| internal(format!("incus arg: {error}")))?;
    cmd = cmd
        .arg(vm_name)
        .map_err(|error| internal(format!("incus arg: {error}")))?;
    cmd = cmd
        .arg("--")
        .map_err(|error| internal(format!("incus arg: {error}")))?;

    let is_shell_script = cmd_args.len() == 3 && cmd_args[0] == "sh" && cmd_args[1] == "-c";
    if is_shell_script {
        cmd = cmd
            .arg("sh")
            .map_err(|error| internal(format!("incus arg: {error}")))?;
        cmd = cmd
            .arg("-c")
            .map_err(|error| internal(format!("incus arg: {error}")))?;
        cmd = cmd
            .trusted_shell_script_arg(&cmd_args[2])
            .map_err(|error| internal(format!("incus script: {error}")))?;
    } else {
        for arg in cmd_args {
            cmd = cmd
                .arg(arg)
                .map_err(|error| internal(format!("incus arg: {error}")))?;
        }
    }
    cmd = cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    if let Some(sender) = &events {
        let label: String = cmd_args.join(" ").chars().take(500).collect();
        let _ = sender.send(ExecEvent::Started { command: label });
    }

    let started = Instant::now();
    let mut child = cmd
        .spawn()
        .map_err(|error| match incus_setup_error(&error.to_string()) {
            Some(err) => err,
            None => internal(format!("incus spawn: {error}")),
        })?;

    let stdout_pipe = child.stdout.take();
    let stderr_pipe = child.stderr.take();
    let stdout_capture = Arc::new(Mutex::new(String::new()));
    let stderr_capture = Arc::new(Mutex::new(String::new()));
    let mut readers = Vec::new();

    if let Some(pipe) = stdout_pipe {
        let capture = stdout_capture.clone();
        let sender = events.clone();
        readers.push(std::thread::spawn(move || {
            pump_lines(pipe, false, sender, capture);
        }));
    }
    if let Some(pipe) = stderr_pipe {
        let capture = stderr_capture.clone();
        let sender = events.clone();
        readers.push(std::thread::spawn(move || {
            pump_lines(pipe, true, sender, capture);
        }));
    }

    let wait = tokio::task::spawn_blocking(move || wait_for_child(child, started, timeout, cancel))
        .await
        .map_err(|error| internal(format!("incus wait join: {error}")))?;
    let (exit_code, cancelled, timed_out) = wait;

    for reader in readers {
        let _ = reader.join();
    }

    let duration_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
    let success = exit_code == Some(0) && !cancelled && !timed_out;
    let stdout = stdout_capture
        .lock()
        .map(|buffer| buffer.clone())
        .unwrap_or_default();
    let stderr = stderr_capture
        .lock()
        .map(|buffer| buffer.clone())
        .unwrap_or_default();

    if let Some(sender) = &events {
        let _ = sender.send(ExecEvent::Finished {
            exit_code,
            duration_ms,
            cancelled,
            timed_out,
        });
    }

    Ok(ExecOutcome {
        success,
        exit_code,
        stdout,
        stderr,
        duration_ms,
        cancelled,
        timed_out,
    })
}

#[cfg(test)]
mod incus_error_tests {
    use super::{incus_setup_error, INCUS_MISSING_MESSAGE, INCUS_PERMISSION_MESSAGE};
    use axum::http::StatusCode;

    #[test]
    fn missing_binary_is_actionable() {
        let (code, message) =
            incus_setup_error("Command execution failed: No such file or directory (os error 2)")
                .expect("missing binary must map to a setup error");
        assert_eq!(code, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(message, INCUS_MISSING_MESSAGE);
        assert!(message.contains("incus-admin"));
    }

    #[test]
    fn socket_permission_is_actionable() {
        let (code, message) = incus_setup_error(
            "Error: You don't have the needed permissions to talk with the incus daemon",
        )
        .expect("socket permission must map to a setup error");
        assert_eq!(code, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(message, INCUS_PERMISSION_MESSAGE);
        assert!(message.contains("usermod -aG incus-admin"));
    }

    #[test]
    fn unreachable_daemon_is_actionable() {
        let (code, message) = incus_setup_error(
            "Error: Failed to connect to local daemon: dial unix socket: no such file or directory",
        )
        .expect("missing daemon must map to a setup error");
        assert_eq!(code, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(message, INCUS_MISSING_MESSAGE);
    }

    #[test]
    fn unrelated_failures_keep_the_generic_error() {
        assert!(incus_setup_error("Error: Instance ag-u1234 is not running").is_none());
        assert!(incus_setup_error("Error: Image not found").is_none());
    }

    #[test]
    fn windows_missing_binary_is_a_capability_error() {
        // #2086 — Windows phrases that used to fall through and be retried five
        // times per chat message.
        for detail in [
            "Command execution failed: The system cannot find the file specified. (os error 3)",
            "'incus' is not recognized as an internal or external command, operable program or batch file.",
        ] {
            let (code, message) =
                incus_setup_error(detail).expect("windows missing binary must map to setup error");
            assert_eq!(code, StatusCode::SERVICE_UNAVAILABLE);
            assert_eq!(message, INCUS_MISSING_MESSAGE);
        }
    }
}
