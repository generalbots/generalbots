use super::*;
use botsecurity_core::error_sanitizer::{log_and_sanitize_str, SafeErrorResponse};
use botsecurity_core::verified_jwt::{verify_saas_jwt, VerifiedJwt};

/// #2753 — ownership guards hand back a sanitised envelope plus the concrete
/// status. The bridge below exists only for the legacy api.rs handlers: it
/// forwards the generic envelope message, never a storage/IO detail.
pub(super) struct OwnedError {
    status: StatusCode,
    body: SafeErrorResponse,
}

impl OwnedError {
    fn storage(status: StatusCode, message: &str) -> Self {
        let body = if status.is_server_error() {
            log_and_sanitize_str(message, "botautomation ownership", None)
        } else {
            SafeErrorResponse::new("error", message)
        };
        Self { status, body }
    }

    fn not_found(resource: &'static str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            body: SafeErrorResponse::not_found(resource),
        }
    }
}

fn ownership_db_error(e: impl std::fmt::Display) -> OwnedError {
    let (status, message) = db_err("verify bot ownership", e);
    OwnedError::storage(status, &message)
}

type LegacyError = (StatusCode, String);

impl From<OwnedError> for LegacyError {
    fn from(error: OwnedError) -> Self {
        (error.status, error.body.message)
    }
}

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

pub(super) fn caller_identity(headers: &HeaderMap, token_param: &str) -> Option<CallerIdentity> {
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
) -> Result<AgentSchedule, OwnedError> {
    let schedule = load_schedule(state, schedule_id)
        .map_err(|(status, message)| OwnedError::storage(status, &message))?;
    match caller_identity(headers, token_param) {
        Some(caller) if is_owned(&caller, &OwnedResource::from(&schedule)) => Ok(schedule),
        _ => Err(OwnedError::not_found("Schedule")),
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
) -> Result<AgentRun, OwnedError> {
    let run = load_run_row(state, run_id)
        .map_err(|(status, message)| OwnedError::storage(status, &message))?;
    let caller = match caller_identity(headers, token_param) {
        Some(caller) => caller,
        None => return Err(OwnedError::not_found("Run")),
    };
    let schedule_id = match run.schedule_id {
        Some(id) => id,
        None => return Err(OwnedError::not_found("Run")),
    };
    let schedule = load_schedule(state, schedule_id)
        .map_err(|(status, message)| OwnedError::storage(status, &message))
        .map_err(|_| OwnedError::not_found("Run"))?;
    if is_owned(&caller, &OwnedResource::from(&schedule)) {
        Ok(run)
    } else {
        Err(OwnedError::not_found("Run"))
    }
}

/// Pure predicate for `create_schedule`'s client-supplied `bot_id`: a verified
/// caller may target a bot only when the bot's org and branch match the caller's
/// signed claims. Unverifiable scope fails closed; a platform admin may target
/// any bot. This closes the vector where an unauthorised bot id drives an LLM
/// goal with that bot's tool allowlist (engine/exec.rs:63).
pub(super) fn bot_is_owned(caller: &CallerIdentity, bot_org: Uuid, bot_branch: Uuid) -> bool {
    if caller.is_admin {
        return true;
    }
    let (Some(org), Some(branch)) = (caller.org, caller.branch) else {
        return false;
    };
    org == bot_org && branch == bot_branch
}

/// Guards `create_schedule`: the client-supplied `bot_id` must resolve to a bot
/// inside the caller's verified org/branch before a schedule may drive goals
/// against it. An unknown, foreign or unverifiable target answers 404 so the
/// endpoint discloses no other tenant's bot.
pub(super) fn assert_owned_bot(
    state: &AutomationService,
    headers: &HeaderMap,
    bot_id: Uuid,
) -> Result<(), OwnedError> {
    let caller = caller_identity(headers, "").ok_or_else(|| OwnedError::not_found("Bot"))?;
    let mut conn = state
        .pool()
        .get()
        .map_err(|e| ownership_db_error(e))?;
    use botschema::bots::dsl as b;
    let bot = b::bots
        .filter(b::id.eq(bot_id))
        .select((b::org_id, b::branch_id))
        .first::<(Uuid, Uuid)>(&mut conn)
        .optional()
        .map_err(ownership_db_error)?
        .ok_or_else(|| OwnedError::not_found("Bot"))?;
    if bot_is_owned(&caller, bot.0, bot.1) {
        Ok(())
    } else {
        Err(OwnedError::not_found("Bot"))
    }
}

/// `GET /api/automations/runs/:id/events` — SSE auth: EventSource cannot set
/// headers, so the suite passes the Bearer token as `?token=`. Either
/// transport must yield a *verified* subject before the ownership guard runs.
pub(super) fn sse_authorized(headers: &HeaderMap, token_param: &str) -> bool {
    caller_identity(headers, token_param).is_some()
}

#[cfg(test)]
#[path = "api_helpers_tests.rs"]
mod ownership_tests;
