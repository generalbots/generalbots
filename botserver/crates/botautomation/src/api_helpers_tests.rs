//! Inline tests for the pure automation ownership predicates (#2762, #2823).
//! Declared from `api_helpers.rs`, so `super` is that module.

use super::*;
use serde_json::json;

fn u(value: &str) -> Uuid {
    Uuid::parse_str(value).expect("test uuid")
}

const OWNER_A: &str = "11111111-1111-1111-1111-111111111111";
const OWNER_B: &str = "22222222-2222-2222-2222-222222222222";
const ORG_A: &str = "33333333-3333-3333-3333-333333333333";
const ORG_B: &str = "44444444-4444-4444-4444-444444444444";
const BRANCH_A: &str = "55555555-5555-5555-5555-555555555555";
const BRANCH_B: &str = "66666666-6666-6666-6666-666666666666";

fn caller(owner: Uuid, org: Option<Uuid>, branch: Option<Uuid>, is_admin: bool) -> CallerIdentity {
    CallerIdentity { owner, org, branch, is_admin }
}

fn target(owner: Uuid, org: Option<Uuid>, branch: Option<Uuid>) -> OwnedResource {
    OwnedResource { owner_user_id: owner, org_id: org, branch_id: branch }
}

/// Two-tenant SSE guard: a tenant A run is streamable only to tenant A;
/// tenant B is refused even though the run id is known.
#[test]
fn sse_stream_refuses_foreign_tenant_run() {
    let resource = target(u(OWNER_A), Some(u(ORG_A)), Some(u(BRANCH_A)));
    assert!(is_owned(
        &caller(u(OWNER_A), Some(u(ORG_A)), Some(u(BRANCH_A)), false),
        &resource
    ));
    assert!(!is_owned(
        &caller(u(OWNER_B), Some(u(ORG_B)), Some(u(BRANCH_B)), false),
        &resource
    ));
    assert!(!is_owned(
        &caller(u(OWNER_A), Some(u(ORG_B)), Some(u(BRANCH_A)), false),
        &resource
    ));
}

/// Two-tenant update guard: a null-org row is denied to ordinary callers
/// from either tenant and reachable only to a platform admin.
#[test]
fn null_org_update_denied_without_platform_admin() {
    let resource = target(u(OWNER_A), None, None);
    assert!(!is_owned(
        &caller(u(OWNER_A), Some(u(ORG_A)), Some(u(BRANCH_A)), false),
        &resource
    ));
    assert!(!is_owned(
        &caller(u(OWNER_B), Some(u(ORG_B)), Some(u(BRANCH_B)), false),
        &resource
    ));
    assert!(is_owned(
        &caller(u(OWNER_A), None, None, true),
        &resource
    ));
}

#[test]
fn missing_org_claim_never_owns_a_tenant_row() {
    let resource = target(u(OWNER_A), Some(u(ORG_A)), Some(u(BRANCH_A)));
    assert!(!is_owned(&caller(u(OWNER_A), None, None, false), &resource));
}

#[test]
fn nil_or_foreign_owner_is_denied() {
    let resource = target(u(OWNER_A), Some(u(ORG_A)), None);
    assert!(!is_owned(&caller(Uuid::nil(), Some(u(ORG_A)), None, false), &resource));
    assert!(!is_owned(&caller(u(OWNER_B), Some(u(ORG_A)), None, false), &resource));
    assert!(!is_owned(
        &caller(u(OWNER_A), Some(u(ORG_A)), None, false),
        &target(Uuid::nil(), Some(u(ORG_A)), None)
    ));
}

#[test]
fn branch_mismatch_is_denied() {
    let resource = target(u(OWNER_A), Some(u(ORG_A)), Some(u(BRANCH_A)));
    assert!(!is_owned(
        &caller(u(OWNER_A), Some(u(ORG_A)), Some(u(BRANCH_B)), false),
        &resource
    ));
}

/// Two-tenant mutation guard (delete/trigger): a tenant A schedule is
/// mutable only by tenant A; tenant B is refused even knowing the id.
#[test]
fn foreign_tenant_schedule_is_not_owned_for_mutation() {
    let resource = target(u(OWNER_A), Some(u(ORG_A)), Some(u(BRANCH_A)));
    assert!(is_owned(
        &caller(u(OWNER_A), Some(u(ORG_A)), Some(u(BRANCH_A)), false),
        &resource
    ));
    assert!(!is_owned(
        &caller(u(OWNER_B), Some(u(ORG_B)), Some(u(BRANCH_B)), false),
        &resource
    ));
    assert!(!is_owned(
        &caller(u(OWNER_B), Some(u(ORG_A)), Some(u(BRANCH_A)), false),
        &resource
    ));
}

/// Two-tenant run guard (cancel/list): runs inherit their parent schedule's
/// owner/org/branch, so a foreign tenant's run is never claimable.
#[test]
fn foreign_tenant_run_is_not_claimable() {
    let resource = target(u(OWNER_A), Some(u(ORG_A)), Some(u(BRANCH_A)));
    assert!(!is_owned(
        &caller(u(OWNER_B), Some(u(ORG_B)), Some(u(BRANCH_B)), false),
        &resource
    ));
    assert!(is_owned(
        &caller(u(OWNER_A), Some(u(ORG_A)), Some(u(BRANCH_A)), false),
        &resource
    ));
}

/// `create_schedule` bot validation: only the caller's own org/branch bot
/// passes; a foreign bot, a missing branch claim and an unverifiable scope
/// all fail closed. A signed admin role may target any bot.
#[test]
fn bot_ownership_is_strict_and_fails_closed() {
    let bot_org = u(ORG_A);
    let bot_branch = u(BRANCH_A);
    assert!(bot_is_owned(
        &caller(u(OWNER_A), Some(bot_org), Some(bot_branch), false),
        bot_org,
        bot_branch
    ));
    assert!(!bot_is_owned(
        &caller(u(OWNER_B), Some(u(ORG_B)), Some(u(BRANCH_B)), false),
        bot_org,
        bot_branch
    ));
    assert!(!bot_is_owned(
        &caller(u(OWNER_A), Some(bot_org), None, false),
        bot_org,
        bot_branch
    ));
    assert!(!bot_is_owned(
        &caller(u(OWNER_A), None, None, false),
        bot_org,
        bot_branch
    ));
    assert!(bot_is_owned(
        &caller(u(OWNER_A), None, None, true),
        bot_org,
        bot_branch
    ));
}

#[test]
fn only_exact_admin_role_claims_grant_platform_admin() {
    assert!(claims_grant_admin(&["admin".to_string()]));
    assert!(claims_grant_admin(&["SuperAdmin".to_string()]));
    assert!(!claims_grant_admin(&["notadmin".to_string()]));
    assert!(!claims_grant_admin(&["admintrainee".to_string()]));
    assert!(!claims_grant_admin(&[]));
}

#[test]
fn numeric_zitadel_subject_maps_to_a_stable_real_uuid() {
    let first = subject_uuid("123456789");
    assert!(!first.is_nil());
    assert_eq!(first, subject_uuid("123456789"));
    assert_ne!(first, subject_uuid("987654321"));
    assert_eq!(subject_uuid(OWNER_A), u(OWNER_A));
}

#[test]
fn schedule_row_exposes_tenant_and_owner_fields() {
    let schedule = AgentSchedule {
        id: u("77777777-7777-7777-7777-777777777777"),
        org_id: Some(u(ORG_A)),
        branch_id: Some(u(BRANCH_A)),
        bot_id: u("88888888-8888-8888-8888-888888888888"),
        title: "report".to_string(),
        goal: "summarize".to_string(),
        cron_expr: "0 9 * * *".to_string(),
        timezone: "UTC".to_string(),
        owner_user_id: u(OWNER_A),
        delivery: json!({}),
        enabled: true,
        max_runtime_secs: 900,
        tool_allowlist: None,
        next_run_at: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    assert_eq!(
        OwnedResource::from(&schedule),
        target(u(OWNER_A), Some(u(ORG_A)), Some(u(BRANCH_A)))
    );
}
