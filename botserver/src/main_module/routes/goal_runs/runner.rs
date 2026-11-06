use std::sync::Arc;
use std::time::Duration;

use botcore::shared::state::AppState;
use botcore::shared::utils::DbPool;
use diesel::prelude::*;

use super::advance::advance;
use super::types::AgentContextRow;

/// Bounded background runner: no task can outlive its checkpoint budget.
/// Shared by create and resume; pause flips the status and the loop exits at
/// the next checkpoint because `advance` refuses a non-running run.
pub(super) fn spawn_goal_runner(
    state: &Arc<AppState>,
    run_id: String,
    interval: u64,
    agent_instruction: Option<String>,
    budget: u32,
) {
    let pool = state.conn.clone();
    #[cfg(feature = "llm")]
    let agent_provider = state.llm_provider.clone();
    #[cfg(not(feature = "llm"))]
    let agent_provider: Option<Arc<dyn botlib::traits::LLMProvider>> = None;
    tokio::spawn(async move {
        for _ in 0..budget {
            tokio::time::sleep(Duration::from_secs(interval)).await;
            // Agent/LLM-driven checkpoint (#1706): ask the configured model for
            // this increment; any error keeps the fixed step below.
            if let (Some(instruction), Some(provider)) =
                (agent_instruction.as_deref(), agent_provider.as_ref())
            {
                let context = {
                    let pool = pool.clone();
                    let run_id = run_id.clone();
                    tokio::task::spawn_blocking(move || agent_context(&pool, &run_id))
                        .await
                        .ok()
                        .flatten()
                };
                if let Some((current, target)) = context {
                    if let Some(step) = crate::core::bot::runs::goal_agent::choose_step(
                        provider,
                        instruction,
                        current,
                        target,
                    )
                    .await
                    {
                        let pool = pool.clone();
                        let run_id = run_id.clone();
                        if let Err(error) = tokio::task::spawn_blocking(move || {
                            set_agent_step(&pool, &run_id, step)
                        })
                        .await
                        .unwrap_or_else(|error| Err(error.to_string()))
                        {
                            log::warn!("goal run agent step update failed: {error}");
                        }
                    }
                }
            }
            let pool = pool.clone();
            let run_id = run_id.clone();
            let log_id = run_id.clone();
            let outcome = tokio::task::spawn_blocking(move || advance(&pool, &run_id)).await;
            match outcome {
                Ok(Ok(row)) if row.status == "running" => continue,
                Ok(Ok(_)) => break,
                Ok(Err(error)) => {
                    log::warn!("goal run tick failed ({}): {error}", log_id);
                    break;
                }
                Err(error) => {
                    log::warn!("goal run tick task failed ({}): {error}", log_id);
                    break;
                }
            }
        }
    });
}

/// Current value and target for the checkpoint prompt, read in one query.
fn agent_context(pool: &DbPool, id: &str) -> Option<(f64, f64)> {
    let mut conn = pool.get().ok()?;
    let row = diesel::sql_query(
        "SELECT COALESCE(r.last_value, k.current_value)::double precision AS current, \
                k.target_value::double precision AS target \
         FROM goal_runs r JOIN okr_key_results k ON k.id = r.key_result_id \
         WHERE r.id = $1::uuid",
    )
    .bind::<diesel::sql_types::Text, _>(id)
    .get_result::<AgentContextRow>(&mut conn)
    .ok()?;
    Some((row.current, row.target))
}

fn set_agent_step(pool: &DbPool, id: &str, step: f64) -> Result<(), String> {
    let mut conn = pool.get().map_err(|error| error.to_string())?;
    diesel::sql_query("UPDATE goal_runs SET step_value = $2 WHERE id = $1::uuid")
        .bind::<diesel::sql_types::Text, _>(id)
        .bind::<diesel::sql_types::Double, _>(step)
        .execute(&mut conn)
        .map(|_| ())
        .map_err(|error| error.to_string())
}
