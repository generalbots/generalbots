use std::sync::Arc;

use botcore::shared::state::AppState;
use diesel::prelude::*;
use uuid::Uuid;

use crate::core::bot::runs::caps::RunCaps;

/// Per-run budget (tokens + wall clock). Per-bot settings (plan/workspace
/// `run-max-tokens` / `run-max-seconds`) override the deployment defaults; 0
/// means unlimited everywhere.
pub(super) fn resolve(state: &Arc<AppState>, bot_uuid: Uuid, user_id: Uuid) -> RunCaps {
    use botcore::config::ConfigManager;
    // Group policy first (most restrictive non-zero across the user's
    // groups), then per-bot/plan keys, then deployment defaults.
    let group_policy = {
        #[derive(diesel::QueryableByName)]
        struct PolicyRow {
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::BigInt>)]
            max_tokens: Option<i64>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::BigInt>)]
            max_seconds: Option<i64>,
        }
        state.conn.get().ok().and_then(|mut conn| {
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
        })
    };
    let group_caps = RunCaps::from_env().restrict_to(
        group_policy
            .as_ref()
            .and_then(|row| row.max_tokens)
            .and_then(|value| u64::try_from(value).ok()),
        group_policy
            .as_ref()
            .and_then(|row| row.max_seconds)
            .and_then(|value| u64::try_from(value).ok()),
    );
    let cfg = ConfigManager::new(state.conn.clone());
    let bot_tokens = cfg.get_config(&bot_uuid, "run-max-tokens", None).ok();
    let bot_seconds = cfg.get_config(&bot_uuid, "run-max-seconds", None).ok();
    RunCaps::merge(bot_tokens.as_deref(), bot_seconds.as_deref(), group_caps)
}
