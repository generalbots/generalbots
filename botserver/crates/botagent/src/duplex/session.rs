//! A long-lived, two-way line session with one child process.

use std::process::Stdio;
use std::sync::mpsc::{self as std_mpsc, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use botlib::security::command_guard::SafeCommand;
use serde::Serialize;
use tokio::sync::{mpsc, watch};

use super::supervise::{
    spawn_stderr_reader, spawn_stdout_reader, spawn_supervisor, spawn_writer, tail_text, Control,
    StderrTail, Supervision,
};
use super::types::{DuplexError, DuplexEvent, DuplexOptions, EndReason, SessionEnd};
use crate::vm::kill_tree;

const EVENT_BUFFER: usize = 1024;

struct Shared {
    writer: Mutex<Option<Sender<Vec<u8>>>>,
    control: Sender<Control>,
    ended: watch::Receiver<Option<SessionEnd>>,
    tail: Arc<Mutex<StderrTail>>,
    max_line_bytes: usize,
}

impl Drop for Shared {
    /// The last handle is gone: close stdin (the writer queue drops with
    /// `self`) and let the supervisor wait the grace period, then kill.
    fn drop(&mut self) {
        if self.control.send(Control::Close).is_err() {
            tracing::debug!("duplex control channel closed before Close could be sent");
        }
    }
}

/// The sending half. Cheap to clone; the session stays open while any
/// clone, the [`DuplexEvents`] or the [`DuplexSession`] is alive.
#[derive(Clone)]
pub struct DuplexSender {
    shared: Arc<Shared>,
}

impl std::fmt::Debug for DuplexSender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DuplexSender")
            .field("ended", &self.ended().map(|end| end.reason))
            .finish_non_exhaustive()
    }
}

impl DuplexSender {
    /// Queues one line (a `\n` is appended). Lines are written in call
    /// order. Fails when the line holds `\n` or `\r`, is over the line cap,
    /// stdin is closed or the session ended.
    pub fn send(&self, line: &str) -> Result<(), DuplexError> {
        if line.contains(['\n', '\r']) {
            return Err(DuplexError::InvalidLine(
                "a line must not contain a newline",
            ));
        }
        if line.len() > self.shared.max_line_bytes {
            return Err(DuplexError::InvalidLine("the line is over the line cap"));
        }
        if let Some(end) = self.ended() {
            return Err(DuplexError::Ended(end.reason));
        }
        let mut bytes = Vec::with_capacity(line.len() + 1);
        bytes.extend_from_slice(line.as_bytes());
        bytes.push(b'\n');
        let writer = self
            .shared
            .writer
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match writer.as_ref() {
            Some(queue) => queue.send(bytes).map_err(|_| DuplexError::StdinClosed),
            None => Err(DuplexError::StdinClosed),
        }
    }

    /// Serialises `value` as one JSON line and sends it.
    pub fn send_json<T: Serialize + ?Sized>(&self, value: &T) -> Result<(), DuplexError> {
        let line = serde_json::to_string(value)
            .map_err(|_| DuplexError::InvalidLine("the value is not serialisable"))?;
        self.send(&line)
    }

    /// Closes stdin once the queued lines are written. The session keeps
    /// reading until the peer exits (or the deadline).
    pub fn close_stdin(&self) {
        self.shared
            .writer
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
    }

    /// Closes stdin and ends the session: the peer gets the grace period to
    /// exit, then its whole process tree is killed. Returns at once; use
    /// [`Self::wait_ended`] for the result.
    pub fn close(&self) {
        self.close_stdin();
        if self.shared.control.send(Control::Close).is_err() {
            tracing::debug!("duplex control channel closed before close_stdin could finish");
        }
    }

    /// `Some` once the session ended ("peer exited" signal, non-blocking).
    pub fn ended(&self) -> Option<SessionEnd> {
        self.shared.ended.borrow().clone()
    }

    /// Waits until the session ended.
    pub async fn wait_ended(&self) -> SessionEnd {
        let mut ended = self.shared.ended.clone();
        loop {
            if let Some(end) = ended.borrow_and_update().clone() {
                return end;
            }
            if ended.changed().await.is_err() {
                return ended.borrow().clone().unwrap_or_else(|| SessionEnd {
                    reason: EndReason::WaitFailed,
                    exit_code: None,
                    stderr_tail: tail_text(&self.shared.tail),
                    elapsed: Duration::ZERO,
                });
            }
        }
    }

    /// What the peer printed to stderr so far (bounded). Diagnostics only.
    pub fn stderr_tail(&self) -> String {
        tail_text(&self.shared.tail)
    }
}

/// The receiving half: stdout lines in order, then exactly one
/// [`DuplexEvent::Ended`].
pub struct DuplexEvents {
    events: mpsc::Receiver<DuplexEvent>,
    done: bool,
    sender: DuplexSender,
}

impl DuplexEvents {
    /// The next event; `None` after [`DuplexEvent::Ended`] was returned.
    pub async fn recv(&mut self) -> Option<DuplexEvent> {
        if self.done {
            return None;
        }
        let event = match self.events.recv().await {
            Some(event) => event,
            None => DuplexEvent::Ended(self.sender.wait_ended().await),
        };
        if matches!(event, DuplexEvent::Ended(_)) {
            self.done = true;
        }
        Some(event)
    }

    /// The next stdout line if one is already queued; `None` when none is
    /// (or the session ended: see [`DuplexSender::ended`]).
    pub fn try_recv_line(&mut self) -> Option<String> {
        if self.done {
            return None;
        }
        match self.events.try_recv() {
            Ok(DuplexEvent::Line(line)) => Some(line),
            Ok(DuplexEvent::Ended(_)) => {
                self.done = true;
                None
            }
            Err(_) => None,
        }
    }
}

/// A child process with piped stdin, stdout and stderr. Dropping every
/// handle closes the session exactly like [`DuplexSender::close`]: no path
/// leaves the process running.
pub struct DuplexSession {
    sender: DuplexSender,
    events: DuplexEvents,
}

impl DuplexSession {
    /// Spawns the guarded command with all three pipes. For the agent
    /// workspace use [`crate::vm::exec_in_vm_duplex`], which builds the
    /// `incus exec -T` command; this entry point takes any `SafeCommand`.
    pub fn spawn(cmd: &SafeCommand, options: DuplexOptions) -> Result<Self, DuplexError> {
        Self::start(cmd, options, None)
    }

    /// [`Self::spawn`], plus `cleanup` run once the child is gone (by any
    /// path) and before [`DuplexEvent::Ended`] is published.
    pub(crate) fn spawn_with_cleanup(
        cmd: &SafeCommand,
        options: DuplexOptions,
        cleanup: SafeCommand,
    ) -> Result<Self, DuplexError> {
        Self::start(cmd, options, Some(cleanup))
    }

    fn start(
        cmd: &SafeCommand,
        options: DuplexOptions,
        cleanup: Option<SafeCommand>,
    ) -> Result<Self, DuplexError> {
        let mut command = cmd.into_std_command();
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = command
            .spawn()
            .map_err(|error| DuplexError::Spawn(error.to_string()))?;
        let (Some(stdin), Some(stdout), Some(stderr)) =
            (child.stdin.take(), child.stdout.take(), child.stderr.take())
        else {
            kill_tree(&mut child);
            return Err(DuplexError::Spawn("a pipe was not opened".to_string()));
        };
        let (events_tx, events_rx) = mpsc::channel(EVENT_BUFFER);
        let (control_tx, control_rx) = std_mpsc::channel();
        let (writer_tx, writer_rx) = std_mpsc::channel();
        let (ended_tx, ended_rx) = watch::channel(None);
        let tail = Arc::new(Mutex::new(StderrTail::new(options.stderr_tail_bytes)));

        spawn_writer(stdin, writer_rx);
        let stdout_done = spawn_stdout_reader(
            stdout,
            options.max_line_bytes,
            events_tx.clone(),
            control_tx.clone(),
        );
        let stderr_done = spawn_stderr_reader(stderr, tail.clone());
        spawn_supervisor(Supervision {
            child,
            control: control_rx,
            deadline: crate::vm::incus_run::capped(options.deadline),
            close_grace: options.close_grace,
            stdout_done,
            stderr_done,
            tail: tail.clone(),
            events: events_tx,
            ended: ended_tx,
            cleanup,
        });

        let sender = DuplexSender {
            shared: Arc::new(Shared {
                writer: Mutex::new(Some(writer_tx)),
                control: control_tx,
                ended: ended_rx,
                tail,
                max_line_bytes: options.max_line_bytes,
            }),
        };
        Ok(Self {
            events: DuplexEvents {
                events: events_rx,
                done: false,
                sender: sender.clone(),
            },
            sender,
        })
    }

    pub fn sender(&self) -> DuplexSender {
        self.sender.clone()
    }

    pub fn send(&self, line: &str) -> Result<(), DuplexError> {
        self.sender.send(line)
    }

    pub fn send_json<T: Serialize + ?Sized>(&self, value: &T) -> Result<(), DuplexError> {
        self.sender.send_json(value)
    }

    pub async fn recv(&mut self) -> Option<DuplexEvent> {
        self.events.recv().await
    }

    /// Closes stdin, waits up to the grace period for the peer to exit, then
    /// kills the tree. Lines still arriving meanwhile are discarded.
    pub async fn close(mut self) -> SessionEnd {
        self.sender.close();
        while let Some(event) = self.events.recv().await {
            if let DuplexEvent::Ended(end) = event {
                return end;
            }
        }
        self.sender.wait_ended().await
    }

    /// Splits into a cloneable sender and the ordered event stream.
    pub fn into_parts(self) -> (DuplexSender, DuplexEvents) {
        (self.sender, self.events)
    }
}
