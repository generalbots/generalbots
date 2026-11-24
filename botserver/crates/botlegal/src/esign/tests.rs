//! Offline unit tests for the approval / e-signature core (issue #2501).

use super::model::{
    ApprovalError, ApprovalStep, Decision, DecisionKind, RouteMode, StepState, WorkflowState,
};
use super::workflow::ApprovalWorkflow;

fn two_step_route(mode: RouteMode) -> ApprovalWorkflow {
    let mut workflow = ApprovalWorkflow::new("doc-1", "v1".to_string(), mode);
    workflow.route(vec![
        ApprovalStep::new("s1", "alice", 5_000, "v1"),
        ApprovalStep::new("s2", "cfo", 6_000, "v1"),
    ]);
    workflow
}

#[test]
fn two_step_route_advances_then_completes() {
    let mut workflow = two_step_route(RouteMode::Sequential);
    assert_eq!(workflow.state(), WorkflowState::Pending);
    assert_eq!(workflow.active_step_ids(), vec!["s1"]);

    let first = workflow
        .decide("s1", Decision::approve("alice", "v1"), 1_000)
        .expect("first approval should succeed");
    assert_eq!(first.from, StepState::Pending);
    assert_eq!(first.to, StepState::Approved);
    assert_eq!(workflow.active_step_ids(), vec!["s2"]);

    let second = workflow
        .decide("s2", Decision::approve("cfo", "v1"), 2_000)
        .expect("second approval should succeed");
    assert_eq!(second.seq, 1);
    assert_eq!(workflow.state(), WorkflowState::Approved);
    assert_eq!(workflow.trail().len(), 2);
    assert!(workflow.steps.iter().all(|s| s.state == StepState::Approved));

    let error = workflow
        .decide("s2", Decision::approve("cfo", "v1"), 2_100)
        .expect_err("a finished step cannot decide again");
    // The step is finished, so the guard refuses it as out-of-order rather
    // than reporting a halted chain (the chain itself is healthy).
    assert_eq!(
        error,
        ApprovalError::StepOutOfOrder {
            step_id: "s2".to_string()
        }
    );
}

#[test]
fn sla_expiry_escalates_once() {
    let mut workflow = ApprovalWorkflow::new("doc-2", "v1".to_string(), RouteMode::Sequential);
    workflow.route(vec![ApprovalStep::new("s1", "alice", 1_000, "v1")]);

    assert!(workflow.sweep_sla(1_000).is_empty(), "deadline is not yet passed");

    let events = workflow.sweep_sla(1_001);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].action, DecisionKind::Expire);
    assert_eq!(events[0].to, StepState::Expired);
    assert!(events[0].escalated);
    assert_eq!(events[0].actor, "system");
    assert_eq!(workflow.steps[0].state, StepState::Expired);
    assert_eq!(workflow.state(), WorkflowState::Expired);

    assert!(workflow.sweep_sla(2_000).is_empty(), "escalation must fire once");
    assert_eq!(workflow.trail().len(), 1);
    assert!(workflow.active_step_ids().is_empty());
}

#[test]
fn delegation_changes_approver_and_still_allows_approval() {
    let mut workflow = ApprovalWorkflow::new("doc-3", "v1".to_string(), RouteMode::Sequential);
    workflow.route(vec![ApprovalStep::new("s1", "alice", 9_000, "v1")]);

    let handed_off = workflow
        .decide(
            "s1",
            Decision::delegate("alice", "bob").with_reason("on leave"),
            1_000,
        )
        .expect("delegation should succeed");
    assert_eq!(handed_off.to, StepState::Delegated);
    assert_eq!(workflow.steps[0].approver, "bob");
    let delegation = workflow.steps[0]
        .delegation
        .as_ref()
        .expect("delegation record should exist");
    assert_eq!(delegation.from, "alice");
    assert_eq!(delegation.to, "bob");
    assert_eq!(delegation.reason, "on leave");

    let approved = workflow
        .decide("s1", Decision::approve("bob", "v1"), 2_000)
        .expect("the delegate may approve from the delegated state");
    assert_eq!(approved.from, StepState::Delegated);
    assert_eq!(workflow.state(), WorkflowState::Approved);
}

#[test]
fn rejection_requires_reason_and_stops_the_chain() {
    let mut workflow = two_step_route(RouteMode::Sequential);

    let missing_reason = workflow
        .decide("s1", Decision::reject("alice", " "), 1_000)
        .expect_err("an empty reason must be refused");
    assert_eq!(missing_reason, ApprovalError::ReasonRequired);

    let rejected = workflow
        .decide(
            "s1",
            Decision::reject("alice", "budget not approved"),
            1_100,
        )
        .expect("a reasoned rejection should succeed");
    assert_eq!(rejected.to, StepState::Rejected);
    assert_eq!(rejected.reason, "budget not approved");
    assert_eq!(workflow.state(), WorkflowState::Rejected);
    assert_eq!(workflow.steps[1].state, StepState::Pending);

    let halted = workflow
        .decide("s2", Decision::approve("cfo", "v1"), 2_000)
        .expect_err("the next step must be blocked");
    assert_eq!(halted, ApprovalError::ChainHalted);
    assert!(workflow.active_step_ids().is_empty());
}

#[test]
fn digest_mismatch_is_refused_and_edits_are_visible_as_stale() {
    let mut workflow = two_step_route(RouteMode::Sequential);

    let mismatched = workflow
        .decide("s1", Decision::approve("alice", "v7"), 1_000)
        .expect_err("approval against the wrong digest must be refused");
    assert_eq!(
        mismatched,
        ApprovalError::DigestMismatch {
            expected: "v1".to_string(),
            got: "v7".to_string(),
        }
    );
    assert_eq!(workflow.steps[0].state, StepState::Pending);
    assert!(workflow.is_stale("v7"));

    workflow
        .decide("s1", Decision::approve("alice", "v1"), 1_100)
        .expect("binding digest should be accepted");
    workflow
        .decide("s2", Decision::approve("cfo", "v1"), 1_200)
        .expect("second approval should succeed");

    let bad_signature = workflow
        .sign("notary", "v9", 2_000, "")
        .expect_err("signing a different digest must be refused");
    assert!(matches!(bad_signature, ApprovalError::DigestMismatch { .. }));
    assert!(workflow.signatures.is_empty());
}

#[test]
fn parallel_route_accepts_out_of_order_decisions() {
    let mut workflow = two_step_route(RouteMode::Parallel);
    assert_eq!(workflow.active_step_ids(), vec!["s1", "s2"]);

    workflow
        .decide("s2", Decision::approve("cfo", "v1"), 1_000)
        .expect("parallel step s2 should be actionable first");
    workflow
        .decide("s1", Decision::approve("alice", "v1"), 1_100)
        .expect("parallel step s1 should also be actionable");
    assert_eq!(workflow.state(), WorkflowState::Approved);
}

#[test]
fn publish_requires_signature_and_export_is_deterministic() {
    let mut workflow = two_step_route(RouteMode::Sequential);
    assert_eq!(
        workflow.export_record().expect_err("unsigned cannot publish"),
        ApprovalError::NotPublished
    );

    workflow
        .decide("s1", Decision::approve("alice", "v1"), 1_000)
        .expect("first approval");
    workflow
        .decide("s2", Decision::approve("cfo", "v1"), 1_100)
        .expect("second approval");
    let signature = workflow
        .sign("notary", "v1", 2_000, "final sign-off")
        .expect("signing should succeed");
    assert_eq!(signature.document_digest, "v1");
    assert!(!signature.signature_hash.is_empty());
    assert_eq!(workflow.state(), WorkflowState::Signed);

    let first = workflow.export_record().expect("signed workflow can be filed");
    let second = workflow.export_record().expect("export is repeatable");
    assert_eq!(first, second);
    assert_eq!(first.export_hash, second.export_hash);
    assert!(first.is_hash_valid());
    assert_eq!(first.schema_version, 1);
    assert_eq!(first.workflow_state, WorkflowState::Signed);
    assert_eq!(first.steps.len(), 2);
    assert_eq!(first.signatures.len(), 1);
    assert_eq!(first.trail.len(), 2);
}
