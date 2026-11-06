use botcore::shared::utils::DbPool;
use diesel::prelude::*;

use super::types::{RunRow, TickRow, RUN_COLUMNS, RUN_FROM};
use crate::core::bot::runs::goal_runner::{next_value, reached, MAX_CHECKPOINTS};

pub(super) fn load_run(
    conn: &mut diesel::PgConnection,
    id: &str,
) -> Result<RunRow, diesel::result::Error> {
    diesel::sql_query(format!("{RUN_COLUMNS}{RUN_FROM} WHERE r.id = $1::uuid"))
        .bind::<diesel::sql_types::Text, _>(id)
        .get_result::<RunRow>(conn)
}

pub(super) fn advance(pool: &DbPool, id: &str) -> Result<RunRow, String> {
    let mut conn = pool.get().map_err(|error| error.to_string())?;
    let run = diesel::sql_query(format!(
        "{RUN_COLUMNS}, r.user_id, r.key_result_id, \
         k.org_id, k.branch_id, k.bot_id, k.start_value::double precision AS start_value, \
         k.current_value::double precision AS current_value{RUN_FROM} \
         WHERE r.id = $1::uuid"
    ))
    .bind::<diesel::sql_types::Text, _>(id)
    .get_result::<TickRow>(&mut conn)
    .map_err(|error| format!("goal run load failed: {error}"))?;
    if run.status != "running" {
        return run_row_from_tick(&run);
    }

    let mut status = "running".to_string();
    let mut last_value = run.last_value;
    if let (Some(kr), Some(org), Some(branch), Some(bot), Some(target), Some(start)) = (
        run.key_result_id,
        run.org_id,
        run.branch_id,
        run.bot_id,
        run.target_value,
        run.start_value,
    ) {
        let current = run.current_value.unwrap_or(start);
        let value = next_value(current, target, run.step_value);
        last_value = Some(value);
        if reached(value, target) {
            status = "completed".to_string();
        }
        diesel::sql_query(
            "INSERT INTO okr_checkins \
                (org_id, branch_id, bot_id, key_result_id, user_id, previous_value, new_value, note) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind::<diesel::sql_types::Uuid, _>(org)
        .bind::<diesel::sql_types::Uuid, _>(branch)
        .bind::<diesel::sql_types::Uuid, _>(bot)
        .bind::<diesel::sql_types::Uuid, _>(kr)
        .bind::<diesel::sql_types::Uuid, _>(run.user_id)
        .bind::<diesel::sql_types::Double, _>(current)
        .bind::<diesel::sql_types::Double, _>(value)
        .bind::<diesel::sql_types::Text, _>(format!(
            "goal runner checkpoint #{}",
            run.checkpoints + 1
        ))
        .execute(&mut conn)
        .map_err(|error| format!("checkin insert failed: {error}"))?;
        let kr_status = if reached(value, target) {
            "completed"
        } else {
            "in_progress"
        };
        diesel::sql_query(
            "UPDATE okr_key_results SET current_value = $2, status = $3, updated_at = NOW() \
             WHERE id = $1",
        )
        .bind::<diesel::sql_types::Uuid, _>(kr)
        .bind::<diesel::sql_types::Double, _>(value)
        .bind::<diesel::sql_types::Text, _>(kr_status)
        .execute(&mut conn)
        .map_err(|error| format!("key result update failed: {error}"))?;
    }
    if run.checkpoints + 1 >= MAX_CHECKPOINTS as i32 {
        status = "completed".to_string();
    }

    diesel::sql_query(
        "UPDATE goal_runs \
         SET checkpoints = checkpoints + 1, last_value = $2, status = $3, \
             next_checkpoint_at = NOW() + ($4 * INTERVAL '1 second'), updated_at = NOW() \
         WHERE id = $1::uuid",
    )
    .bind::<diesel::sql_types::Text, _>(id)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Double>, _>(last_value)
    .bind::<diesel::sql_types::Text, _>(&status)
    .bind::<diesel::sql_types::Integer, _>(run.interval_seconds)
    .execute(&mut conn)
    .map_err(|error| format!("goal run update failed: {error}"))?;

    diesel::sql_query(format!("{RUN_COLUMNS}{RUN_FROM} WHERE r.id = $1::uuid"))
        .bind::<diesel::sql_types::Text, _>(id)
        .get_result::<RunRow>(&mut conn)
        .map_err(|error| format!("goal run reload failed: {error}"))
}

fn run_row_from_tick(run: &TickRow) -> Result<RunRow, String> {
    Ok(RunRow {
        id: run.id.clone(),
        status: run.status.clone(),
        checkpoints: run.checkpoints,
        interval_seconds: run.interval_seconds,
        step_value: run.step_value,
        last_value: run.last_value,
        target_value: run.target_value,
    })
}
