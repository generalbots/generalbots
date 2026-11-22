//! Workflow regression tests covering every issue #2501 acceptance criterion
//! that is implementable inside this crate.

use chrono::{Duration, Utc};

use super::engine;
use super::store::ApprovalStore;
use super::types::{
    ApprovalStatus, ApprovalStep, DecisionKind, Delegation, SlaEventKind, StepKind, StepMode,
};

fn step(id: &str, mode: StepMode, kind: StepKind, approvers: &[&str], sla_secs: i64) -> ApprovalStep {
    ApprovalStep {
        id: id.to_string(),
        title: id.to_string(),
        mode,
        kind,
        approvers: approvers.iter().map(|name| (*name).to_string()).collect(),
        groups: Vec::new(),
        sla_secs,
        escalate_to: Some("chief-of-staff".to_string()),
        quorum: 0,
    }
}

fn contract_route() -> Vec<ApprovalStep> {
    vec![
        step("reviewers", StepMode::Parallel, StepKind::Review, &["alice", "bob"], 0),
        step("cfo", StepMode::Sequential, StepKind::Approval, &["cfo"], 0),
        step("esign", StepMode::Sequential, StepKind::Signature, &["signer"], 0),
    ]
}

fn store_with_contract() -> (ApprovalStore, uuid::Uuid) {
    let store = ApprovalStore::new();
    let record = store
        .create("tenant-1", "contract-q3", "contract", "Q3 contract", "author", "v7 content")
        .map_err(|e| e)
        .expect("create");
    store
        .set_route(record.id, contract_route(), "Please review", None)
        .map_err(|e| e)
        .expect("route");
    (store, record.id)
}

#[test]
fn parallel_then_cfo_then_signature_keeps_full_trail() {
    let (store, id) = store_with_contract();
    let after_alice = store
        .decide(id, "alice", DecisionKind::Approve, "", "", Some("wrong-hash"), Utc::now())
        .expect_err("stale hash rejected");
    assert_eq!(after_alice, "version_mismatch");

    let mut record = store
        .decide(id, "alice", DecisionKind::Approve, "", "looks good", None, Utc::now())
        .expect("alice approves");
    assert_eq!(record.status, ApprovalStatus::InReview);
    assert_eq!(record.current_step, 0);

    record = store
        .decide(id, "bob", DecisionKind::Approve, "", "ok", None, Utc::now())
        .expect("bob approves");
    assert_eq!(record.status, ApprovalStatus::InReview);
    assert_eq!(record.current_step, 1);
    assert_eq!(record.next_actor(Utc::now()).as_deref(), Some("cfo"));

    record = store
        .decide(id, "cfo", DecisionKind::Approve, "", "signed off", None, Utc::now())
        .expect("cfo approves");
    assert_eq!(record.status, ApprovalStatus::AwaitingSignature);
    assert_eq!(record.current_step, 2);

    record = store.sign(id, "signer", "electronic signature").expect("signature");
    assert_eq!(record.status, ApprovalStatus::Signed);
    assert_eq!(record.decisions.len(), 4);
    assert!(store.verify_trail(id).expect("verify"));

    let all = store.list().expect("list");
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].decisions[0].version_hash, record.version_hash);
}

#[test]
fn sla_breach_emits_reminder_and_escalation_exactly_once() {
    let store = ApprovalStore::new();
    let record = store
        .create("tenant-1", "deck", "deck", "Q3 deck", "author", "slides")
        .expect("create");
    let route = vec![step("review", StepMode::Parallel, StepKind::Review, &["alice"], 100)];
    store.set_route(record.id, route, "", None).expect("route");
    let start = Utc::now();

    let first = store.check_sla_at(start + Duration::seconds(200)).expect("first sweep");
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].kind, SlaEventKind::Reminder);
    assert!(store
        .check_sla_at(start + Duration::seconds(200))
        .expect("second sweep")
        .is_empty());

    let third = store.check_sla_at(start + Duration::seconds(500)).expect("third sweep");
    assert_eq!(third.len(), 1);
    assert_eq!(third[0].kind, SlaEventKind::Escalation);
    assert_eq!(third[0].escalate_to.as_deref(), Some("chief-of-staff"));
    assert!(store
        .check_sla_at(start + Duration::seconds(900))
        .expect("fourth sweep")
        .is_empty());
}

#[test]
fn rejection_needs_reason_and_returns_to_draft() {
    let (store, id) = store_with_contract();
    let err = store
        .decide(id, "alice", DecisionKind::Reject, "", "", None, Utc::now())
        .expect_err("reason required");
    assert_eq!(err, "reason_required");
    let record = store
        .decide(id, "alice", DecisionKind::Reject, "confidential", "", None, Utc::now())
        .expect("reject with reason");
    assert_eq!(record.status, ApprovalStatus::Draft);
    assert_eq!(record.current_step, 0);
    assert_eq!(record.decisions[0].reason, "confidential");
}

#[test]
fn later_edits_invalidate_the_signed_approval() {
    let (store, id) = store_with_contract();
    store.sign(id, "signer", "").expect_err("signature step not active yet");
    for actor in ["alice", "bob", "cfo"] {
        store
            .decide(id, actor, DecisionKind::Approve, "", "", None, Utc::now())
            .expect("approve");
    }
    store.sign(id, "signer", "done").expect("sign");
    let record = store.update_content(id, "v8 content").expect("edit");
    assert_eq!(record.status, ApprovalStatus::Invalidated);
    let err = store.publish(id).expect_err("cannot publish invalidated");
    assert_eq!(err, "only signed approvals can be published");
}

#[test]
fn publishing_creates_immutable_signed_artefact_and_records_entry() {
    let (store, id) = store_with_contract();
    for actor in ["alice", "bob", "cfo"] {
        store
            .decide(id, actor, DecisionKind::Approve, "", "", None, Utc::now())
            .expect("approve");
    }
    store.sign(id, "signer", "done").expect("sign");
    let artifact = store.publish(id).expect("publish");
    assert!(artifact.immutable);
    assert!(artifact.records_entry.contains("records://tenant-1/contract-q3/"));
    assert_eq!(artifact.signed_hash.len(), 64);
    let record = store.get(id).expect("get").expect("present");
    assert_eq!(record.status, ApprovalStatus::Published);
    let err = store
        .update_content(id, "tampered")
        .expect_err("published record is immutable");
    assert_eq!(err, "published approval is immutable");
}

#[test]
fn delegation_redirects_the_assignee_and_trail_stays_verifiable() {
    let (store, id) = store_with_contract();
    let created = Utc::now();
    let delegation = Delegation {
        from: "alice".to_string(),
        to: "proxy".to_string(),
        reason: "on leave".to_string(),
        expires_at: Some(created + Duration::days(1)),
        created_at: created,
    };
    let record = store.delegate(id, delegation).expect("delegate");
    assert!(record.effective_assignees(Utc::now()).contains(&"proxy".to_string()));
    let err = store
        .decide(id, "alice", DecisionKind::Approve, "", "", None, Utc::now())
        .expect_err("delegator cannot decide");
    assert_eq!(err, "not_an_assignee");
    store
        .decide(id, "proxy", DecisionKind::Approve, "", "", None, Utc::now())
        .expect("proxy decides");
    assert!(store.verify_trail(id).expect("verify"));
}

#[test]
fn engine_rejects_sequential_out_of_turn_decisions() {
    let mut record = engine::blank_record("t", "d", "c", "o", "hash");
    let route = vec![step("cfo", StepMode::Sequential, StepKind::Approval, &["cfo", "ceo"], 0)];
    engine::start_route(&mut record, route, "", None, Utc::now()).expect("start");
    let err = engine::apply_decision(
        &mut record,
        b"secret",
        "ceo",
        DecisionKind::Approve,
        "",
        "",
        None,
        Utc::now(),
    )
    .expect_err("out of turn");
    assert_eq!(err, "out_of_turn");
}
