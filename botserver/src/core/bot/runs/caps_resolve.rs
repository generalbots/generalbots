//! Runtime resolution of per-run caps (#1696).
//!
//! The arithmetic lives in the dependency-free `caps` module (unit-tested
//! standalone). This module owns the lookup order that both streaming paths
//! must share:
//!
//! 1. most restrictive non-zero policy across the user's RBAC groups;
//! 2. the bot/plan keys `run-max-tokens` / `run-max-seconds`;
//! 3. the deployment defaults `GB_RUN_MAX_TOKENS` / `GB_RUN_MAX_SECONDS`.
//!
//! Group policies can only tighten, never raise, so a tenant admin cannot
//! bypass a workspace limit through a bot override.

use std::sync::Arc;

use botcore::shared::state::AppState;
use diesel::prelude::*;
use uuid::Uuid;

use super::caps::RunCaps;

#[derive(Debug, diesel::QueryableByName)]
struct PolicyRow {
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::BigInt>)]
    max_tokens: Option<i64>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::BigInt>)]
    max_seconds: Option<i64>,
}

fn group_policy(state: &Arc<AppState>, user_id: Uuid) -> Option<PolicyRow> {
    let mut conn = state.conn.get().ok()?;
    diesel::sql_query(
        "SELECT min(NULLIF(p.max_tokens, 0))::bigint AS max_tokens, \
                min(NULLIF(p.max_seconds, 0))::bigint AS max_seconds \
         FROM group_cap_policies p \
         JOIN rbac_groups g ON g.name = p.group_name \
         JOIN rbac_user_groups ug ON ug.group_id = g.id \
         WHERE ug.user_id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .get_result::<PolicyRow>(&mut conn)
    .ok()
}

/// Resolves the effective caps for one chat run.
#[must_use]
pub fn resolve_for_run(state: &Arc<AppState>, bot_uuid: Uuid, user_id: Uuid) -> RunCaps {
    use botcore::config::ConfigManager;

    let policy = group_policy(state, user_id);
    let group_caps = RunCaps::from_env().restrict_to(
        policy
            .as_ref()
            .and_then(|row| row.max_tokens)
            .and_then(|value| u64::try_from(value).ok()),
        policy
            .as_ref()
            .and_then(|row| row.max_seconds)
            .and_then(|value| u64::try_from(value).ok()),
    );
    let cfg = ConfigManager::new(state.conn.clone());
    let bot_tokens = cfg.get_config(&bot_uuid, "run-max-tokens", None).ok();
    let bot_seconds = cfg.get_config(&bot_uuid, "run-max-seconds", None).ok();
    let caps = RunCaps::merge(bot_tokens.as_deref(), bot_seconds.as_deref(), group_caps);
    log::info!(
        "run caps resolved: max_tokens={} max_seconds={} (bot {}, defaults t={} s={})",
        caps.max_tokens,
        caps.max_seconds,
        bot_uuid,
        group_caps.max_tokens,
        group_caps.max_seconds
    );
    caps
}
