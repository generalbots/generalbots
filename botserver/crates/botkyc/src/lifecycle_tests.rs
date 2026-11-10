//! Tests for the KYC transition guard and the workflow document mapping.
//!
//! The table below is the published contract of the KYC surface: which wire
//! statuses are accepted, which transitions are refused, and how a workflow's
//! `steps_completed` JSONB becomes the document list the UI renders.

use super::*;
use botbiometry::kyc::KycState;
use serde_json::json;

#[test]
fn every_published_wire_status_is_accepted() {
    let cases = [
        ("pending", KycState::Pending),
        ("incomplete", KycState::Incomplete),
        ("under_review", KycState::UnderReview),
        ("in_review", KycState::UnderReview),
        ("review", KycState::UnderReview),
        ("liveness_required", KycState::LivenessRequired),
        ("approved", KycState::Approved),
        ("rejected", KycState::Rejected),
        ("expired", KycState::Expired),
    ];
    for (wire, expected) in cases {
        assert_eq!(resolve_requested(wire), Ok(expected), "status {wire:?}");
    }
}

#[test]
fn unknown_and_miscased_statuses_are_rejected() {
    for wire in ["", "APPROVED", "Approved", "done", "pending ", "canceled"] {
        let error = resolve_requested(wire).expect_err("must reject status");
        assert_eq!(error, format!("Unknown KYC status '{wire}'"));
    }
}

#[test]
fn terminal_cases_cannot_be_reopened() {
    let refused = [
        ("approved", KycState::Pending),
        ("approved", KycState::Rejected),
        ("rejected", KycState::Approved),
        ("expired", KycState::UnderReview),
    ];
    for (current, requested) in refused {
        let error = check_transition(current, requested).expect_err("must refuse");
        assert_eq!(
            error,
            format!(
                "terminal ({current}) and cannot transition to '{}'",
                requested.as_status()
            )
        );
    }
}

#[test]
fn re_applying_the_same_terminal_status_is_allowed() {
    for state in [KycState::Approved, KycState::Rejected, KycState::Expired] {
        assert_eq!(check_transition(state.as_status(), state), Ok(()));
    }
}

#[test]
fn open_cases_may_move_to_any_known_status() {
    let open = ["pending", "incomplete", "under_review", "liveness_required"];
    let targets = [
        KycState::Pending,
        KycState::Incomplete,
        KycState::UnderReview,
        KycState::LivenessRequired,
        KycState::Approved,
        KycState::Rejected,
        KycState::Expired,
    ];
    for current in open {
        for requested in targets {
            assert_eq!(check_transition(current, requested), Ok(()), "{current} -> {requested:?}");
        }
    }
}

#[test]
fn an_unknown_stored_status_stays_editable() {
    assert_eq!(check_transition("legacy_status", KycState::Pending), Ok(()));
    assert_eq!(check_transition("", KycState::Approved), Ok(()));
}

#[test]
fn documents_come_from_the_string_elements_of_steps_completed() {
    let cases = [
        (json!(["id_front", "selfie"]), vec!["id_front", "selfie"]),
        (json!([]), vec![]),
        (json!(null), vec![]),
        (json!({"step": "id_front"}), vec![]),
        (json!("id_front"), vec![]),
        (json!([1, "selfie", null, {"a": 1}, "proof_of_address"]), vec!["selfie", "proof_of_address"]),
    ];
    for (input, expected) in cases {
        assert_eq!(documents_from_steps(&input), expected, "input {input}");
    }
}
