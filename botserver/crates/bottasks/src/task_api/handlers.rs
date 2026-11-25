use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Json, Response},
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::state::TasksState;
use crate::task_api::engine::{TaskEngine, TASK_NOT_FOUND};
use crate::task_api::engine_persist::EnginePersistence;
use crate::task_api::html_renderers_cards::render_cards;
use crate::task_api::html_renderers_terminal::render_terminal;
use crate::task_api::utils::get_user_id_from_headers;
use crate::types::{CreateTaskRequest, TaskManifest, UpdateTaskRequest};
use crate::scheduler_exec;

#[derive(Deserialize)]
pub struct ListQuery {
    pub branch_id: Option<String>,
}

/// Resolves the tenant branch. The server-minted `branch_id` JWT claim wins
/// (issue #734); unknown/unauthenticated callers fall back to the configured
/// default branch so legacy calls still work — every mutation is additionally
/// constrained by the branch, so a tenant can never touch another tenant's
/// task.
fn resolve_branch(state: &Arc<TasksState>, headers: &HeaderMap) -> Uuid {
    botsecurity_core::tenant::branch_from_claims(headers)
        .or_else(|| {
            (state.get_config)("default_branch_id")
                .ok()
                .and_then(|id| id.parse::<Uuid>().ok())
        })
        .unwrap_or_else(Uuid::nil)
}

/// A task id that does not resolve inside the caller's verified branch is a
/// 404 — it must never fall through to a manifest read or write (issue #2822).
fn task_not_found_response() -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({"error": TASK_NOT_FOUND})),
    )
        .into_response()
}

/// Maps the branch-scoped engine's not-found sentinel to 404, leaving every
/// other failure on the established JSON error path.
fn engine_error_response(error: String) -> Response {
    if error == TASK_NOT_FOUND {
        task_not_found_response()
    } else {
        Json(serde_json::json!({"error": error})).into_response()
    }
}

pub async fn handle_list_tasks(
    State(state): State<Arc<TasksState>>,
    _query: Query<ListQuery>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let _session = match get_user_id_from_headers(&state, &headers) {
        Ok(id) => id,
        Err(e) => return Json(serde_json::json!({"error": e})).into_response(),
    };

    let engine = TaskEngine::new(state.clone());
    // Tenant scope comes from the authenticated token; the client-supplied
    // branch_id query param is ignored (issue #734) so a tenant can never
    // list another tenant's tasks by spoofing it.
    let branch_id = resolve_branch(&state, &headers);

    match engine.list_tasks(Some(branch_id)) {
        Ok(tasks) => Json(serde_json::json!({"tasks": tasks})).into_response(),
        Err(e) => Json(serde_json::json!({"error": e})).into_response(),
    }
}

pub async fn handle_create_task(
    State(state): State<Arc<TasksState>>,
    headers: HeaderMap,
    Json(payload): Json<CreateTaskRequest>,
) -> impl IntoResponse {
    let branch_id = resolve_branch(&state, &headers);

    let bot_id = match (state.get_config)("default_bot_id") {
        Ok(id) => id.parse::<Uuid>().unwrap_or_else(|_| Uuid::nil()),
        Err(_) => Uuid::nil(),
    };

    let engine = TaskEngine::new(state.clone());

    match engine.create_task(branch_id, &payload.title, payload.description.as_deref()) {
        Ok(task) => {
            let manifest = TaskManifest::new(&payload.title);
            let persistence = EnginePersistence::new(state.clone());
            if let Err(e) = persistence.save_manifest(task.id, &manifest).await {
                log::error!("Failed to save manifest: {}", e);
            }

            if let Some(ref schedule) = payload.schedule {
                if let Err(e) = crate::scheduler::create_auto_task(
                    &state,
                    branch_id,
                    bot_id,
                    &payload.title,
                    Some(schedule),
                ) {
                    log::error!("Failed to create auto task: {}", e);
                }
            }

            Json(serde_json::json!({"task": task})).into_response()
        }
        Err(e) => Json(serde_json::json!({"error": e})).into_response(),
    }
}

pub async fn handle_get_task(
    State(state): State<Arc<TasksState>>,
    headers: HeaderMap,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    let engine = TaskEngine::new(state.clone());
    let branch_id = resolve_branch(&state, &headers);
    let id = match task_id.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => return Json(serde_json::json!({"error": "Invalid task ID"})).into_response(),
    };

    match engine.get_task(id, branch_id) {
        Ok(task) => Json(serde_json::json!({"task": task})).into_response(),
        Err(e) => Json(serde_json::json!({"error": e})).into_response(),
    }
}

pub async fn handle_execute_task(
    State(state): State<Arc<TasksState>>,
    headers: HeaderMap,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    let id = match task_id.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => return Json(serde_json::json!({"error": "Invalid task ID"})).into_response(),
    };
    let branch_id = resolve_branch(&state, &headers);

    let engine = TaskEngine::new(state.clone());
    // Defense in depth: prove the task belongs to the caller's verified branch
    // before touching the shared manifest cache. `resolve_branch` ignores
    // client-supplied org/branch, so a foreign id is refused here (#2822).
    if engine.get_task(id, branch_id).is_err() {
        return task_not_found_response();
    }
    if let Err(e) = engine.update_task_status(id, branch_id, "running") {
        return engine_error_response(e);
    }

    let persistence = EnginePersistence::new(state.clone());
    let mut manifest = persistence
        .load_manifest(id)
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| TaskManifest::new("Auto-generated task"));

    match scheduler_exec::execute_task(&state, id, &mut manifest).await {
        Ok(()) => {
            if let Err(e) = engine.update_task_status(id, branch_id, &manifest.status) {
                log::warn!("Failed to update task status after execution: {e}");
            }
            if let Err(e) = persistence.save_manifest(id, &manifest).await {
                log::warn!("Failed to save task manifest after execution: {e}");
            }
            Json(serde_json::json!({"status": "completed", "manifest": manifest})).into_response()
        }
        Err(e) => {
            if let Err(e) = engine.update_task_status(id, branch_id, "failed") {
                log::warn!("Failed to mark task as failed: {e}");
            }
            if let Err(e) = persistence.save_manifest(id, &manifest).await {
                log::warn!("Failed to save failed-task manifest: {e}");
            }
            Json(serde_json::json!({"error": e})).into_response()
        }
    }
}

pub async fn handle_cancel_task(
    State(state): State<Arc<TasksState>>,
    headers: HeaderMap,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    let id = match task_id.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => return Json(serde_json::json!({"error": "Invalid task ID"})).into_response(),
    };
    let branch_id = resolve_branch(&state, &headers);

    let engine = TaskEngine::new(state.clone());
    // Defense in depth: same ownership check as execute — never load, mutate
    // or save a manifest for a task outside the caller's branch (#2822).
    if engine.get_task(id, branch_id).is_err() {
        return task_not_found_response();
    }
    if let Err(e) = engine.update_task_status(id, branch_id, "cancelled") {
        return engine_error_response(e);
    }

    let persistence = EnginePersistence::new(state.clone());
    if let Ok(Some(mut manifest)) = persistence.load_manifest(id).await {
        if let Err(e) = scheduler_exec::cancel_task(&state, id, &mut manifest).await {
            log::warn!("Failed to cancel running task {id}: {e}");
        }
        if let Err(e) = persistence.save_manifest(id, &manifest).await {
            log::warn!("Failed to save manifest for cancelled task {id}: {e}");
        }
    }

    Json(serde_json::json!({"status": "cancelled"})).into_response()
}

pub async fn handle_update_task(
    State(state): State<Arc<TasksState>>,
    headers: HeaderMap,
    Path(task_id): Path<String>,
    Json(payload): Json<UpdateTaskRequest>,
) -> impl IntoResponse {
    let id = match task_id.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => return Json(serde_json::json!({"error": "Invalid task ID"})).into_response(),
    };

    let branch_id = resolve_branch(&state, &headers);
    let engine = TaskEngine::new(state.clone());

    match engine.update_task(id, branch_id, &payload) {
        Ok(task) => Json(serde_json::json!({"task": task})).into_response(),
        Err(e) => Json(serde_json::json!({"error": e})).into_response(),
    }
}

pub async fn handle_complete_task(
    State(state): State<Arc<TasksState>>,
    headers: HeaderMap,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    let id = match task_id.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => return Json(serde_json::json!({"error": "Invalid task ID"})).into_response(),
    };

    let branch_id = resolve_branch(&state, &headers);
    let engine = TaskEngine::new(state.clone());

    match engine.complete_task(id, branch_id) {
        Ok(task) => Json(serde_json::json!({"task": task})).into_response(),
        Err(e) => Json(serde_json::json!({"error": e})).into_response(),
    }
}

pub async fn handle_reopen_task(
    State(state): State<Arc<TasksState>>,
    headers: HeaderMap,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    let id = match task_id.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => return Json(serde_json::json!({"error": "Invalid task ID"})).into_response(),
    };

    let branch_id = resolve_branch(&state, &headers);
    let engine = TaskEngine::new(state.clone());

    match engine.reopen_task(id, branch_id) {
        Ok(task) => Json(serde_json::json!({"task": task})).into_response(),
        Err(e) => Json(serde_json::json!({"error": e})).into_response(),
    }
}

pub async fn handle_delete_task(
    State(state): State<Arc<TasksState>>,
    headers: HeaderMap,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    let id = match task_id.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => return Json(serde_json::json!({"error": "Invalid task ID"})).into_response(),
    };

    let branch_id = resolve_branch(&state, &headers);
    let engine = TaskEngine::new(state.clone());

    match engine.delete_task(id, branch_id) {
        Ok(()) => Json(serde_json::json!({"status": "deleted"})).into_response(),
        Err(e) => Json(serde_json::json!({"error": e})).into_response(),
    }
}

pub async fn handle_get_manifest(
    State(state): State<Arc<TasksState>>,
    headers: HeaderMap,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    let id = match task_id.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => return Json(serde_json::json!({"error": "Invalid task ID"})).into_response(),
    };

    let branch_id = resolve_branch(&state, &headers);
    let engine = TaskEngine::new(state.clone());
    if engine.get_task(id, branch_id).is_err() {
        return Json(serde_json::json!({"error": "Task not found"})).into_response();
    }

    let persistence = EnginePersistence::new(state.clone());
    match persistence.load_manifest(id).await {
        Ok(Some(manifest)) => Json(serde_json::json!({"manifest": manifest})).into_response(),
        Ok(None) => Json(serde_json::json!({"error": "Manifest not found"})).into_response(),
        Err(e) => Json(serde_json::json!({"error": e})).into_response(),
    }
}

pub async fn handle_get_cards(
    State(state): State<Arc<TasksState>>,
    headers: HeaderMap,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    let id = match task_id.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => return Html("<div class='error'>Invalid task ID</div>".to_string()).into_response(),
    };

    let branch_id = resolve_branch(&state, &headers);
    let engine = TaskEngine::new(state.clone());
    if engine.get_task(id, branch_id).is_err() {
        return Html("<div class='error'>Task not found</div>".to_string()).into_response();
    }

    let persistence = EnginePersistence::new(state.clone());
    match persistence.load_manifest(id).await {
        Ok(Some(manifest)) => Html(render_cards(&manifest)).into_response(),
        Ok(None) => Html("<div class='error'>Manifest not found</div>".to_string()).into_response(),
        Err(e) => Html(format!("<div class='error'>{}</div>", html_escape(&e))).into_response(),
    }
}

pub async fn handle_get_terminal(
    State(state): State<Arc<TasksState>>,
    headers: HeaderMap,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    let id = match task_id.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => return Html("<div class='error'>Invalid task ID</div>".to_string()).into_response(),
    };

    let branch_id = resolve_branch(&state, &headers);
    let engine = TaskEngine::new(state.clone());
    if engine.get_task(id, branch_id).is_err() {
        return Html("<div class='error'>Task not found</div>".to_string()).into_response();
    }

    let persistence = EnginePersistence::new(state.clone());
    match persistence.load_manifest(id).await {
        Ok(Some(manifest)) => Html(render_terminal(&manifest)).into_response(),
        Ok(None) => Html("<div class='error'>Manifest not found</div>".to_string()).into_response(),
        Err(e) => Html(format!("<div class='error'>{}</div>", html_escape(&e))).into_response(),
    }
}

pub async fn handle_get_log(
    State(state): State<Arc<TasksState>>,
    headers: HeaderMap,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    let id = match task_id.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => return Json(serde_json::json!({"error": "Invalid task ID"})).into_response(),
    };

    let branch_id = resolve_branch(&state, &headers);
    let engine = TaskEngine::new(state.clone());
    if engine.get_task(id, branch_id).is_err() {
        return Json(serde_json::json!({"error": "Task not found"})).into_response();
    }

    let persistence = EnginePersistence::new(state.clone());
    match persistence.load_manifest(id).await {
        Ok(Some(manifest)) => {
            let log_entries: Vec<serde_json::Value> = manifest
                .terminal_output
                .iter()
                .map(|line| {
                    serde_json::json!({
                        "text": line.text,
                        "type": line.line_type
                    })
                })
                .collect();
            Json(serde_json::json!({"log": log_entries})).into_response()
        }
        Ok(None) => Json(serde_json::json!({"error": "Manifest not found"})).into_response(),
        Err(e) => Json(serde_json::json!({"error": e})).into_response(),
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
