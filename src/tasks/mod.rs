pub mod scheduler;

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
) -> impl IntoResponse {
    log::info!("[TASK_GET] *** Handler called for task: {} ***", id);

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
            "SELECT id, title, status, priority, intent, error, progress, current_step, total_steps, step_results, created_at, started_at, completed_at
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
            log::info!("[TASK_GET] Returning task: {} - {}", task.id, task.title);
            let status_class = match task.status.as_str() {
                "completed" | "done" => "completed",
                "running" | "pending" => "running",
                "failed" | "error" => "error",
                _ => "pending"
            };
            let progress_percent = (task.progress * 100.0) as u8;
            let created = task.created_at.format("%Y-%m-%d %H:%M").to_string();

            // Calculate runtime
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
            let intent_text = task.intent.clone().unwrap_or_else(|| task.title.clone());
            let error_html = task.error.clone().map(|e| format!(
                r#"<div class="error-alert">
                    <span class="error-icon">⚠</span>
                    <span class="error-text">{}</span>
                </div>"#, e
            )).unwrap_or_default();

            let current_step = task.current_step;
            let total_steps = if task.total_steps > 0 { task.total_steps } else { 1 };

            let status_label = match task.status.as_str() {
                "completed" | "done" => "Completed",
                "running" => "Running",
                "pending" => "Pending",
                "failed" | "error" => "Failed",
                "paused" => "Paused",
                "waiting_approval" => "Awaiting Approval",
                _ => &task.status
            };

            // Build progress log HTML from step_results
            let progress_log_html = build_progress_log_html(&task.step_results, current_step, total_steps);

            // Build terminal output from recent activity
            let terminal_html = build_terminal_html(&task.step_results, &task.status);

            let html = format!(r#"
                <div class="task-detail-rich" data-task-id="{task_id}">
                    <!-- Header with title and status badge -->
                    <div class="detail-header-rich">
                        <h2 class="detail-title-rich">{title}</h2>
                        <span class="status-badge-rich status-{status_class}">{status_label}</span>
                    </div>

                    <!-- Status Section -->
                    <div class="detail-section-box status-section">
                        <div class="section-label">STATUS</div>
                        <div class="status-content">
                            <div class="status-main">
                                <span class="status-dot status-{status_class}"></span>
                                <span class="status-text">{title}</span>
                            </div>
                            <div class="status-meta">
                                <span class="meta-runtime">Runtime: {runtime}</span>
                                <span class="meta-estimated">Step {current_step}/{total_steps}</span>
                            </div>
                        </div>
                        {error_html}
                        <div class="status-details">
                            <div class="status-row">
                                <span class="status-indicator {status_indicator}"></span>
                                <span class="status-step-name">{status_label} (Step {current_step}/{total_steps})</span>
                                <span class="status-step-note">{priority} priority</span>
                            </div>
                        </div>
                    </div>

                    <!-- Progress Bar -->
                    <div class="detail-progress-rich">
                        <div class="progress-bar-rich">
                            <div class="progress-fill-rich" style="width: {progress_percent}%"></div>
                        </div>
                        <div class="progress-info-rich">
                            <span class="progress-label-rich">Progress: {progress_percent}%</span>
                        </div>
                    </div>

                    <!-- Progress Log Section -->
                    <div class="detail-section-box progress-log-section">
                        <div class="section-label">PROGRESS LOG</div>
                        <div class="progress-log-content" id="progress-log-{task_id}">
                            {progress_log_html}
                        </div>
                    </div>

                    <!-- Terminal Section -->
                    <div class="detail-section-box terminal-section-rich">
                        <div class="section-header-rich">
                            <div class="section-label">
                                <span class="terminal-dot-rich {terminal_active}"></span>
                                TERMINAL (LIVE AGENT ACTIVITY)
                            </div>
                            <div class="terminal-stats-rich">
                                <span>Step: <strong>{current_step}</strong> of <strong>{total_steps}</strong></span>
                            </div>
                        </div>
                        <div class="terminal-output-rich" id="terminal-output-{task_id}">
                            {terminal_html}
                        </div>
                        <div class="terminal-footer-rich">
                            <span class="terminal-eta">Started: <strong>{created}</strong></span>
                        </div>
                    </div>

                    <!-- Intent Section -->
                    <div class="detail-section-box intent-section">
                        <div class="section-label">INTENT</div>
                        <p class="intent-text-rich">{intent_text}</p>
                    </div>

                    <!-- Actions -->
                    <div class="detail-actions-rich">
                        <button class="btn-action-rich btn-pause" onclick="pauseTask('{task_id}')">
                            <span class="btn-icon">⏸</span> Pause
                        </button>
                        <button class="btn-action-rich btn-cancel" onclick="cancelTask('{task_id}')">
                            <span class="btn-icon">✗</span> Cancel
                        </button>
                        <button class="btn-action-rich btn-detailed" onclick="showDetailedView('{task_id}')">
                            Detailed View
                        </button>
                    </div>
                </div>
            "#,
                task_id = task_id,
                title = task.title,
                status_class = status_class,
                status_label = status_label,
                runtime = runtime,
                current_step = current_step,
                total_steps = total_steps,
                error_html = error_html,
                status_indicator = if task.status == "running" { "active" } else { "" },
                priority = task.priority,
                progress_percent = progress_percent,
                progress_log_html = progress_log_html,
                terminal_active = if task.status == "running" { "active" } else { "" },
                terminal_html = terminal_html,
                created = created,
                intent_text = intent_text,
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

/// Build HTML for the progress log section from step_results JSON
fn build_progress_log_html(step_results: &Option<serde_json::Value>, current_step: i32, total_steps: i32) -> String {
    let mut html = String::new();

    if let Some(serde_json::Value::Array(steps)) = step_results {
        if steps.is_empty() {
            // No steps yet - show current status
            html.push_str(&format!(r#"
                <div class="log-group">
                    <div class="log-group-header">
                        <span class="log-group-name">Task Execution</span>
                        <span class="log-step-badge">Step {}/{}</span>
                        <span class="log-status-badge running">In Progress</span>
                    </div>
                    <div class="log-group-items">
                        <div class="log-item">
                            <span class="log-dot running"></span>
                            <span class="log-item-name">Waiting for execution steps...</span>
                        </div>
                    </div>
                </div>
            "#, current_step, total_steps));
        } else {
            // Group steps and show real data
            for (idx, step) in steps.iter().enumerate() {
                let step_name = step.get("step_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Step");
                let step_status = step.get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("pending");
                let step_order = step.get("step_order")
                    .and_then(|v| v.as_i64())
                    .unwrap_or((idx + 1) as i64);
                let duration_ms = step.get("duration_ms")
                    .and_then(|v| v.as_i64());

                let status_class = match step_status {
                    "completed" | "Completed" => "completed",
                    "running" | "Running" => "running",
                    "failed" | "Failed" => "failed",
                    _ => "pending"
                };

                let duration_str = duration_ms.map(|ms| {
                    if ms > 60000 {
                        format!("{}m {}s", ms / 60000, (ms % 60000) / 1000)
                    } else if ms > 1000 {
                        format!("{}s", ms / 1000)
                    } else {
                        format!("{}ms", ms)
                    }
                }).unwrap_or_else(|| "--".to_string());

                html.push_str(&format!(r#"
                    <div class="log-item">
                        <span class="log-dot {status_class}"></span>
                        <span class="log-item-name">{step_name}</span>
                        <span class="log-item-badge">Step {step_order}/{total_steps}</span>
                        <span class="log-item-status">{step_status}</span>
                        <span class="log-duration">Duration: {duration_str}</span>
                    </div>
                "#,
                    status_class = status_class,
                    step_name = step_name,
                    step_order = step_order,
                    total_steps = total_steps,
                    step_status = step_status,
                    duration_str = duration_str,
                ));

                // Show logs if present
                if let Some(serde_json::Value::Array(logs)) = step.get("logs") {
                    for log_entry in logs.iter().take(3) {
                        let msg = log_entry.get("message")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        if !msg.is_empty() {
                            html.push_str(&format!(r#"
                                <div class="log-subitem">
                                    <span class="log-subdot {status_class}"></span>
                                    <span class="log-subitem-name">{msg}</span>
                                </div>
                            "#, status_class = status_class, msg = msg));
                        }
                    }
                }
            }
        }
    } else {
        // No step results - show placeholder based on current progress
        html.push_str(&format!(r#"
            <div class="log-group">
                <div class="log-group-header">
                    <span class="log-group-name">Task Progress</span>
                    <span class="log-step-badge">Step {}/{}</span>
                    <span class="log-status-badge pending">Pending</span>
                </div>
                <div class="log-group-items">
                    <div class="log-item">
                        <span class="log-dot pending"></span>
                        <span class="log-item-name">No execution steps recorded yet</span>
                    </div>
                </div>
            </div>
        "#, current_step, total_steps));
    }

    html
}

/// Build HTML for terminal output from step results
fn build_terminal_html(step_results: &Option<serde_json::Value>, status: &str) -> String {
    let mut html = String::new();
    let mut lines: Vec<String> = Vec::new();

    if let Some(serde_json::Value::Array(steps)) = step_results {
        for step in steps.iter() {
            // Add step name as a line
            if let Some(step_name) = step.get("step_name").and_then(|v| v.as_str()) {
                let step_status = step.get("status").and_then(|v| v.as_str()).unwrap_or("");
                let prefix = match step_status {
                    "completed" | "Completed" => "✓",
                    "running" | "Running" => "►",
                    "failed" | "Failed" => "✗",
                    _ => "○"
                };
                lines.push(format!("{} {}", prefix, step_name));
            }

            // Add log messages
            if let Some(serde_json::Value::Array(logs)) = step.get("logs") {
                for log_entry in logs.iter() {
                    if let Some(msg) = log_entry.get("message").and_then(|v| v.as_str()) {
                        lines.push(format!("  {}", msg));
                    }
                }
            }
        }
    }

    if lines.is_empty() {
        // Show default message based on status
        let default_msg = match status {
            "running" => "Task is running...",
            "pending" => "Waiting to start...",
            "completed" | "done" => "Task completed successfully",
            "failed" | "error" => "Task failed - check error details",
            "paused" => "Task is paused",
            _ => "Initializing..."
        };
        html.push_str(&format!(r#"<div class="terminal-line current">{}</div>"#, default_msg));
    } else {
        // Show last 10 lines, with the last one marked as current
        let start = if lines.len() > 10 { lines.len() - 10 } else { 0 };
        for (idx, line) in lines[start..].iter().enumerate() {
            let is_last = idx == lines[start..].len() - 1;
            let class = if is_last && status == "running" { "terminal-line current" } else { "terminal-line" };
            html.push_str(&format!(r#"<div class="{}">{}</div>"#, class, line));
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

        let _ = write!(
            html,
            r#"
            <div class="task-card {completed_class} {status_class}" data-task-id="{}" onclick="selectTask('{}')">
                <div class="task-card-header">
                    <span class="task-card-title">{}</span>
                    <span class="task-card-status {}">{}</span>
                </div>
                <div class="task-card-body">
                    <div class="task-card-priority">
                        <span class="priority-badge priority-{}">{}</span>
                    </div>
                    {due_date_html}
                </div>
                <div class="task-card-footer">
                    <button class="task-action-btn" data-action="priority" data-task-id="{}" onclick="event.stopPropagation()">
                        ⭐
                    </button>
                    <button class="task-action-btn" data-action="delete" data-task-id="{}" onclick="event.stopPropagation()">
                        🗑️
                    </button>
                </div>
            </div>
            "#,
            task.id,
            task.id,
            task.title,
            status_class,
            task.status,
            task.priority,
            task.priority,
            task.id,
            task.id
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
