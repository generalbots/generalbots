//! Conversation-scoped autotasks (#1613).
//!
//! Long-running work started from a conversation — "remind me every Monday",
//! "keep this service running" — is persisted as a first-class row and mirrored
//! to a cron entry or a systemd unit inside the user's agent workspace. The row
//! is the source of truth the chat reads; the workspace entry is the live
//! execution. Reconciliation flips a row to `orphaned` when the workspace no
//! longer knows about the job.

mod workspace;

use std::sync::Arc;

use axum::extract::{Json, Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::Router;
use botcore::shared::state::AppState;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use botschema::conversation_autotasks as table;

pub(super) const STATUS_SCHEDULED: &str = "scheduled";
pub(super) const STATUS_RUNNING: &str = "running";
const STATUS_STOPPED: &str = "stopped";
pub(super) const STATUS_ORPHANED: &str = "orphaned";
pub(super) const STATUS_FAILED: &str = "failed";
pub(super) const STATUS_PENDING: &str = "pending";

type ApiError = (StatusCode, Json<Value>);

fn api_error(status: StatusCode, message: &str) -> ApiError {
    (status, Json(json!({ "error": message })))
}

#[derive(Queryable, Serialize)]
pub(super) struct AutotaskRow {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub bot_id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub kind: String,
    pub schedule: String,
    pub schedule_human: Option<String>,
    pub command: String,
    pub unit_name: Option<String>,
    pub status: String,
    pub output_path: Option<String>,
    pub last_run_at: Option<DateTime<Utc>>,
    pub next_run_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = table)]
struct NewAutotask<'a> {
    conversation_id: Uuid,
    bot_id: Uuid,
    user_id: Uuid,
    title: &'a str,
    kind: &'a str,
    schedule: &'a str,
    schedule_human: Option<&'a str>,
    command: &'a str,
    unit_name: Option<&'a str>,
    status: &'a str,
    next_run_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAutotaskRequest {
    pub title: String,
    pub schedule: String,
    pub command: String,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub schedule_human: Option<String>,
    #[serde(default)]
    pub timezone: Option<String>,
}

fn resolve_user(headers: &HeaderMap) -> Result<Uuid, ApiError> {
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

fn next_run_for(schedule: &str) -> Option<DateTime<Utc>> {
    #[cfg(feature = "automation")]
    {
        use std::str::FromStr;
        let parsed = cron::Schedule::from_str(schedule).ok()?;
        return parsed.after(&Utc::now()).next();
    }
    #[cfg(not(feature = "automation"))]
    {
        let _ = schedule;
        None
    }
}

fn respond(rows: &[AutotaskRow]) -> Value {
    let active = rows
        .iter()
        .filter(|row| row.status == STATUS_RUNNING || row.status == STATUS_SCHEDULED)
        .count();
    let persistent = rows
        .iter()
        .filter(|row| row.kind == "systemd" && row.status == STATUS_RUNNING)
        .count();
    json!({
        "autotasks": rows,
        "active": active,
        "persistent": persistent,
        "has_work": active > 0,
    })
}

fn load_rows(
    state: &AppState,
    conversation_id: Uuid,
    user_id: Uuid,
) -> Result<Vec<AutotaskRow>, ApiError> {
    let mut conn = state
        .conn
        .get()
        .map_err(|error| api_error(StatusCode::SERVICE_UNAVAILABLE, &error.to_string()))?;
    table::table
        .filter(table::conversation_id.eq(conversation_id))
        .filter(table::user_id.eq(user_id))
        .order(table::created_at.desc())
        .load::<AutotaskRow>(&mut conn)
        .map_err(|error| api_error(StatusCode::INTERNAL_SERVER_ERROR, &error.to_string()))
}

/// GET /api/conversations/:conversation_id/autotasks
pub async fn list(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(conversation_id): Path<Uuid>,
) -> Result<Json<Value>, ApiError> {
    let user_id = resolve_user(&headers)?;
    let rows = load_rows(&state, conversation_id, user_id)?;
    // Reconcile lazily: the workspace is usually warm when the conversation is
    // open, and this keeps the indicator honest without a boot-time VM sweep.
    for row in &rows {
        if row.status == STATUS_RUNNING || row.status == STATUS_SCHEDULED {
            workspace::refresh(&state, row).await;
        }
    }
    let rows = load_rows(&state, conversation_id, user_id)?;
    Ok(Json(respond(&rows)))
}

/// POST /api/conversations/:conversation_id/autotasks
pub async fn create(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(conversation_id): Path<Uuid>,
    Json(body): Json<CreateAutotaskRequest>,
) -> Result<Json<Value>, ApiError> {
    let user_id = resolve_user(&headers)?;
    let title = body.title.trim();
    let schedule = body.schedule.trim();
    let command = body.command.trim();
    if title.is_empty() || schedule.is_empty() || command.is_empty() {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            "title, schedule and command are required",
        ));
    }
    let kind = body.kind.as_deref().unwrap_or("cron").to_ascii_lowercase();
    if kind != "cron" && kind != "systemd" {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            "kind must be 'cron' or 'systemd'",
        ));
    }
    let bot_id = conversation_bot(&state, conversation_id, user_id)?;
    let mut conn = state
        .conn
        .get()
        .map_err(|error| api_error(StatusCode::SERVICE_UNAVAILABLE, &error.to_string()))?;
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
        .map_err(|error| api_error(StatusCode::INTERNAL_SERVER_ERROR, &error.to_string()))?;

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
    let updated = diesel::update(table::table.find(row.id))
        .set((
            table::status.eq(&status),
            table::unit_name.eq(Some(unit)),
            table::last_error.eq(error),
            table::updated_at.eq(Utc::now()),
        ))
        .get_result::<AutotaskRow>(&mut conn)
        .map_err(|db_error| api_error(StatusCode::INTERNAL_SERVER_ERROR, &db_error.to_string()))?;
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
    let mut conn = state
        .conn
        .get()
        .map_err(|error| api_error(StatusCode::SERVICE_UNAVAILABLE, &error.to_string()))?;
    let row = table::table
        .filter(table::id.eq(id))
        .filter(table::user_id.eq(user_id))
        .first::<AutotaskRow>(&mut conn)
        .optional()
        .map_err(|error| api_error(StatusCode::INTERNAL_SERVER_ERROR, &error.to_string()))?
        .ok_or_else(|| api_error(StatusCode::NOT_FOUND, "autotask not found"))?;
    drop(conn);

    let removal_error = workspace::remove(&row).await;
    let mut conn = state
        .conn
        .get()
        .map_err(|error| api_error(StatusCode::SERVICE_UNAVAILABLE, &error.to_string()))?;
    let updated = diesel::update(table::table.find(row.id))
        .set((
            table::status.eq(STATUS_STOPPED),
            table::last_error.eq(removal_error),
            table::updated_at.eq(Utc::now()),
        ))
        .get_result::<AutotaskRow>(&mut conn)
        .map_err(|error| api_error(StatusCode::INTERNAL_SERVER_ERROR, &error.to_string()))?;
    Ok(Json(json!({ "autotask": updated })))
}

/// Resolves the bot that owns a conversation and refuses another user's
/// session. `user_sessions.user_id` is text in the canonical schema, so the
/// lookup binds text and parses the owner back into a UUID.
fn conversation_bot(
    state: &AppState,
    conversation_id: Uuid,
    user_id: Uuid,
) -> Result<Uuid, ApiError> {
    use diesel::sql_types::Text;

    #[derive(QueryableByName)]
    struct SessionRow {
        #[diesel(sql_type = Text)]
        bot_id: String,
        #[diesel(sql_type = Text)]
        user_id: String,
    }

    let mut conn = state
        .conn
        .get()
        .map_err(|error| api_error(StatusCode::SERVICE_UNAVAILABLE, &error.to_string()))?;
    let row = diesel::sql_query(
        "SELECT bot_id::text AS bot_id, user_id::text AS user_id \
         FROM user_sessions WHERE id::text = $1 LIMIT 1",
    )
    .bind::<Text, _>(conversation_id.to_string())
    .get_result::<SessionRow>(&mut conn)
    .optional()
    .map_err(|error| api_error(StatusCode::INTERNAL_SERVER_ERROR, &error.to_string()))?
    .ok_or_else(|| api_error(StatusCode::NOT_FOUND, "conversation not found"))?;
    let owner = Uuid::parse_str(row.user_id.trim())
        .map_err(|_| api_error(StatusCode::INTERNAL_SERVER_ERROR, "bad session owner"))?;
    if owner != user_id {
        return Err(api_error(StatusCode::NOT_FOUND, "conversation not found"));
    }
    Uuid::parse_str(row.bot_id.trim())
        .map_err(|_| api_error(StatusCode::INTERNAL_SERVER_ERROR, "bad session bot"))
}

/// Marks every live row as orphaned at boot (#1613). Lazy reconciliation in
/// `list` promotes it back as soon as the workspace confirms the entry, so a
/// job that survived a restart is reported correctly on first open.
pub fn reconcile_on_boot(state: &AppState) {
    let mut conn = match state.conn.get() {
        Ok(conn) => conn,
        Err(error) => {
            log::warn!("conversation autotask reconciliation skipped: {error}");
            return;
        }
    };
    let touched = diesel::update(table::table)
        .filter(table::status.eq_any(vec![STATUS_RUNNING, STATUS_SCHEDULED]))
        .set((
            table::status.eq(STATUS_ORPHANED),
            table::updated_at.eq(Utc::now()),
        ))
        .execute(&mut conn);
    match touched {
        Ok(count) if count > 0 => {
            log::info!("conversation autotasks: {count} row(s) pending reconciliation")
        }
        Ok(_) => {}
        Err(error) => log::warn!("conversation autotask reconciliation failed: {error}"),
    }
}

/// Routes are merged into the API router whose state is `Arc<AppState>`.
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/api/conversations/:conversation_id/autotasks",
            get(list).post(create),
        )
        .route("/api/conversation-autotasks/:id/cancel", post(cancel))
}
