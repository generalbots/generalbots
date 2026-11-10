//! Handoff state machine (CHAN-5, CHAN-6 in #2070).
//!
//! The service is in-memory, so every transition — including the ones the
//! database-backed path relies on — is decidable offline.

use super::*;
use std::str::FromStr;

fn snapshot() -> ContextSnapshot {
    ContextSnapshot {
        recent_messages: Vec::new(),
        session_state: HashMap::new(),
        bot_memory: HashMap::new(),
        active_tools: Vec::new(),
        current_intent: None,
    }
}

fn service_with_request() -> (HandoffService, Uuid, Uuid, Uuid) {
    let mut service = HandoffService::new();
    let bot_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let request = service.request_handoff(
        "session-1".to_string(),
        bot_id,
        user_id,
        snapshot(),
        "user asked for a human".to_string(),
    );
    (service, request.id, bot_id, user_id)
}

#[test]
fn every_status_parses_case_insensitively() {
    for (input, expected) in [
        ("requested", HandoffStatus::Requested),
        ("ACCEPTED", HandoffStatus::Accepted),
        ("Rejected", HandoffStatus::Rejected),
        ("completed", HandoffStatus::Completed),
        ("Escalated", HandoffStatus::Escalated),
    ] {
        assert_eq!(HandoffStatus::from_str(input), Ok(expected));
    }
}

#[test]
fn unknown_statuses_are_rejected_with_the_input_in_the_message() {
    for input in ["", "Requested ", "done"] {
        match HandoffStatus::from_str(input) {
            Err(message) => assert_eq!(message, format!("Unknown handoff status: {input}")),
            Ok(status) => panic!("{input:?} must be rejected, got {status:?}"),
        }
    }
}

#[test]
fn display_round_trips_through_from_str() {
    for status in [
        HandoffStatus::Requested,
        HandoffStatus::Accepted,
        HandoffStatus::Rejected,
        HandoffStatus::Completed,
        HandoffStatus::Escalated,
    ] {
        let wire = status.to_string();
        assert_eq!(HandoffStatus::from_str(&wire), Ok(status));
    }
}

#[test]
fn a_new_request_starts_requested_and_unassigned() {
    let (service, request_id, bot_id, user_id) = service_with_request();

    let pending = service.list_pending();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].id, request_id);
    assert_eq!(pending[0].status, HandoffStatus::Requested);
    assert!(pending[0].assigned_agent.is_none());
    assert_eq!(pending[0].bot_id, bot_id);
    assert_eq!(pending[0].user_id, user_id);
}

#[test]
fn the_happy_path_runs_requested_accepted_completed() {
    let (mut service, request_id, _, _) = service_with_request();
    let agent = Uuid::new_v4();

    let accepted = service.assign_agent(request_id, agent).expect("assign");
    assert_eq!(accepted.status, HandoffStatus::Accepted);
    assert_eq!(accepted.assigned_agent, Some(agent));

    let completed = service.complete_handoff(request_id).expect("complete");
    assert_eq!(completed.status, HandoffStatus::Completed);

    // A completed request is no longer pending.
    assert!(service.list_pending().is_empty());
}

#[test]
fn assigning_twice_is_an_invalid_transition_from_accepted() {
    let (mut service, request_id, _, _) = service_with_request();
    service.assign_agent(request_id, Uuid::new_v4()).expect("assign");

    match service.assign_agent(request_id, Uuid::new_v4()) {
        Err(HandoffError::InvalidTransition { from, to }) => {
            assert_eq!(from, HandoffStatus::Accepted);
            assert_eq!(to, HandoffStatus::Accepted);
        }
        other => panic!("expected InvalidTransition, got {other:?}"),
    }
}

#[test]
fn completing_a_request_that_was_never_accepted_is_invalid() {
    let (mut service, request_id, _, _) = service_with_request();

    match service.complete_handoff(request_id) {
        Err(HandoffError::InvalidTransition { from, to }) => {
            assert_eq!(from, HandoffStatus::Requested);
            assert_eq!(to, HandoffStatus::Completed);
        }
        other => panic!("expected InvalidTransition, got {other:?}"),
    }
}

#[test]
fn completing_twice_is_invalid() {
    let (mut service, request_id, _, _) = service_with_request();
    service.assign_agent(request_id, Uuid::new_v4()).expect("assign");
    service.complete_handoff(request_id).expect("complete");

    assert!(matches!(
        service.complete_handoff(request_id),
        Err(HandoffError::InvalidTransition {
            from: HandoffStatus::Completed,
            to: HandoffStatus::Completed
        })
    ));
}

#[test]
fn an_unknown_request_is_reported_as_not_found() {
    let (mut service, _, _, _) = service_with_request();
    let unknown = Uuid::new_v4();

    assert!(matches!(
        service.assign_agent(unknown, Uuid::new_v4()),
        Err(HandoffError::RequestNotFound(id)) if id == unknown
    ));
    assert!(matches!(
        service.complete_handoff(unknown),
        Err(HandoffError::RequestNotFound(id)) if id == unknown
    ));
    assert!(matches!(
        service.escalate_handoff(unknown),
        Err(HandoffError::RequestNotFound(id)) if id == unknown
    ));
    assert!(matches!(
        service.get_context_history(unknown),
        Err(HandoffError::RequestNotFound(id)) if id == unknown
    ));
}

#[test]
fn escalation_is_permitted_from_any_state_including_completed() {
    let (mut service, request_id, _, _) = service_with_request();
    service.assign_agent(request_id, Uuid::new_v4()).expect("assign");
    service.complete_handoff(request_id).expect("complete");

    let escalated = service.escalate_handoff(request_id).expect("escalate");
    assert_eq!(escalated.status, HandoffStatus::Escalated);
    // Pinned: escalate is a one-way override, it does not clear the agent.
    assert!(escalated.assigned_agent.is_some());
}

#[test]
fn only_requested_rows_are_listed_as_pending() {
    let mut service = HandoffService::new();
    let bot_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let first = service.request_handoff(
        "s1".to_string(),
        bot_id,
        user_id,
        snapshot(),
        "a".to_string(),
    );
    let second = service.request_handoff(
        "s2".to_string(),
        bot_id,
        user_id,
        snapshot(),
        "b".to_string(),
    );
    service.assign_agent(first.id, Uuid::new_v4()).expect("assign");

    let pending = service.list_pending();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].id, second.id);

    // Rejecting is not implemented as a transition; escalation is.
    service.escalate_handoff(second.id).expect("escalate");
    assert!(service.list_pending().is_empty());
}

#[test]
fn list_by_user_filters_by_owner() {
    let mut service = HandoffService::new();
    let bot_id = Uuid::new_v4();
    let ana = Uuid::new_v4();
    let bruno = Uuid::new_v4();

    let ana_request = service.request_handoff(
        "s1".to_string(),
        bot_id,
        ana,
        snapshot(),
        "a".to_string(),
    );
    service.request_handoff("s2".to_string(), bot_id, bruno, snapshot(), "b".to_string());

    let listed = service.list_by_user(ana);
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, ana_request.id);
}

#[test]
fn the_context_snapshot_is_retrievable_after_a_transition() {
    let (mut service, request_id, _, _) = service_with_request();
    service.assign_agent(request_id, Uuid::new_v4()).expect("assign");

    let context = service.get_context_history(request_id).expect("context");
    assert!(context.recent_messages.is_empty());
    assert!(context.current_intent.is_none());
}

#[test]
fn error_messages_name_both_ends_of_a_rejected_transition() {
    let error = HandoffError::InvalidTransition {
        from: HandoffStatus::Accepted,
        to: HandoffStatus::Accepted,
    };
    assert_eq!(
        error.to_string(),
        "Invalid handoff transition from accepted to accepted"
    );
}
