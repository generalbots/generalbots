//! The plain types of a duplex session: options, events, the end report
//! and errors.

use std::time::Duration;

/// Whole-session limit when the caller names none.
pub const DEFAULT_SESSION_DEADLINE: Duration = Duration::from_secs(30 * 60);
/// Longest stdout line accepted; a longer one ends the session.
pub const DEFAULT_MAX_LINE_BYTES: usize = 8 * 1024 * 1024;
/// How long a closed session may take to exit before its tree is killed.
pub const DEFAULT_CLOSE_GRACE: Duration = Duration::from_secs(5);
const DEFAULT_STDERR_TAIL_BYTES: usize = 8 * 1024;

#[derive(Debug, Clone)]
pub struct DuplexOptions {
    /// The session is killed when it is still running after this long.
    pub deadline: Duration,
    /// Longest line, in bytes, accepted in either direction.
    pub max_line_bytes: usize,
    /// After [`DuplexSender::close`], wait this long for the peer to exit.
    pub close_grace: Duration,
    /// How much of the peer's stderr is kept for [`SessionEnd::stderr_tail`].
    pub stderr_tail_bytes: usize,
}

impl Default for DuplexOptions {
    fn default() -> Self {
        Self {
            deadline: DEFAULT_SESSION_DEADLINE,
            max_line_bytes: DEFAULT_MAX_LINE_BYTES,
            close_grace: DEFAULT_CLOSE_GRACE,
            stderr_tail_bytes: DEFAULT_STDERR_TAIL_BYTES,
        }
    }
}

/// Why a session ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndReason {
    /// The peer exited on its own.
    PeerExited,
    /// The session was closed (or dropped): the peer exited after its stdin
    /// closed, or was killed when the grace period ran out.
    Closed,
    /// The session deadline passed; the process tree was killed.
    Deadline,
    /// The peer wrote a line longer than the cap; the tree was killed.
    LineTooLong,
    /// Reading the peer's stdout failed; the tree was killed.
    ReadFailed,
    /// The process state could not be read; the tree was killed.
    WaitFailed,
}

/// The final state of a session. `stderr_tail` is for diagnostics only and
/// may hold anything the peer printed: never log it or show it unsanitised.
#[derive(Debug, Clone)]
pub struct SessionEnd {
    pub reason: EndReason,
    /// `None` when the process was killed.
    pub exit_code: Option<i32>,
    pub stderr_tail: String,
    pub elapsed: Duration,
}

#[derive(Debug)]
pub enum DuplexEvent {
    /// One stdout line, without its `\n` (or `\r\n`).
    Line(String),
    /// Always the last event of a session.
    Ended(SessionEnd),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DuplexError {
    Spawn(String),
    /// The outgoing line holds a newline, is too long or is not serialisable.
    InvalidLine(&'static str),
    /// stdin was already closed by this side.
    StdinClosed,
    /// The session already ended.
    Ended(EndReason),
}

impl std::fmt::Display for DuplexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Spawn(detail) => write!(f, "duplex spawn failed: {detail}"),
            Self::InvalidLine(why) => write!(f, "duplex line rejected: {why}"),
            Self::StdinClosed => f.write_str("duplex stdin is closed"),
            Self::Ended(reason) => write!(f, "duplex session ended: {reason:?}"),
        }
    }
}

impl std::error::Error for DuplexError {}
