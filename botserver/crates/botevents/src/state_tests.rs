//! Full transition matrix of [`RunState`].
//!
//! The matrix is the published contract of the run owner: every cell is
//! asserted so a new state or a widened edge is a deliberate, reviewed change
//! instead of an accident.

use super::RunState;

const ALL: [RunState; 8] = [
    RunState::Running,
    RunState::WaitingInput,
    RunState::WaitingApproval,
    RunState::Paused,
    RunState::Cancelling,
    RunState::Succeeded,
    RunState::Failed,
    RunState::Cancelled,
];

/// Every legal edge, written out so the matrix below is readable.
const LEGAL: [(RunState, RunState); 22] = [
    (RunState::Running, RunState::WaitingInput),
    (RunState::Running, RunState::WaitingApproval),
    (RunState::Running, RunState::Paused),
    (RunState::Running, RunState::Cancelling),
    (RunState::Running, RunState::Succeeded),
    (RunState::Running, RunState::Failed),
    (RunState::Running, RunState::Cancelled),
    (RunState::WaitingInput, RunState::Running),
    (RunState::WaitingInput, RunState::Cancelling),
    (RunState::WaitingInput, RunState::Cancelled),
    (RunState::WaitingInput, RunState::Failed),
    (RunState::WaitingApproval, RunState::Running),
    (RunState::WaitingApproval, RunState::Cancelling),
    (RunState::WaitingApproval, RunState::Cancelled),
    (RunState::WaitingApproval, RunState::Failed),
    (RunState::Paused, RunState::Running),
    (RunState::Paused, RunState::Cancelling),
    (RunState::Paused, RunState::Cancelled),
    (RunState::Paused, RunState::Failed),
    (RunState::Cancelling, RunState::Cancelled),
    (RunState::Cancelling, RunState::Failed),
    (RunState::Cancelling, RunState::Succeeded),
];

#[test]
fn transition_matrix_matches_the_contract() {
    for from in ALL {
        for to in ALL {
            let expected = LEGAL.contains(&(from, to));
            assert_eq!(
                from.can_transition_to(to),
                expected,
                "{from} -> {to} must be {expected}"
            );
        }
    }
}

#[test]
fn terminal_states_have_no_outgoing_edge() {
    for state in [RunState::Succeeded, RunState::Failed, RunState::Cancelled] {
        assert!(state.is_terminal());
        assert!(state.is_valid_finish_state());
        for next in ALL {
            assert!(!state.can_transition_to(next), "{state} -> {next}");
        }
    }
}

#[test]
fn open_states_are_never_terminal() {
    for state in [
        RunState::Running,
        RunState::WaitingInput,
        RunState::WaitingApproval,
        RunState::Paused,
        RunState::Cancelling,
    ] {
        assert!(!state.is_terminal(), "{state}");
        assert!(!state.is_valid_finish_state(), "{state}");
    }
}

#[test]
fn only_input_and_approval_are_waiting_states() {
    let waiting = ALL
        .iter()
        .copied()
        .filter(|state| state.is_waiting())
        .collect::<Vec<_>>();
    assert_eq!(waiting, vec![RunState::WaitingInput, RunState::WaitingApproval]);
}

#[test]
fn display_uses_the_wire_spelling() {
    let expected = [
        (RunState::Running, "running"),
        (RunState::WaitingInput, "waiting_input"),
        (RunState::WaitingApproval, "waiting_approval"),
        (RunState::Paused, "paused"),
        (RunState::Cancelling, "cancelling"),
        (RunState::Succeeded, "succeeded"),
        (RunState::Failed, "failed"),
        (RunState::Cancelled, "cancelled"),
    ];
    for (state, wire) in expected {
        assert_eq!(state.to_string(), wire);
        assert_eq!(
            serde_json::to_value(state).expect("serializes"),
            serde_json::Value::String(wire.to_string())
        );
    }
}

#[test]
fn serde_round_trips_every_state() {
    for state in ALL {
        let json = serde_json::to_string(&state).expect("serializes");
        let back: RunState = serde_json::from_str(&json).expect("deserializes");
        assert_eq!(back, state);
    }
    assert!(serde_json::from_str::<RunState>("\"unknown\"").is_err());
}
