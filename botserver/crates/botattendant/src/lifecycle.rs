//! Pure contact-center lifecycle rules.
//!
//! Session numbering, the wait/handle timers, the session state machine, the
//! agent activity stamps and the status→CSS mapping all live here so the
//! handlers only load a row, apply a decision and persist it.

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Width of the numeric part of a session number.
const SESSION_NUMBER_DIGITS: usize = 6;

/// Agent status: available for new sessions.
pub const AGENT_ONLINE: &str = "online";
/// Agent status: at capacity.
pub const AGENT_BUSY: &str = "busy";
/// Agent status: on a break.
pub const AGENT_BREAK: &str = "break";
/// Agent status: not working.
pub const AGENT_OFFLINE: &str = "offline";

/// `SES-000123` for the 123rd session of a branch.
///
/// The counter is the row count, so a negative count (a broken aggregate) still
/// produces the first number instead of a negative one.
pub fn format_session_number(count: i64) -> String {
    let next = count.saturating_add(1).max(1);
    format!("SES-{next:0width$}", width = SESSION_NUMBER_DIGITS)
}

/// Seconds a customer waited before assignment, floored at zero so a clock
/// that steps backwards never produces a negative wait.
pub fn wait_seconds(started_at: DateTime<Utc>, now: DateTime<Utc>) -> i32 {
    (now - started_at).num_seconds().clamp(0, i32::MAX as i64) as i32
}

/// Seconds an agent handled the session; `None` when it was never assigned.
pub fn handle_seconds(assigned_at: Option<DateTime<Utc>>, now: DateTime<Utc>) -> Option<i32> {
    assigned_at.map(|assigned| (now - assigned).num_seconds().clamp(0, i32::MAX as i64) as i32)
}

/// Lifecycle status of a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStatus {
    /// Queued, no agent yet.
    Waiting,
    /// Owned by an agent.
    Active,
    /// Closed; terminal.
    Ended,
}

impl SessionStatus {
    /// Parses the wire status; `None` for anything this crate does not own.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "waiting" => Some(Self::Waiting),
            "active" => Some(Self::Active),
            "ended" => Some(Self::Ended),
            _ => None,
        }
    }

    /// Wire status of this state.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Waiting => "waiting",
            Self::Active => "active",
            Self::Ended => "ended",
        }
    }

    /// Whether a session may move from `self` to `next`.
    ///
    /// Only `Ended` is terminal: closing an already-closed session is
    /// idempotent, but nothing reopens it. Waiting and active sessions may
    /// bounce between each other (assignment, transfer back to a queue).
    pub fn can_transition_to(self, next: Self) -> bool {
        match (self, next) {
            (Self::Ended, Self::Ended) => true,
            (Self::Ended, _) => false,
            _ => true,
        }
    }
}

/// True when the session may still be worked on (assigned, transferred,
/// messaged). Unknown statuses are treated as open: an older row must not
/// become read-only because this crate learned a new label.
pub fn accepts_work(status: &str) -> bool {
    match SessionStatus::parse(status) {
        Some(current) => current.can_transition_to(SessionStatus::Active),
        None => true,
    }
}

/// Activity timestamps written with an agent status change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActivityStamps {
    /// Set when the agent goes on a break, cleared otherwise.
    pub break_started_at: Option<DateTime<Utc>>,
    /// Set when the agent becomes available, cleared otherwise.
    pub available_since: Option<DateTime<Utc>>,
}

/// Stamps implied by an agent status transition at `now`.
pub fn activity_stamps(status: &str, now: DateTime<Utc>) -> ActivityStamps {
    ActivityStamps {
        break_started_at: if status == AGENT_BREAK { Some(now) } else { None },
        available_since: if status == AGENT_ONLINE { Some(now) } else { None },
    }
}

/// Whether an agent message must stamp the session's first-response time.
///
/// Internal notes are not customer-visible, so they never count as a response.
pub fn stamps_first_response(
    first_response_at: Option<DateTime<Utc>>,
    is_internal: bool,
) -> bool {
    first_response_at.is_none() && !is_internal
}

/// Where a transfer sends the session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferTarget {
    /// Hand the session to another agent.
    Agent(Uuid),
    /// Send the session back to a queue.
    Queue(Uuid),
    /// Neither destination was supplied: the transfer is only logged.
    Unrouted,
}

/// Resolves the destination of a transfer. An agent wins when both an agent and
/// a queue are supplied, matching the order the handler always applied.
pub fn transfer_target(to_agent: Option<Uuid>, to_queue: Option<Uuid>) -> TransferTarget {
    match (to_agent, to_queue) {
        (Some(agent), _) => TransferTarget::Agent(agent),
        (None, Some(queue)) => TransferTarget::Queue(queue),
        (None, None) => TransferTarget::Unrouted,
    }
}

/// CSS class for a session status badge.
pub fn session_status_class(status: &str) -> &'static str {
    match status {
        "waiting" => "status-waiting",
        "active" => "status-active",
        "ended" => "status-ended",
        _ => "status-default",
    }
}

/// CSS class for an agent status indicator.
pub fn agent_status_class(status: &str) -> &'static str {
    match status {
        AGENT_ONLINE => "status-online",
        AGENT_BUSY => "status-busy",
        AGENT_BREAK => "status-break",
        _ => "status-offline",
    }
}

#[cfg(test)]
#[path = "lifecycle_tests.rs"]
mod lifecycle_tests;
