//! Deadlines for Incus CLI calls.
//!
//! On Windows `incus` is a wrapper that runs `wsl.exe … incus`. While the WSL
//! distro is stopped, booting or busy, one call could hang for minutes and
//! hold an HTTP connection the whole time (the desktop WebView has only six).
//! Every call now has a deadline chosen from the operation; on expiry the
//! whole process tree is terminated and the caller gets `incus_timeout`.
//!
//! A read that must answer quickly (a status or metrics poll) runs inside
//! [`within_budget`]: every Incus call made inside it is capped at the budget,
//! so the processes it started are killed instead of being abandoned.

use axum::http::StatusCode;
use botlib::security::command_guard::SafeCommand;
use std::future::Future;
use std::io::{Read, Write};
use std::process::{Child, Output, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// Error code prefix of a call that did not answer in time.
pub const INCUS_TIMEOUT_CODE: &str = "incus_timeout";
/// Listing, state and configuration calls.
pub const DEFAULT_DEADLINE: Duration = Duration::from_secs(20);
/// Status and metrics reads that a user interface is waiting on.
pub const PROBE_DEADLINE: Duration = Duration::from_secs(5);
/// `start`, `stop`, `restart`, `delete`.
const POWER_DEADLINE: Duration = Duration::from_secs(120);
/// Snapshots, copies, image transfers and file push/pull.
const DATA_DEADLINE: Duration = Duration::from_secs(600);
/// A command run inside the workspace when its caller names no limit
/// (provisioning scripts install packages and CLIs).
pub const EXEC_DEADLINE: Duration = Duration::from_secs(900);
/// How long the pipe readers may take to see end-of-file once the process
/// is gone; longer means a grandchild still holds the pipe open.
const READER_GRACE: Duration = Duration::from_secs(2);
const POLL: Duration = Duration::from_millis(25);

tokio::task_local! {
    static DEADLINE_CAP: Duration;
}

/// `deadline`, lowered to the budget of an enclosing [`within_budget`].
pub fn capped(deadline: Duration) -> Duration {
    DEADLINE_CAP
        .try_with(|cap| deadline.min(*cap))
        .unwrap_or(deadline)
}

/// The deadline of one `incus <args>` call, before any budget cap.
pub(crate) fn base_deadline(args: &[String]) -> Duration {
    let verb = args.first().map(String::as_str).unwrap_or_else(|| {
        tracing::debug!("incus call has no verb; using the default deadline");
        ""
    });
    let sub = args.get(1).map(String::as_str).unwrap_or_else(|| {
        tracing::debug!("incus call has no subcommand; using the default deadline");
        ""
    });
    let read_only_sub = matches!(sub, "list" | "show" | "info" | "get");
    match verb {
        "launch" | "init" => Duration::from_secs(super::incus::cold_launch_wait_secs()),
        "start" | "stop" | "restart" | "delete" => POWER_DEADLINE,
        "snapshot" | "image" | "file" if read_only_sub => DEFAULT_DEADLINE,
        "snapshot" | "image" | "file" | "copy" | "publish" | "export" | "import"
        | "restore" => DATA_DEADLINE,
        "exec" => EXEC_DEADLINE,
        _ => DEFAULT_DEADLINE,
    }
}

/// The deadline of one `incus <args>` call.
pub(crate) fn deadline_for(args: &[String]) -> Duration {
    capped(base_deadline(args))
}

/// Runs `future` with every Incus call inside it capped at `budget`. `None`
/// when the whole future did not finish within the budget (plus a short
/// grace for the kill of the last process).
pub async fn within_budget<F: Future>(budget: Duration, future: F) -> Option<F::Output> {
    let budget = capped(budget);
    DEADLINE_CAP
        .scope(budget, async move {
            tokio::time::timeout(budget + Duration::from_millis(500), future)
                .await
                .ok()
        })
        .await
}

/// The error a caller receives when an Incus call ran out of time.
pub(crate) fn timeout_error(operation: &str, deadline: Duration, orphaned: bool) -> (StatusCode, String) {
    let secs = deadline.as_secs().max(1);
    if orphaned {
        tracing::warn!(
            "incus {operation} did not answer within {secs}s; the process tree was terminated \
             but a pipe stayed open (a child of the wrapper may still run)"
        );
    } else {
        tracing::warn!("incus {operation} did not answer within {secs}s; the process tree was terminated");
    }
    (
        StatusCode::GATEWAY_TIMEOUT,
        format!("{INCUS_TIMEOUT_CODE}: the agent workspace did not answer within {secs} s"),
    )
}

/// True for the error built by [`timeout_error`].
pub fn is_timeout(error: &(StatusCode, String)) -> bool {
    error.0 == StatusCode::GATEWAY_TIMEOUT && error.1.starts_with(INCUS_TIMEOUT_CODE)
}

#[derive(Debug)]
pub(crate) enum RunFailure {
    Spawn(String),
    TimedOut { orphaned: bool },
}

/// Runs the guarded command to completion or until `deadline`, capturing
/// stdout and stderr. Blocking: call it from `spawn_blocking`.
pub(crate) fn output_within(cmd: &SafeCommand, deadline: Duration) -> Result<Output, RunFailure> {
    let mut command = cmd.into_std_command();
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let child = command
        .spawn()
        .map_err(|error| RunFailure::Spawn(error.to_string()))?;
    wait_output(child, deadline)
}

/// [`output_within`] with `input` written to the command's standard input,
/// which is then closed. Bytes travel through the pipe, so their size is not
/// bounded by the command line.
pub(crate) fn output_with_input(cmd: &SafeCommand, input: Vec<u8>, deadline: Duration) -> Result<Output, RunFailure> {
    let mut command = cmd.into_std_command();
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command
        .spawn()
        .map_err(|error| RunFailure::Spawn(error.to_string()))?;
    if let Some(mut pipe) = child.stdin.take() {
        std::thread::spawn(move || {
            if let Err(error) = pipe.write_all(&input) {
                tracing::debug!("incus stdin: write ended early: {error}");
            }
        });
    }
    wait_output(child, deadline)
}

fn wait_output(mut child: Child, deadline: Duration) -> Result<Output, RunFailure> {
    let stdout = pipe_reader(child.stdout.take());
    let stderr = pipe_reader(child.stderr.take());
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                return Ok(Output {
                    status,
                    stdout: drain(&stdout).unwrap_or_else(|| {
                        tracing::debug!("incus stdout reader did not answer; using empty output");
                        Vec::new()
                    }),
                    stderr: drain(&stderr).unwrap_or_else(|| {
                        tracing::debug!("incus stderr reader did not answer; using empty output");
                        Vec::new()
                    }),
                });
            }
            Ok(None) => {}
            Err(error) => {
                kill_tree(&mut child);
                return Err(RunFailure::Spawn(error.to_string()));
            }
        }
        if started.elapsed() >= deadline {
            kill_tree(&mut child);
            let orphaned = drain(&stdout).is_none() || drain(&stderr).is_none();
            return Err(RunFailure::TimedOut { orphaned });
        }
        std::thread::sleep(POLL);
    }
}

fn pipe_reader<R: Read + Send + 'static>(pipe: Option<R>) -> mpsc::Receiver<Vec<u8>> {
    let (sender, receiver) = mpsc::channel();
    match pipe {
        Some(mut pipe) => {
            std::thread::spawn(move || {
                let mut buffer = Vec::new();
                if let Err(error) = pipe.read_to_end(&mut buffer) {
                    tracing::debug!("incus pipe read failed: {error}");
                }
                if sender.send(buffer).is_err() {
                    tracing::debug!("incus pipe reader: no consumer for the captured output");
                }
            });
        }
        None => {
            if sender.send(Vec::new()).is_err() {
                tracing::debug!("incus pipe reader: no consumer for the empty output");
            }
        }
    }
    receiver
}

fn drain(receiver: &mpsc::Receiver<Vec<u8>>) -> Option<Vec<u8>> {
    receiver.recv_timeout(READER_GRACE).ok()
}

/// Terminates `child` and everything it started. On Windows the Incus
/// wrapper's own child is `wsl.exe`, which `Child::kill` alone would leave
/// running, so the tree goes through `taskkill /T`.
pub(crate) fn kill_tree(child: &mut Child) {
    #[cfg(windows)]
    {
        let pid = child.id().to_string();
        match SafeCommand::new("taskkill")
            .and_then(|command| command.args(&["/F", "/T", "/PID", &pid]))
            .and_then(|command| command.execute())
        {
            Ok(output) if !output.status.success() => tracing::debug!(
                "taskkill /T for incus pid {pid} exited {}",
                output.status
            ),
            Ok(_) => {}
            Err(error) => tracing::warn!("taskkill for incus pid {pid} rejected: {error}"),
        }
    }
    if let Err(error) = child.kill() {
        tracing::debug!("kill_tree: child was already gone: {error}");
    }
    if let Err(error) = child.wait() {
        tracing::debug!("kill_tree: could not reap the child: {error}");
    }
}

#[cfg(test)]
#[path = "incus_run_tests.rs"]
mod tests;
