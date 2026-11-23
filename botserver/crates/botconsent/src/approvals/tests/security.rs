//! #2901 approval-bypass exploit-chain and identity-binding tests.

use super::*;

/// Issue #2901: two users in two tenants walk the full
/// create → delegate → decide → sign → publish chain. Every hop is denied:
/// the owner cannot self-approve, a delegation cannot impersonate a third
/// party, the store is keyed by (tenant, id), and `list()` never crosses the
/// tenant boundary. The record never reaches Signed/Published.
#[test]
fn compound_approval_bypass_chain_is_denied() {
    let owner_a = user(1);
    let user_b = user(2);
    let principal_a = CallerPrincipal::new(owner_a, TENANT_A.to_string());
    let principal_b = CallerPrincipal::new(user_b, TENANT_B.to_string());
    let store = ApprovalStore::new();

    // 1) CREATE — the owner is the server-derived principal A. A body field
    //    `owner` no longer exists, so the creator cannot name a straw owner.
    let record = store
        .create(TENANT_A, "doc", "contract", "", &principal_a.actor, "v1")
        .expect("create");
    assert_eq!(record.owner, owner_a.to_string());

    // The attacker lists only themselves in the route to self-approve.
    let a_token = principal_a.actor.clone();
    let self_route = vec![step(
        "self",
        StepMode::Parallel,
        StepKind::Approval,
        &[a_token.as_str()],
        0,
    )];
    store
        .set_route(TENANT_A, record.id, self_route, "", None)
        .expect("route");

    // 2) DECIDE — self-approval is rejected at the engine boundary.
    let self_err = store
        .decide(
            TENANT_A,
            record.id,
            &principal_a.actor,
            DecisionKind::Approve,
            "",
            "",
            None,
        )
        .expect_err("self approval denied");
    assert_eq!(self_err, "owner_cannot_approve");

    // 3) DELEGATE — `from` is always the caller in the API. A forged
    //    delegation from another party's identity cannot redirect the route.
    let forged = Delegation {
        from: "cfo-of-tenant-b".to_string(),
        to: principal_a.actor.clone(),
        reason: String::new(),
        expires_at: None,
        created_at: Utc::now(),
    };
    let forge_err = store
        .delegate(TENANT_A, record.id, forged)
        .expect_err("forged delegation denied");
    assert_eq!(forge_err, "delegator is not an assignee");

    // A genuine delegation from A to B is allowed by the store (the API also
    // checks that B is an active user of A's tenant before accepting it).
    let delegation = Delegation {
        from: principal_a.actor.clone(),
        to: principal_b.actor.clone(),
        reason: "on leave".to_string(),
        expires_at: None,
        created_at: Utc::now(),
    };
    store
        .delegate(TENANT_A, record.id, delegation)
        .expect("delegate");

    // 4) DECIDE cross-tenant — B authenticates in tenant B, so the record
    //    addressed as (tenant-b, id) simply does not exist.
    let cross_err = store
        .decide(
            TENANT_B,
            record.id,
            &principal_b.actor,
            DecisionKind::Approve,
            "",
            "",
            None,
        )
        .expect_err("cross-tenant decide denied");
    assert_eq!(cross_err, "approval not found");
    assert!(store.get(TENANT_B, record.id).expect("get").is_none());
    assert!(store.list(TENANT_B).expect("list").is_empty());

    // 5) SIGN + PUBLISH — still unreachable from tenant B.
    let sign_err = store
        .sign(TENANT_B, record.id, &principal_b.actor, "forged")
        .expect_err("cross-tenant sign denied");
    assert_eq!(sign_err, "approval not found");
    let publish_err = store
        .publish(TENANT_B, record.id)
        .expect_err("cross-tenant publish denied");
    assert_eq!(publish_err, "approval not found");

    let after = store
        .get(TENANT_A, record.id)
        .expect("get")
        .expect("present");
    assert_eq!(after.status, ApprovalStatus::InReview);
    assert!(after.decisions.is_empty());
    assert!(after.published.is_none());
    assert!(store.verify_trail(TENANT_A, record.id).expect("verify"));
}

#[test]
fn signed_decision_binds_the_server_derived_actor() {
    let owner = user(3);
    let approver = user(4);
    let principal = CallerPrincipal::new(approver, TENANT_A.to_string());
    let store = ApprovalStore::new();
    let record = store
        .create(TENANT_A, "d", "c", "", &owner.to_string(), "v1")
        .expect("create");
    let token = principal.actor.clone();
    let route = vec![step(
        "s1",
        StepMode::Parallel,
        StepKind::Approval,
        &[token.as_str()],
        0,
    )];
    store
        .set_route(TENANT_A, record.id, route, "", None)
        .expect("route");

    let decided = store
        .decide(
            TENANT_A,
            record.id,
            &principal.actor,
            DecisionKind::Approve,
            "",
            "",
            None,
        )
        .expect("decide");
    assert_eq!(decided.decisions[0].actor, approver.to_string());
    assert!(store.verify_trail(TENANT_A, record.id).expect("verify"));
}
