//! HTTP surface for browser tasks, browsing memory and the admin policy.
//!
//! Routes (BRIEF contract): `POST /api/browser/tasks`,
//! `GET /api/browser/tasks/:id/events` (SSE),
//! `POST /api/browser/tasks/:id/pause|resume|cancel`,
//! `GET /api/browser/memory`. Additions: `GET /api/browser/tasks` (owner
//! listing), `POST /api/browser/tasks/:id/advance` and
//! `POST /api/browser/tasks/:id/finish` for the external execution driver,
//! `GET/PUT /api/browser/policy` (admin-only).

use crate::caller::{resolve_principal, resolve_principal_sse, resolve_user_id};
use crate::jwt::{bearer_token, verified_claims};
use crate::models::ProgressDoc;
use crate::ownership::{ownership_matches, TaskPrincipal};
use crate::policy::{PolicyConfig, TASK_KIND_CHECKOUT, TASK_KIND_DEFAULT};
use crate::tasks::{self, StepResult, TaskError};
use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    response::sse::{Event, KeepAlive, Sse},
    Json,
};
use botsecurity_core::error_sanitizer::{log_and_sanitize_str, SafeErrorResponse};
use botsecurity_core::verified_jwt::VerifiedJwt;
use diesel::prelude::*;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

const SSE_POLL_INTERVAL_MS: u64 = 1000;
const SSE_MAX_TICKS: u64 = 3600;
const POLICY_PREFERENCE_KEY: &str = "browser_policy";

/// Platform-global sentinel scope used to store the admin policy in the
/// existing `user_preferences` table without a dedicated migration.
const GLOBAL_SCOPE_USER: Uuid = Uuid::nil();

pub type SharedService = Arc<crate::BrowserPolicyService>;

/// Maps a task error to the shared sanitised envelope. Storage/driver detail
/// goes through `log_and_sanitize_str` so it is logged in full but never
/// returned; the other arms carry only deliberate, client-facing text.
fn api_error(context: &str, e: &TaskError) -> SafeErrorResponse {
    match e {
        TaskError::NotFound => SafeErrorResponse::not_found("Browser task"),
        TaskError::InvalidState(_) | TaskError::Policy(_) | TaskError::Budget(_) => {
            tracing::error!("browser policy {context}: {e}");
            SafeErrorResponse::conflict("Task state rejected")
        }
        TaskError::Db(_) | TaskError::Memory(_) => {
            log_and_sanitize_str(&e.to_string(), context, None)
        }
    }
}

fn unauthorized() -> SafeErrorResponse {
    SafeErrorResponse::unauthorized()
}

/// #2882 — derives task scope only from verified claims. `branch_id` is the
/// signed per-bot workspace (`bot_id`); `org_id` is the signed `.gborg` tenant.
fn resolve_task_scope(claims: &VerifiedJwt) -> (Option<Uuid>, Option<Uuid>) {
    (claims.org_id, claims.branch_id)
}

/// Verified email claim of the caller, used by the admin policy gate.
///
/// `Ok(None)` = verified token without an `email` claim (never an admin);
/// `Err(401)` = the token cannot be verified, so the gate fails closed.
fn verified_email(headers: &HeaderMap) -> Result<Option<String>, SafeErrorResponse> {
    let token = bearer_token(headers).ok_or_else(unauthorized)?;
    let claims = verified_claims(token).ok_or_else(unauthorized)?;
    Ok(claims.email)
}

fn is_super_admin(svc: &SharedService, headers: &HeaderMap) -> Result<bool, SafeErrorResponse> {
    // #2112 — the admin decision is made on a **signature-verified** email
    // claim. The previous path read `email` straight out of the decoded
    // payload, so a forged `alg:none` token could name `admin@localhost` and
    // rewrite the browsing policy.
    let Some(email) = verified_email(headers)? else {
        return Ok(false);
    };
    // Canonical platform-admin check (#1387) — same root-org membership
    // model as drive tenant isolation and botcloud admin gates.
    let mut conn = svc.pool().get().map_err(|e| {
        log_and_sanitize_str(&e.to_string(), "browser policy admin check: pool unavailable", None)
    })?;
    botsecurity_auth::platform_admin::is_platform_admin_for_email(&mut conn, &email).map_err(|e| {
        log_and_sanitize_str(&e.to_string(), "browser policy admin check query failed", None)
    })
}

/// #2882 — no `org_id`/`bot_id`: scope comes from verified claims, not client input.
#[derive(Debug, Deserialize)]
pub struct CreateTaskBody {
    pub goal: String,
    #[serde(default)]
    pub domains: Vec<String>,
    pub budget_steps: Option<i32>,
    pub policy: Option<PolicyConfig>,
    pub task_kind: Option<String>,
}

pub async fn create_task_handler(
    State(svc): State<SharedService>,
    headers: HeaderMap,
    Json(body): Json<CreateTaskBody>,
) -> Result<Json<serde_json::Value>, SafeErrorResponse> {
    let token = bearer_token(&headers).ok_or_else(unauthorized)?;
    let claims = verified_claims(token).ok_or_else(unauthorized)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| SafeErrorResponse::new("unauthorized", "Invalid token subject"))?;
    let (org_id, bot_id) = resolve_task_scope(&claims);
    if body.goal.trim().is_empty() {
        return Err(SafeErrorResponse::bad_request("Goal must not be empty"));
    }
    let task_kind = match body.task_kind.as_deref() {
        None | Some(TASK_KIND_DEFAULT) => TASK_KIND_DEFAULT,
        Some(TASK_KIND_CHECKOUT) => TASK_KIND_CHECKOUT,
        Some(other) => {
            return Err(SafeErrorResponse::bad_request(format!("Unknown task kind '{other}'")))
        }
    };
    let t = tasks::create_task(
        svc.pool(),
        tasks::NewTaskRequest {
            user_id,
            org_id,
            bot_id,
            goal: body.goal,
            domains: body.domains,
            budget_steps: body.budget_steps.unwrap_or(60),
            policy_cfg: body.policy,
            task_kind,
        },
    )
    .map_err(|e| api_error("create task", &e))?;
    Ok(Json(serde_json::to_value(&t).unwrap_or_else(|_| serde_json::json!({}))))
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    limit: Option<i64>,
}

pub async fn list_tasks_handler(
    State(svc): State<SharedService>,
    headers: HeaderMap,
    Query(q): Query<ListQuery>,
) -> Result<Json<serde_json::Value>, SafeErrorResponse> {
    let principal = resolve_principal(&headers)?;
    let items = tasks::list_scoped_tasks(svc.pool(), &principal, q.limit.unwrap_or(50))
        .map_err(|e| api_error("list tasks", &e))?;
    Ok(Json(serde_json::json!({ "items": items })))
}

async fn control(
    svc: SharedService,
    headers: HeaderMap,
    task_id: Uuid,
    f: fn(&tasks::DbPool, Uuid, &TaskPrincipal) -> Result<crate::models::BrowserTask, TaskError>,
) -> Result<Json<serde_json::Value>, SafeErrorResponse> {
    let principal = resolve_principal(&headers)?;
    // `f` loads through `load_owned_task`: foreign and unknown ids both become
    // 404 before any state change is attempted (#2877).
    let t = f(svc.pool(), task_id, &principal).map_err(|e| api_error("task control", &e))?;
    Ok(Json(serde_json::json!({ "status": t.status })))
}

pub async fn pause_handler(State(svc): State<SharedService>, headers: HeaderMap, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, SafeErrorResponse> {
    control(svc, headers, id, tasks::pause_task).await
}

pub async fn resume_handler(State(svc): State<SharedService>, headers: HeaderMap, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, SafeErrorResponse> {
    control(svc, headers, id, tasks::resume_task).await
}

pub async fn cancel_handler(State(svc): State<SharedService>, headers: HeaderMap, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, SafeErrorResponse> {
    control(svc, headers, id, tasks::cancel_task).await
}

#[derive(Debug, Deserialize)]
pub struct AdvanceBody {
    pub action: String,
    pub url: String,
    #[serde(default = "default_true")]
    pub ok: bool,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub cost_milli: u64,
    pub title: Option<String>,
    pub facts: Option<serde_json::Value>,
}

fn default_true() -> bool {
    true
}

/// Driver-facing endpoint invoking `tasks::advance_task`.
pub async fn advance_handler(
    State(svc): State<SharedService>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(body): Json<AdvanceBody>,
) -> Result<Json<serde_json::Value>, SafeErrorResponse> {
    let principal = resolve_principal(&headers)?;
    let (_, recorded) = tasks::advance_task(
        svc.pool(),
        id,
        principal,
        StepResult {
            action: body.action,
            url: body.url,
            ok: body.ok,
            note: body.note,
            cost_milli: body.cost_milli,
            title: body.title,
            facts: body.facts,
        },
    )
    .await
    .map_err(|e| api_error("advance", &e))?;
    Ok(Json(serde_json::to_value(&recorded).unwrap_or_else(|_| serde_json::json!({}))))
}

#[derive(Debug, Deserialize)]
pub struct FinishBody {
    pub result: serde_json::Value,
    pub citations: Option<serde_json::Value>,
}

pub async fn finish_handler(
    State(svc): State<SharedService>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(body): Json<FinishBody>,
) -> Result<Json<serde_json::Value>, SafeErrorResponse> {
    let principal = resolve_principal(&headers)?;
    let t = tasks::finish_task(svc.pool(), id, &principal, body.result, body.citations)
        .map_err(|e| api_error("finish", &e))?;
    Ok(Json(serde_json::json!({ "status": t.status })))
}

struct SseTick {
    task: crate::models::BrowserTask,
    new_steps: Vec<serde_json::Value>,
}

fn poll_tick(
    svc: &SharedService,
    task_id: Uuid,
    emitted: usize,
    caller: &TaskPrincipal,
) -> Option<SseTick> {
    use crate::schema::browser_tasks::dsl as bt;

    let mut conn = svc.pool().get().ok()?;
    let task = bt::browser_tasks
        .find(task_id)
        .first::<crate::models::BrowserTask>(&mut conn)
        .optional()
        .ok()??;
    // #2877 — re-assert ownership on every poll so the stream never emits a
    // task that the caller is not allowed to see.
    if !ownership_matches(&TaskPrincipal::of_task(&task), caller) {
        return None;
    }
    let progress = ProgressDoc::parse(task.progress.as_ref());
    let new_steps = progress.steps.iter().skip(emitted).filter_map(|s| serde_json::to_value(s).ok()).collect();
    Some(SseTick { task, new_steps })
}

/// `GET /api/browser/tasks/:id/events` — Server-Sent Events stream polling the
/// persisted progress document every second; emits each new step once and a
/// terminal `task_status` event before closing.
pub async fn task_events(
    State(svc): State<SharedService>,
    headers: HeaderMap,
    Path(task_id): Path<Uuid>,
    Query(query): Query<std::collections::HashMap<String, String>>,
) -> Result<Sse<impl futures::Stream<Item = Result<Event, std::convert::Infallible>>>, SafeErrorResponse>
{
    let token_param = match query.get("token") {
        Some(token) => token.as_str(),
        None => "",
    };
    let principal = resolve_principal_sse(&headers, token_param)?;
    // #2877 — authorize the id before opening the stream: a foreign or unknown
    // task yields 404, never an event feed.
    tasks::load_owned_task(svc.pool(), task_id, &principal)
        .map_err(|e| api_error("task events", &e))?;
    let stream = async_stream::stream! {
        let mut emitted: usize = 0;
        for _ in 0..SSE_MAX_TICKS {
            if let Some(tick) = poll_tick(&svc, task_id, emitted, &principal) {
                for step in tick.new_steps {
                    emitted += 1;
                    yield Ok(Event::default().event("step").data(step.to_string()));
                }
                if matches!(tick.task.status.as_str(), tasks::STATUS_FINISHED | tasks::STATUS_FAILED | tasks::STATUS_CANCELLED) {
                    if let Ok(data) = serde_json::to_string(&tick.task) {
                        yield Ok(Event::default().event("task_status").data(data));
                    }
                    break;
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(SSE_POLL_INTERVAL_MS)).await;
        }
    };
    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

#[derive(Debug, Deserialize)]
pub struct MemoryQuery {
    limit: Option<i64>,
}

pub async fn memory_handler(
    State(svc): State<SharedService>,
    headers: HeaderMap,
    Query(q): Query<MemoryQuery>,
) -> Result<Json<serde_json::Value>, SafeErrorResponse> {
    let user_id = resolve_user_id(&headers)?;
    let items = crate::memory::top_facts(svc.pool(), user_id, q.limit.unwrap_or(50))
        .map_err(|e| api_error("memory", &TaskError::Memory(e)))?;
    Ok(Json(serde_json::json!({ "items": items })))
}

pub async fn delete_memory_handler(
    State(svc): State<SharedService>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, SafeErrorResponse> {
    let user_id = resolve_user_id(&headers)?;
    let removed = crate::memory::purge_user(svc.pool(), user_id)
        .map_err(|e| api_error("purge memory", &TaskError::Memory(e)))?;
    Ok(Json(serde_json::json!({ "removed": removed })))
}

fn read_global_policy(svc: &SharedService) -> PolicyConfig {
    use crate::schema::user_preferences::dsl as up;

    let Ok(mut conn) = svc.pool().get() else {
        tracing::warn!("browser policy read: pool unavailable, using defaults");
        return PolicyConfig::default();
    };
    let value = match up::user_preferences
        .filter(up::user_id.eq(GLOBAL_SCOPE_USER))
        .filter(up::preference_key.eq(POLICY_PREFERENCE_KEY))
        .select(up::preference_value)
        .first::<serde_json::Value>(&mut conn)
        .optional()
    {
        Ok(Some(value)) => value,
        Ok(None) => return PolicyConfig::default(),
        Err(e) => {
            tracing::warn!("browser policy read failed, using defaults: {e}");
            return PolicyConfig::default();
        }
    };
    match serde_json::from_value::<PolicyConfig>(value) {
        Ok(cfg) => cfg,
        Err(e) => {
            tracing::warn!("browser policy document invalid, using defaults: {e}");
            PolicyConfig::default()
        }
    }
}

pub async fn get_policy_handler(
    State(svc): State<SharedService>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, SafeErrorResponse> {
    if !is_super_admin(&svc, &headers)? {
        return Err(SafeErrorResponse::new(
            "forbidden",
            "Administrator access required",
        ));
    }
    let cfg = read_global_policy(&svc);
    Ok(Json(serde_json::to_value(&cfg).unwrap_or_else(|_| serde_json::json!({}))))
}

pub async fn put_policy_handler(
    State(svc): State<SharedService>,
    headers: HeaderMap,
    Json(cfg): Json<PolicyConfig>,
) -> Result<Json<serde_json::Value>, SafeErrorResponse> {
    if !is_super_admin(&svc, &headers)? {
        return Err(SafeErrorResponse::new(
            "forbidden",
            "Administrator access required",
        ));
    }
    use crate::schema::user_preferences::dsl as up;

    let value = serde_json::to_value(&cfg)
        .map_err(|_| SafeErrorResponse::bad_request("Invalid policy document"))?;
    let mut conn = svc.pool().get().map_err(|e| {
        log_and_sanitize_str(&e.to_string(), "browser policy persist: pool unavailable", None)
    })?;
    diesel::insert_into(up::user_preferences)
        .values((
            up::id.eq(Uuid::new_v4()),
            up::user_id.eq(GLOBAL_SCOPE_USER),
            up::preference_key.eq(POLICY_PREFERENCE_KEY),
            up::preference_value.eq(value.clone()),
        ))
        .on_conflict((up::user_id, up::preference_key))
        .do_update()
        .set((up::preference_value.eq(value.clone()), up::updated_at.eq(diesel::dsl::now)))
        .execute(&mut conn)
        .map_err(|e| log_and_sanitize_str(&e.to_string(), "browser policy persist failed", None))?;
    Ok(Json(value))
}

/// Router fragment merged by the integrator under the authenticated scope.
pub fn configure_routes() -> axum::Router<SharedService> {
    use axum::routing::{get, post};

    axum::Router::new()
        .route("/api/browser/tasks", get(list_tasks_handler).post(create_task_handler))
        .route("/api/browser/tasks/:id/events", get(task_events))
        .route("/api/browser/tasks/:id/pause", post(pause_handler))
        .route("/api/browser/tasks/:id/resume", post(resume_handler))
        .route("/api/browser/tasks/:id/cancel", post(cancel_handler))
        .route("/api/browser/tasks/:id/advance", post(advance_handler))
        .route("/api/browser/tasks/:id/finish", post(finish_handler))
        .route("/api/browser/memory", get(memory_handler).delete(delete_memory_handler))
        .route("/api/browser/policy", get(get_policy_handler).put(put_policy_handler))
}

#[cfg(test)]
#[path = "api_tests.rs"]
mod api_tests;
