//! HTTP API for the automations dashboard (#1170): schedule CRUD, manual
//! triggers, run listing, cancellation and the SSE progress stream.

use crate::cron::CronExpr;
use crate::engine;
use crate::models::{AgentRun, AgentSchedule, NewAgentRun, NewAgentSchedule, ScheduleCreateBody, ScheduleUpdateBody};
use crate::schema::{agent_runs, agent_schedules};
use crate::state::AutomationService;
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::sse::{Event, KeepAlive, Sse},
    Json,
};
use botsecurity_core::tenant;
use diesel::prelude::*;
use serde::Deserialize;
use serde_json::json;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

const MAX_SCHEDULES_PER_ORG: usize = 50;
const RUNS_PAGE_LIMIT: i64 = 100;
const SSE_POLL_INTERVAL_MS: u64 = 1000;
const SSE_MAX_TICKS: u64 = 3600;

#[derive(Debug, Deserialize)]
pub struct CreateQuery {
    pub bot_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct ListRunsQuery {
    #[serde(default)]
    pub schedule_id: Option<Uuid>,
    #[serde(default)]
    pub status: Option<String>,
}

/// Owner identity from the server-minted JWT `sub` claim; callers reaching
/// this handler without a UUID subject (middleware-enforced upstream) are
/// attributed to the nil sentinel rather than rejected.
fn resolve_owner(headers: &HeaderMap) -> Uuid {
    tenant::user_id_from_claims(headers)
        .and_then(|sub| Uuid::parse_str(&sub).ok())
        .unwrap_or_else(Uuid::nil)
}

fn db_err(context: &str, e: impl std::fmt::Display) -> (StatusCode, String) {
    tracing::error!("automation api {context}: {e}");
    (StatusCode::INTERNAL_SERVER_ERROR, "Internal storage error".to_string())
}

pub async fn list_schedules(
    State(state): State<Arc<AutomationService>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = state.pool().get().map_err(|e| db_err("list schedules", e))?;
    let org = tenant::org_from_claims(&headers);
    let items = match org {
        Some(org_id) => agent_schedules::dsl::agent_schedules
            .filter(agent_schedules::dsl::org_id.eq(org_id))
            .order(agent_schedules::dsl::created_at.desc())
            .load::<AgentSchedule>(&mut conn)
            .map_err(|e| db_err("query schedules", e))?,
        None => agent_schedules::dsl::agent_schedules
            .order(agent_schedules::dsl::created_at.desc())
            .limit(RUNS_PAGE_LIMIT)
            .load::<AgentSchedule>(&mut conn)
            .map_err(|e| db_err("query schedules", e))?,
    };
    Ok(Json(json!({ "schedules": items })))
}

pub async fn create_schedule(
    State(state): State<Arc<AutomationService>>,
    headers: HeaderMap,
    Query(query): Query<CreateQuery>,
    Json(body): Json<ScheduleCreateBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    if body.title.trim().is_empty() || body.goal.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Title and goal are required".to_string()));
    }
    let cron = CronExpr::parse(&body.cron_expr)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid cron expression: {e}")))?;

    let mut conn = state.pool().get().map_err(|e| db_err("create schedule", e))?;

    let org = tenant::org_from_claims(&headers);
    let owner = resolve_owner(&headers);
    let existing = match org {
        Some(org_id) => agent_schedules::dsl::agent_schedules
            .filter(agent_schedules::dsl::org_id.eq(org_id))
            .count()
            .get_result::<i64>(&mut conn)
            .map_err(|e| db_err("quota check", e))?,
        None => agent_schedules::dsl::agent_schedules
            .filter(agent_schedules::dsl::owner_user_id.eq(owner))
            .count()
            .get_result::<i64>(&mut conn)
            .map_err(|e| db_err("quota check", e))?,
    };
    if existing as usize >= MAX_SCHEDULES_PER_ORG {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            format!("Schedule quota of {MAX_SCHEDULES_PER_ORG} reached"),
        ));
    }

    let now = chrono::Utc::now();
    let prefs = body.delivery.unwrap_or_default();
    let delivery_json = serde_json::to_value(&prefs)
        .unwrap_or_else(|_| json!({ "email": true, "sms": false, "channels": [] }));
    let max_runtime = body.max_runtime_secs.unwrap_or(900).clamp(30, 86400);
    let new_schedule = NewAgentSchedule {
        id: Uuid::new_v4(),
        org_id: org,
        branch_id: tenant::branch_from_claims(&headers),
        bot_id: query.bot_id,
        title: body.title.trim().to_string(),
        goal: body.goal.clone(),
        cron_expr: cron.as_str().to_string(),
        timezone: body.timezone.unwrap_or_else(|| "UTC".to_string()),
        owner_user_id: owner,
        delivery: delivery_json,
        enabled: true,
        max_runtime_secs: max_runtime,
        tool_allowlist: body.tool_allowlist.map(|list| json!(list)),
        next_run_at: cron.next_after(now),
    };
    diesel::insert_into(agent_schedules::dsl::agent_schedules)
        .values(&new_schedule)
        .execute(&mut conn)
        .map_err(|e| db_err("insert schedule", e))?;
    drop(conn);

    let saved = load_schedule(&state, new_schedule.id)?;
    Ok(Json(json!({ "schedule": saved })))
}

fn load_schedule(state: &AutomationService, id: Uuid) -> Result<AgentSchedule, (StatusCode, String)> {
    let mut conn = state.pool().get().map_err(|e| db_err("load schedule", e))?;
    agent_schedules::dsl::agent_schedules
        .find(id)
        .first::<AgentSchedule>(&mut conn)
        .optional()
        .map_err(|e| db_err("load schedule", e))?
        .ok_or((StatusCode::NOT_FOUND, "Schedule not found".to_string()))
}

pub async fn update_schedule(
    State(state): State<Arc<AutomationService>>,
    headers: HeaderMap,
    Path(schedule_id): Path<Uuid>,
    Json(body): Json<ScheduleUpdateBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let schedule = load_schedule(&state, schedule_id)?;
    if let Some(org) = tenant::org_from_claims(&headers) {
        if schedule.org_id.is_some_and(|o| o != org) {
            return Err((StatusCode::FORBIDDEN, "Not authorized".to_string()));
        }
    }
    let mut conn = state.pool().get().map_err(|e| db_err("update schedule", e))?;
    use agent_schedules::dsl::*;

    macro_rules! set_field {
        ($col:ident, $value:expr) => {
            diesel::update(agent_schedules.find(schedule_id))
                .set($col.eq($value))
                .execute(&mut conn)
                .map_err(|e| db_err("update schedule", e))?;
        };
    }
    if let Some(v) = body.title.as_deref() {
        set_field!(title, v.to_string());
    }
    if let Some(v) = &body.goal {
        set_field!(goal, v.clone());
    }
    let cron_changed = if let Some(ref expr) = body.cron_expr {
        CronExpr::parse(expr).map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid cron expression: {e}")))?;
        set_field!(cron_expr, expr.clone());
        true
    } else {
        false
    };
    if let Some(v) = &body.timezone {
        set_field!(timezone, v.clone());
    }
    if let Some(v) = body.enabled {
        set_field!(enabled, v);
    }
    if let Some(prefs) = &body.delivery {
        let value = serde_json::to_value(prefs).unwrap_or_else(|_| json!({}));
        set_field!(delivery, value);
    }
    if let Some(v) = body.max_runtime_secs {
        set_field!(max_runtime_secs, v.clamp(30, 86400));
    }
    if let Some(list) = &body.tool_allowlist {
        set_field!(tool_allowlist, Some(json!(list)));
    }
    if cron_changed || body.enabled.is_some() {
        let current = agent_schedules
            .find(schedule_id)
            .select((cron_expr, enabled))
            .first::<(String, bool)>(&mut conn)
            .map_err(|e| db_err("reload for next_run_at", e))?;
        let (current_expr, schedule_enabled) = current;
        let next = CronExpr::parse(&current_expr)
            .ok()
            .and_then(|c| c.next_after(chrono::Utc::now()))
            .filter(|_| schedule_enabled);
        set_field!(next_run_at, next);
    }
    set_field!(updated_at, chrono::Utc::now());
    drop(conn);

    let saved = load_schedule(&state, schedule_id)?;
    Ok(Json(json!({ "schedule": saved })))
}

pub async fn delete_schedule(
    State(state): State<Arc<AutomationService>>,
    Path(schedule_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = state.pool().get().map_err(|e| db_err("delete schedule", e))?;
    let deleted = diesel::delete(agent_schedules::dsl::agent_schedules.find(schedule_id))
        .execute(&mut conn)
        .map_err(|e| db_err("delete schedule", e))?;
    if deleted == 0 {
        return Err((StatusCode::NOT_FOUND, "Schedule not found".to_string()));
    }
    Ok(Json(json!({ "status": "deleted" })))
}

pub async fn trigger_run(
    State(state): State<Arc<AutomationService>>,
    Path(schedule_id): Path<Uuid>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {
    let schedule = load_schedule(&state, schedule_id)?;
    let mut conn = state.pool().get().map_err(|e| db_err("trigger run", e))?;
    let new_run = NewAgentRun {
        id: Uuid::new_v4(),
        schedule_id: Some(schedule.id),
        bot_id: schedule.bot_id,
        trigger_kind: "manual".to_string(),
        status: engine::STATUS_QUEUED.to_string(),
    };
    let run_id = new_run.id;
    diesel::insert_into(agent_runs::dsl::agent_runs)
        .values(&new_run)
        .execute(&mut conn)
        .map_err(|e| db_err("insert run", e))?;
    drop(conn);

    tokio::spawn(engine::execute_run(state.clone(), run_id));
    Ok((StatusCode::ACCEPTED, Json(json!({ "run_id": run_id, "status": "queued" }))))
}

pub async fn list_runs(
    State(state): State<Arc<AutomationService>>,
    Query(query): Query<ListRunsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = state.pool().get().map_err(|e| db_err("list runs", e))?;
    let mut boxed = agent_runs::dsl::agent_runs
        .order(agent_runs::dsl::created_at.desc())
        .limit(RUNS_PAGE_LIMIT)
        .into_boxed();
    if let Some(sid) = query.schedule_id {
        boxed = boxed.filter(agent_runs::dsl::schedule_id.eq(sid));
    }
    if let Some(st) = query.status {
        boxed = boxed.filter(agent_runs::dsl::status.eq(st));
    }
    let runs = boxed
        .load::<AgentRun>(&mut conn)
        .map_err(|e| db_err("query runs", e))?;
    Ok(Json(json!({ "runs": runs })))
}

pub async fn cancel_run(
    State(state): State<Arc<AutomationService>>,
    Path(run_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let existing = load_run_row(&state, run_id)?;
    if is_terminal(&existing.status) {
        return Ok(Json(json!({ "status": "already_finished", "run_status": existing.status })));
    }
    let flag = state.cancel_flag(run_id);
    flag.store(true, std::sync::atomic::Ordering::Relaxed);
    // A queued run has no engine task polling the flag yet; cancel directly.
    let mut conn = state.pool().get().map_err(|e| db_err("cancel run", e))?;
    diesel::update(agent_runs::dsl::agent_runs.find(run_id))
        .filter(agent_runs::dsl::status.eq(engine::STATUS_QUEUED))
        .set((
            agent_runs::dsl::status.eq(engine::STATUS_CANCELLED),
            agent_runs::dsl::finished_at.eq(diesel::dsl::now),
        ))
        .execute(&mut conn)
        .map_err(|e| db_err("cancel run", e))?;
    Ok(Json(json!({ "status": "cancel_requested" })))
}

fn load_run_row(
    state: &AutomationService,
    run_id: Uuid,
) -> Result<AgentRun, (StatusCode, String)> {
    let mut conn = state.pool().get().map_err(|e| db_err("load run", e))?;
    agent_runs::dsl::agent_runs
        .find(run_id)
        .first::<AgentRun>(&mut conn)
        .optional()
        .map_err(|e| db_err("load run", e))?
        .ok_or((StatusCode::NOT_FOUND, "Run not found".to_string()))
}

fn is_terminal(status: &str) -> bool {
    matches!(
        status,
        engine::STATUS_COMPLETED
            | engine::STATUS_FAILED
            | engine::STATUS_CANCELLED
            | engine::STATUS_TIMEOUT
    )
}

struct SseTick {
    spans: Vec<crate::models::AgentSpan>,
    total_spans: i64,
    run: Option<AgentRun>,
}

fn poll_tick(state: &AutomationService, run_id: Uuid, emitted: usize) -> Option<SseTick> {
    use agent_runs::dsl as r;
    use agent_spans::dsl as s;
    let mut conn = match state.pool().get() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("automation sse {run_id}: db pool unavailable: {e}");
            return None;
        }
    };
    let spans = match s::agent_spans
        .filter(s::run_id.eq(run_id))
        .order(s::started_at.asc())
        .offset(emitted as i64)
        .limit(RUNS_PAGE_LIMIT)
        .load::<crate::models::AgentSpan>(&mut conn)
    {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("automation sse {run_id}: span poll failed: {e}");
            return None;
        }
    };
    let total = match s::agent_spans
        .filter(s::run_id.eq(run_id))
        .count()
        .get_result::<i64>(&mut conn)
    {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("automation sse {run_id}: span count failed: {e}");
            return None;
        }
    };
    let run = match r::agent_runs.find(run_id).first::<AgentRun>(&mut conn).optional() {
        Ok(v) => v.flatten(),
        Err(e) => {
            tracing::error!("automation sse {run_id}: run poll failed: {e}");
            return None;
        }
    };
    Some(SseTick { spans, total_spans: total, run })
}

/// `GET /api/automations/runs/:id/events` — Server-Sent Events stream that
/// polls the database every second, emitting newly persisted spans and a
/// terminal run-status event before closing.
/// SSE auth: EventSource cannot set headers, so the suite passes the
/// Bearer token as `?token=`. Either transport must yield a subject.
fn sse_authorized(headers: &HeaderMap, token_param: &str) -> bool {
    let header_token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap_or_default();
    let candidate = if header_token.is_empty() { token_param } else { header_token };
    if candidate.is_empty() {
        return false;
    }
    tenant::user_id_from_claims_subject(candidate).is_some()
}

pub async fn run_events(
    State(state): State<Arc<AutomationService>>,
    Path(run_id): Path<Uuid>,
    Query(query): Query<std::collections::HashMap<String, String>>,
    headers: HeaderMap,
) -> Result<Sse<impl futures::Stream<Item = Result<Event, Infallible>>>, (StatusCode, String)> {
    if !sse_authorized(&headers, query.get("token").map(|s| s.as_str()).unwrap_or_default()) {
        return Err((StatusCode::UNAUTHORIZED, "Authentication required".to_string()));
    }
    let stream = async_stream::stream! {
        let mut emitted: usize = 0;
        for _ in 0..SSE_MAX_TICKS {
            if let Some(tick_data) = poll_tick(&state, run_id, emitted) {
                for span in tick_data.spans {
                    if let Ok(data) = serde_json::to_string(&span) {
                        yield Ok(Event::default().event("span").data(data));
                    }
                    emitted += 1;
                }
                if let Some(run) = tick_data.run {
                    if is_terminal(&run.status) && emitted as i64 >= tick_data.total_spans {
                        if let Ok(data) = serde_json::to_string(&run) {
                            yield Ok(Event::default().event("run_status").data(data));
                        }
                        break;
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(SSE_POLL_INTERVAL_MS)).await;
        }
    };
    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

/// Router fragment merged by the integrator under the authenticated scope.
pub fn configure_routes() -> axum::Router<Arc<AutomationService>> {
    use axum::routing::{delete, get, post, put};
    axum::Router::new()
        .route(
            "/api/automations/schedules",
            get(list_schedules).post(create_schedule),
        )
        .route(
            "/api/automations/schedules/:id",
            put(update_schedule).delete(delete_schedule),
        )
        .route("/api/automations/schedules/:id/run", post(trigger_run))
        .route("/api/automations/runs", get(list_runs))
        .route("/api/automations/runs/:id/cancel", post(cancel_run))
        .route("/api/automations/runs/:id/events", get(run_events))
}
