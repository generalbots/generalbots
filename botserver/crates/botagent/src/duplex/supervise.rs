//! The threads behind a [`DuplexSession`](super::DuplexSession): one reader
//! per output pipe, one writer for stdin and one supervisor that owns the
//! child process. Only the supervisor ever waits on or kills the child, so a
//! session ends exactly once and always through [`kill_tree`] when the peer
//! does not leave by itself.

use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStderr, ChildStdin, ChildStdout};
use std::sync::mpsc::{self as std_mpsc, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use botlib::security::command_guard::SafeCommand;
use tokio::sync::{mpsc, watch};

use super::types::{DuplexEvent, EndReason, SessionEnd};
use crate::vm::incus_run::output_within;
use crate::vm::kill_tree;

const POLL: Duration = Duration::from_millis(25);
/// How long the stdout reader may take to reach end-of-file once the child is
/// gone; longer means a grandchild still holds the pipe open.
const READER_GRACE: Duration = Duration::from_secs(2);
const STDERR_GRACE: Duration = Duration::from_millis(300);
const READ_BUFFER: usize = 64 * 1024;
const STDERR_CHUNK: usize = 4096;
/// Limit for the cleanup command run after the session ends.
const CLEANUP_DEADLINE: Duration = Duration::from_secs(30);

/// Requests to the supervisor.
pub(super) enum Control {
    /// Close stdin was requested: wait the grace period, then kill the tree.
    Close,
    /// End now: kill the tree and report this reason.
    Abort(EndReason),
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum LineRead {
    Line,
    Eof,
    TooLong,
}

/// Reads one `\n`-terminated line into `line` (without the terminator).
/// A line longer than `cap` bytes is never buffered past the cap: the call
/// returns [`LineRead::TooLong`] and the caller ends the session. A final
/// line without a terminator is still delivered before end-of-file.
pub(super) fn read_line_capped<R: BufRead>(
    reader: &mut R,
    line: &mut Vec<u8>,
    cap: usize,
) -> std::io::Result<LineRead> {
    line.clear();
    loop {
        let available = match reader.fill_buf() {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        };
        if available.is_empty() {
            return Ok(if line.is_empty() {
                LineRead::Eof
            } else {
                LineRead::Line
            });
        }
        let newline = available.iter().position(|byte| *byte == b'\n');
        let take = newline.unwrap_or(available.len());
        if line.len() + take > cap {
            return Ok(LineRead::TooLong);
        }
        line.extend_from_slice(&available[..take]);
        match newline {
            Some(position) => {
                reader.consume(position + 1);
                return Ok(LineRead::Line);
            }
            None => reader.consume(take),
        }
    }
}

/// The last bytes the peer wrote to stderr, for diagnostics only. It is
/// never logged here: callers decide what, if anything, may be shown.
pub(super) struct StderrTail {
    bytes: VecDeque<u8>,
    cap: usize,
    trimmed: bool,
}

impl StderrTail {
    pub(super) fn new(cap: usize) -> Self {
        Self {
            bytes: VecDeque::new(),
            cap: cap.max(1),
            trimmed: false,
        }
    }

    pub(super) fn push(&mut self, chunk: &[u8]) {
        self.bytes.extend(chunk.iter().copied());
        while self.bytes.len() > self.cap {
            self.bytes.pop_front();
            self.trimmed = true;
        }
    }

    /// The tail as text; after trimming, the cut first line is dropped.
    pub(super) fn text(&self) -> String {
        let bytes: Vec<u8> = self.bytes.iter().copied().collect();
        let start = if self.trimmed {
            bytes
                .iter()
                .position(|byte| *byte == b'\n')
                .map_or(0, |position| position + 1)
        } else {
            0
        };
        String::from_utf8_lossy(&bytes[start..])
            .trim_end()
            .to_string()
    }
}

pub(super) fn tail_text(tail: &Mutex<StderrTail>) -> String {
    tail.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .text()
}

fn into_text(line: Vec<u8>) -> String {
    match String::from_utf8(line) {
        Ok(text) => text,
        Err(error) => String::from_utf8_lossy(error.as_bytes()).into_owned(),
    }
}

/// Delivers stdout lines in order. Returns the "reader finished" signal.
pub(super) fn spawn_stdout_reader(
    pipe: ChildStdout,
    cap: usize,
    events: mpsc::Sender<DuplexEvent>,
    control: Sender<Control>,
) -> Receiver<()> {
    let (done, finished) = std_mpsc::channel();
    std::thread::spawn(move || {
        let mut reader = BufReader::with_capacity(READ_BUFFER, pipe);
        let mut line = Vec::new();
        let mut listener_gone = false;
        loop {
            match read_line_capped(&mut reader, &mut line, cap) {
                Ok(LineRead::Line) => {
                    if line.last() == Some(&b'\r') {
                        line.pop();
                    }
                    let text = into_text(std::mem::take(&mut line));
                    // Keep draining when nobody listens, so the peer never
                    // blocks on a full pipe while the session winds down.
                    if !listener_gone && events.blocking_send(DuplexEvent::Line(text)).is_err() {
                        listener_gone = true;
                    }
                }
                Ok(LineRead::Eof) => break,
                Ok(LineRead::TooLong) => {
                    let _ = control.send(Control::Abort(EndReason::LineTooLong));
                    break;
                }
                Err(_) => {
                    let _ = control.send(Control::Abort(EndReason::ReadFailed));
                    break;
                }
            }
        }
        let _ = done.send(());
    });
    finished
}

/// Keeps the bounded stderr tail. Returns the "reader finished" signal.
pub(super) fn spawn_stderr_reader(
    mut pipe: ChildStderr,
    tail: Arc<Mutex<StderrTail>>,
) -> Receiver<()> {
    let (done, finished) = std_mpsc::channel();
    std::thread::spawn(move || {
        let mut chunk = [0u8; STDERR_CHUNK];
        loop {
            match pipe.read(&mut chunk) {
                Ok(0) => break,
                Ok(read) => tail
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push(&chunk[..read]),
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
                Err(_) => break,
            }
        }
        let _ = done.send(());
    });
    finished
}

/// Writes queued lines to stdin in order. When every queue sender is gone
/// the thread ends and drops stdin, so the peer reads end-of-file.
pub(super) fn spawn_writer(mut stdin: ChildStdin, lines: Receiver<Vec<u8>>) {
    std::thread::spawn(move || {
        for bytes in lines {
            if stdin
                .write_all(&bytes)
                .and_then(|()| stdin.flush())
                .is_err()
            {
                break;
            }
        }
    });
}

pub(super) struct Supervision {
    pub(super) child: Child,
    pub(super) control: Receiver<Control>,
    pub(super) deadline: Duration,
    pub(super) close_grace: Duration,
    pub(super) stdout_done: Receiver<()>,
    pub(super) stderr_done: Receiver<()>,
    pub(super) tail: Arc<Mutex<StderrTail>>,
    pub(super) events: mpsc::Sender<DuplexEvent>,
    pub(super) ended: watch::Sender<Option<SessionEnd>>,
    /// Run once the child is gone, before `Ended` is published: removes
    /// what the child left behind where `kill_tree` cannot reach.
    pub(super) cleanup: Option<SafeCommand>,
}

pub(super) fn spawn_supervisor(supervision: Supervision) {
    std::thread::spawn(move || supervise(supervision));
}

fn supervise(mut run: Supervision) {
    let started = Instant::now();
    let (reason, exit_code) = watch_child(&mut run, started);
    if let Some(cleanup) = run.cleanup.take() {
        match output_within(&cleanup, CLEANUP_DEADLINE) {
            Ok(output) if output.status.success() => {}
            Ok(output) => tracing::warn!("duplex cleanup exited {}", output.status),
            Err(failure) => tracing::warn!("duplex cleanup did not run: {failure:?}"),
        }
    }
    // Every line read before the end is delivered before `Ended`.
    let _ = run.stdout_done.recv_timeout(READER_GRACE);
    let _ = run.stderr_done.recv_timeout(STDERR_GRACE);
    let end = SessionEnd {
        reason,
        exit_code,
        stderr_tail: tail_text(&run.tail),
        elapsed: started.elapsed(),
    };
    tracing::debug!(
        "duplex session ended: {:?}, exit code {:?}, after {} ms",
        end.reason,
        end.exit_code,
        end.elapsed.as_millis()
    );
    run.ended.send_replace(Some(end.clone()));
    let _ = run.events.blocking_send(DuplexEvent::Ended(end));
}

fn watch_child(run: &mut Supervision, started: Instant) -> (EndReason, Option<i32>) {
    let mut closing_since: Option<Instant> = None;
    let mut control_open = true;
    loop {
        match run.child.try_wait() {
            Ok(Some(status)) => {
                let reason = if closing_since.is_some() {
                    EndReason::Closed
                } else {
                    EndReason::PeerExited
                };
                return (reason, status.code());
            }
            Ok(None) => {}
            Err(_) => {
                kill_tree(&mut run.child);
                return (EndReason::WaitFailed, None);
            }
        }
        if started.elapsed() >= run.deadline {
            kill_tree(&mut run.child);
            return (EndReason::Deadline, None);
        }
        if closing_since.is_some_and(|since| since.elapsed() >= run.close_grace) {
            kill_tree(&mut run.child);
            return (EndReason::Closed, None);
        }
        let request = if control_open {
            match run.control.recv_timeout(POLL) {
                Ok(request) => Some(request),
                Err(RecvTimeoutError::Timeout) => None,
                Err(RecvTimeoutError::Disconnected) => {
                    control_open = false;
                    Some(Control::Close)
                }
            }
        } else {
            std::thread::sleep(POLL);
            None
        };
        match request {
            Some(Control::Close) => {
                closing_since.get_or_insert_with(Instant::now);
            }
            Some(Control::Abort(reason)) => {
                kill_tree(&mut run.child);
                return (reason, None);
            }
            None => {}
        }
    }
}
