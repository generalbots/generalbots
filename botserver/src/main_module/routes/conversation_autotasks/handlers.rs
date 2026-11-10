use std::sync::Arc;

use axum::extract::{Json, Path, State};
use axum::http::{HeaderMap, StatusCode};
use botcore::shared::state::AppState;
use chrono::Utc;
use diesel::prelude::*;
use serde_json::{json, Value};
use uuid::Uuid;

use super::store::{conversation_bot, load_all_rows, load_rows};
use super::types::{next_run_for, respond, AutotaskRow, CreateAutotaskRequest, NewAutotask};
use super::{
    api_error, api_error_db, resolve_user, workspace, ApiError, STATUS_FAILED,
    STATUS_ORPHANED,
    STATUS_PENDING, STATUS_RUNNING, STATUS_SCHEDULED, STATUS_STOPPED,
};
use botschema::conversation_autotasks as table;

/// GET /api/conversations/:conversation_id/autotasks
pub async fn list(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(conversation_id): Path<Uuid>,
) -> Result<Json<Value>, ApiError> {
    let user_id = resolve_user(&headers)?;
    let locale = botcore::i18n::locale_from_headers(&headers);
    let rows = load_rows(&locale, &state, conversation_id, user_id)?;
    // Reconcile lazily: the workspace is usually warm when the conversation is
    // open, and this keeps the indicator honest without a boot-time VM sweep.
    for row in &rows {
        // `orphaned` is not terminal: a workspace that was briefly down comes
        // back with the job still installed. Re-check it or the indicator
        // stays dark forever (#1693 follow-up).
        if row.status == STATUS_RUNNING
            || row.status == STATUS_SCHEDULED
            || row.status == STATUS_ORPHANED
        {
            workspace::refresh(&state, row).await;
        }
    }
    let rows = load_rows(&locale, &state, conversation_id, user_id)?;
    Ok(Json(respond(&rows)))
}

/// GET /api/conversation-autotasks — list every cron/systemd job owned by the
/// current user, across conversations, for the automations dashboard.
pub async fn list_all(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    let user_id = resolve_user(&headers)?;
    let locale = botcore::i18n::locale_from_headers(&headers);
    let rows = load_all_rows(&locale, &state, user_id)?;
    // Lazy reconciliation keeps the displayed status honest.
    for row in &rows {
        if row.status == STATUS_RUNNING
            || row.status == STATUS_SCHEDULED
            || row.status == STATUS_ORPHANED
        {
            workspace::refresh(&state, row).await;
        }
    }
    let rows = load_all_rows(&locale, &state, user_id)?;
    Ok(Json(json!({ "autotasks": rows })))
}

/// POST /api/conversations/:conversation_id/autotasks
pub async fn create(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(conversation_id): Path<Uuid>,
    Json(body): Json<CreateAutotaskRequest>,
) -> Result<Json<Value>, ApiError> {
    let user_id = resolve_user(&headers)?;
    let locale = botcore::i18n::locale_from_headers(&headers);
    let title = body.title.trim();
    let schedule = body.schedule.trim();
    let command = body.command.trim();
    if title.is_empty() || schedule.is_empty() || command.is_empty() {
        return Err(api_error(&locale, StatusCode::BAD_REQUEST, "error-autotask-fields-required"));
    }
    let kind = body.kind.as_deref().unwrap_or("cron").to_ascii_lowercase();
    if kind != "cron" && kind != "systemd" {
        return Err(api_error(&locale, StatusCode::BAD_REQUEST, "error-autotask-kind-invalid"));
    }
    let bot_id = conversation_bot(&locale, &state, conversation_id, user_id)?;
    let mut conn = state
        .conn
        .get()
        .map_err(|error| api_error_db(&locale, StatusCode::SERVICE_UNAVAILABLE, error))?;
    let new_row = NewAutotask {
        conversation_id,
        bot_id,
        user_id,
        title,
        kind: &kind,
        schedule,
        schedule_human: body.schedule_human.as_deref().map(str::trim),
        command,
        unit_name: None,
        status: STATUS_PENDING,
        next_run_at: next_run_for(schedule),
    };
    let row = diesel::insert_into(table::table)
        .values(&new_row)
        .get_result::<AutotaskRow>(&mut conn)
        .map_err(|error| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, error))?;
    // Never hold a pooled connection across the workspace call: installing a
    // job can take seconds and would starve the pool.
    drop(conn);

    let unit = workspace::unit_name_for(row.id);
    let outcome = workspace::install(
        user_id,
        bot_id,
        conversation_id,
        row.id,
        &kind,
        schedule,
        command,
        &unit,
    )
    .await;
    let (status, error) = match outcome {
        Ok(status) => (status, None),
        Err(error) => (STATUS_FAILED.to_string(), Some(error)),
    };
    let mut conn = state
        .conn
        .get()
        .map_err(|error| api_error_db(&locale, StatusCode::SERVICE_UNAVAILABLE, error))?;
    let updated = diesel::update(table::table.find(row.id))
        .set((
            table::status.eq(&status),
            table::unit_name.eq(Some(unit)),
            table::last_error.eq(error),
            table::updated_at.eq(Utc::now()),
        ))
        .get_result::<AutotaskRow>(&mut conn)
        .map_err(|db_error| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, db_error))?;
    log::info!(
        "conversation autotask {} created ({} / {}) status={}",
        updated.id,
        updated.kind,
        updated.schedule,
        updated.status
    );
    Ok(Json(json!({ "autotask": updated })))
}

/// POST /api/conversation-autotasks/:id/cancel
pub async fn cancel(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, ApiError> {
    let user_id = resolve_user(&headers)?;
    let locale = botcore::i18n::locale_from_headers(&headers);
    let mut conn = state
        .conn
        .get()
        .map_err(|error| api_error_db(&locale, StatusCode::SERVICE_UNAVAILABLE, error))?;
    let row = table::table
        .filter(table::id.eq(id))
        .filter(table::user_id.eq(user_id))
        .first::<AutotaskRow>(&mut conn)
        .optional()
        .map_err(|error| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, error))?
        .ok_or_else(|| api_error(&locale, StatusCode::NOT_FOUND, "error-autotask-not-found"))?;
    drop(conn);

    let removal_error = workspace::remove(&row).await;
    let mut conn = state
        .conn
        .get()
        .map_err(|error| api_error_db(&locale, StatusCode::SERVICE_UNAVAILABLE, error))?;
    let updated = diesel::update(table::table.find(row.id))
        .set((
            table::status.eq(STATUS_STOPPED),
            table::last_error.eq(removal_error),
            table::updated_at.eq(Utc::now()),
        ))
        .get_result::<AutotaskRow>(&mut conn)
        .map_err(|error| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, error))?;
    Ok(Json(json!({ "autotask": updated })))
}

/// DELETE /api/conversation-autotasks/:id — remove the job from the workspace
/// and delete the row.
pub async fn delete(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, ApiError> {
    let user_id = resolve_user(&headers)?;
    let locale = botcore::i18n::locale_from_headers(&headers);
    let mut conn = state
        .conn
        .get()
        .map_err(|error| api_error_db(&locale, StatusCode::SERVICE_UNAVAILABLE, error))?;
    let row = table::table
        .filter(table::id.eq(id))
        .filter(table::user_id.eq(user_id))
        .first::<AutotaskRow>(&mut conn)
        .optional()
        .map_err(|error| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, error))?
        .ok_or_else(|| api_error(&locale, StatusCode::NOT_FOUND, "error-autotask-not-found"))?;
    drop(conn);

    let _ = workspace::remove(&row).await;
    let mut conn = state
        .conn
        .get()
        .map_err(|error| api_error_db(&locale, StatusCode::SERVICE_UNAVAILABLE, error))?;
    diesel::delete(table::table.filter(table::id.eq(id)))
        .execute(&mut conn)
        .map_err(|error| api_error_db(&locale, StatusCode::INTERNAL_SERVER_ERROR, error))?;
    Ok(Json(json!({ "deleted": true })))
}
