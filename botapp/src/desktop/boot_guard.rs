//! Decides when the boot behind the splash has failed.
//!
//! The splash used to wait for the API with no way out: a server that died
//! while starting left it on screen forever. The boot now fails when the
//! sidecar this app spawned exits before the API answers, or when the whole
//! boot passes an overall deadline. Pure, so the rules are testable without a
//! process or a window.

use super::sidecar::ChildState;
use std::time::Duration;

/// Overall boot deadline. A cold start on the reference machine (services
/// stopped, first run after a reboot) takes 40 to 50 s; the first start after
/// an upgrade adds database migrations, and an antivirus scanning a large
/// stack can slow every service down. 180 s is more than three times the
/// slowest cold start seen, short enough that a hung boot is reported while
/// the person is still waiting for it. `GB_BOOT_DEADLINE_SECS` overrides it.
pub const DEFAULT_BOOT_DEADLINE: Duration = Duration::from_secs(180);

/// Why the boot failed; the splash words each case in its own language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BootFailure {
    /// The sidecar exited before the API answered.
    Exited { code: Option<i32> },
    /// The API did not answer within the deadline.
    Deadline { seconds: u64 },
    /// Another program holds the sidecar port.
    PortTaken { port: u16, program: String },
    /// The sidecar could not be launched at all (binary or runtime missing).
    NotStarted,
}

/// The outcome of one look at the boot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BootVerdict {
    Ready,
    Waiting,
    Failed(BootFailure),
}

/// What the boot watcher saw on one poll.
#[derive(Debug, Clone, Copy)]
pub struct BootObservation {
    /// `/api/product` answered with a success status.
    pub api_ready: bool,
    /// The owned child's state on this poll.
    pub child: ChildState,
    /// This boot spawned the child (as opposed to attaching to one).
    pub spawned: bool,
    /// Time since the boot started.
    pub elapsed: Duration,
}

/// Judges one observation. A ready API wins over everything, then a dead
/// child, then the deadline.
pub fn judge(seen: BootObservation, deadline: Duration) -> BootVerdict {
    if seen.api_ready {
        return BootVerdict::Ready;
    }
    match seen.child {
        ChildState::Exited(code) => return BootVerdict::Failed(BootFailure::Exited { code }),
        // The child this boot spawned is gone without an exit having been
        // seen here: another caller collected it.
        ChildState::NotOwned if seen.spawned => {
            return BootVerdict::Failed(BootFailure::Exited { code: None })
        }
        ChildState::NotOwned | ChildState::Running => {}
    }
    if seen.elapsed >= deadline {
        return BootVerdict::Failed(BootFailure::Deadline {
            seconds: deadline.as_secs(),
        });
    }
    BootVerdict::Waiting
}

/// Parses `GB_BOOT_DEADLINE_SECS`; anything but a positive number keeps the
/// default.
pub fn parse_deadline(raw: Option<&str>) -> Duration {
    raw.and_then(|value| value.trim().parse::<u64>().ok())
        .filter(|seconds| *seconds > 0)
        .map(Duration::from_secs)
        .unwrap_or(DEFAULT_BOOT_DEADLINE)
}

/// The deadline for this run.
pub fn boot_deadline() -> Duration {
    parse_deadline(std::env::var("GB_BOOT_DEADLINE_SECS").ok().as_deref())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seen(api_ready: bool, child: ChildState, spawned: bool, secs: u64) -> BootObservation {
        BootObservation {
            api_ready,
            child,
            spawned,
            elapsed: Duration::from_secs(secs),
        }
    }

    const LIMIT: Duration = Duration::from_secs(180);

    #[test]
    fn a_server_that_answers_in_time_is_ready() {
        assert_eq!(judge(seen(true, ChildState::Running, true, 42), LIMIT), BootVerdict::Ready);
        // Attached to an assistant this app did not spawn.
        assert_eq!(judge(seen(true, ChildState::NotOwned, false, 3), LIMIT), BootVerdict::Ready);
        // The answer counts even on the poll that passes the deadline.
        assert_eq!(judge(seen(true, ChildState::Running, true, 500), LIMIT), BootVerdict::Ready);
    }

    #[test]
    fn a_running_boot_inside_the_deadline_keeps_waiting() {
        assert_eq!(judge(seen(false, ChildState::Running, true, 0), LIMIT), BootVerdict::Waiting);
        assert_eq!(judge(seen(false, ChildState::Running, true, 179), LIMIT), BootVerdict::Waiting);
        assert_eq!(judge(seen(false, ChildState::NotOwned, false, 60), LIMIT), BootVerdict::Waiting);
    }

    #[test]
    fn a_server_that_exits_before_answering_fails_at_once() {
        assert_eq!(
            judge(seen(false, ChildState::Exited(Some(1)), true, 7), LIMIT),
            BootVerdict::Failed(BootFailure::Exited { code: Some(1) })
        );
        assert_eq!(
            judge(seen(false, ChildState::Exited(None), true, 7), LIMIT),
            BootVerdict::Failed(BootFailure::Exited { code: None })
        );
        // The exit was collected elsewhere: the spawned child is still gone.
        assert_eq!(
            judge(seen(false, ChildState::NotOwned, true, 7), LIMIT),
            BootVerdict::Failed(BootFailure::Exited { code: None })
        );
    }

    #[test]
    fn a_boot_past_the_deadline_fails() {
        assert_eq!(
            judge(seen(false, ChildState::Running, true, 180), LIMIT),
            BootVerdict::Failed(BootFailure::Deadline { seconds: 180 })
        );
        assert_eq!(
            judge(seen(false, ChildState::NotOwned, false, 181), LIMIT),
            BootVerdict::Failed(BootFailure::Deadline { seconds: 180 })
        );
    }

    #[test]
    fn the_deadline_override_must_be_a_positive_number() {
        assert_eq!(parse_deadline(None), DEFAULT_BOOT_DEADLINE);
        assert_eq!(parse_deadline(Some(" 20 ")), Duration::from_secs(20));
        assert_eq!(parse_deadline(Some("0")), DEFAULT_BOOT_DEADLINE);
        assert_eq!(parse_deadline(Some("soon")), DEFAULT_BOOT_DEADLINE);
        assert_eq!(DEFAULT_BOOT_DEADLINE, Duration::from_secs(180));
    }
}
