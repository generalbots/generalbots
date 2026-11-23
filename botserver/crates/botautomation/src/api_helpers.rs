use super::*;
use botsecurity_core::verified_jwt::{verify_saas_jwt, VerifiedJwt};

/// Canonical administrator role names (shared rule #2633). Mirrored locally so
/// the crate needs no RBAC dependency; only *signed* role claims of a verified
/// token ever reach this list — a client header or query parameter never does.
const ADMIN_ROLE_NAMES: &[&str] = &[
    "admin",
    "administrator",
    "superadmin",
    "super_admin",
    "super-admin",
    "super",
];

pub(super) fn is_terminal(status: &str) -> bool {
    matches!(
        status,
        engine::STATUS_COMPLETED
            | engine::STATUS_FAILED
            | engine::STATUS_CANCELLED
            | engine::STATUS_TIMEOUT
    )
}

pub(super) struct SseTick {
    pub(super) spans: Vec<crate::models::AgentSpan>,
    pub(super) total_spans: i64,
    pub(super) run: Option<AgentRun>,
}

pub(super) fn poll_tick(state: &AutomationService, run_id: Uuid, emitted: usize) -> Option<SseTick> {
    use agent_runs::dsl as r;
    use agent_spans::dsl as s;
    let mut conn = match state.pool().get() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("automation sse {run_id}: db pool unavailable: {e}");
            return None;
        }
    };
    let spans = match s::agent_spans
        .filter(s::run_id.eq(run_id))
        .order(s::started_at.asc())
        .offset(emitted as i64)
        .limit(RUNS_PAGE_LIMIT)
        .load::<crate::models::AgentSpan>(&mut conn)
    {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("automation sse {run_id}: span poll failed: {e}");
            return None;
        }
    };
    let total = match s::agent_spans
        .filter(s::run_id.eq(run_id))
        .count()
        .get_result::<i64>(&mut conn)
    {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("automation sse {run_id}: span count failed: {e}");
            return None;
        }
    };
    let run = match r::agent_runs.find(run_id).first::<AgentRun>(&mut conn).optional() {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("automation sse {run_id}: run poll failed: {e}");
            return None;
        }
    };
    Some(SseTick { spans, total_spans: total, run })
}

/// Identity of the caller, derived exclusively from a verified server-minted
/// JWT. A token that cannot be verified yields no identity at all, so every
/// ownership decision fails closed (#2823).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CallerIdentity {
    pub(super) owner: Uuid,
    pub(super) org: Option<Uuid>,
    pub(super) branch: Option<Uuid>,
    pub(super) is_admin: bool,
}

impl CallerIdentity {
    fn from_claims(claims: &VerifiedJwt) -> Self {
        Self {
            owner: subject_uuid(&claims.sub),
            org: claims.org_id,
            branch: claims.branch_id,
            is_admin: claims_grant_admin(&claims.roles),
        }
    }
}

/// Same stable subject derivation used by the directory and marketplace, so a
/// verified numeric Zitadel `sub` owns a real UUID instead of the nil sentinel.
fn subject_uuid(sub: &str) -> Uuid {
    match Uuid::parse_str(sub.trim()) {
        Ok(id) => id,
        Err(_) => Uuid::new_v5(&Uuid::NAMESPACE_DNS, format!("zitadel:{sub}").as_bytes()),
    }
}

fn claims_grant_admin(roles: &[String]) -> bool {
    roles
        .iter()
        .any(|role| ADMIN_ROLE_NAMES.iter().any(|admin| role.eq_ignore_ascii_case(admin)))
}

/// Verified claims from `Authorization`, falling back to the SSE `?token=`
/// only when no header token is present. Both transports go through the same
/// signature/expiry verifier; no client-supplied org/branch parameter is read.
fn caller_claims(headers: &HeaderMap, token_param: &str) -> Option<VerifiedJwt> {
    let auth = headers.get("authorization").and_then(|v| v.to_str().ok());
    let token = auth.and_then(tenant::token_from_authorization).or_else(|| {
        let candidate = token_param.strip_prefix("Bearer ").unwrap_or(token_param).trim();
        if candidate.is_empty() {
            None
        } else {
            Some(candidate.to_string())
        }
    })?;
    match verify_saas_jwt(&token) {
        Ok(claims) => Some(claims),
        Err(reason) => {
            tracing::debug!("automation ownership: token refused: {reason}");
            None
        }
    }
}

fn caller_identity(headers: &HeaderMap, token_param: &str) -> Option<CallerIdentity> {
    caller_claims(headers, token_param).map(|claims| CallerIdentity::from_claims(&claims))
}

/// Owner written into new schedule rows. Uses the same verified identity as
/// the ownership guard, so a row created by a caller is later manageable by
/// that caller and by nobody else.
pub(super) fn owner_from_headers(headers: &HeaderMap) -> Uuid {
    caller_identity(headers, "").map_or(Uuid::nil(), |caller| caller.owner)
}

/// The tenant/owner fields that a schedule or run resolves to. Runs inherit
/// them from their parent schedule; a run without one is never claimable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct OwnedResource {
    pub(super) owner_user_id: Uuid,
    pub(super) org_id: Option<Uuid>,
    pub(super) branch_id: Option<Uuid>,
}

impl From<&AgentSchedule> for OwnedResource {
    fn from(schedule: &AgentSchedule) -> Self {
        Self {
            owner_user_id: schedule.owner_user_id,
            org_id: schedule.org_id,
            branch_id: schedule.branch_id,
        }
    }
}

/// Pure ownership decision shared by `update_schedule` and `run_events`.
///
/// Fail closed: an unverified/nil owner, a missing org claim, a foreign org,
/// a null-org resource or an owner mismatch is denied. A null-org resource is
/// reachable only by a platform admin (signed admin role claim). A legacy row
/// with no branch scope is gated by org + owner only.
pub(super) fn is_owned(caller: &CallerIdentity, target: &OwnedResource) -> bool {
    if caller.is_admin {
        return true;
    }
    if caller.owner.is_nil() || target.owner_user_id.is_nil() || caller.owner != target.owner_user_id {
        return false;
    }
    let (Some(caller_org), Some(target_org)) = (caller.org, target.org_id) else {
        return false;
    };
    if caller_org != target_org {
        return false;
    }
    if let Some(branch) = target.branch_id {
        if caller.branch != Some(branch) {
            return false;
        }
    }
    true
}

/// Guards `update_schedule`: load the row and require the verified caller to
/// own it before any field is mutated. Foreign/unknown ids answer 404 so the
/// endpoint does not disclose another tenant's schedule.
pub(super) fn assert_owned_schedule(
    state: &AutomationService,
    headers: &HeaderMap,
    token_param: &str,
    schedule_id: Uuid,
) -> Result<AgentSchedule, (StatusCode, String)> {
    let schedule = load_schedule(state, schedule_id)?;
    match caller_identity(headers, token_param) {
        Some(caller) if is_owned(&caller, &OwnedResource::from(&schedule)) => Ok(schedule),
        _ => Err((StatusCode::NOT_FOUND, "Schedule not found".to_string())),
    }
}

/// Guards `run_events`: resolve the run's parent schedule and verify ownership
/// *before* the SSE stream is opened, so spans/results of a foreign run never
/// leak. A schedule-less run has no claimable owner and is refused.
pub(super) fn assert_owned_run(
    state: &AutomationService,
    headers: &HeaderMap,
    token_param: &str,
    run_id: Uuid,
) -> Result<AgentRun, (StatusCode, String)> {
    let run = load_run_row(state, run_id)?;
    let caller = match caller_identity(headers, token_param) {
        Some(caller) => caller,
        None => return Err((StatusCode::NOT_FOUND, "Run not found".to_string())),
    };
    let schedule_id = match run.schedule_id {
        Some(id) => id,
        None => return Err((StatusCode::NOT_FOUND, "Run not found".to_string())),
    };
    let schedule = load_schedule(state, schedule_id)
        .map_err(|_| (StatusCode::NOT_FOUND, "Run not found".to_string()))?;
    if is_owned(&caller, &OwnedResource::from(&schedule)) {
        Ok(run)
    } else {
        Err((StatusCode::NOT_FOUND, "Run not found".to_string()))
    }
}

/// `GET /api/automations/runs/:id/events` — SSE auth: EventSource cannot set
/// headers, so the suite passes the Bearer token as `?token=`. Either
/// transport must yield a *verified* subject before the ownership guard runs.
pub(super) fn sse_authorized(headers: &HeaderMap, token_param: &str) -> bool {
    caller_identity(headers, token_param).is_some()
}

#[cfg(test)]
mod ownership_tests {
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
}
