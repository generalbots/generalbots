//! Run lifecycle states and their legal transitions.
//!
//! A run has exactly one owner. The state machine below is the authority the
//! owner consults before it accepts a transition, so a late `RUN_FINISHED`
//! cannot revive a cancelled run and a terminal run can never move again.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunState {
    Running,
    WaitingInput,
    WaitingApproval,
    Paused,
    Cancelling,
    Succeeded,
    Failed,
    Cancelled,
}

impl RunState {
    #[must_use]
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::Cancelled)
    }

    #[must_use]
    pub fn is_waiting(self) -> bool {
        matches!(self, Self::WaitingInput | Self::WaitingApproval)
    }

    /// Whether the owner may move from `self` to `next`.
    ///
    /// Waiting states resume into `running`; `cancelling` is the only state
    /// allowed to end from `paused`/`waiting` without resuming first. Terminal
    /// states have no outgoing edge at all.
    #[must_use]
    pub fn can_transition_to(self, next: Self) -> bool {
        use RunState::{
            Cancelled, Cancelling, Failed, Paused, Running, Succeeded, WaitingApproval,
            WaitingInput,
        };
        match self {
            Running => matches!(
                next,
                WaitingInput | WaitingApproval | Paused | Cancelling | Succeeded | Failed | Cancelled
            ),
            WaitingInput | WaitingApproval | Paused => {
                matches!(next, Running | Cancelling | Cancelled | Failed)
            }
            Cancelling => matches!(next, Cancelled | Failed | Succeeded),
            Succeeded | Failed | Cancelled => false,
        }
    }

    /// The AG-UI payload type that carries this state.
    #[must_use]
    pub fn is_valid_finish_state(self) -> bool {
        self.is_terminal()
    }
}

impl std::fmt::Display for RunState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Running => "running",
            Self::WaitingInput => "waiting_input",
            Self::WaitingApproval => "waiting_approval",
            Self::Paused => "paused",
            Self::Cancelling => "cancelling",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        };
        f.write_str(name)
    }
}

#[cfg(test)]
mod tests {
    use super::RunState;

    #[test]
    fn terminal_states_are_final() {
        for state in [RunState::Succeeded, RunState::Failed, RunState::Cancelled] {
            assert!(state.is_terminal());
            for next in [
                RunState::Running,
                RunState::Succeeded,
                RunState::Failed,
                RunState::Cancelled,
            ] {
                assert!(!state.can_transition_to(next));
            }
        }
    }

    #[test]
    fn running_can_wait_and_resume() {
        assert!(RunState::Running.can_transition_to(RunState::WaitingInput));
        assert!(RunState::WaitingInput.can_transition_to(RunState::Running));
        assert!(RunState::Paused.can_transition_to(RunState::Running));
        assert!(!RunState::WaitingApproval.can_transition_to(RunState::Paused));
    }
}
