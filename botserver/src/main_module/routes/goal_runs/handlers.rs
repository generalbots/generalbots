use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use botcore::shared::state::AppState;
use diesel::prelude::*;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use super::advance::{advance, load_run};
use super::runner::spawn_goal_runner;
use super::types::{
    api_error, run_json, user_from_headers, ApiError, CreateBody, KrValues, ResumeRow, RunRow,
    RUN_COLUMNS, RUN_FROM,
};
use crate::core::bot::runs::goal_runner::{clamp_interval, MAX_CHECKPOINTS};

pub(super) async fn create(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<CreateBody>,
) -> Result<Json<Value>, ApiError> {
    let user = user_from_headers(&headers)?;
    let interval = clamp_interval(body.interval_seconds.unwrap_or(60));
    let agent_instruction = body
        .instruction
        .clone()
        .filter(|value| !value.trim().is_empty());
    let mut conn = state
        .conn
        .get()
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    // Step defaults to a tenth of the distance so a 2h goal reaches its target
    // without an explicit tuning value.
    let step = match body.step_value {
        Some(step) if step > 0.0 => step,
        _ => {
            let row = diesel::sql_query(
                "SELECT start_value::double precision AS start_value, \
                        target_value::double precision AS target_value \
                 FROM okr_key_results WHERE id = $1",
            )
            .bind::<diesel::sql_types::Uuid, _>(body.key_result_id.unwrap_or_else(Uuid::nil))
            .get_result::<KrValues>(&mut conn)
            .ok();
            match row {
                Some(values) if values.target_value != values.start_value => {
                    ((values.target_value - values.start_value) / 10.0)
                        .abs()
                        .max(0.01)
                }
                _ => 1.0,
            }
        }
    };

    let run = diesel::sql_query(
        "INSERT INTO goal_runs \
            (objective_id, key_result_id, user_id, step_value, interval_seconds, next_checkpoint_at, instruction) \
         VALUES ($1, $2, $3, $4, $5, NOW() + ($5 * INTERVAL '1 second'), $6) \
         RETURNING id::text AS id, status, checkpoints, interval_seconds, \
                   step_value::double precision AS step_value, \
                   last_value::double precision AS last_value, NULL::double precision AS target_value",
    )
    .bind::<diesel::sql_types::Uuid, _>(body.objective_id)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(body.key_result_id)
    .bind::<diesel::sql_types::Uuid, _>(user)
    .bind::<diesel::sql_types::Double, _>(step)
    .bind::<diesel::sql_types::Integer, _>(interval as i32)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(agent_instruction.clone())
    .get_result::<RunRow>(&mut conn)
    .map_err(|error| {
        api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("goal run insert failed: {error}"),
        )
    })?;

    spawn_goal_runner(
        &state,
        run.id.clone(),
        interval,
        agent_instruction,
        MAX_CHECKPOINTS,
    );

    Ok(Json(json!({ "run": run_json(&run) })))
}

/// POST /api/goals/runs/:id/pause — the background task exits at its next
/// checkpoint (advance refuses a non-running run).
pub(super) async fn pause(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    let _user = user_from_headers(&headers)?;
    let mut conn = state
        .conn
        .get()
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;
    let run = diesel::sql_query(
        "UPDATE goal_runs SET status = 'paused', updated_at = NOW() \
         WHERE id = $1::uuid AND status = 'running' \
         RETURNING id::text AS id, status, checkpoints, interval_seconds, \
                   step_value::double precision AS step_value, \
                   last_value::double precision AS last_value, NULL::double precision AS target_value",
    )
    .bind::<diesel::sql_types::Text, _>(&id)
    .get_result::<RunRow>(&mut conn)
    .map_err(|_| api_error(StatusCode::BAD_REQUEST, "goal run is not running"))?;
    Ok(Json(json!({ "run": run_json(&run) })))
}

/// POST /api/goals/runs/:id/resume — flips a paused run back to running and
/// restarts the bounded runner with the same instruction and checkpoint budget.
pub(super) async fn resume(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    let _user = user_from_headers(&headers)?;
    let mut conn = state
        .conn
        .get()
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;
    let resumed = diesel::sql_query(
        "UPDATE goal_runs \
         SET status = 'running', updated_at = NOW(), \
             next_checkpoint_at = NOW() + (interval_seconds * INTERVAL '1 second') \
         WHERE id = $1::uuid AND status = 'paused' \
         RETURNING interval_seconds, checkpoints, instruction",
    )
    .bind::<diesel::sql_types::Text, _>(&id)
    .get_result::<ResumeRow>(&mut conn)
    .map_err(|_| api_error(StatusCode::BAD_REQUEST, "goal run is not paused"))?;
    let interval = clamp_interval(u64::try_from(resumed.interval_seconds.max(0)).unwrap_or(60));
    let done = u32::try_from(resumed.checkpoints).unwrap_or(MAX_CHECKPOINTS);
    let budget = MAX_CHECKPOINTS.saturating_sub(done);
    if budget > 0 {
        spawn_goal_runner(&state, id.clone(), interval, resumed.instruction, budget);
    }
    let run = load_run(&mut conn, &id).map_err(|e| {
        api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("goal run load failed: {e}"),
        )
    })?;
    Ok(Json(json!({ "run": run_json(&run) })))
}

pub(super) async fn tick_now(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let _user = user_from_headers(&headers)?;
    let pool = state.conn.clone();
    let run = tokio::task::spawn_blocking(move || advance(&pool, &id))
        .await
        .map_err(|error| api_error(StatusCode::INTERNAL_SERVER_ERROR, &error.to_string()))?
        .map_err(|error| api_error(StatusCode::BAD_REQUEST, &error))?;
    Ok(Json(json!({ "run": run_json(&run) })))
}

pub(super) async fn stop(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let _user = user_from_headers(&headers)?;
    let mut conn = state
        .conn
        .get()
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;
    diesel::sql_query(
        "UPDATE goal_runs SET status = 'stopped', updated_at = NOW() \
         WHERE id = $1::uuid AND status = 'running'",
    )
    .bind::<diesel::sql_types::Text, _>(id)
    .execute(&mut conn)
    .map_err(|error| api_error(StatusCode::INTERNAL_SERVER_ERROR, &error.to_string()))?;
    Ok(Json(json!({ "stopped": true })))
}

#[derive(Debug, Deserialize)]
pub(super) struct ListQuery {
    objective_id: String,
}

pub(super) async fn list(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> Result<Json<Value>, ApiError> {
    let _user = user_from_headers(&headers)?;
    let mut conn = state
        .conn
        .get()
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;
    let rows = diesel::sql_query(format!(
        "{RUN_COLUMNS}{RUN_FROM} WHERE r.objective_id = $1::uuid ORDER BY r.created_at DESC LIMIT 50"
    ))
    .bind::<diesel::sql_types::Text, _>(&query.objective_id)
    .load::<RunRow>(&mut conn)
    .map_err(|error| api_error(StatusCode::INTERNAL_SERVER_ERROR, &error.to_string()))?;
    Ok(Json(json!({
        "runs": rows.iter().map(run_json).collect::<Vec<_>>(),
    })))
}
