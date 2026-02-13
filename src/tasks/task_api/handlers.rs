//! HTTP handlers for task API
use crate::core::shared::state::AppState;
use crate::tasks::task_api::{html_renderers, utils};
use crate::tasks::types::TaskResponse;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use axum::routing::{delete, get, post, put};
use axum::Router;
use diesel::prelude::*;
use log::{error, info, warn};
use std::sync::Arc;
use uuid::Uuid;

/// Handler for task creation
pub async fn handle_task_create(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<crate::tasks::types::CreateTaskRequest>,
) -> Result<Json<TaskResponse>, StatusCode> {
    let task_engine = &state.task_engine;

    match task_engine.create_task(payload).await {
        Ok(task) => Ok(Json(task)),
        Err(e) => {
            error!("Failed to create task: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Handler for task update
pub async fn handle_task_update(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<crate::tasks::types::TaskUpdate>,
) -> Result<Json<TaskResponse>, StatusCode> {
    let task_engine = &state.task_engine;

    match task_engine.update_task(id, payload).await {
        Ok(task) => Ok(Json(task.into())),
        Err(e) => {
            error!("Failed to update task: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Handler for task deletion
pub async fn handle_task_delete(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let task_engine = &state.task_engine;

    match task_engine.delete_task(id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete task: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Handler for listing all tasks
pub async fn handle_task_list(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let conn = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| {
            error!("[TASK_LIST] DB connection error: {}", e);
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        #[derive(Debug, QueryableByName, serde::Serialize)]
        struct AutoTaskRow {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            pub id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Text)]
            pub title: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            pub status: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            pub priority: String,
            #[diesel(sql_type = diesel::sql_types::Double)]
            pub progress: f64,
        }

        let tasks = diesel::sql_query(
            "SELECT id, title, status, priority, progress FROM auto_tasks ORDER BY created_at DESC"
        )
        .load::<AutoTaskRow>(&mut db_conn)
        .map_err(|e| {
            error!("[TASK_LIST] Query error: {}", e);
            e
        })?;

        Ok::<Vec<AutoTaskRow>, diesel::result::Error>(tasks)
    })
    .await;

    match result {
        Ok(Ok(tasks)) => (StatusCode::OK, axum::Json(tasks)).into_response(),
        Ok(Err(e)) => {
            error!("[TASK_LIST] DB error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        }
        Err(e) => {
            error!("[TASK_LIST] Task join error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
        }
    }
}

/// Handler for getting a single task
pub async fn handle_task_get(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    info!("[TASK_GET] *** Handler called for task: {} ***", id);

    // Check if client wants JSON (for polling) vs HTML (for HTMX)
    let wants_json = headers
        .get(axum::http::header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.contains("application/json"))
        .unwrap_or(false);

    let conn = state.conn.clone();
    let task_id = id.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn
            .get()
            .map_err(|e| {
                error!("[TASK_GET] DB connection error: {}", e);
                format!("DB connection error: {}", e)
            })?;

        #[derive(Debug, QueryableByName, serde::Serialize)]
        struct AutoTaskRow {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            pub id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Text)]
            pub title: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            pub status: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            pub priority: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            pub intent: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            pub error: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Double)]
            pub progress: f64,
            #[diesel(sql_type = diesel::sql_types::Integer)]
            pub current_step: i32,
            #[diesel(sql_type = diesel::sql_types::Integer)]
            pub total_steps: i32,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Jsonb>)]
            pub step_results: Option<serde_json::Value>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Jsonb>)]
            pub manifest_json: Option<serde_json::Value>,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            pub created_at: chrono::DateTime<chrono::Utc>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
            pub started_at: Option<chrono::DateTime<chrono::Utc>>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
            pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
        }

        let parsed_uuid = match Uuid::parse_str(&task_id) {
            Ok(u) => {
                info!("[TASK_GET] Parsed UUID: {}", u);
                u
            }
            Err(e) => {
                error!("[TASK_GET] Invalid task ID '{}': {}", task_id, e);
                return Err(format!("Invalid task ID: {}", task_id));
            }
        };

        let task: Option<AutoTaskRow> = diesel::sql_query(
            "SELECT id, title, status, priority, intent, error, progress, current_step, total_steps, step_results, manifest_json, created_at, started_at, completed_at
             FROM auto_tasks WHERE id = $1 LIMIT 1"
        )
        .bind::<diesel::sql_types::Uuid, _>(parsed_uuid)
        .get_result(&mut db_conn)
        .map_err(|e| {
            error!("[TASK_GET] Query error for {}: {}", parsed_uuid, e);
            e
        })
        .ok();

        info!("[TASK_GET] Query result for {}: found={}", parsed_uuid, task.is_some());
        Ok::<_, String>(task)
    })
    .await
    .unwrap_or_else(|e| {
        error!("Task query failed: {}", e);
        Err(format!("Task query failed: {}", e))
    });

    match result {
        Ok(Some(task)) => {
            info!("[TASK_GET] Returning task: {} - {} (wants_json={})", task.id, task.title, wants_json);

            // Return JSON for API polling clients
            if wants_json {
                return (
                    StatusCode::OK,
                    [(axum::http::header::CONTENT_TYPE, "application/json")],
                    serde_json::json!({
                        "id": task.id.to_string(),
                        "title": task.title,
                        "status": task.status,
                        "priority": task.priority,
                        "intent": task.intent,
                        "error": task.error,
                        "progress": (task.progress * 100.0) as u8,
                        "current_step": task.current_step,
                        "total_steps": task.total_steps,
                        "created_at": task.created_at.to_rfc3339(),
                        "started_at": task.started_at.map(|t| t.to_rfc3339()),
                        "completed_at": task.completed_at.map(|t| t.to_rfc3339())
                    }).to_string()
                ).into_response();
            }

            // Return HTML for HTMX
            let status_class = match task.status.as_str() {
                "completed" | "done" => "completed",
                "running" | "pending" => "running",
                "failed" | "error" => "error",
                _ => "pending"
            };

            let runtime = if let Some(started) = task.started_at {
                let end_time = task.completed_at.unwrap_or_else(chrono::Utc::now);
                let duration = end_time.signed_duration_since(started);
                let mins = duration.num_minutes();
                let secs = duration.num_seconds() % 60;
                if mins > 0 {
                    format!("{}m {}s", mins, secs)
                } else {
                    format!("{}s", secs)
                }
            } else {
                "Not started".to_string()
            };

            let task_id = task.id.to_string();
            let error_html = task.error.clone().map(|e| format!(
                r#"<div class="error-alert">
                    <span class="error-icon">⚠</span>
                    <span class="error-text">{}</span>
                </div>"#, e
            )).unwrap_or_default();

            let status_label = match task.status.as_str() {
                "completed" | "done" => "Completed",
                "running" => "Running",
                "pending" => "Pending",
                "failed" | "error" => "Failed",
                "paused" => "Paused",
                "waiting_approval" => "Awaiting Approval",
                _ => &task.status
            };

            // Build terminal output from recent activity
            let terminal_html = html_renderers::build_terminal_html(&task.step_results, &task.status);

            // Extract app_url from step_results if task is completed
            let app_url = if task.status == "completed" || task.status == "done" {
                utils::extract_app_url_from_results(&task.step_results, &task.title)
            } else {
                None
            };

            let app_button_html = app_url.map(|url| format!(
                r#"<a href="{}" target="_blank" class="btn-action-rich btn-open-app" rel="noopener noreferrer">
                    <span class="btn-icon">🚀</span> Open App
                </a>"#,
                url
            )).unwrap_or_default();

            let cancel_button_html = match task.status.as_str() {
                "completed" | "done" | "failed" | "error" => String::new(),
                _ => format!(
                    r#"<button class="btn-action-rich btn-cancel" onclick="cancelTask('{task_id}')">
                            <span class="btn-icon">✗</span> Cancel
                        </button>"#
                ),
            };

            let (status_html, progress_log_html) = html_renderers::build_taskmd_html(&state, &task_id, &task.title, &runtime, task.manifest_json.as_ref());

            let html = format!(r#"
                <div class="task-detail-rich" data-task-id="{task_id}">
                    <!-- Header - compact -->
                    <div class="taskmd-header">
                        <h1 class="taskmd-title">{title}</h1>
                        <span class="taskmd-status-badge status-{status_class}">{status_label}</span>
                    </div>

                    {error_html}

                    <!-- STATUS Section -->
                    <div class="taskmd-section taskmd-section-status">
                        <div class="taskmd-section-header">STATUS</div>
                        <div class="taskmd-status-content">
                            {status_html}
                        </div>
                    </div>

                    <!-- PROGRESS LOG Section -->
                    <div class="taskmd-section taskmd-section-progress">
                        <div class="taskmd-section-header">PROGRESS LOG</div>
                        <div class="taskmd-progress-content" id="progress-log-{task_id}">
                            {progress_log_html}
                        </div>
                    </div>

                    <!-- TERMINAL Section -->
                    <div class="taskmd-section taskmd-section-terminal taskmd-terminal">
                        <div class="taskmd-terminal-header">
                            <div class="taskmd-terminal-title">
                                <span class="terminal-dot {terminal_active}"></span>
                                <span>TERMINAL (LIVE AGENT ACTIVITY)</span>
                            </div>
                            <div class="taskmd-terminal-stats">
                                <span>Processed: <strong id="terminal-processed-{task_id}">{processed_count}</strong> items</span>
                                <span class="stat-sep">|</span>
                                <span>Speed: <strong>{processing_speed}</strong></span>
                                <span class="stat-sep">|</span>
                                <span>ETA: <strong id="terminal-eta-{task_id}">{eta_display}</strong></span>
                            </div>
                        </div>
                        <div class="taskmd-terminal-output" id="terminal-output-{task_id}">
                            {terminal_html}
                        </div>
                    </div>

                    <!-- Actions -->
                    <div class="taskmd-actions">
                        {app_button_html}
                        {cancel_button_html}
                    </div>
                </div>
            "#,
                task_id = task_id,
                title = task.title,
                status_class = status_class,
                status_label = status_label,
                error_html = error_html,
                status_html = status_html,
                progress_log_html = progress_log_html,
                terminal_active = if task.status == "running" { "active" } else { "" },
                terminal_html = terminal_html,
                app_button_html = app_button_html,
                cancel_button_html = cancel_button_html,
                processed_count = utils::get_manifest_processed_count(&state, &task_id),
                processing_speed = utils::get_manifest_speed(&state, &task_id),
                eta_display = utils::get_manifest_eta(&state, &task_id),
            );
            (StatusCode::OK, axum::response::Html(html)).into_response()
        }
        Ok(None) => {
            warn!("[TASK_GET] Task not found: {}", id);
            (StatusCode::NOT_FOUND, axum::response::Html("<div class='error'>Task not found</div>".to_string())).into_response()
        }
        Err(e) => {
            error!("[TASK_GET] Error fetching task {}: {}", id, e);
            (StatusCode::INTERNAL_SERVER_ERROR, axum::response::Html(format!("<div class='error'>{}</div>", e))).into_response()
        }
    }
}

/// Configure task routes for the Axum router
pub fn configure_task_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/tasks", post(handle_task_create))
        .route("/tasks", get(handle_task_list))
        .route("/tasks/:id", get(handle_task_get))
        .route("/tasks/:id", put(handle_task_update))
        .route("/tasks/:id", delete(handle_task_delete))
}
