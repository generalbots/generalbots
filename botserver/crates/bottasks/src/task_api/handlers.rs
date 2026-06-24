use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    response::{Html, IntoResponse, Json},
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::state::TasksState;
use crate::task_api::engine::TaskEngine;
use crate::task_api::engine_persist::EnginePersistence;
use crate::task_api::html_renderers_cards::render_cards;
use crate::task_api::html_renderers_terminal::render_terminal;
use crate::task_api::utils::get_user_id_from_headers;
use crate::types::{CreateTaskRequest, TaskManifest};
use crate::scheduler_exec;

#[derive(Deserialize)]
pub struct ListQuery {
    pub bot_id: Option<String>,
}

pub async fn handle_list_tasks(
    State(state): State<Arc<TasksState>>,
    Query(query): Query<ListQuery>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let _user_id = match get_user_id_from_headers(&state, &headers) {
        Ok(id) => id,
        Err(e) => return Json(serde_json::json!({"error": e})).into_response(),
    };

    let engine = TaskEngine::new(state.clone());
    let bot_id = query.bot_id.and_then(|s| s.parse::<Uuid>().ok());

    match engine.list_tasks(bot_id) {
        Ok(tasks) => Json(serde_json::json!({"tasks": tasks})).into_response(),
        Err(e) => Json(serde_json::json!({"error": e})).into_response(),
    }
}

pub async fn handle_create_task(
    State(state): State<Arc<TasksState>>,
    headers: HeaderMap,
    Json(payload): Json<CreateTaskRequest>,
) -> impl IntoResponse {
    let user_id = match get_user_id_from_headers(&state, &headers) {
        Ok(id) => id,
        Err(e) => return Json(serde_json::json!({"error": e})).into_response(),
    };

    let bot_id = match (state.get_config)("default_bot_id") {
        Ok(id) => id.parse::<Uuid>().unwrap_or_default(),
        Err(_) => Uuid::nil(),
    };

    let engine = TaskEngine::new(state.clone());

    match engine.create_task(bot_id, &payload.title, payload.description.as_deref(), Some(user_id)) {
        Ok(task) => {
            let manifest = TaskManifest::new(&payload.title);
            let persistence = EnginePersistence::new(state.clone());
            if let Err(e) = persistence.save_manifest(task.id, &manifest).await {
                log::error!("Failed to save manifest: {}", e);
            }

            if let Some(ref schedule) = payload.schedule {
                if let Err(e) = crate::scheduler::create_auto_task(
                    &state,
                    bot_id,
                    &payload.title,
                    payload.description.as_deref(),
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
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    let engine = TaskEngine::new(state.clone());
    let id = match task_id.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => return Json(serde_json::json!({"error": "Invalid task ID"})).into_response(),
    };

    match engine.get_task(id) {
        Ok(task) => Json(serde_json::json!({"task": task})).into_response(),
        Err(e) => Json(serde_json::json!({"error": e})).into_response(),
    }
}

pub async fn handle_execute_task(
    State(state): State<Arc<TasksState>>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    let id = match task_id.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => return Json(serde_json::json!({"error": "Invalid task ID"})).into_response(),
    };

    let engine = TaskEngine::new(state.clone());
    if let Err(e) = engine.update_task_status(id, "running") {
        return Json(serde_json::json!({"error": e})).into_response();
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
            let _ = engine.update_task_status(id, &manifest.status);
            let _ = persistence.save_manifest(id, &manifest).await;
            Json(serde_json::json!({"status": "completed", "manifest": manifest})).into_response()
        }
        Err(e) => {
            let _ = engine.update_task_status(id, "failed");
            let _ = persistence.save_manifest(id, &manifest).await;
            Json(serde_json::json!({"error": e})).into_response()
        }
    }
}

pub async fn handle_cancel_task(
    State(state): State<Arc<TasksState>>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    let id = match task_id.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => return Json(serde_json::json!({"error": "Invalid task ID"})).into_response(),
    };

    let engine = TaskEngine::new(state.clone());
    if let Err(e) = engine.update_task_status(id, "cancelled") {
        return Json(serde_json::json!({"error": e})).into_response();
    }

    let persistence = EnginePersistence::new(state.clone());
    if let Ok(Some(mut manifest)) = persistence.load_manifest(id).await {
        let _ = scheduler_exec::cancel_task(&state, id, &mut manifest).await;
        let _ = persistence.save_manifest(id, &manifest).await;
    }

    Json(serde_json::json!({"status": "cancelled"})).into_response()
}

pub async fn handle_get_manifest(
    State(state): State<Arc<TasksState>>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    let id = match task_id.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => return Json(serde_json::json!({"error": "Invalid task ID"})).into_response(),
    };

    let persistence = EnginePersistence::new(state.clone());
    match persistence.load_manifest(id).await {
        Ok(Some(manifest)) => Json(serde_json::json!({"manifest": manifest})).into_response(),
        Ok(None) => Json(serde_json::json!({"error": "Manifest not found"})).into_response(),
        Err(e) => Json(serde_json::json!({"error": e})).into_response(),
    }
}

pub async fn handle_get_cards(
    State(state): State<Arc<TasksState>>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    let id = match task_id.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => return Html("<div class='error'>Invalid task ID</div>".to_string()).into_response(),
    };

    let persistence = EnginePersistence::new(state.clone());
    match persistence.load_manifest(id).await {
        Ok(Some(manifest)) => Html(render_cards(&manifest)).into_response(),
        Ok(None) => Html("<div class='error'>Manifest not found</div>".to_string()).into_response(),
        Err(e) => Html(format!("<div class='error'>{}</div>", html_escape(&e))).into_response(),
    }
}

pub async fn handle_get_terminal(
    State(state): State<Arc<TasksState>>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    let id = match task_id.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => return Html("<div class='error'>Invalid task ID</div>".to_string()).into_response(),
    };

    let persistence = EnginePersistence::new(state.clone());
    match persistence.load_manifest(id).await {
        Ok(Some(manifest)) => Html(render_terminal(&manifest)).into_response(),
        Ok(None) => Html("<div class='error'>Manifest not found</div>".to_string()).into_response(),
        Err(e) => Html(format!("<div class='error'>{}</div>", html_escape(&e))).into_response(),
    }
}

pub async fn handle_get_log(
    State(state): State<Arc<TasksState>>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    let id = match task_id.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => return Json(serde_json::json!({"error": "Invalid task ID"})).into_response(),
    };

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
