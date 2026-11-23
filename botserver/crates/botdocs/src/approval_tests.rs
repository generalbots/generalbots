//! Unit tests for the approval state machine (issue #2501).

use crate::approval::*;

const V: &str = "v7-hash";

fn flow_at_in_review() -> ApprovalFlow {
    let mut flow = ApprovalFlow::new("doc-1");
    flow.transition(
        TransitionCommand::new(TransitionKind::Submit, "alice", ActorRole::Author, V),
        100,
    )
    .expect("submit");
    flow.transition(
        TransitionCommand::new(
            TransitionKind::StartReview,
            "bob",
            ActorRole::Reviewer,
            "",
        ),
        101,
    )
    .expect("review");
    flow
}

#[test]
fn happy_path_reaches_signed_with_full_trail() {
    let mut flow = flow_at_in_review();
    assert_eq!(flow.state, ApprovalState::InReview);

    let approved = flow
        .transition(
            TransitionCommand::new(TransitionKind::Approve, "cfo", ActorRole::Approver, V),
            102,
        )
        .expect("approve");
    assert_eq!(approved.to, ApprovalState::Approved);
    assert_eq!(flow.bound_version_hash, V);

    let signed = flow
        .transition(
            TransitionCommand::new(TransitionKind::Sign, "cfo", ActorRole::Signer, V),
            103,
        )
        .expect("sign");
    assert_eq!(flow.state, ApprovalState::Signed);
    assert!(!signed.signature_hash.is_empty());
    assert_eq!(signed.signature_hash, compute_signature_hash(V, "cfo", 103));

    assert_eq!(flow.trail().len(), 4);
    for (i, event) in flow.trail().iter().enumerate() {
        assert_eq!(event.seq, i as u64);
    }
}

#[test]
fn invalid_transitions_are_rejected_without_mutating() {
    let mut flow = ApprovalFlow::new("doc-1");
    let err = flow
        .transition(
            TransitionCommand::new(TransitionKind::Approve, "cfo", ActorRole::Approver, V),
            1,
        )
        .expect_err("cannot approve a draft");
    assert_eq!(
        err,
        ApprovalError::InvalidTransition {
            from: ApprovalState::Draft,
            action: TransitionKind::Approve,
        }
    );
    assert!(flow.trail().is_empty());
    assert_eq!(flow.state, ApprovalState::Draft);

    let err = flow
        .transition(
            TransitionCommand::new(TransitionKind::Sign, "cfo", ActorRole::Signer, V),
            1,
        )
        .expect_err("cannot sign a draft");
    assert!(matches!(err, ApprovalError::InvalidTransition { .. }));
}

#[test]
fn rejection_requires_a_reason_and_returns_to_draft() {
    let mut flow = flow_at_in_review();
    let err = flow
        .transition(
            TransitionCommand::new(TransitionKind::Reject, "cfo", ActorRole::Approver, V),
            104,
        )
        .expect_err("reason required");
    assert_eq!(err, ApprovalError::ReasonRequired);
    assert_eq!(flow.state, ApprovalState::InReview);

    flow.transition(
        TransitionCommand::new(TransitionKind::Reject, "cfo", ActorRole::Approver, V)
            .with_comment("clauses 4 and 9 are unacceptable"),
        104,
    )
    .expect("reject with reason");
    assert_eq!(flow.state, ApprovalState::Rejected);
    assert_eq!(
        flow.trail().last().expect("event").comment,
        "clauses 4 and 9 are unacceptable"
    );

    flow.transition(
        TransitionCommand::new(
            TransitionKind::ReturnToDraft,
            "alice",
            ActorRole::Author,
            "",
        ),
        105,
    )
    .expect("return to draft");
    assert_eq!(flow.state, ApprovalState::Draft);
    assert!(flow.bound_version_hash.is_empty());
}

#[test]
fn role_guards_are_enforced() {
    let mut flow = flow_at_in_review();
    let err = flow
        .transition(
            TransitionCommand::new(TransitionKind::Approve, "bob", ActorRole::Reviewer, V),
            106,
        )
        .expect_err("reviewer cannot approve");
    assert_eq!(
        err,
        ApprovalError::RoleNotAllowed {
            role: ActorRole::Reviewer,
            action: TransitionKind::Approve,
        }
    );

    flow.transition(
        TransitionCommand::new(TransitionKind::Approve, "cfo", ActorRole::Approver, V),
        107,
    )
    .expect("approve");
    let err = flow
        .transition(
            TransitionCommand::new(TransitionKind::Sign, "alice", ActorRole::Author, V),
            108,
        )
        .expect_err("author cannot sign");
    assert!(matches!(err, ApprovalError::RoleNotAllowed { .. }));
}

#[test]
fn approval_and_signature_bind_to_the_submitted_version() {
    let mut flow = flow_at_in_review();
    let err = flow
        .transition(
            TransitionCommand::new(
                TransitionKind::Approve,
                "cfo",
                ActorRole::Approver,
                "edited-version",
            ),
            109,
        )
        .expect_err("version changed after submit");
    assert!(matches!(
        err,
        ApprovalError::VersionMismatch { ref expected, ref got }
            if expected == V && got == "edited-version"
    ));
    assert_eq!(flow.state, ApprovalState::InReview);

    flow.transition(
        TransitionCommand::new(TransitionKind::Approve, "cfo", ActorRole::Approver, V),
        110,
    )
    .expect("approve bound version");
    let err = flow
        .transition(
            TransitionCommand::new(
                TransitionKind::Sign,
                "cfo",
                ActorRole::Signer,
                "edited-version",
            ),
            111,
        )
        .expect_err("cannot sign a tampered version");
    assert!(matches!(err, ApprovalError::VersionMismatch { .. }));
}

#[test]
fn version_hash_and_actor_are_required_where_needed() {
    let mut flow = ApprovalFlow::new("doc-1");
    let err = flow
        .transition(
            TransitionCommand::new(TransitionKind::Submit, "alice", ActorRole::Author, ""),
            1,
        )
        .expect_err("submit needs a version hash");
    assert_eq!(err, ApprovalError::VersionHashRequired);

    let err = flow
        .transition(
            TransitionCommand::new(TransitionKind::Submit, "  ", ActorRole::Author, V),
            1,
        )
        .expect_err("blank actor");
    assert_eq!(err, ApprovalError::EmptyActor);
}

#[test]
fn signature_hash_is_deterministic_and_staleness_is_visible() {
    let a = compute_signature_hash(V, "cfo", 103);
    let b = compute_signature_hash(V, "cfo", 103);
    assert_eq!(a, b);
    assert_ne!(a, compute_signature_hash(V, "cfo", 104));
    assert_ne!(a, compute_signature_hash("other", "cfo", 103));

    let mut flow = flow_at_in_review();
    flow.transition(
        TransitionCommand::new(TransitionKind::Approve, "cfo", ActorRole::Approver, V),
        102,
    )
    .expect("approve");
    assert!(!flow.is_stale(V));
    assert!(flow.is_stale("newer-content"));

    flow.transition(
        TransitionCommand::new(TransitionKind::Sign, "cfo", ActorRole::Signer, V),
        103,
    )
    .expect("sign");
    assert!(flow.is_stale("newer-content"));
}

#[test]
fn can_transition_reflects_state_and_role_only() {
    let flow = ApprovalFlow::new("doc-1");
    assert!(flow.can_transition(TransitionKind::Submit, ActorRole::Author));
    assert!(!flow.can_transition(TransitionKind::Submit, ActorRole::Signer));
    assert!(!flow.can_transition(TransitionKind::Sign, ActorRole::Signer));
}
