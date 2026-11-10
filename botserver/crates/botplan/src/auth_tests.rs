//! Identity and authorization boundary for plan routes (issue #2136).
//!
//! Tokens are minted HS256 with the secret the verifier resolves in this
//! process, so these tests pin exactly the accept/reject boundary production
//! uses: a forged, unsigned, expired or unknown credential never yields a
//! caller, and a plan the caller neither owns nor belongs to is reported as
//! "not found".

use super::{authorize, authorize_ws, Access, Caller, PlanError};
use crate::test_support::{b64url, caller, headers_for, live_claims, token};
use crate::Plan;
use axum::http::{HeaderMap, StatusCode};
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

const OWNER: &str = "user-owner";
const MEMBER: &str = "user-member";
const STRANGER: &str = "user-stranger";
const BRANCH: Uuid = Uuid::from_u128(0x1111);
const OTHER_BRANCH: Uuid = Uuid::from_u128(0x2222);

fn plan(owner: &str, members: &[&str], branch: Option<Uuid>) -> Plan {
    Plan {
        id: "plan-1".to_string(),
        title: "Plano".to_string(),
        description: None,
        owner_id: owner.to_string(),
        org_id: None,
        branch_id: branch.map(|value| value.to_string()),
        members: members.iter().map(|member| member.to_string()).collect(),
        tasks: HashMap::new(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

#[test]
fn a_verified_token_yields_the_caller_and_its_tenant() {
    let resolved = Caller::from_token(&token(live_claims(OWNER, Some(BRANCH)))).expect("caller");
    assert_eq!(resolved.user_id, OWNER);
    assert_eq!(resolved.email.as_deref(), Some("user-owner@example.test"));
    assert_eq!(resolved.branch_id, Some(BRANCH));
}

#[test]
fn a_nil_branch_claim_means_no_tenant_scope() {
    let resolved =
        Caller::from_token(&token(live_claims(OWNER, Some(Uuid::nil())))).expect("caller");
    assert_eq!(resolved.branch_id, None);
}

#[test]
fn a_missing_token_is_refused() {
    let refusal = Caller::from_headers(&HeaderMap::new()).expect_err("no token");
    assert_eq!(refusal, PlanError::Unauthenticated("missing_token"));
    assert_eq!(refusal.status(), StatusCode::UNAUTHORIZED);
}

#[test]
fn an_unknown_opaque_token_is_refused() {
    // `gb_*` suite tokens are resolved through the session cache; an unknown
    // one (no cache entry in a unit test) must not become an identity.
    let refusal = Caller::from_token("gb_not_a_real_session").expect_err("unknown session");
    assert_eq!(refusal.status(), StatusCode::UNAUTHORIZED);
}

#[test]
fn a_forged_signature_is_refused() {
    let valid = token(live_claims(OWNER, Some(BRANCH)));
    let mut parts: Vec<String> = valid.split('.').map(str::to_string).collect();
    parts[1] = b64url(
        serde_json::json!({
            "sub": "victim",
            "exp": Utc::now().timestamp() + 3600,
        })
        .to_string()
        .as_bytes(),
    );
    let refusal = Caller::from_token(&parts.join(".")).expect_err("tampered token");
    assert_eq!(refusal, PlanError::Unauthenticated("invalid_signature"));
}

#[test]
fn an_unsigned_token_is_refused() {
    let header = b64url(br#"{"alg":"none","typ":"JWT"}"#);
    let payload = b64url(
        serde_json::json!({ "sub": OWNER, "exp": Utc::now().timestamp() + 3600 })
            .to_string()
            .as_bytes(),
    );
    let refusal = Caller::from_token(&format!("{header}.{payload}.")).expect_err("alg none");
    assert_eq!(refusal, PlanError::Unauthenticated("unsupported_algorithm"));
}

#[test]
fn an_expired_token_is_refused() {
    let expired = token(serde_json::json!({
        "sub": OWNER,
        "exp": Utc::now().timestamp() - 60,
    }));
    let refusal = Caller::from_token(&expired).expect_err("expired");
    assert_eq!(refusal, PlanError::Unauthenticated("token_expired"));
}

#[test]
fn the_owner_may_read_write_and_administer() {
    let plan = plan(OWNER, &[], Some(BRANCH));
    let owner = caller(OWNER, Some(BRANCH));
    assert!(authorize(&plan, &owner, Access::Read).is_ok());
    assert!(authorize(&plan, &owner, Access::Write).is_ok());
    assert!(authorize(&plan, &owner, Access::Owner).is_ok());
}

#[test]
fn a_member_may_read_and_write_but_not_administer() {
    let plan = plan(OWNER, &[MEMBER], Some(BRANCH));
    let member = caller(MEMBER, Some(BRANCH));
    assert!(authorize(&plan, &member, Access::Read).is_ok());
    assert!(authorize(&plan, &member, Access::Write).is_ok());
    assert_eq!(
        authorize(&plan, &member, Access::Owner),
        Err(PlanError::Forbidden("owner_only"))
    );
}

#[test]
fn a_stranger_sees_a_plan_as_not_found() {
    let plan = plan(OWNER, &[], Some(BRANCH));
    let stranger = caller(STRANGER, Some(BRANCH));
    assert_eq!(
        authorize(&plan, &stranger, Access::Read),
        Err(PlanError::NotFound)
    );
    assert_eq!(
        authorize(&plan, &stranger, Access::Write),
        Err(PlanError::NotFound)
    );
    assert_eq!(
        authorize(&plan, &stranger, Access::Owner),
        Err(PlanError::NotFound)
    );
}

#[test]
fn the_legacy_anonymous_owner_owns_nothing() {
    let plan = plan("anonymous", &[], None);
    let legacy = caller("anonymous", None);
    assert_eq!(
        authorize(&plan, &legacy, Access::Read),
        Err(PlanError::NotFound)
    );
}

#[test]
fn another_tenant_cannot_reach_the_plan() {
    let plan = plan(OWNER, &[MEMBER], Some(BRANCH));
    let other_branch = caller(MEMBER, Some(OTHER_BRANCH));
    assert_eq!(
        authorize(&plan, &other_branch, Access::Read),
        Err(PlanError::NotFound)
    );
    // Even the owner, presenting a token from another branch, is refused.
    let owner_elsewhere = caller(OWNER, Some(OTHER_BRANCH));
    assert_eq!(
        authorize(&plan, &owner_elsewhere, Access::Read),
        Err(PlanError::NotFound)
    );
}

#[test]
fn a_branchless_caller_only_reaches_branchless_plans() {
    let scoped = plan(OWNER, &[], Some(BRANCH));
    let unscoped = plan(OWNER, &[], None);
    let suite_user = caller(OWNER, None);
    assert_eq!(
        authorize(&scoped, &suite_user, Access::Read),
        Err(PlanError::NotFound)
    );
    assert!(authorize(&unscoped, &suite_user, Access::Read).is_ok());
}

#[test]
fn the_socket_refuses_an_anonymous_upgrade() {
    let plan = plan(OWNER, &[], Some(BRANCH));
    let refusal = authorize_ws(&plan, None, &HeaderMap::new()).expect_err("anonymous");
    assert_eq!(refusal.status(), StatusCode::UNAUTHORIZED);
    let empty = authorize_ws(&plan, Some("   "), &HeaderMap::new()).expect_err("blank token");
    assert_eq!(empty.status(), StatusCode::UNAUTHORIZED);
}

#[test]
fn the_socket_refuses_a_guest_on_a_private_plan() {
    let plan = plan(OWNER, &[], Some(BRANCH));
    let guest_token = token(live_claims(STRANGER, Some(BRANCH)));
    let refusal = authorize_ws(&plan, Some(guest_token.as_str()), &HeaderMap::new())
        .expect_err("guest");
    assert_eq!(refusal, PlanError::NotFound);
    assert_eq!(refusal.status(), StatusCode::NOT_FOUND);
}

#[test]
fn the_socket_accepts_the_verified_owner_by_query_token_or_header() {
    let plan = plan(OWNER, &[], Some(BRANCH));
    let owner_token = token(live_claims(OWNER, Some(BRANCH)));
    let by_query =
        authorize_ws(&plan, Some(owner_token.as_str()), &HeaderMap::new()).expect("query token");
    assert_eq!(by_query.user_id, OWNER);
    let by_header = authorize_ws(&plan, None, &headers_for(OWNER, Some(BRANCH))).expect("header");
    assert_eq!(by_header.user_id, OWNER);
}
