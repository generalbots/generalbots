//! Workflow regression tests (#2501).

use super::*;

#[test]
fn parallel_then_cfo_then_signature_keeps_full_trail() {
    let (store, id) = store_with_contract();
    let after_alice = store
        .decide(
            TENANT_A,
            id,
            "alice",
            DecisionKind::Approve,
            "",
            "",
            Some("wrong-hash"),
        )
        .expect_err("stale hash rejected");
    assert_eq!(after_alice, "version_mismatch");

    let mut record = store
        .decide(
            TENANT_A,
            id,
            "alice",
            DecisionKind::Approve,
            "",
            "looks good",
            None,
        )
        .expect("alice approves");
    assert_eq!(record.status, ApprovalStatus::InReview);
    assert_eq!(record.current_step, 0);

    record = store
        .decide(
            TENANT_A,
            id,
            "bob",
            DecisionKind::Approve,
            "",
            "ok",
            None,
        )
        .expect("bob approves");
    assert_eq!(record.status, ApprovalStatus::InReview);
    assert_eq!(record.current_step, 1);
    assert_eq!(record.next_actor(Utc::now()).as_deref(), Some("cfo"));

    record = store
        .decide(
            TENANT_A,
            id,
            "cfo",
            DecisionKind::Approve,
            "",
            "signed off",
            None,
        )
        .expect("cfo approves");
    // The engine keeps InReview until the explicit signature step is signed.
    assert_eq!(record.status, ApprovalStatus::InReview);
    assert_eq!(record.current_step, 2);

    record = store
        .sign(TENANT_A, id, "signer", "electronic signature")
        .expect("signature");
    assert_eq!(record.status, ApprovalStatus::Signed);
    assert_eq!(record.decisions.len(), 4);
    assert!(store.verify_trail(TENANT_A, id).expect("verify"));

    let all = store.list(TENANT_A).expect("list");
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].decisions[0].version_hash, record.version_hash);
}

#[test]
fn sla_breach_emits_reminder_and_escalation_exactly_once() {
    let store = ApprovalStore::new();
    let record = store
        .create(TENANT_A, "deck", "deck", "Q3 deck", "author", "slides")
        .expect("create");
    let route = vec![step(
        "review",
        StepMode::Parallel,
        StepKind::Review,
        &["alice"],
        100,
    )];
    store
        .set_route(TENANT_A, record.id, route, "", None)
        .expect("route");
    let start = Utc::now();

    let first = store
        .check_sla_at(TENANT_A, start + Duration::seconds(200))
        .expect("first sweep");
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].kind, SlaEventKind::Reminder);
    assert!(store
        .check_sla_at(TENANT_A, start + Duration::seconds(200))
        .expect("second sweep")
        .is_empty());

    let third = store
        .check_sla_at(TENANT_A, start + Duration::seconds(500))
        .expect("third sweep");
    assert_eq!(third.len(), 1);
    assert_eq!(third[0].kind, SlaEventKind::Escalation);
    assert_eq!(third[0].escalate_to.as_deref(), Some("chief-of-staff"));
    assert!(store
        .check_sla_at(TENANT_A, start + Duration::seconds(900))
        .expect("fourth sweep")
        .is_empty());
}

#[test]
fn rejection_needs_reason_and_returns_to_draft() {
    let (store, id) = store_with_contract();
    let err = store
        .decide(
            TENANT_A,
            id,
            "alice",
            DecisionKind::Reject,
            "",
            "",
            None,
        )
        .expect_err("reason required");
    assert_eq!(err, "reason_required");
    let record = store
        .decide(
            TENANT_A,
            id,
            "alice",
            DecisionKind::Reject,
            "confidential",
            "",
            None,
        )
        .expect("reject with reason");
    assert_eq!(record.status, ApprovalStatus::Draft);
    assert_eq!(record.current_step, 0);
    assert_eq!(record.decisions[0].reason, "confidential");
}

#[test]
fn later_edits_invalidate_the_signed_approval() {
    let (store, id) = store_with_contract();
    store
        .sign(TENANT_A, id, "signer", "")
        .expect_err("signature step not active yet");
    for actor in ["alice", "bob", "cfo"] {
        store
            .decide(
                TENANT_A,
                id,
                actor,
                DecisionKind::Approve,
                "",
                "",
                None,
            )
            .expect("approve");
    }
    store.sign(TENANT_A, id, "signer", "done").expect("sign");
    let record = store
        .update_content(TENANT_A, id, "v8 content")
        .expect("edit");
    assert_eq!(record.status, ApprovalStatus::Invalidated);
    let err = store
        .publish(TENANT_A, id)
        .expect_err("cannot publish invalidated");
    assert_eq!(err, "only signed approvals can be published");
}

#[test]
fn publishing_creates_immutable_signed_artefact_and_records_entry() {
    let (store, id) = store_with_contract();
    for actor in ["alice", "bob", "cfo"] {
        store
            .decide(
                TENANT_A,
                id,
                actor,
                DecisionKind::Approve,
                "",
                "",
                None,
            )
            .expect("approve");
    }
    store.sign(TENANT_A, id, "signer", "done").expect("sign");
    let artifact = store.publish(TENANT_A, id).expect("publish");
    assert!(artifact.immutable);
    assert!(artifact
        .records_entry
        .contains("records://tenant-a/contract-q3/"));
    assert_eq!(artifact.signed_hash.len(), 64);
    let record = store.get(TENANT_A, id).expect("get").expect("present");
    assert_eq!(record.status, ApprovalStatus::Published);
    let err = store
        .update_content(TENANT_A, id, "tampered")
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
    let record = store.delegate(TENANT_A, id, delegation).expect("delegate");
    assert!(record
        .effective_assignees(Utc::now())
        .contains(&"proxy".to_string()));
    let err = store
        .decide(
            TENANT_A,
            id,
            "alice",
            DecisionKind::Approve,
            "",
            "",
            None,
        )
        .expect_err("delegator cannot decide");
    assert_eq!(err, "not_an_assignee");
    store
        .decide(
            TENANT_A,
            id,
            "proxy",
            DecisionKind::Approve,
            "",
            "",
            None,
        )
        .expect("proxy decides");
    assert!(store.verify_trail(TENANT_A, id).expect("verify"));
}

#[test]
fn engine_rejects_sequential_out_of_turn_decisions() {
    let mut record = engine::blank_record("t", "d", "c", "o", "hash");
    let route = vec![step(
        "cfo",
        StepMode::Sequential,
        StepKind::Approval,
        &["cfo", "ceo"],
        0,
    )];
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
