//! Pooled group budgets and usage reports (#1697).
//!
//! One allowance per group (`group_budgets`) plus attributed spend rows
//! (`group_spend`). The report aggregates rows through the tested pure model
//! in `core::bot::runs::group_budget`, so totals and per-member breakdowns do
//! not drift from the arithmetic the UI relies on.
//!
//! Routes:
//! - `GET  /api/groups/:group/budget`  — allowance, spend, remaining, percent;
//! - `PUT  /api/groups/:group/budget`  — create/update the allowance;
//! - `POST /api/groups/:group/spend`   — attribute spend to a member/run;
//! - `GET  /api/groups/:group/usage`   — pooled report with per-member rows.

use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use botcore::shared::state::AppState;
use diesel::prelude::*;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::core::bot::runs::group_budget::{aggregate, BudgetSummary, SpendEntry};

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

#[derive(Debug, diesel::QueryableByName)]
struct BudgetRow {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    budget_milli: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    spent_milli: i64,
}

#[derive(Debug, diesel::QueryableByName)]
struct BudgetIdRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    id: String,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    budget_milli: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    spent_milli: i64,
}

#[derive(Debug, diesel::QueryableByName)]
struct SpendRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    user_id: String,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    amount_milli: i64,
}

fn summary_json(budget_milli: u64, spent_milli: u64) -> Value {
    let summary = BudgetSummary {
        budget_milli,
        spent_milli,
    };
    json!({
        "budget_milli": summary.budget_milli,
        "spent_milli": summary.spent_milli,
        "remaining_milli": summary.remaining_milli(),
        "percent_used": summary.percent_used(),
        "over_budget": summary.over_budget(),
    })
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/api/groups/:group/budget",
            get(get_budget).put(put_budget),
        )
        .route("/api/groups/:group/spend", post(add_spend))
        .route("/api/groups/:group/usage", get(usage))
}

async fn get_budget(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(group): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let _user = user_from_headers(&headers)?;
    let mut conn = state
        .conn
        .get()
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;
    let row = diesel::sql_query(
        "SELECT budget_milli, spent_milli FROM group_budgets WHERE group_name = $1",
    )
    .bind::<diesel::sql_types::Text, _>(&group)
    .get_result::<BudgetRow>(&mut conn);
    match row {
        Ok(row) => Ok(Json(json!({
            "group": group,
            "exists": true,
            "budget": summary_json(row.budget_milli.max(0) as u64, row.spent_milli.max(0) as u64),
        }))),
        Err(diesel::result::Error::NotFound) => Ok(Json(json!({
            "group": group,
            "exists": false,
            "budget": summary_json(0, 0),
        }))),
        Err(error) => Err(api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("budget lookup failed: {error}"),
        )),
    }
}

#[derive(Debug, Deserialize)]
struct BudgetBody {
    budget_milli: i64,
}

async fn put_budget(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(group): Path<String>,
    Json(body): Json<BudgetBody>,
) -> Result<Json<Value>, ApiError> {
    let user = user_from_headers(&headers)?;
    if body.budget_milli < 0 {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            "budget_milli must be zero or positive",
        ));
    }
    let mut conn = state
        .conn
        .get()
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;
    let row = diesel::sql_query(
        "INSERT INTO group_budgets (owner_id, group_name, budget_milli) \
         VALUES ($1, $2, $3) \
         ON CONFLICT (group_name) DO UPDATE \
         SET budget_milli = EXCLUDED.budget_milli, updated_at = NOW() \
         RETURNING budget_milli, spent_milli",
    )
    .bind::<diesel::sql_types::Uuid, _>(user)
    .bind::<diesel::sql_types::Text, _>(&group)
    .bind::<diesel::sql_types::BigInt, _>(body.budget_milli)
    .get_result::<BudgetRow>(&mut conn)
    .map_err(|error| {
        api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("budget upsert failed: {error}"),
        )
    })?;
    Ok(Json(json!({
        "group": group,
        "exists": true,
        "budget": summary_json(row.budget_milli.max(0) as u64, row.spent_milli.max(0) as u64),
    })))
}

#[derive(Debug, Deserialize)]
struct SpendBody {
    amount_milli: i64,
    #[serde(default)]
    user_id: Option<Uuid>,
    #[serde(default)]
    run_id: Option<String>,
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    note: Option<String>,
}

async fn add_spend(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(group): Path<String>,
    Json(body): Json<SpendBody>,
) -> Result<Json<Value>, ApiError> {
    let caller = user_from_headers(&headers)?;
    if body.amount_milli < 0 {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            "amount_milli must be zero or positive",
        ));
    }
    let member = body.user_id.unwrap_or(caller);
    let kind = body.kind.unwrap_or_else(|| "run".to_string());
    let mut conn = state
        .conn
        .get()
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    let updated = diesel::sql_query(
        "UPDATE group_budgets \
         SET spent_milli = spent_milli + $2, updated_at = NOW() \
         WHERE group_name = $1 \
         RETURNING id::text AS id, budget_milli, spent_milli",
    )
    .bind::<diesel::sql_types::Text, _>(&group)
    .bind::<diesel::sql_types::BigInt, _>(body.amount_milli)
    .get_result::<BudgetIdRow>(&mut conn)
    .optional()
    .map_err(|error| {
        api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("budget update failed: {error}"),
        )
    })?;
    let Some(budget) = updated else {
        return Err(api_error(StatusCode::NOT_FOUND, "group budget not found"));
    };
    let budget_id = Uuid::parse_str(&budget.id)
        .map_err(|_| api_error(StatusCode::INTERNAL_SERVER_ERROR, "invalid budget id"))?;
    diesel::sql_query(
        "INSERT INTO group_spend (budget_id, user_id, run_id, kind, amount_milli, note) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind::<diesel::sql_types::Uuid, _>(budget_id)
    .bind::<diesel::sql_types::Uuid, _>(member)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(body.run_id)
    .bind::<diesel::sql_types::Text, _>(&kind)
    .bind::<diesel::sql_types::BigInt, _>(body.amount_milli)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(body.note)
    .execute(&mut conn)
    .map_err(|error| {
        api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("spend insert failed: {error}"),
        )
    })?;
    Ok(Json(json!({
        "group": group,
        "member": member,
        "kind": kind,
        "amount_milli": body.amount_milli,
        "budget": summary_json(
            budget.budget_milli.max(0) as u64,
            budget.spent_milli.max(0) as u64,
        ),
    })))
}

async fn usage(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(group): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, ApiError> {
    let _user = user_from_headers(&headers)?;
    let limit = params
        .get("limit")
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(200)
        .clamp(1, 1000);
    let mut conn = state
        .conn
        .get()
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;
    let budget = diesel::sql_query(
        "SELECT id::text AS id, budget_milli, spent_milli \
         FROM group_budgets WHERE group_name = $1",
    )
    .bind::<diesel::sql_types::Text, _>(&group)
    .get_result::<BudgetIdRow>(&mut conn)
    .optional()
    .map_err(|error| {
        api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("budget lookup failed: {error}"),
        )
    })?;
    let Some(budget) = budget else {
        return Err(api_error(StatusCode::NOT_FOUND, "group budget not found"));
    };
    let budget_id = Uuid::parse_str(&budget.id)
        .map_err(|_| api_error(StatusCode::INTERNAL_SERVER_ERROR, "invalid budget id"))?;
    let rows = diesel::sql_query(
        "SELECT user_id::text AS user_id, amount_milli \
         FROM group_spend WHERE budget_id = $1 \
         ORDER BY created_at DESC LIMIT $2",
    )
    .bind::<diesel::sql_types::Uuid, _>(budget_id)
    .bind::<diesel::sql_types::BigInt, _>(limit)
    .load::<SpendRow>(&mut conn)
    .map_err(|error| {
        api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("spend load failed: {error}"),
        )
    })?;
    let entries: Vec<SpendEntry<'_>> = rows
        .iter()
        .map(|row| SpendEntry {
            user_id: row.user_id.as_str(),
            amount_milli: row.amount_milli.max(0) as u64,
        })
        .collect();
    let report = aggregate(entries);
    Ok(Json(json!({
        "group": group,
        "budget": summary_json(
            budget.budget_milli.max(0) as u64,
            budget.spent_milli.max(0) as u64,
        ),
        "report": {
            "total_milli": report.total_milli,
            "members": report.members.iter().map(|member| json!({
                "user_id": member.user_id,
                "amount_milli": member.amount_milli,
                "entries": member.entries,
            })).collect::<Vec<_>>(),
        },
    })))
}
