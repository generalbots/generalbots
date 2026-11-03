//! Streaming process execution (#1667).
//!
//! `exec_in_vm` used to return `(bool, stdout, stderr)` after the process
//! exited: progress was invisible and the real exit code was flattened into a
//! boolean. This module keeps the buffered API for existing callers but adds a
//! streaming path with line events, a budget for payload size, a timeout and
//! cooperative cancellation.

use std::io::BufRead;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::http::StatusCode;
use serde::Serialize;
use tokio::sync::mpsc;

use crate::vm::{exec_in_vm_streaming, ExecOutcome};

const MAX_LINE_CHARS: usize = 8_000;
const MAX_CAPTURE_BYTES: usize = 256 * 1024;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ExecEvent {
    Started { command: String },
    Stdout { line: String },
    Stderr { line: String },
    Finished {
        exit_code: Option<i32>,
        duration_ms: u64,
        cancelled: bool,
        timed_out: bool,
    },
}

/// Runs a command and publishes line events while it runs.
pub async fn stream_command(
    state: &crate::AgentService,
    vm_name: &str,
    args: &[String],
    timeout: Option<Duration>,
    cancel: Option<Arc<AtomicBool>>,
    events: Option<mpsc::UnboundedSender<ExecEvent>>,
) -> Result<ExecOutcome, (StatusCode, String)> {
    exec_in_vm_streaming(state, vm_name, args, timeout, cancel, events).await
}

/// Shared by the stdout and stderr pumps.
pub(crate) fn pump_lines<R: std::io::Read + Send + 'static>(
    reader: R,
    is_stderr: bool,
    events: Option<mpsc::UnboundedSender<ExecEvent>>,
    capture: Arc<Mutex<String>>,
) {
    let mut buffered = std::io::BufReader::new(reader);
    let mut line = String::new();
    loop {
        line.clear();
        match buffered.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                let trimmed = line.trim_end_matches(['\r', '\n']);
                let text: String = trimmed.chars().take(MAX_LINE_CHARS).collect();
                if let Some(sender) = &events {
                    let event = if is_stderr {
                        ExecEvent::Stderr { line: text.clone() }
                    } else {
                        ExecEvent::Stdout { line: text.clone() }
                    };
                    let _ = sender.send(event);
                }
                if let Ok(mut buffer) = capture.lock() {
                    if buffer.len() < MAX_CAPTURE_BYTES {
                        buffer.push_str(trimmed);
                        buffer.push('\n');
                    } else if !buffer.ends_with("[output truncated]\n") {
                        buffer.push_str("[output truncated]\n");
                    }
                }
            }
            Err(_) => break,
        }
    }
}

/// Registers the streaming entry point with `vm.rs` without making that file
/// depend on this module's event type. `exec_in_vm` builds the guarded
/// command; this helper only watches the child.
pub(crate) fn wait_for_child(
    mut child: std::process::Child,
    started: Instant,
    timeout: Option<Duration>,
    cancel: Option<Arc<AtomicBool>>,
) -> (Option<i32>, bool, bool) {
    let deadline = timeout.map(|value| started + value);
    let mut cancelled = false;
    let mut timed_out = false;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return (status.code(), cancelled, timed_out),
            Ok(None) => {}
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return (None, cancelled, timed_out);
            }
        }
        if cancel
            .as_ref()
            .is_some_and(|flag| flag.load(Ordering::SeqCst))
        {
            cancelled = true;
            let _ = child.kill();
            let _ = child.wait();
            return (None, cancelled, timed_out);
        }
        if deadline.is_some_and(|value| Instant::now() >= value) {
            timed_out = true;
            let _ = child.kill();
            let _ = child.wait();
            return (None, cancelled, timed_out);
        }
        std::thread::sleep(Duration::from_millis(25));
    }
}
