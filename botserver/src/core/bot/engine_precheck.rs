//! Runs the subscription engines' safety self-checks before the person's first
//! turn needs them.
//!
//! Codex and Claude Code each prove once per workspace (Codex: per CLI
//! version) that they ask before acting; the first turn after a server start
//! used to wait 5–8 s for that. Shortly after start, and right after a
//! sign-in, the check now runs in the background for every engine that is
//! linked and whose workspace is running. Start-up never waits for it, a
//! failed or inconclusive run only leaves the turn to check again, and an
//! engine found acting unasked stays refused exactly as before.

use std::time::Duration;

use diesel::sql_types::{Text, Uuid as SqlUuid};
use diesel::RunQueryDsl;
use uuid::Uuid;

use botcore::shared::utils::DbPool;

use crate::core::ai_accounts::{CLAUDE_CODE, CODEX_CLI};

/// Lets start-up settle (workspaces resume, pools fill) before checking.
#[cfg(feature = "agent-vm")]
const AFTER_START: Duration = Duration::from_secs(20);
/// The sign-in finishes writing the engine's credentials first.
const AFTER_SIGN_IN: Duration = Duration::from_secs(3);

/// Engines whose self-check this module runs ahead of the first turn.
pub(crate) fn prechecked(provider: &str) -> bool {
    provider == CODEX_CLI || provider == CLAUDE_CODE
}

#[derive(diesel::QueryableByName)]
struct LinkedEngine {
    #[diesel(sql_type = SqlUuid)]
    user_id: Uuid,
    #[diesel(sql_type = Text)]
    provider: String,
    #[diesel(sql_type = Text)]
    vm_name: String,
}

fn linked_engines(pool: &DbPool) -> Vec<LinkedEngine> {
    let Ok(mut conn) = pool.get() else {
        return Vec::new();
    };
    diesel::sql_query(
        "SELECT user_id, provider, vm_name FROM user_llm_accounts \
         WHERE status = 'linked' AND vm_name IS NOT NULL AND provider IN ($1, $2) \
         ORDER BY updated_at DESC",
    )
    .bind::<Text, _>(CODEX_CLI)
    .bind::<Text, _>(CLAUDE_CODE)
    .load::<LinkedEngine>(&mut conn)
    .unwrap_or_else(|error| {
        log::warn!("engine pre-check: linked engines could not be read: {error}");
        Vec::new()
    })
}

/// Checks every linked engine shortly after the server started, one at a
/// time, in the background.
#[cfg(feature = "agent-vm")]
pub fn after_start(pool: DbPool) {
    tokio::spawn(async move {
        tokio::time::sleep(AFTER_START).await;
        for engine in linked_engines(&pool) {
            run(engine.user_id, &engine.provider, &engine.vm_name).await;
        }
    });
}

/// Checks the engine the person just signed in to, in the background.
pub fn after_sign_in(user_id: Uuid, provider: &'static str, vm_name: String) {
    if !prechecked(provider) {
        return;
    }
    tokio::spawn(async move {
        tokio::time::sleep(AFTER_SIGN_IN).await;
        run(user_id, provider, &vm_name).await;
    });
}

async fn run(user_id: Uuid, provider: &str, vm_name: &str) {
    let Some(service) = crate::core::bot::agent_vm_hook::service() else {
        return;
    };
    let Ok(Some(row)) = botagent::vm::get_user_vm(service, &user_id) else {
        return;
    };
    if row.vm_name != vm_name || row.status != botagent::vm::STATUS_RUNNING {
        return;
    }
    let live = botagent::vm::workspace_state(service, &user_id, &row.vm_name, &row.status).await;
    if live != botagent::vm::WorkspaceState::Running {
        return;
    }
    let outcome = match provider {
        CODEX_CLI => botagent::codex::precheck(service, vm_name)
            .await
            .map_err(|error| format!("{error:?}")),
        CLAUDE_CODE => botagent::claude::bridge::precheck(service, vm_name)
            .await
            .map_err(|(_, message)| message),
        _ => return,
    };
    match outcome {
        Ok(()) => botlog::safe_info!("engine pre-check: {provider} in {vm_name} asks before acting"),
        Err(error) => botlog::safe_warn!("engine pre-check: {provider} in {vm_name}: {error}"),
    }
}

#[cfg(test)]
mod tests {
    use super::prechecked;

    #[test]
    fn only_the_engines_with_a_self_check_are_prechecked() {
        assert!(prechecked("codex-cli"));
        assert!(prechecked("claude-code"));
        assert!(!prechecked("antigravity"));
        assert!(!prechecked("chatgpt-plan"));
    }
}
