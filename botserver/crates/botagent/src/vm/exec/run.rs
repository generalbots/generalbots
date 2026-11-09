//! Buffered and streaming command execution with the cold-start retry (#1798).
//!
//! Split from `exec.rs` so that file stays under the size limit. #2046 added
//! `run_or_stream`, which publishes output lines live; the buffered path is
//! unchanged.

use axum::http::StatusCode;
use tokio::sync::mpsc;

use crate::stream::ExecEvent;
use crate::AgentService;

use super::super::incus::exec_in_vm;

pub(super) fn exec_channel_not_ready(stderr: &str) -> bool {
    let text = stderr.to_ascii_lowercase();
    (text.contains("vm agent")
        && (text.contains("not connected")
            || text.contains("not running")
            || text.contains("starting")))
        || text.contains("instance is not running")
        || text.contains("instance is starting")
        || (text.contains("incus") && text.contains("not ready"))
}

const EXEC_RETRY_ATTEMPTS: u32 = 3;

pub(super) async fn run_with_timeout(
    state: &AgentService,
    vm_name: &str,
    command: String,
    timeout_ms: u64,
) -> Result<(bool, String, String), (StatusCode, String)> {
    for attempt in 0..EXEC_RETRY_ATTEMPTS {
        let result = tokio::time::timeout(
            std::time::Duration::from_millis(timeout_ms),
            exec_in_vm(state, vm_name, &["sh".into(), "-c".into(), command.clone()]),
        )
        .await;
        match result {
            Err(_) => return Ok((false, String::new(), "Execution timed out".to_string())),
            Ok(Err(exec_error)) => return Err(exec_error),
            Ok(Ok((true, stdout, stderr))) => return Ok((true, stdout, stderr)),
            Ok(Ok((false, stdout, stderr))) => {
                if !exec_channel_not_ready(&stderr) || attempt + 1 >= EXEC_RETRY_ATTEMPTS {
                    return Ok((false, stdout, stderr));
                }
                tracing::warn!(
                    "vm exec channel for {vm_name} not ready (attempt {}/{EXEC_RETRY_ATTEMPTS}): {stderr}",
                    attempt + 1
                );
                tokio::time::sleep(std::time::Duration::from_secs(2 * u64::from(attempt + 1)))
                    .await;
            }
        }
    }
    Ok((
        false,
        String::new(),
        "the workspace exec channel did not become ready".to_string(),
    ))
}

/// Runs a command either buffered (default) or streaming its output lines.
///
/// #2046 — a streamed run keeps the partial stdout when the program is killed
/// at the timeout, which is what the trace table shows while the failure is
/// explained. Both variants return the same (success, stdout, stderr) shape.
pub(super) async fn run_or_stream(
    state: &AgentService,
    vm_name: &str,
    command: String,
    timeout_ms: u64,
    events: Option<mpsc::UnboundedSender<ExecEvent>>,
) -> Result<(bool, String, String), (StatusCode, String)> {
    let Some(events) = events else {
        return run_with_timeout(state, vm_name, command, timeout_ms).await;
    };
    // Outer guard: the inner wait already enforces the timeout, but a wedged
    // Incus channel must not outlive this call.
    let guarded = tokio::time::timeout(
        std::time::Duration::from_millis(timeout_ms.saturating_add(10_000)),
        crate::stream::stream_command(
            state,
            vm_name,
            &["sh".into(), "-c".into(), command],
            Some(std::time::Duration::from_millis(timeout_ms)),
            None,
            Some(events),
        ),
    )
    .await;
    match guarded {
        Err(_) => Ok((false, String::new(), "Execution timed out".to_string())),
        Ok(Err(exec_error)) => Err(exec_error),
        Ok(Ok(outcome)) => {
            let mut stderr = outcome.stderr;
            if outcome.timed_out {
                if stderr.trim().is_empty() {
                    stderr = "Execution timed out".to_string();
                } else if !stderr.contains("Execution timed out") {
                    stderr = format!("{stderr}\nExecution timed out");
                }
            }
            Ok((outcome.success, outcome.stdout, stderr))
        }
    }
}
