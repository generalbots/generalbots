use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use diesel::prelude::*;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::core::bot::runs::goal_runner::progress_percent;

pub(super) type ApiError = (StatusCode, Json<Value>);

pub(super) fn api_error(status: StatusCode, message: &str) -> ApiError {
    (status, Json(json!({ "error": message })))
}

pub(super) fn user_from_headers(headers: &HeaderMap) -> Result<Uuid, ApiError> {
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

#[derive(Debug, Deserialize)]
pub(super) struct CreateBody {
    pub(super) objective_id: Uuid,
    #[serde(default)]
    pub(super) key_result_id: Option<Uuid>,
    #[serde(default)]
    pub(super) interval_seconds: Option<u64>,
    #[serde(default)]
    pub(super) step_value: Option<f64>,
    /// Optional agent instruction: when present and an LLM is configured, the
    /// model chooses each checkpoint's increment instead of `step_value`.
    #[serde(default)]
    pub(super) instruction: Option<String>,
}

#[derive(Debug, diesel::QueryableByName)]
pub(super) struct RunRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub(super) id: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub(super) status: String,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub(super) checkpoints: i32,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub(super) interval_seconds: i32,
    #[diesel(sql_type = diesel::sql_types::Double)]
    pub(super) step_value: f64,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Double>)]
    pub(super) last_value: Option<f64>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Double>)]
    pub(super) target_value: Option<f64>,
}

// Columns and FROM are kept apart: `advance` adds extra columns for the tick.
// Concatenating them after FROM produced 'syntax error at or near ::"' on
// every checkpoint.
pub(super) const RUN_COLUMNS: &str =
    "SELECT r.id::text AS id, r.status, r.checkpoints, r.interval_seconds, \
     r.step_value::double precision AS step_value, r.last_value::double precision AS last_value, \
     k.target_value::double precision AS target_value";
pub(super) const RUN_FROM: &str =
    " FROM goal_runs r LEFT JOIN okr_key_results k ON k.id = r.key_result_id ";

#[derive(Debug, diesel::QueryableByName)]
pub(super) struct ResumeRow {
    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub(super) interval_seconds: i32,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub(super) checkpoints: i32,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub(super) instruction: Option<String>,
}

#[derive(Debug, diesel::QueryableByName)]
pub(super) struct AgentContextRow {
    #[diesel(sql_type = diesel::sql_types::Double)]
    pub(super) current: f64,
    #[diesel(sql_type = diesel::sql_types::Double)]
    pub(super) target: f64,
}

#[derive(Debug, diesel::QueryableByName)]
pub(super) struct KrValues {
    #[diesel(sql_type = diesel::sql_types::Double)]
    pub(super) start_value: f64,
    #[diesel(sql_type = diesel::sql_types::Double)]
    pub(super) target_value: f64,
}

#[derive(Debug, diesel::QueryableByName)]
pub(super) struct TickRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub(super) id: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub(super) status: String,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub(super) checkpoints: i32,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub(super) interval_seconds: i32,
    #[diesel(sql_type = diesel::sql_types::Double)]
    pub(super) step_value: f64,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Double>)]
    pub(super) last_value: Option<f64>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Double>)]
    pub(super) target_value: Option<f64>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
    pub(super) key_result_id: Option<Uuid>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
    pub(super) org_id: Option<Uuid>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
    pub(super) branch_id: Option<Uuid>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
    pub(super) bot_id: Option<Uuid>,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub(super) user_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Double>)]
    pub(super) start_value: Option<f64>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Double>)]
    pub(super) current_value: Option<f64>,
}

pub(super) fn run_json(run: &RunRow) -> Value {
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
