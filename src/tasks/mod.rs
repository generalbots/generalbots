pub mod scheduler;

use crate::auto_task::TaskManifest;
use crate::core::urls::ApiUrls;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{delete, get, post, put},
    Router,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::Write as FmtWrite;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::shared::state::AppState;
use crate::shared::utils::DbPool;

pub use scheduler::TaskScheduler;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: Option<String>,
    pub assignee_id: Option<Uuid>,
    pub reporter_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub priority: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub tags: Option<Vec<String>>,
    pub estimated_hours: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskFilters {
    pub status: Option<String>,
    pub priority: Option<String>,
    pub assignee: Option<String>,
    pub project_id: Option<Uuid>,
    pub tag: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskUpdate {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub assignee: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = crate::core::shared::models::schema::tasks)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub assignee_id: Option<Uuid>,
    pub reporter_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub due_date: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
    pub dependencies: Vec<Uuid>,
    pub estimated_hours: Option<f64>,
    pub actual_hours: Option<f64>,
    pub progress: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResponse {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub assignee: Option<String>,
    pub reporter: Option<String>,
    pub status: String,
    pub priority: String,
    pub due_date: Option<DateTime<Utc>>,
    pub estimated_hours: Option<f64>,
    pub actual_hours: Option<f64>,
    pub tags: Vec<String>,
    pub parent_task_id: Option<Uuid>,
    pub subtasks: Vec<Uuid>,
    pub dependencies: Vec<Uuid>,
    pub attachments: Vec<String>,
    pub comments: Vec<TaskComment>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub progress: i32,
}

impl From<Task> for TaskResponse {
    fn from(task: Task) -> Self {
        Self {
            id: task.id,
            title: task.title,
            description: task.description.unwrap_or_default(),
            assignee: task.assignee_id.map(|id| id.to_string()),
            reporter: task.reporter_id.map(|id| id.to_string()),
            status: task.status,
            priority: task.priority,
            due_date: task.due_date,
            estimated_hours: task.estimated_hours,
            actual_hours: task.actual_hours,
            tags: task.tags,
            parent_task_id: None,
            subtasks: vec![],
            dependencies: task.dependencies,
            attachments: vec![],
            comments: vec![],
            created_at: task.created_at,
            updated_at: task.updated_at,
            completed_at: task.completed_at,
            progress: task.progress,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Completed,
    OnHold,
    Review,
    Blocked,
    Cancelled,
    Done,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Urgent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskComment {
    pub id: Uuid,
    pub task_id: Uuid,
    pub author: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskTemplate {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub default_assignee: Option<String>,
    pub default_priority: TaskPriority,
    pub default_tags: Vec<String>,
    pub checklist: Vec<ChecklistItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistItem {
    pub id: Uuid,
    pub task_id: Uuid,
    pub description: String,
    pub completed: bool,
    pub completed_by: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskBoard {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub columns: Vec<BoardColumn>,
    pub owner: String,
    pub members: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardColumn {
    pub id: Uuid,
    pub name: String,
    pub position: i32,
    pub status_mapping: TaskStatus,
    pub task_ids: Vec<Uuid>,
    pub wip_limit: Option<i32>,
}

#[derive(Debug)]
pub struct TaskEngine {
    _db: DbPool,
    cache: Arc<RwLock<Vec<Task>>>,
}

impl TaskEngine {
    pub fn new(db: DbPool) -> Self {
        Self {
            _db: db,
            cache: Arc::new(RwLock::new(vec![])),
        }
    }

    pub async fn create_task(
        &self,
        request: CreateTaskRequest,
    ) -> Result<TaskResponse, Box<dyn std::error::Error>> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        let task = Task {
            id,
            title: request.title,
            description: request.description,
            status: "todo".to_string(),
            priority: request.priority.unwrap_or_else(|| "medium".to_string()),
            assignee_id: request.assignee_id,
            reporter_id: request.reporter_id,
            project_id: request.project_id,
            due_date: request.due_date,
            tags: request.tags.unwrap_or_default(),
            dependencies: vec![],
            estimated_hours: request.estimated_hours,
            actual_hours: None,
            progress: 0,
            created_at: now,
            updated_at: now,
            completed_at: None,
        };

        let created_task = self.create_task_with_db(task).await?;

        Ok(created_task.into())
    }

    pub async fn list_tasks(
        &self,
        filters: TaskFilters,
    ) -> Result<Vec<TaskResponse>, Box<dyn std::error::Error + Send + Sync>> {
        let cache = self.cache.read().await;
        let mut tasks: Vec<Task> = cache.clone();
        drop(cache);

        if let Some(status) = filters.status {
            tasks.retain(|t| t.status == status);
        }
        if let Some(priority) = filters.priority {
            tasks.retain(|t| t.priority == priority);
        }
        if let Some(assignee) = filters.assignee {
            if let Ok(assignee_id) = Uuid::parse_str(&assignee) {
                tasks.retain(|t| t.assignee_id == Some(assignee_id));
            }
        }
        if let Some(project_id) = filters.project_id {
            tasks.retain(|t| t.project_id == Some(project_id));
        }
        if let Some(tag) = filters.tag {
            tasks.retain(|t| t.tags.contains(&tag));
        }

        tasks.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        if let Some(limit) = filters.limit {
            tasks.truncate(limit);
        }

        Ok(tasks.into_iter().map(|t| t.into()).collect())
    }

    pub async fn update_status(
        &self,
        id: Uuid,
        status: String,
    ) -> Result<TaskResponse, Box<dyn std::error::Error + Send + Sync>> {
        let mut cache = self.cache.write().await;

        if let Some(task) = cache.iter_mut().find(|t| t.id == id) {
            task.status.clone_from(&status);
            if status == "completed" || status == "done" {
                task.completed_at = Some(Utc::now());
                task.progress = 100;
            }
            task.updated_at = Utc::now();
            Ok(task.clone().into())
        } else {
            Err("Task not found".into())
        }
    }
}

pub async fn handle_task_create(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<Json<TaskResponse>, StatusCode> {
    let task_engine = &state.task_engine;

    match task_engine.create_task(payload).await {
        Ok(task) => Ok(Json(task)),
        Err(e) => {
            log::error!("Failed to create task: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn handle_task_update(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<TaskUpdate>,
) -> Result<Json<TaskResponse>, StatusCode> {
    let task_engine = &state.task_engine;

    match task_engine.update_task(id, payload).await {
        Ok(task) => Ok(Json(task.into())),
        Err(e) => {
            log::error!("Failed to update task: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn handle_task_delete(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let task_engine = &state.task_engine;

    match task_engine.delete_task(id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            log::error!("Failed to delete task: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn handle_task_get(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    log::info!("[TASK_GET] *** Handler called for task: {} ***", id);

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
                log::error!("[TASK_GET] DB connection error: {}", e);
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
                log::info!("[TASK_GET] Parsed UUID: {}", u);
                u
            }
            Err(e) => {
                log::error!("[TASK_GET] Invalid task ID '{}': {}", task_id, e);
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
            log::error!("[TASK_GET] Query error for {}: {}", parsed_uuid, e);
            e
        })
        .ok();

        log::info!("[TASK_GET] Query result for {}: found={}", parsed_uuid, task.is_some());
        Ok::<_, String>(task)
    })
    .await
    .unwrap_or_else(|e| {
        log::error!("Task query failed: {}", e);
        Err(format!("Task query failed: {}", e))
    });

    match result {
        Ok(Some(task)) => {
            log::info!("[TASK_GET] Returning task: {} - {} (wants_json={})", task.id, task.title, wants_json);

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
            let terminal_html = build_terminal_html(&task.step_results, &task.status);

            // Extract app_url from step_results if task is completed
            let app_url = if task.status == "completed" || task.status == "done" {
                extract_app_url_from_results(&task.step_results, &task.title)
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

            let (status_html, progress_log_html) = build_taskmd_html(&state, &task_id, &task.title, &runtime, task.manifest_json.as_ref());

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
                processed_count = get_manifest_processed_count(&state, &task_id),
                processing_speed = get_manifest_speed(&state, &task_id),
                eta_display = get_manifest_eta(&state, &task_id),
            );
            (StatusCode::OK, axum::response::Html(html)).into_response()
        }
        Ok(None) => {
            log::warn!("[TASK_GET] Task not found: {}", id);
            (StatusCode::NOT_FOUND, axum::response::Html("<div class='error'>Task not found</div>".to_string())).into_response()
        }
        Err(e) => {
            log::error!("[TASK_GET] Error fetching task {}: {}", id, e);
            (StatusCode::INTERNAL_SERVER_ERROR, axum::response::Html(format!("<div class='error'>{}</div>", e))).into_response()
        }
    }
}

fn extract_app_url_from_results(step_results: &Option<serde_json::Value>, title: &str) -> Option<String> {
    if let Some(serde_json::Value::Array(steps)) = step_results {
        for step in steps.iter() {
            if let Some(logs) = step.get("logs").and_then(|v| v.as_array()) {
                for log in logs.iter() {
                    if let Some(msg) = log.get("message").and_then(|v| v.as_str()) {
                        if msg.contains("/apps/") {
                            if let Some(start) = msg.find("/apps/") {
                                let rest = &msg[start..];
                                let end = rest.find(|c: char| c.is_whitespace() || c == '"' || c == '\'').unwrap_or(rest.len());
                                let url = rest[..end].to_string();
                                // Add trailing slash if not present
                                if url.ends_with('/') {
                                    return Some(url);
                                } else {
                                    return Some(format!("{}/", url));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let app_name = title
        .to_lowercase()
        .replace(' ', "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect::<String>();

    if !app_name.is_empty() {
        Some(format!("/apps/{}/", app_name))
    } else {
        None
    }
}

// Helper functions to get real manifest stats
fn get_manifest_processed_count(state: &Arc<AppState>, task_id: &str) -> String {
    if let Ok(manifests) = state.task_manifests.read() {
        if let Some(manifest) = manifests.get(task_id) {
            return manifest.processing_stats.data_points_processed.to_string();
        }
    }
    "0".to_string()
}

fn get_manifest_speed(state: &Arc<AppState>, task_id: &str) -> String {
    if let Ok(manifests) = state.task_manifests.read() {
        if let Some(manifest) = manifests.get(task_id) {
            let speed = manifest.processing_stats.sources_per_min;
            if speed > 0.0 {
                return format!("{:.1}/min", speed);
            }
        }
    }
    "calculating...".to_string()
}

fn get_manifest_eta(state: &Arc<AppState>, task_id: &str) -> String {
    if let Ok(manifests) = state.task_manifests.read() {
        if let Some(manifest) = manifests.get(task_id) {
            let eta_secs = manifest.processing_stats.estimated_remaining_seconds;
            if eta_secs > 0 {
                if eta_secs >= 60 {
                    return format!("~{} min", eta_secs / 60);
                } else {
                    return format!("~{} sec", eta_secs);
                }
            } else if manifest.status == crate::auto_task::ManifestStatus::Completed {
                return "Done".to_string();
            }
        }
    }
    "calculating...".to_string()
}

fn build_taskmd_html(state: &Arc<AppState>, task_id: &str, title: &str, runtime: &str, db_manifest: Option<&serde_json::Value>) -> (String, String) {
    log::info!("[TASKMD_HTML] Building TASK.md view for task_id: {}", task_id);

    // First, try to get manifest from in-memory cache (for active/running tasks)
    if let Ok(manifests) = state.task_manifests.read() {
        if let Some(manifest) = manifests.get(task_id) {
            log::info!("[TASKMD_HTML] Found manifest in memory for task: {} with {} sections", manifest.app_name, manifest.sections.len());
            let status_html = build_status_section_html(manifest, title, runtime);
            let progress_html = build_progress_log_html(manifest);
            return (status_html, progress_html);
        }
    }

    // If not in memory, try to load from database (for completed/historical tasks)
    if let Some(manifest_json) = db_manifest {
        log::info!("[TASKMD_HTML] Found manifest in database for task: {}", task_id);
        if let Ok(manifest) = serde_json::from_value::<TaskManifest>(manifest_json.clone()) {
            log::info!("[TASKMD_HTML] Parsed DB manifest for task: {} with {} sections", manifest.app_name, manifest.sections.len());
            let status_html = build_status_section_html(&manifest, title, runtime);
            let progress_html = build_progress_log_html(&manifest);
            return (status_html, progress_html);
        } else {
            // Try parsing as web JSON format (the format we store)
            if let Ok(web_manifest) = parse_web_manifest_json(manifest_json) {
                log::info!("[TASKMD_HTML] Parsed web manifest from DB for task: {}", task_id);
                let status_html = build_status_section_from_web_json(&web_manifest, title, runtime);
                let progress_html = build_progress_log_from_web_json(&web_manifest);
                return (status_html, progress_html);
            }
            log::warn!("[TASKMD_HTML] Failed to parse manifest JSON for task: {}", task_id);
        }
    }

    log::info!("[TASKMD_HTML] No manifest found for task: {}", task_id);

    let default_status = format!(r#"
        <div class="status-row">
            <span class="status-title">{}</span>
            <span class="status-time">Runtime: {}</span>
        </div>
    "#, title, runtime);

    (default_status, r#"<div class="progress-empty">No steps executed yet</div>"#.to_string())
}

// Parse the web JSON format that we store in the database
fn parse_web_manifest_json(json: &serde_json::Value) -> Result<serde_json::Value, ()> {
    // The web format has sections with status as strings, etc.
    if json.get("sections").is_some() {
        Ok(json.clone())
    } else {
        Err(())
    }
}

fn build_status_section_from_web_json(manifest: &serde_json::Value, title: &str, runtime: &str) -> String {
    let mut html = String::new();

    let current_action = manifest
        .get("current_status")
        .and_then(|s| s.get("current_action"))
        .and_then(|a| a.as_str())
        .unwrap_or("Processing...");

    let estimated_seconds = manifest
        .get("estimated_seconds")
        .and_then(|e| e.as_u64())
        .unwrap_or(0);

    let estimated = if estimated_seconds >= 60 {
        format!("{} min", estimated_seconds / 60)
    } else {
        format!("{} sec", estimated_seconds)
    };

    let runtime_display = if runtime == "0s" || runtime == "calculating..." {
        "Not started".to_string()
    } else {
        runtime.to_string()
    };

    html.push_str(&format!(r#"
        <div class="status-row status-main">
            <span class="status-title">{}</span>
            <span class="status-time">Runtime: {} <span class="status-indicator"></span></span>
        </div>
        <div class="status-row status-current">
            <span class="status-dot active"></span>
            <span class="status-text">{}</span>
            <span class="status-time">Estimated: {} <span class="status-gear">⚙</span></span>
        </div>
    "#, title, runtime_display, current_action, estimated));

    html
}

fn build_progress_log_from_web_json(manifest: &serde_json::Value) -> String {
    let mut html = String::new();
    html.push_str(r#"<div class="taskmd-tree">"#);

    let total_steps = manifest
        .get("total_steps")
        .and_then(|t| t.as_u64())
        .unwrap_or(60) as u32;

    let sections = match manifest.get("sections").and_then(|s| s.as_array()) {
        Some(s) => s,
        None => {
            html.push_str("</div>");
            return html;
        }
    };

    for section in sections {
        let section_id = section.get("id").and_then(|i| i.as_str()).unwrap_or("unknown");
        let section_name = section.get("name").and_then(|n| n.as_str()).unwrap_or("Unknown");
        let section_status = section.get("status").and_then(|s| s.as_str()).unwrap_or("Pending");

        // Progress fields are nested inside a "progress" object in the web JSON format
        let progress = section.get("progress");
        let current_step = progress
            .and_then(|p| p.get("current"))
            .and_then(|c| c.as_u64())
            .unwrap_or(0) as u32;
        let global_step_start = progress
            .and_then(|p| p.get("global_start"))
            .and_then(|g| g.as_u64())
            .unwrap_or(0) as u32;

        let section_class = match section_status.to_lowercase().as_str() {
            "completed" => "completed expanded",
            "running" => "running expanded",
            "failed" => "failed",
            "skipped" => "skipped",
            _ => "pending",
        };

        let global_current = global_step_start + current_step;

        html.push_str(&format!(r#"
            <div class="tree-section {}" data-section-id="{}">
                <div class="tree-row tree-level-0" onclick="this.parentElement.classList.toggle('expanded')">
                    <span class="tree-name">{}</span>
                    <span class="tree-step-badge">Step {}/{}</span>
                    <span class="tree-status {}">{}</span>
                    <span class="tree-section-dot {}"></span>
                </div>
                <div class="tree-children">
        "#, section_class, section_id, section_name, global_current, total_steps, section_class, section_status, section_class));

        // Render children
        if let Some(children) = section.get("children").and_then(|c| c.as_array()) {
            for child in children {
                let child_id = child.get("id").and_then(|i| i.as_str()).unwrap_or("unknown");
                let child_name = child.get("name").and_then(|n| n.as_str()).unwrap_or("Unknown");
                let child_status = child.get("status").and_then(|s| s.as_str()).unwrap_or("Pending");

                // Progress fields are nested inside a "progress" object in the web JSON format
                let child_progress = child.get("progress");
                let child_current = child_progress
                    .and_then(|p| p.get("current"))
                    .and_then(|c| c.as_u64())
                    .unwrap_or(0) as u32;
                let child_total = child_progress
                    .and_then(|p| p.get("total"))
                    .and_then(|t| t.as_u64())
                    .unwrap_or(0) as u32;

                let child_class = match child_status.to_lowercase().as_str() {
                    "completed" => "completed expanded",
                    "running" => "running expanded",
                    "failed" => "failed",
                    "skipped" => "skipped",
                    _ => "pending",
                };

                html.push_str(&format!(r#"
                    <div class="tree-child {}" data-child-id="{}">
                        <div class="tree-row tree-level-1" onclick="this.parentElement.classList.toggle('expanded')">
                            <span class="tree-indent"></span>
                            <span class="tree-name">{}</span>
                            <span class="tree-step-badge">Step {}/{}</span>
                            <span class="tree-status {}">{}</span>
                        </div>
                        <div class="tree-items">
                "#, child_class, child_id, child_name, child_current, child_total, child_class, child_status));

                // Render items
                if let Some(items) = child.get("items").and_then(|i| i.as_array()) {
                    for item in items {
                        let item_name = item.get("name").and_then(|n| n.as_str()).unwrap_or("Unknown");
                        let item_status = item.get("status").and_then(|s| s.as_str()).unwrap_or("Pending");
                        let duration = item.get("duration_seconds").and_then(|d| d.as_u64());

                        let item_class = match item_status.to_lowercase().as_str() {
                            "completed" => "completed",
                            "running" => "running",
                            _ => "pending",
                        };

                        let check_mark = if item_status.to_lowercase() == "completed" { "✓" } else { "" };
                        let duration_str = duration
                            .map(|s| if s >= 60 { format!("Duration: {} min", s / 60) } else { format!("Duration: {} sec", s) })
                            .unwrap_or_default();

                        html.push_str(&format!(r#"
                            <div class="tree-item {}">
                                <span class="item-dot {}"></span>
                                <span class="item-name">{}</span>
                                <div class="item-info">
                                    <span class="item-duration">{}</span>
                                    <span class="item-check {}">{}</span>
                                </div>
                            </div>
                        "#, item_class, item_class, item_name, duration_str, item_class, check_mark));
                    }
                }

                html.push_str("</div></div>"); // Close tree-items and tree-child
            }
        }

        html.push_str("</div></div>"); // Close tree-children and tree-section
    }

    html.push_str("</div>"); // Close taskmd-tree
    html
}

fn build_status_section_html(manifest: &TaskManifest, title: &str, runtime: &str) -> String {
    let mut html = String::new();

    let current_action = manifest.current_status.current_action.as_deref().unwrap_or("Processing...");

    // Format estimated time nicely
    let estimated = if manifest.estimated_seconds >= 60 {
        format!("{} min", manifest.estimated_seconds / 60)
    } else {
        format!("{} sec", manifest.estimated_seconds)
    };

    // Format runtime nicely
    let runtime_display = if runtime == "0s" || runtime == "calculating..." {
        "Not started".to_string()
    } else {
        runtime.to_string()
    };

    html.push_str(&format!(r#"
        <div class="status-row status-main">
            <span class="status-title">{}</span>
            <span class="status-time">Runtime: {} <span class="status-indicator"></span></span>
        </div>
        <div class="status-row status-current">
            <span class="status-dot active"></span>
            <span class="status-text">{}</span>
            <span class="status-time">Estimated: {} <span class="status-gear">⚙</span></span>
        </div>
    "#, title, runtime_display, current_action, estimated));

    if let Some(ref dp) = manifest.current_status.decision_point {
        html.push_str(&format!(r#"
            <div class="status-row status-decision">
                <span class="status-dot pending"></span>
                <span class="status-text">Decision Point Coming (Step {}/{})</span>
                <span class="status-badge">{}</span>
            </div>
        "#, dp.step_current, dp.step_total, dp.message));
    }

    html
}

fn build_progress_log_html(manifest: &TaskManifest) -> String {
    let mut html = String::new();
    html.push_str(r#"<div class="taskmd-tree">"#);

    let total_steps = manifest.total_steps;

    log::info!("[PROGRESS_HTML] Building progress log, {} sections, total_steps={}", manifest.sections.len(), total_steps);

    for section in &manifest.sections {
        log::info!("[PROGRESS_HTML] Section '{}': children={}, items={}, item_groups={}",
            section.name, section.children.len(), section.items.len(), section.item_groups.len());
        let section_class = match section.status {
            crate::auto_task::SectionStatus::Completed => "completed expanded",
            crate::auto_task::SectionStatus::Running => "running expanded",
            crate::auto_task::SectionStatus::Failed => "failed",
            crate::auto_task::SectionStatus::Skipped => "skipped",
            _ => "pending",
        };

        let status_text = match section.status {
            crate::auto_task::SectionStatus::Completed => "Completed",
            crate::auto_task::SectionStatus::Running => "Running",
            crate::auto_task::SectionStatus::Failed => "Failed",
            crate::auto_task::SectionStatus::Skipped => "Skipped",
            _ => "Pending",
        };

        // Use global step count (e.g., "Step 24/60")
        let global_current = section.global_step_start + section.current_step;

        html.push_str(&format!(r#"
            <div class="tree-section {}" data-section-id="{}">
                <div class="tree-row tree-level-0" onclick="this.parentElement.classList.toggle('expanded')">
                    <span class="tree-name">{}</span>
                    <span class="tree-step-badge">Step {}/{}</span>
                    <span class="tree-status {}">{}</span>
                    <span class="tree-section-dot {}"></span>
                </div>
                <div class="tree-children">
        "#, section_class, section.id, section.name, global_current, total_steps, section_class, status_text, section_class));

        for child in &section.children {
            log::info!("[PROGRESS_HTML]   Child '{}': items={}, item_groups={}",
                child.name, child.items.len(), child.item_groups.len());
            let child_class = match child.status {
                crate::auto_task::SectionStatus::Completed => "completed expanded",
                crate::auto_task::SectionStatus::Running => "running expanded",
                crate::auto_task::SectionStatus::Failed => "failed",
                crate::auto_task::SectionStatus::Skipped => "skipped",
                _ => "pending",
            };

            let child_status = match child.status {
                crate::auto_task::SectionStatus::Completed => "Completed",
                crate::auto_task::SectionStatus::Running => "Running",
                crate::auto_task::SectionStatus::Failed => "Failed",
                crate::auto_task::SectionStatus::Skipped => "Skipped",
                _ => "Pending",
            };

            html.push_str(&format!(r#"
                <div class="tree-child {}" data-child-id="{}" onclick="this.classList.toggle('expanded')">
                    <div class="tree-row tree-level-1">
                        <span class="tree-indent"></span>
                        <span class="tree-name">{}</span>
                        <span class="tree-step-badge">Step {}/{}</span>
                        <span class="tree-status {}">{}</span>
                    </div>
                    <div class="tree-items">
            "#, child_class, child.id, child.name, child.current_step, child.total_steps, child_class, child_status));

            // Render item groups first (grouped fields like "email, password_hash, email_verified")
            for group in &child.item_groups {
                let group_class = match group.status {
                    crate::auto_task::ItemStatus::Completed => "completed",
                    crate::auto_task::ItemStatus::Running => "running",
                    _ => "pending",
                };
                let check_mark = if group.status == crate::auto_task::ItemStatus::Completed { "✓" } else { "" };

                let group_duration = group.duration_seconds
                    .map(|s| if s >= 60 { format!("Duration: {} min", s / 60) } else { format!("Duration: {} sec", s) })
                    .unwrap_or_default();

                let group_name = group.display_name();

                html.push_str(&format!(r#"
                    <div class="tree-item {}" data-item-id="{}">
                        <span class="tree-item-dot {}"></span>
                        <span class="tree-item-name">{}</span>
                        <span class="tree-item-duration">{}</span>
                        <span class="tree-item-check {}">{}</span>
                    </div>
                "#, group_class, group.id, group_class, group_name, group_duration, group_class, check_mark));
            }

            // Then individual items
            for item in &child.items {
                let item_class = match item.status {
                    crate::auto_task::ItemStatus::Completed => "completed",
                    crate::auto_task::ItemStatus::Running => "running",
                    _ => "pending",
                };
                let check_mark = if item.status == crate::auto_task::ItemStatus::Completed { "✓" } else { "" };

                let item_duration = item.duration_seconds
                    .map(|s| if s >= 60 { format!("Duration: {} min", s / 60) } else { format!("Duration: {} sec", s) })
                    .unwrap_or_default();

                html.push_str(&format!(r#"
                    <div class="tree-item {}" data-item-id="{}">
                        <span class="tree-item-dot {}"></span>
                        <span class="tree-item-name">{}</span>
                        <span class="tree-item-duration">{}</span>
                        <span class="tree-item-check {}">{}</span>
                    </div>
                "#, item_class, item.id, item_class, item.name, item_duration, item_class, check_mark));
            }

            html.push_str("</div></div>");
        }

        // Render section-level item groups
        for group in &section.item_groups {
            let group_class = match group.status {
                crate::auto_task::ItemStatus::Completed => "completed",
                crate::auto_task::ItemStatus::Running => "running",
                _ => "pending",
            };
            let check_mark = if group.status == crate::auto_task::ItemStatus::Completed { "✓" } else { "" };

            let group_duration = group.duration_seconds
                .map(|s| if s >= 60 { format!("Duration: {} min", s / 60) } else { format!("Duration: {} sec", s) })
                .unwrap_or_default();

            let group_name = group.display_name();

            html.push_str(&format!(r#"
                <div class="tree-item {}" data-item-id="{}">
                    <span class="tree-item-dot {}"></span>
                    <span class="tree-item-name">{}</span>
                    <span class="tree-item-duration">{}</span>
                    <span class="tree-item-check {}">{}</span>
                </div>
            "#, group_class, group.id, group_class, group_name, group_duration, group_class, check_mark));
        }

        // Render section-level items
        for item in &section.items {
            let item_class = match item.status {
                crate::auto_task::ItemStatus::Completed => "completed",
                crate::auto_task::ItemStatus::Running => "running",
                _ => "pending",
            };
            let check_mark = if item.status == crate::auto_task::ItemStatus::Completed { "✓" } else { "" };

            let item_duration = item.duration_seconds
                .map(|s| if s >= 60 { format!("Duration: {} min", s / 60) } else { format!("Duration: {} sec", s) })
                .unwrap_or_default();

            html.push_str(&format!(r#"
                <div class="tree-item {}" data-item-id="{}">
                    <span class="tree-item-dot {}"></span>
                    <span class="tree-item-name">{}</span>
                    <span class="tree-item-duration">{}</span>
                    <span class="tree-item-check {}">{}</span>
                </div>
            "#, item_class, item.id, item_class, item.name, item_duration, item_class, check_mark));
        }

        html.push_str("</div></div>");
    }

    html.push_str("</div>");

    if manifest.sections.is_empty() {
        return r#"<div class="progress-empty">No steps executed yet</div>"#.to_string();
    }

    html
}



/// Build HTML for the progress log section from step_results JSON
fn build_terminal_html(step_results: &Option<serde_json::Value>, status: &str) -> String {
    let mut html = String::new();
    let mut output_lines: Vec<(String, bool)> = Vec::new();

    if let Some(serde_json::Value::Array(steps)) = step_results {
        for step in steps.iter() {
            let step_status = step.get("status").and_then(|v| v.as_str()).unwrap_or("");
            let is_current = step_status == "running" || step_status == "Running";

            if let Some(serde_json::Value::Array(logs)) = step.get("logs") {
                for log_entry in logs.iter() {
                    if let Some(msg) = log_entry.get("message").and_then(|v| v.as_str()) {
                        if !msg.trim().is_empty() {
                            output_lines.push((msg.to_string(), is_current));
                        }
                    }
                    if let Some(code) = log_entry.get("code").and_then(|v| v.as_str()) {
                        if !code.trim().is_empty() {
                            for line in code.lines().take(20) {
                                output_lines.push((format!("  {}", line), is_current));
                            }
                        }
                    }
                    if let Some(output) = log_entry.get("output").and_then(|v| v.as_str()) {
                        if !output.trim().is_empty() {
                            for line in output.lines().take(10) {
                                output_lines.push((format!("→ {}", line), is_current));
                            }
                        }
                    }
                }
            }
        }
    }

    if output_lines.is_empty() {
        let msg = match status {
            "running" => "Agent working...",
            "pending" => "Waiting to start...",
            "completed" | "done" => "✓ Task completed",
            "failed" | "error" => "✗ Task failed",
            "paused" => "Task paused",
            _ => "Initializing..."
        };
        html.push_str(&format!(r#"<div class="terminal-line">{}</div>"#, msg));
    } else {
        let start = if output_lines.len() > 15 { output_lines.len() - 15 } else { 0 };
        for (line, is_current) in output_lines[start..].iter() {
            let class = if *is_current { "terminal-line current" } else { "terminal-line" };
            let escaped = line.replace('<', "&lt;").replace('>', "&gt;");
            html.push_str(&format!(r#"<div class="{}">{}</div>"#, class, escaped));
        }
    }

    html
}

impl TaskEngine {
    pub async fn create_task_with_db(
        &self,
        task: Task,
    ) -> Result<Task, Box<dyn std::error::Error>> {
        use crate::shared::models::schema::tasks::dsl::*;
        use diesel::prelude::*;

        let conn = self._db.clone();
        let task_clone = task.clone();

        let created_task =
            tokio::task::spawn_blocking(move || -> Result<Task, diesel::result::Error> {
                let mut db_conn = conn.get().map_err(|e| {
                    diesel::result::Error::DatabaseError(
                        diesel::result::DatabaseErrorKind::UnableToSendCommand,
                        Box::new(e.to_string()),
                    )
                })?;

                diesel::insert_into(tasks)
                    .values(&task_clone)
                    .get_result(&mut db_conn)
            })
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        let mut cache = self.cache.write().await;
        cache.push(created_task.clone());
        drop(cache);

        Ok(created_task)
    }

    pub async fn update_task(
        &self,
        id: Uuid,
        updates: TaskUpdate,
    ) -> Result<Task, Box<dyn std::error::Error + Send + Sync>> {
        let updated_at = Utc::now();

        let mut cache = self.cache.write().await;
        if let Some(task) = cache.iter_mut().find(|t| t.id == id) {
            task.updated_at = updated_at;

            if let Some(title) = updates.title {
                task.title = title;
            }
            if let Some(description) = updates.description {
                task.description = Some(description);
            }
            if let Some(status) = updates.status {
                task.status.clone_from(&status);
                if status == "completed" || status == "done" {
                    task.completed_at = Some(Utc::now());
                    task.progress = 100;
                }
            }
            if let Some(priority) = updates.priority {
                task.priority = priority;
            }
            if let Some(assignee) = updates.assignee {
                task.assignee_id = Uuid::parse_str(&assignee).ok();
            }
            if let Some(due_date) = updates.due_date {
                task.due_date = Some(due_date);
            }
            if let Some(tags) = updates.tags {
                task.tags = tags;
            }

            let result = task.clone();
            drop(cache);
            return Ok(result);
        }
        drop(cache);

        Err("Task not found".into())
    }

    pub async fn delete_task(
        &self,
        id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let dependencies = self.get_task_dependencies(id).await?;
        if !dependencies.is_empty() {
            return Err("Cannot delete task with dependencies".into());
        }

        let mut cache = self.cache.write().await;
        cache.retain(|t| t.id != id);
        drop(cache);

        self.refresh_cache()
            .await
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                Box::new(std::io::Error::other(e.to_string()))
            })?;
        Ok(())
    }

    pub async fn get_user_tasks(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<Task>, Box<dyn std::error::Error>> {
        let cache = self.cache.read().await;
        let user_tasks: Vec<Task> = cache
            .iter()
            .filter(|t| {
                t.assignee_id.map(|a| a == user_id).unwrap_or(false)
                    || t.reporter_id.map(|r| r == user_id).unwrap_or(false)
            })
            .cloned()
            .collect();
        drop(cache);

        Ok(user_tasks)
    }

    pub async fn get_tasks_by_status(
        &self,
        status: TaskStatus,
    ) -> Result<Vec<Task>, Box<dyn std::error::Error + Send + Sync>> {
        let cache = self.cache.read().await;
        let status_str = format!("{:?}", status);
        let mut tasks: Vec<Task> = cache
            .iter()
            .filter(|t| t.status == status_str)
            .cloned()
            .collect();
        drop(cache);
        tasks.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(tasks)
    }

    pub async fn get_overdue_tasks(
        &self,
    ) -> Result<Vec<Task>, Box<dyn std::error::Error + Send + Sync>> {
        let now = Utc::now();
        let cache = self.cache.read().await;
        let mut tasks: Vec<Task> = cache
            .iter()
            .filter(|t| t.due_date.is_some_and(|due| due < now) && t.status != "completed")
            .cloned()
            .collect();
        drop(cache);
        tasks.sort_by(|a, b| a.due_date.cmp(&b.due_date));
        Ok(tasks)
    }

    pub fn add_comment(
        &self,
        task_id: Uuid,
        author: &str,
        content: &str,
    ) -> Result<TaskComment, Box<dyn std::error::Error>> {
        let comment = TaskComment {
            id: Uuid::new_v4(),
            task_id,
            author: author.to_string(),
            content: content.to_string(),
            created_at: Utc::now(),
            updated_at: None,
        };

        log::info!("Added comment to task {}: {}", task_id, content);

        Ok(comment)
    }

    pub async fn create_subtask(
        &self,
        parent_id: Uuid,
        subtask_data: CreateTaskRequest,
    ) -> Result<Task, Box<dyn std::error::Error + Send + Sync>> {
        {
            let cache = self.cache.read().await;
            if !cache.iter().any(|t| t.id == parent_id) {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Parent task not found",
                ))
                    as Box<dyn std::error::Error + Send + Sync>);
            }
        }

        let subtask = self.create_task(subtask_data).await.map_err(
            |e| -> Box<dyn std::error::Error + Send + Sync> {
                Box::new(std::io::Error::other(e.to_string()))
            },
        )?;

        let created = Task {
            id: subtask.id,
            title: subtask.title,
            description: Some(subtask.description),
            status: subtask.status,
            priority: subtask.priority,
            assignee_id: subtask
                .assignee
                .as_ref()
                .and_then(|a| Uuid::parse_str(a).ok()),
            reporter_id: subtask
                .reporter
                .as_ref()
                .and_then(|r| Uuid::parse_str(r).ok()),
            project_id: None,
            due_date: subtask.due_date,
            tags: subtask.tags,
            dependencies: subtask.dependencies,
            estimated_hours: subtask.estimated_hours,
            actual_hours: subtask.actual_hours,
            progress: subtask.progress,
            created_at: subtask.created_at,
            updated_at: subtask.updated_at,
            completed_at: subtask.completed_at,
        };

        Ok(created)
    }

    pub async fn get_task_dependencies(
        &self,
        task_id: Uuid,
    ) -> Result<Vec<Task>, Box<dyn std::error::Error + Send + Sync>> {
        let task = self.get_task(task_id).await?;
        let mut dependencies = Vec::new();

        for dep_id in task.dependencies {
            if let Ok(dep_task) = self.get_task(dep_id).await {
                dependencies.push(dep_task);
            }
        }

        Ok(dependencies)
    }

    pub async fn get_task(
        &self,
        id: Uuid,
    ) -> Result<Task, Box<dyn std::error::Error + Send + Sync>> {
        let cache = self.cache.read().await;
        if let Some(task) = cache.iter().find(|t| t.id == id).cloned() {
            drop(cache);
            return Ok(task);
        }
        drop(cache);

        let conn = self._db.clone();
        let task_id = id;

        let task = tokio::task::spawn_blocking(move || {
            use crate::shared::models::schema::tasks::dsl::*;
            use diesel::prelude::*;

            let mut db_conn = conn.get().map_err(|e| {
                Box::<dyn std::error::Error + Send + Sync>::from(format!("DB error: {e}"))
            })?;

            tasks
                .filter(id.eq(task_id))
                .first::<Task>(&mut db_conn)
                .map_err(|e| {
                    Box::<dyn std::error::Error + Send + Sync>::from(format!("Task not found: {e}"))
                })
        })
        .await
        .map_err(|e| {
            Box::<dyn std::error::Error + Send + Sync>::from(format!("Task error: {e}"))
        })??;

        let mut cache = self.cache.write().await;
        cache.push(task.clone());
        drop(cache);

        Ok(task)
    }

    pub async fn get_all_tasks(
        &self,
    ) -> Result<Vec<Task>, Box<dyn std::error::Error + Send + Sync>> {
        let cache = self.cache.read().await;
        let mut tasks: Vec<Task> = cache.clone();
        drop(cache);
        tasks.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(tasks)
    }

    pub async fn assign_task(
        &self,
        id: Uuid,
        assignee: String,
    ) -> Result<Task, Box<dyn std::error::Error + Send + Sync>> {
        let assignee_id = Uuid::parse_str(&assignee).ok();
        let updated_at = Utc::now();

        let mut cache = self.cache.write().await;
        if let Some(task) = cache.iter_mut().find(|t| t.id == id) {
            task.assignee_id = assignee_id;
            task.updated_at = updated_at;
            let result = task.clone();
            drop(cache);
            return Ok(result);
        }
        drop(cache);

        Err("Task not found".into())
    }

    pub async fn set_dependencies(
        &self,
        task_id: Uuid,
        dependency_ids: Vec<Uuid>,
    ) -> Result<TaskResponse, Box<dyn std::error::Error + Send + Sync>> {
        let mut cache = self.cache.write().await;
        if let Some(task) = cache.iter_mut().find(|t| t.id == task_id) {
            task.dependencies = dependency_ids;
            task.updated_at = Utc::now();
        }

        let task = self.get_task(task_id).await?;
        Ok(task.into())
    }

    pub async fn calculate_progress(
        &self,
        task_id: Uuid,
    ) -> Result<u8, Box<dyn std::error::Error + Send + Sync>> {
        let task = self.get_task(task_id).await?;

        Ok(match task.status.as_str() {
            "in_progress" | "in-progress" => 50,
            "review" => 75,
            "completed" | "done" => 100,
            "blocked" => {
                ((task.actual_hours.unwrap_or(0.0) / task.estimated_hours.unwrap_or(1.0)) * 100.0)
                    as u8
            }
            // "todo", "cancelled", and any other status default to 0
            _ => 0,
        })
    }

    pub async fn create_from_template(
        &self,
        _template_id: Uuid,
        assignee_id: Option<Uuid>,
    ) -> Result<Task, Box<dyn std::error::Error + Send + Sync>> {
        let template = TaskTemplate {
            id: Uuid::new_v4(),
            name: "Default Template".to_string(),
            description: Some("Default template".to_string()),
            default_assignee: None,
            default_priority: TaskPriority::Medium,
            default_tags: vec![],
            checklist: vec![],
        };

        let now = Utc::now();
        let task = Task {
            id: Uuid::new_v4(),
            title: format!("Task from template: {}", template.name),
            description: template.description.clone(),
            status: "todo".to_string(),
            priority: "medium".to_string(),
            assignee_id,
            reporter_id: Some(Uuid::new_v4()),
            project_id: None,
            due_date: None,
            estimated_hours: None,
            actual_hours: None,
            tags: template.default_tags,
            dependencies: Vec::new(),
            progress: 0,
            created_at: now,
            updated_at: now,
            completed_at: None,
        };

        let task_request = CreateTaskRequest {
            title: task.title,
            description: task.description,
            assignee_id: task.assignee_id,
            reporter_id: task.reporter_id,
            project_id: task.project_id,
            priority: Some(task.priority),
            due_date: task.due_date,
            tags: Some(task.tags),
            estimated_hours: task.estimated_hours,
        };
        let created = self.create_task(task_request).await.map_err(
            |e| -> Box<dyn std::error::Error + Send + Sync> {
                Box::new(std::io::Error::other(e.to_string()))
            },
        )?;

        for item in template.checklist {
            let _checklist_item = ChecklistItem {
                id: Uuid::new_v4(),
                task_id: created.id,
                description: item.description.clone(),
                completed: false,
                completed_by: None,
                completed_at: None,
            };

            log::info!(
                "Added checklist item to task {}: {}",
                created.id,
                item.description
            );
        }

        let task = Task {
            id: created.id,
            title: created.title,
            description: Some(created.description),
            status: created.status,
            priority: created.priority,
            assignee_id: created
                .assignee
                .as_ref()
                .and_then(|a| Uuid::parse_str(a).ok()),
            reporter_id: created.reporter.as_ref().and_then(|r| {
                if r == "system" {
                    None
                } else {
                    Uuid::parse_str(r).ok()
                }
            }),
            project_id: None,
            tags: created.tags,
            dependencies: created.dependencies,
            due_date: created.due_date,
            estimated_hours: created.estimated_hours,
            actual_hours: created.actual_hours,
            progress: created.progress,
            created_at: created.created_at,
            updated_at: created.updated_at,
            completed_at: created.completed_at,
        };
        Ok(task)
    }

    fn _notify_assignee(assignee: &str, task: &Task) -> Result<(), Box<dyn std::error::Error>> {
        log::info!(
            "Notifying {} about new task assignment: {}",
            assignee,
            task.title
        );
        Ok(())
    }

    async fn refresh_cache(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use crate::shared::models::schema::tasks::dsl::*;
        use diesel::prelude::*;

        let conn = self._db.clone();

        let task_list = tokio::task::spawn_blocking(
            move || -> Result<Vec<Task>, Box<dyn std::error::Error + Send + Sync>> {
                let mut db_conn = conn.get()?;

                tasks
                    .order(created_at.desc())
                    .load::<Task>(&mut db_conn)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
            },
        )
        .await??;

        let mut cache = self.cache.write().await;
        *cache = task_list;

        Ok(())
    }

    pub async fn get_statistics(
        &self,
        user_id: Option<Uuid>,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        use chrono::Utc;

        let cache = self.cache.read().await;
        let task_list = if let Some(uid) = user_id {
            cache
                .iter()
                .filter(|t| {
                    t.assignee_id.map(|a| a == uid).unwrap_or(false)
                        || t.reporter_id.map(|r| r == uid).unwrap_or(false)
                })
                .cloned()
                .collect()
        } else {
            cache.clone()
        };
        drop(cache);

        let mut todo_count = 0;
        let mut in_progress_count = 0;
        let mut done_count = 0;
        let mut overdue_count = 0;
        let mut total_completion_ratio = 0.0;
        let mut ratio_count = 0;

        let now = Utc::now();

        for task in &task_list {
            match task.status.as_str() {
                "todo" => todo_count += 1,
                "in_progress" => in_progress_count += 1,
                "done" => done_count += 1,
                _ => {}
            }

            if let Some(due) = task.due_date {
                if due < now && task.status != "done" {
                    overdue_count += 1;
                }
            }

            if let (Some(actual), Some(estimated)) = (task.actual_hours, task.estimated_hours) {
                if estimated > 0.0 {
                    total_completion_ratio += actual / estimated;
                    ratio_count += 1;
                }
            }
        }

        let avg_completion_ratio = if ratio_count > 0 {
            Some(total_completion_ratio / f64::from(ratio_count))
        } else {
            None
        };

        Ok(serde_json::json!({
            "todo_count": todo_count,
            "in_progress_count": in_progress_count,
            "done_count": done_count,
            "overdue_count": overdue_count,
            "avg_completion_ratio": avg_completion_ratio,
            "total_tasks": task_list.len()
        }))
    }
}

pub mod handlers {
    use super::*;
    use axum::extract::{Path as AxumPath, Query as AxumQuery, State as AxumState};
    use axum::http::StatusCode;
    use axum::response::{IntoResponse, Json as AxumJson};

    pub async fn create_task_handler(
        AxumState(engine): AxumState<Arc<TaskEngine>>,
        AxumJson(task_resp): AxumJson<TaskResponse>,
    ) -> impl IntoResponse {
        let task = Task {
            id: task_resp.id,
            title: task_resp.title,
            description: Some(task_resp.description),
            assignee_id: task_resp.assignee.and_then(|s| Uuid::parse_str(&s).ok()),
            reporter_id: task_resp.reporter.and_then(|s| Uuid::parse_str(&s).ok()),
            project_id: None,
            status: task_resp.status,
            priority: task_resp.priority,
            due_date: task_resp.due_date,
            estimated_hours: task_resp.estimated_hours,
            actual_hours: task_resp.actual_hours,
            tags: task_resp.tags,
            dependencies: vec![],
            progress: 0,
            created_at: task_resp.created_at,
            updated_at: task_resp.updated_at,
            completed_at: None,
        };

        match engine.create_task_with_db(task).await {
            Ok(created) => (StatusCode::CREATED, AxumJson(serde_json::json!(created))),
            Err(e) => {
                log::error!("Failed to create task: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    AxumJson(serde_json::json!({"error": e.to_string()})),
                )
            }
        }
    }

    pub async fn get_tasks_handler(
        AxumState(engine): AxumState<Arc<TaskEngine>>,
        AxumQuery(query): AxumQuery<serde_json::Value>,
    ) -> impl IntoResponse {
        let status_filter = query
            .get("status")
            .and_then(|v| v.as_str())
            .and_then(|s| serde_json::from_str::<TaskStatus>(&format!("\"{}\"", s)).ok());

        let user_id = query
            .get("user_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok());

        let tasks = if let Some(status) = status_filter {
            match engine.get_tasks_by_status(status).await {
                Ok(t) => t,
                Err(e) => {
                    log::error!("Failed to get tasks by status: {}", e);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        AxumJson(serde_json::json!({"error": e.to_string()})),
                    );
                }
            }
        } else if let Some(uid) = user_id {
            match engine.get_user_tasks(uid).await {
                Ok(t) => t,
                Err(e) => {
                    log::error!("Failed to get user tasks: {}", e);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        AxumJson(serde_json::json!({"error": e.to_string()})),
                    );
                }
            }
        } else {
            match engine.get_all_tasks().await {
                Ok(t) => t,
                Err(e) => {
                    log::error!("Failed to get all tasks: {}", e);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        AxumJson(serde_json::json!({"error": e.to_string()})),
                    );
                }
            }
        };

        let responses: Vec<TaskResponse> = tasks
            .into_iter()
            .map(|t| TaskResponse {
                id: t.id,
                title: t.title,
                description: t.description.unwrap_or_default(),
                assignee: t.assignee_id.map(|id| id.to_string()),
                reporter: t.reporter_id.map(|id| id.to_string()),
                status: t.status,
                priority: t.priority,
                due_date: t.due_date,
                estimated_hours: t.estimated_hours,
                actual_hours: t.actual_hours,
                tags: t.tags,
                parent_task_id: None,
                subtasks: vec![],
                dependencies: t.dependencies,
                attachments: vec![],
                comments: vec![],
                created_at: t.created_at,
                updated_at: t.updated_at,
                completed_at: t.completed_at,
                progress: t.progress,
            })
            .collect();

        (StatusCode::OK, AxumJson(serde_json::json!(responses)))
    }

    pub async fn update_task_handler(
        AxumState(_engine): AxumState<Arc<TaskEngine>>,
        AxumPath(_id): AxumPath<Uuid>,
        AxumJson(_updates): AxumJson<TaskUpdate>,
    ) -> impl IntoResponse {
        let updated = serde_json::json!({
            "message": "Task updated",
            "task_id": _id
        });
        (StatusCode::OK, AxumJson(updated))
    }

    pub async fn get_statistics_handler(
        AxumState(_engine): AxumState<Arc<TaskEngine>>,
        AxumQuery(_query): AxumQuery<serde_json::Value>,
    ) -> impl IntoResponse {
        let stats = serde_json::json!({
            "todo_count": 0,
            "in_progress_count": 0,
            "done_count": 0,
            "overdue_count": 0,
            "total_tasks": 0
        });
        (StatusCode::OK, AxumJson(stats))
    }
}

pub async fn handle_task_list(
    State(state): State<Arc<AppState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<TaskResponse>>, StatusCode> {
    let tasks = if let Some(user_id) = params.get("user_id") {
        let user_uuid = Uuid::parse_str(user_id).unwrap_or_else(|_| Uuid::nil());
        match state.task_engine.get_user_tasks(user_uuid).await {
            Ok(tasks) => Ok(tasks),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }?
    } else if let Some(status_str) = params.get("status") {
        let status = match status_str.as_str() {
            "in_progress" => TaskStatus::InProgress,
            "review" => TaskStatus::Review,
            "done" => TaskStatus::Done,
            "blocked" => TaskStatus::Blocked,
            "completed" => TaskStatus::Completed,
            "cancelled" => TaskStatus::Cancelled,
            // "todo" and any other status default to Todo
            _ => TaskStatus::Todo,
        };
        state
            .task_engine
            .get_tasks_by_status(status)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        state
            .task_engine
            .get_all_tasks()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };

    Ok(Json(
        tasks
            .into_iter()
            .map(|t| t.into())
            .collect::<Vec<TaskResponse>>(),
    ))
}

pub async fn handle_task_assign(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<TaskResponse>, StatusCode> {
    let assignee = payload["assignee"]
        .as_str()
        .ok_or(StatusCode::BAD_REQUEST)?;

    match state
        .task_engine
        .assign_task(id, assignee.to_string())
        .await
    {
        Ok(updated) => Ok(Json(updated.into())),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn handle_task_status_update(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<TaskResponse>, StatusCode> {
    let status_str = payload["status"].as_str().ok_or(StatusCode::BAD_REQUEST)?;
    let status = match status_str {
        "todo" => "todo",
        "in_progress" => "in_progress",
        "review" => "review",
        "done" => "completed",
        "blocked" => "blocked",
        "cancelled" => "cancelled",
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    let updates = TaskUpdate {
        title: None,
        description: None,
        status: Some(status.to_string()),
        priority: None,
        assignee: None,
        due_date: None,
        tags: None,
    };

    match state.task_engine.update_task(id, updates).await {
        Ok(updated_task) => Ok(Json(updated_task.into())),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn handle_task_priority_set(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<TaskResponse>, StatusCode> {
    let priority_str = payload["priority"]
        .as_str()
        .ok_or(StatusCode::BAD_REQUEST)?;
    let priority = match priority_str {
        "low" => "low",
        "medium" => "medium",
        "high" => "high",
        "urgent" => "urgent",
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    let updates = TaskUpdate {
        title: None,
        description: None,
        status: None,
        priority: Some(priority.to_string()),
        assignee: None,
        due_date: None,
        tags: None,
    };

    match state.task_engine.update_task(id, updates).await {
        Ok(updated_task) => Ok(Json(updated_task.into())),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn handle_task_set_dependencies(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<TaskResponse>, StatusCode> {
    let deps = payload["dependencies"]
        .as_array()
        .ok_or(StatusCode::BAD_REQUEST)?
        .iter()
        .filter_map(|v| v.as_str().and_then(|s| Uuid::parse_str(s).ok()))
        .collect::<Vec<_>>();

    match state.task_engine.set_dependencies(id, deps).await {
        Ok(updated) => Ok(Json(updated)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub fn configure_task_routes() -> Router<Arc<AppState>> {
    log::info!("[ROUTES] Registering task routes with /api/tasks/:id pattern");

    Router::new()
        // Task list and create
        .route(
            "/api/tasks",
            post(handle_task_create).get(handle_task_list_htmx),
        )
        // Specific routes MUST come before parameterized route
        .route("/api/tasks/stats", get(handle_task_stats_htmx))
        .route("/api/tasks/stats/json", get(handle_task_stats))
        .route("/api/tasks/time-saved", get(handle_time_saved))
        .route("/api/tasks/completed", delete(handle_clear_completed))
        // Parameterized task routes - use :id for axum path params
        .route(
            "/api/tasks/:id",
            get(handle_task_get)
                .put(handle_task_update)
                .delete(handle_task_delete)
                .patch(handle_task_patch),
        )
        .route("/api/tasks/:id/assign", post(handle_task_assign))
        .route("/api/tasks/:id/status", put(handle_task_status_update))
        .route("/api/tasks/:id/priority", put(handle_task_priority_set))
        .route("/api/tasks/:id/dependencies", put(handle_task_set_dependencies))
        .route("/api/tasks/:id/cancel", post(handle_task_cancel))
}

pub async fn handle_task_cancel(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    log::info!("[TASK_CANCEL] Cancelling task: {}", id);

    let conn = state.conn.clone();
    let task_id = id.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn
            .get()
            .map_err(|e| format!("DB connection error: {}", e))?;

        let parsed_uuid = Uuid::parse_str(&task_id)
            .map_err(|e| format!("Invalid task ID: {}", e))?;

        diesel::sql_query(
            "UPDATE auto_tasks SET status = 'cancelled', updated_at = NOW() WHERE id = $1"
        )
        .bind::<diesel::sql_types::Uuid, _>(parsed_uuid)
        .execute(&mut db_conn)
        .map_err(|e| format!("Failed to cancel task: {}", e))?;

        Ok::<_, String>(())
    })
    .await
    .unwrap_or_else(|e| Err(format!("Task execution error: {}", e)));

    match result {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "message": "Task cancelled"
            })),
        ).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": e
            })),
        ).into_response(),
    }
}

pub fn configure(router: Router<Arc<TaskEngine>>) -> Router<Arc<TaskEngine>> {
    use axum::routing::{get, post, put};

    router
        .route(ApiUrls::TASKS, post(handlers::create_task_handler))
        .route(ApiUrls::TASKS, get(handlers::get_tasks_handler))
        .route(
            &ApiUrls::TASK_BY_ID.replace(":id", "{id}"),
            put(handlers::update_task_handler),
        )
        .route(
            "/api/tasks/statistics",
            get(handlers::get_statistics_handler),
        )
}

pub async fn handle_task_list_htmx(
    State(state): State<Arc<AppState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let filter = params
        .get("filter")
        .cloned()
        .unwrap_or_else(|| "all".to_string());

    let conn = state.conn.clone();
    let filter_clone = filter.clone();

    let tasks = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn
            .get()
            .map_err(|e| format!("DB connection error: {}", e))?;

        let mut query = String::from(
            "SELECT id, title, status, priority, intent as description, NULL::timestamp as due_date FROM auto_tasks WHERE 1=1",
        );

        match filter_clone.as_str() {
            "complete" | "completed" => query.push_str(" AND status IN ('done', 'completed')"),
            "active" => query.push_str(" AND status IN ('running', 'pending', 'in_progress')"),
            "awaiting" => query.push_str(" AND status IN ('awaiting_decision', 'awaiting', 'waiting')"),
            "paused" => query.push_str(" AND status = 'paused'"),
            "blocked" => query.push_str(" AND status IN ('blocked', 'failed', 'error')"),
            "priority" => query.push_str(" AND priority IN ('high', 'urgent')"),
            _ => {}
        }

        query.push_str(" ORDER BY created_at DESC LIMIT 50");

        diesel::sql_query(&query)
            .load::<TaskRow>(&mut db_conn)
            .map_err(|e| format!("Query failed: {}", e))
    })
    .await
    .unwrap_or_else(|e| {
        log::error!("Task query failed: {}", e);
        Err(format!("Task query failed: {}", e))
    })
    .unwrap_or_default();

    let mut html = String::new();

    for task in tasks {
        let is_completed = task.status == "done" || task.status == "completed";
        let completed_class = if is_completed { "completed" } else { "" };

        let due_date_html = if let Some(due) = &task.due_date {
            format!(
                r#"<span class="task-due-date"> {}</span>"#,
                due.format("%Y-%m-%d")
            )
        } else {
            String::new()
        };
        let status_class = match task.status.as_str() {
            "completed" | "done" => "status-complete",
            "running" | "pending" | "in_progress" => "status-running",
            "failed" | "error" | "blocked" => "status-error",
            "paused" => "status-paused",
            "awaiting" | "awaiting_decision" => "status-awaiting",
            _ => "status-pending"
        };

        let is_app_task = task.title.to_lowercase().contains("create") ||
                          task.title.to_lowercase().contains("app") ||
                          task.title.to_lowercase().contains("crm") ||
                          task.title.to_lowercase().contains("calculator");

        let task_icon = if is_app_task {
            r#"<svg class="task-type-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M9 3v18"/><path d="M14 9h3"/><path d="M14 14h3"/></svg>"#
        } else {
            r#"<svg class="task-type-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 11l3 3L22 4"/><path d="M21 12v7a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2h11"/></svg>"#
        };

        let app_url = if (task.status == "completed" || task.status == "done") && is_app_task {
            let app_name = task.title
                .to_lowercase()
                .replace("create ", "")
                .replace("a ", "")
                .replace("an ", "")
                .split_whitespace()
                .collect::<Vec<_>>()
                .join("-");
            Some(format!("/apps/{}/", app_name))
        } else {
            None
        };

        let open_app_btn = app_url.as_ref().map(|url| format!(
            r#"<a href="{}" target="_blank" class="btn-open-app" onclick="event.stopPropagation()" rel="noopener noreferrer">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 13v6a2 2 0 01-2 2H5a2 2 0 01-2-2V8a2 2 0 012-2h6"/><polyline points="15 3 21 3 21 9"/><line x1="10" y1="14" x2="21" y2="3"/></svg>
                Open App
            </a>"#,
            url
        )).unwrap_or_default();

        let _ = write!(
            html,
            r#"
            <div class="task-card {completed_class} {status_class}" data-task-id="{task_id}" onclick="selectTask('{task_id}')">
                <div class="task-card-header">
                    {task_icon}
                    <span class="task-card-title">{title}</span>
                    <span class="task-card-status {status_class}">{status}</span>
                </div>
                <div class="task-card-body">
                    <div class="task-card-priority">
                        <span class="priority-badge priority-{priority}">{priority}</span>
                    </div>
                    {due_date_html}
                    {open_app_btn}
                </div>
                <div class="task-card-footer">
                    <button class="task-action-btn" data-action="priority" data-task-id="{task_id}" onclick="event.stopPropagation()">
                        ⭐
                    </button>
                    <button class="task-action-btn" data-action="delete" data-task-id="{task_id}" onclick="event.stopPropagation()">
                        🗑️
                    </button>
                </div>
            </div>
            "#,
            task_id = task.id,
            task_icon = task_icon,
            title = task.title,
            status_class = status_class,
            status = task.status,
            priority = task.priority,
            due_date_html = due_date_html,
            open_app_btn = open_app_btn,
            completed_class = completed_class,
        );
    }

    if html.is_empty() {
        html = format!(
            r#"
            <div class="empty-state">
                <svg width="80" height="80" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1">
                    <polyline points="9 11 12 14 22 4"></polyline>
                    <path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"></path>
                </svg>
                <h3>No {} tasks</h3>
                <p>{}</p>
            </div>
            "#,
            filter,
            if filter == "all" {
                "Create your first task to get started"
            } else {
                "Switch to another view or add new tasks"
            }
        );
    }

    axum::response::Html(html)
}

pub async fn handle_task_stats_htmx(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let conn = state.conn.clone();

    let stats = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn
            .get()
            .map_err(|e| format!("DB connection error: {}", e))?;

        let total: i64 = diesel::sql_query("SELECT COUNT(*) as count FROM auto_tasks")
            .get_result::<CountResult>(&mut db_conn)
            .map(|r| r.count)
            .unwrap_or(0);

        let active: i64 =
            diesel::sql_query("SELECT COUNT(*) as count FROM auto_tasks WHERE status IN ('running', 'pending', 'in_progress')")
                .get_result::<CountResult>(&mut db_conn)
                .map(|r| r.count)
                .unwrap_or(0);

        let completed: i64 =
            diesel::sql_query("SELECT COUNT(*) as count FROM auto_tasks WHERE status IN ('done', 'completed')")
                .get_result::<CountResult>(&mut db_conn)
                .map(|r| r.count)
                .unwrap_or(0);

        let awaiting: i64 =
            diesel::sql_query("SELECT COUNT(*) as count FROM auto_tasks WHERE status IN ('awaiting_decision', 'awaiting', 'waiting')")
                .get_result::<CountResult>(&mut db_conn)
                .map(|r| r.count)
                .unwrap_or(0);

        let paused: i64 =
            diesel::sql_query("SELECT COUNT(*) as count FROM auto_tasks WHERE status = 'paused'")
                .get_result::<CountResult>(&mut db_conn)
                .map(|r| r.count)
                .unwrap_or(0);

        let blocked: i64 =
            diesel::sql_query("SELECT COUNT(*) as count FROM auto_tasks WHERE status IN ('blocked', 'failed', 'error')")
                .get_result::<CountResult>(&mut db_conn)
                .map(|r| r.count)
                .unwrap_or(0);

        let priority: i64 =
            diesel::sql_query("SELECT COUNT(*) as count FROM auto_tasks WHERE priority IN ('high', 'urgent')")
                .get_result::<CountResult>(&mut db_conn)
                .map(|r| r.count)
                .unwrap_or(0);

        let time_saved = format!("{}h", completed * 2);

        Ok::<_, String>(TaskStats {
            total: total as usize,
            active: active as usize,
            completed: completed as usize,
            awaiting: awaiting as usize,
            paused: paused as usize,
            blocked: blocked as usize,
            priority: priority as usize,
            time_saved,
        })
    })
    .await
    .unwrap_or_else(|e| {
        log::error!("Stats query failed: {}", e);
        Err(format!("Stats query failed: {}", e))
    })
    .unwrap_or(TaskStats {
        total: 0,
        active: 0,
        completed: 0,
        awaiting: 0,
        paused: 0,
        blocked: 0,
        priority: 0,
        time_saved: "0h".to_string(),
    });

    let html = format!(
        "{} tasks
        <script>
            document.getElementById('count-all').textContent = '{}';
            document.getElementById('count-complete').textContent = '{}';
            document.getElementById('count-active').textContent = '{}';
            document.getElementById('count-awaiting').textContent = '{}';
            document.getElementById('count-paused').textContent = '{}';
            document.getElementById('count-blocked').textContent = '{}';
            document.getElementById('time-saved-value').textContent = '{}';
        </script>",
        stats.total, stats.total, stats.completed, stats.active, stats.awaiting, stats.paused, stats.blocked, stats.time_saved
    );

    axum::response::Html(html)
}

pub async fn handle_task_stats(State(state): State<Arc<AppState>>) -> Json<TaskStats> {
    let conn = state.conn.clone();

    let stats = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn
            .get()
            .map_err(|e| format!("DB connection error: {}", e))?;

        let total: i64 = diesel::sql_query("SELECT COUNT(*) as count FROM auto_tasks")
            .get_result::<CountResult>(&mut db_conn)
            .map(|r| r.count)
            .unwrap_or(0);

        let active: i64 =
            diesel::sql_query("SELECT COUNT(*) as count FROM auto_tasks WHERE status IN ('running', 'pending', 'in_progress')")
                .get_result::<CountResult>(&mut db_conn)
                .map(|r| r.count)
                .unwrap_or(0);

        let completed: i64 =
            diesel::sql_query("SELECT COUNT(*) as count FROM auto_tasks WHERE status IN ('done', 'completed')")
                .get_result::<CountResult>(&mut db_conn)
                .map(|r| r.count)
                .unwrap_or(0);

        let awaiting: i64 =
            diesel::sql_query("SELECT COUNT(*) as count FROM auto_tasks WHERE status IN ('awaiting_decision', 'awaiting', 'waiting')")
                .get_result::<CountResult>(&mut db_conn)
                .map(|r| r.count)
                .unwrap_or(0);

        let paused: i64 =
            diesel::sql_query("SELECT COUNT(*) as count FROM auto_tasks WHERE status = 'paused'")
                .get_result::<CountResult>(&mut db_conn)
                .map(|r| r.count)
                .unwrap_or(0);

        let blocked: i64 =
            diesel::sql_query("SELECT COUNT(*) as count FROM auto_tasks WHERE status IN ('blocked', 'failed', 'error')")
                .get_result::<CountResult>(&mut db_conn)
                .map(|r| r.count)
                .unwrap_or(0);

        let priority: i64 =
            diesel::sql_query("SELECT COUNT(*) as count FROM auto_tasks WHERE priority IN ('high', 'urgent')")
                .get_result::<CountResult>(&mut db_conn)
                .map(|r| r.count)
                .unwrap_or(0);

        let time_saved = format!("{}h", completed * 2);

        Ok::<_, String>(TaskStats {
            total: total as usize,
            active: active as usize,
            completed: completed as usize,
            awaiting: awaiting as usize,
            paused: paused as usize,
            blocked: blocked as usize,
            priority: priority as usize,
            time_saved,
        })
    })
    .await
    .unwrap_or_else(|e| {
        log::error!("Stats query failed: {}", e);
        Err(format!("Stats query failed: {}", e))
    })
    .unwrap_or(TaskStats {
        total: 0,
        active: 0,
        completed: 0,
        awaiting: 0,
        paused: 0,
        blocked: 0,
        priority: 0,
        time_saved: "0h".to_string(),
    });

    Json(stats)
}

pub async fn handle_time_saved(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let conn = state.conn.clone();

    let time_saved = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(_) => return "0h".to_string(),
        };

        let completed: i64 =
            diesel::sql_query("SELECT COUNT(*) as count FROM auto_tasks WHERE status IN ('done', 'completed')")
                .get_result::<CountResult>(&mut db_conn)
                .map(|r| r.count)
                .unwrap_or(0);

        format!("{}h", completed * 2)
    })
    .await
    .unwrap_or_else(|_| "0h".to_string());

    axum::response::Html(format!(
        r#"<span class="time-label">Active Time Saved:</span>
        <span class="time-value" id="time-saved-value">{}</span>"#,
        time_saved
    ))
}

pub async fn handle_clear_completed(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let conn = state.conn.clone();

    tokio::task::spawn_blocking(move || {
        let mut db_conn = conn
            .get()
            .map_err(|e| format!("DB connection error: {}", e))?;

        diesel::sql_query("DELETE FROM auto_tasks WHERE status IN ('done', 'completed')")
            .execute(&mut db_conn)
            .map_err(|e| format!("Delete failed: {}", e))?;

        Ok::<_, String>(())
    })
    .await
    .unwrap_or_else(|e| {
        log::error!("Clear completed failed: {}", e);
        Err(format!("Clear completed failed: {}", e))
    })
    .ok();

    log::info!("Cleared completed tasks");

    handle_task_list_htmx(State(state), Query(std::collections::HashMap::new())).await
}

pub async fn handle_task_patch(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(update): Json<TaskPatch>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, String)> {
    log::info!("Updating task {} with {:?}", id, update);

    let conn = state.conn.clone();
    let task_id = id
        .parse::<Uuid>()
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid task ID: {}", e)))?;

    tokio::task::spawn_blocking(move || {
        let mut db_conn = conn
            .get()
            .map_err(|e| format!("DB connection error: {}", e))?;

        if let Some(completed) = update.completed {
            diesel::sql_query("UPDATE tasks SET completed = $1 WHERE id = $2")
                .bind::<diesel::sql_types::Bool, _>(completed)
                .bind::<diesel::sql_types::Uuid, _>(task_id)
                .execute(&mut db_conn)
                .map_err(|e| format!("Update failed: {}", e))?;
        }

        if let Some(priority) = update.priority {
            diesel::sql_query("UPDATE tasks SET priority = $1 WHERE id = $2")
                .bind::<diesel::sql_types::Bool, _>(priority)
                .bind::<diesel::sql_types::Uuid, _>(task_id)
                .execute(&mut db_conn)
                .map_err(|e| format!("Update failed: {}", e))?;
        }

        if let Some(text) = update.text {
            diesel::sql_query("UPDATE tasks SET title = $1 WHERE id = $2")
                .bind::<diesel::sql_types::Text, _>(text)
                .bind::<diesel::sql_types::Uuid, _>(task_id)
                .execute(&mut db_conn)
                .map_err(|e| format!("Update failed: {}", e))?;
        }

        Ok::<_, String>(())
    })
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Task join error: {}", e),
        )
    })?
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(()),
        message: Some("Task updated".to_string()),
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskStats {
    pub total: usize,
    pub active: usize,
    pub completed: usize,
    pub awaiting: usize,
    pub paused: usize,
    pub blocked: usize,
    pub priority: usize,
    pub time_saved: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskPatch {
    pub completed: Option<bool>,
    pub priority: Option<bool>,
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

#[derive(Debug, QueryableByName)]
struct TaskRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub title: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub status: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub priority: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    #[allow(dead_code)]
    pub description: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
    pub due_date: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, QueryableByName)]
struct CountResult {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub count: i64,
}
