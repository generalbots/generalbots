//! `api::run_ownership` — ownership gate for by-id Vibe run reads (#2855).
//!
//! Trace/spans/metrics endpoints used to look up a bare `run_id` from the URL
//! and return the payload to any logged-in caller. Scope is now resolved only
//! from server-verified data: the acting user recorded on the run at creation
//! and its bot's tenant, versus the authenticated principal and the
//! signature-verified JWT branch claim. No client-supplied org/branch is ever
//! consulted. Foreign and unknown ids both surface as 404.

use super::*;

/// Verified ownership scope of a run or a caller (#2855).
///
/// A run's scope is the user recorded at creation plus the org/branch of the
/// bot it executed on. A caller's scope is the authenticated user plus the
/// org/branch claims minted by the server at login. Both share this shape so
/// the pure [`run_owned_by`] predicate can compare them directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RunOwnershipScope {
    pub(crate) user_id: Uuid,
    pub(crate) org_id: Uuid,
    pub(crate) branch_id: Uuid,
}

/// Pure ownership predicate (#2855).
///
/// The acting user on the run is the primary boundary — it is stamped from
/// the verified caller at creation, so a caller can never read a run created
/// by somebody else, even inside the same tenant. The org/branch claims are a
/// second boundary: whenever the caller carries a verified claim, the run's
/// bot scope must match it exactly. A nil caller claim means "no assertion"
/// (legacy token or desktop-local install) and leaves the user-id boundary as
/// the sole gate — it never widens access to another user's run.
pub(crate) fn run_owned_by(run: &RunOwnershipScope, caller: &RunOwnershipScope) -> bool {
    if run.user_id != caller.user_id {
        return false;
    }
    scope_component_matches(run.org_id, caller.org_id)
        && scope_component_matches(run.branch_id, caller.branch_id)
}

fn scope_component_matches(run: Uuid, caller: Uuid) -> bool {
    if caller.is_nil() {
        return true;
    }
    run == caller
}

/// Builds the caller's verified scope. `branch_id` must originate from a
/// signature-verified JWT claim (`branch_from_jwt_pool`), never from a query
/// parameter or request body; `org_id` comes from the authenticated principal.
pub(crate) fn caller_run_scope(
    user: &AuthenticatedUser,
    verified_branch_id: Option<Uuid>,
) -> RunOwnershipScope {
    RunOwnershipScope {
        user_id: user.user_id,
        org_id: user.organization_id.unwrap_or_else(Uuid::nil),
        branch_id: verified_branch_id.unwrap_or_else(Uuid::nil),
    }
}

/// Resolves the run's tenant scope from its bot row (server-side). `None` when
/// the bot is missing makes the run unknown/foreign to the caller.
pub(crate) fn run_ownership_scope(api: &VibeApiInner, run: &VibeRun) -> Option<RunOwnershipScope> {
    let mut conn = api.state.db_pool().get().ok()?;
    #[derive(QueryableByName)]
    struct BotScopeRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        org_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        branch_id: Uuid,
    }
    let row = diesel::sql_query("SELECT org_id, branch_id FROM bots WHERE id = $1")
        .bind::<diesel::sql_types::Uuid, _>(run.bot_id)
        .get_result::<BotScopeRow>(&mut conn)
        .optional()
        .ok()??;
    Some(RunOwnershipScope {
        user_id: run.user_id,
        org_id: row.org_id,
        branch_id: row.branch_id,
    })
}

/// Loads a run by id only when it belongs to the verified caller. Covers live
/// state, the in-memory map and the persisted store (runs survive restarts);
/// a missing row, a missing bot scope or a foreign owner all become `None`,
/// which handlers translate into the same 404.
pub(crate) async fn load_owned_run(
    api: &VibeApiInner,
    run_id: Uuid,
    caller: &RunOwnershipScope,
) -> Option<VibeRun> {
    let candidate = {
        let active = api.state.active_runs().read().await;
        active.get(&run_id).cloned()
    };
    let candidate = match candidate {
        Some(run) => Some(run),
        None => {
            let runs = api.runs.read().await;
            runs.get(&run_id).cloned()
        }
    };
    let candidate = candidate.or_else(|| api.runs_store.get_run(run_id))?;
    let scope = run_ownership_scope(api, &candidate)?;
    if run_owned_by(&scope, caller) {
        Some(candidate)
    } else {
        None
    }
}

/// Uniform 404 for foreign or unknown run ids (#2855).
pub(crate) fn run_not_found() -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "success": false, "error": "Run not found" })),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scope(user: u128, org: u128, branch: u128) -> RunOwnershipScope {
        RunOwnershipScope {
            user_id: Uuid::from_u128(user),
            org_id: Uuid::from_u128(org),
            branch_id: Uuid::from_u128(branch),
        }
    }

    #[test]
    fn owner_in_the_same_tenant_is_allowed() {
        assert!(run_owned_by(&scope(1, 10, 20), &scope(1, 10, 20)));
    }

    #[test]
    fn a_different_user_is_never_allowed_even_in_the_same_tenant() {
        assert!(!run_owned_by(&scope(1, 10, 20), &scope(2, 10, 20)));
    }

    #[test]
    fn a_foreign_org_or_branch_is_denied() {
        assert!(!run_owned_by(&scope(1, 10, 20), &scope(1, 11, 20)));
        assert!(!run_owned_by(&scope(1, 10, 20), &scope(1, 10, 21)));
    }

    #[test]
    fn nil_caller_claims_fall_back_to_the_user_boundary() {
        let caller = scope(1, 0, 0);
        assert!(run_owned_by(&scope(1, 10, 20), &caller));
        // Still a different user, so still denied.
        assert!(!run_owned_by(&scope(2, 10, 20), &caller));
    }

    #[test]
    fn an_asserting_caller_cannot_read_an_unscoped_legacy_run() {
        assert!(!run_owned_by(&scope(1, 0, 0), &scope(1, 10, 20)));
        assert!(!run_owned_by(&scope(1, 10, 0), &scope(1, 10, 20)));
    }
}
