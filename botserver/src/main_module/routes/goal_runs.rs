//! Long-horizon goal runs (#1705).
//!
//! `POST /api/goals/runs` starts a bounded background runner for one key
//! result: every `interval_seconds` it advances the value by `step_value`
//! (clamped at the target), writes an `okr_checkins` row and updates the run.
//! It stops at the target, after `MAX_CHECKPOINTS`, or on `/stop`.
//!
//! Arithmetic lives in the tested `runs::goal_runner` module.

use std::sync::Arc;
use std::time::Duration;

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::{Json, Router};
use botcore::shared::state::AppState;
use botcore::shared::utils::DbPool;
use diesel::prelude::*;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::core::bot::runs::goal_runner::{
    clamp_interval, next_value, progress_percent, reached, MAX_CHECKPOINTS,
};

type ApiError = (StatusCode, Json<Value>);

fn api_error(status: StatusCode, message: &str) -> ApiError {
    (status, Json(json!({ "error": message })))
}

fn user_from_headers(headers: &HeaderMap) -> Result<Uuid, ApiError> {
    let token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or_else(|| api_error(StatusCode::UNAUTHORIZED, "missing token"))?;
    let (claims_user, _roles) =
        crate::main_module::routes::anonymous_auth::resolve_token_user(token)
            .ok_or_else(|| api_error(StatusCode::UNAUTHORIZED, "invalid token"))?;
    Ok(crate::security::user_role::derive_stable_user_uuid(
        &claims_user,
    ))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/goals/runs", post(create).get(list))
        .route("/api/goals/runs/:id/tick", post(tick_now))
        .route("/api/goals/runs/:id/stop", post(stop))
}

#[derive(Debug, Deserialize)]
struct CreateBody {
    objective_id: Uuid,
    #[serde(default)]
    key_result_id: Option<Uuid>,
    #[serde(default)]
    interval_seconds: Option<u64>,
    #[serde(default)]
    step_value: Option<f64>,
}

#[derive(Debug, diesel::QueryableByName)]
struct RunRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    id: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    status: String,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    checkpoints: i32,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    interval_seconds: i32,
    #[diesel(sql_type = diesel::sql_types::Double)]
    step_value: f64,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Double>)]
    last_value: Option<f64>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Double>)]
    target_value: Option<f64>,
}

// Columns and FROM are kept apart: `advance` adds extra columns for the
// tick. Concatenating them after FROM produced 'syntax error at or near ::"'
// on every checkpoint.
const RUN_COLUMNS: &str = "SELECT r.id::text AS id, r.status, r.checkpoints, r.interval_seconds, \
    r.step_value::double precision AS step_value, r.last_value::double precision AS last_value, \
    k.target_value::double precision AS target_value";
const RUN_FROM: &str = " FROM goal_runs r LEFT JOIN okr_key_results k ON k.id = r.key_result_id ";

async fn create(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<CreateBody>,
) -> Result<Json<Value>, ApiError> {
    let user = user_from_headers(&headers)?;
    let interval = clamp_interval(body.interval_seconds.unwrap_or(60));
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
                    ((values.target_value - values.start_value) / 10.0).abs().max(0.01)
                }
                _ => 1.0,
            }
        }
    };

    let run = diesel::sql_query(
        "INSERT INTO goal_runs \
            (objective_id, key_result_id, user_id, step_value, interval_seconds, next_checkpoint_at) \
         VALUES ($1, $2, $3, $4, $5, NOW() + ($5 * INTERVAL '1 second')) \
         RETURNING id::text AS id, status, checkpoints, interval_seconds, \
                   step_value::double precision AS step_value, \
                   last_value::double precision AS last_value, NULL::double precision AS target_value",
    )
    .bind::<diesel::sql_types::Uuid, _>(body.objective_id)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(body.key_result_id)
    .bind::<diesel::sql_types::Uuid, _>(user)
    .bind::<diesel::sql_types::Double, _>(step)
    .bind::<diesel::sql_types::Integer, _>(interval as i32)
    .get_result::<RunRow>(&mut conn)
    .map_err(|error| {
        api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("goal run insert failed: {error}"),
        )
    })?;

    // Bounded background runner: no task can outlive its checkpoint budget.
    let pool = state.conn.clone();
    let run_id = run.id.clone();
    tokio::spawn(async move {
        for _ in 0..MAX_CHECKPOINTS {
            tokio::time::sleep(Duration::from_secs(interval)).await;
            let pool = pool.clone();
            let run_id = run_id.clone();
            let outcome = tokio::task::spawn_blocking(move || advance(&pool, &run_id)).await;
            match outcome {
                Ok(Ok(row)) if row.status == "running" => continue,
                _ => break,
            }
        }
    });

    Ok(Json(json!({ "run": run_json(&run) })))
}

#[derive(Debug, diesel::QueryableByName)]
struct KrValues {
    #[diesel(sql_type = diesel::sql_types::Double)]
    start_value: f64,
    #[diesel(sql_type = diesel::sql_types::Double)]
    target_value: f64,
}

#[derive(Debug, diesel::QueryableByName)]
struct TickRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    id: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    status: String,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    checkpoints: i32,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    interval_seconds: i32,
    #[diesel(sql_type = diesel::sql_types::Double)]
    step_value: f64,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Double>)]
    last_value: Option<f64>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Double>)]
    target_value: Option<f64>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
    key_result_id: Option<Uuid>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
    org_id: Option<Uuid>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
    bot_id: Option<Uuid>,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    user_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Double>)]
    start_value: Option<f64>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Double>)]
    current_value: Option<f64>,
}

fn advance(pool: &DbPool, id: &str) -> Result<RunRow, String> {
    let mut conn = pool.get().map_err(|error| error.to_string())?;
    let run = diesel::sql_query(format!(
        "{RUN_COLUMNS}, r.user_id, r.key_result_id, \
         k.org_id, k.bot_id, k.start_value::double precision AS start_value, \
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
    if let (Some(kr), Some(org), Some(bot), Some(target), Some(start)) = (
        run.key_result_id,
        run.org_id,
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
                (org_id, bot_id, key_result_id, user_id, previous_value, new_value, note) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind::<diesel::sql_types::Uuid, _>(org)
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

fn run_json(run: &RunRow) -> Value {
    let percent = match (run.target_value, run.last_value) {
        (Some(target), Some(value)) => progress_percent(0.0, target, value),
        _ => 0,
    };
    json!({
        "id": run.id,
        "status": run.status,
        "checkpoints": run.checkpoints,
        "interval_seconds": run.interval_seconds,
        "step_value": run.step_value,
        "last_value": run.last_value,
        "target_value": run.target_value,
        "progress_percent": percent,
    })
}

async fn tick_now(
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

async fn stop(
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
struct ListQuery {
    objective_id: String,
}

async fn list(
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
