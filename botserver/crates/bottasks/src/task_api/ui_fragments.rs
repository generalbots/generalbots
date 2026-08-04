//! HTMX fragment endpoints for the Tasks app.
//!
//! The Tasks UI (`task-window.html`) loads its list via
//! `GET /api/ui/tasks?filter=all` and related routes. These handlers render
//! HTML fragments directly from the live `tasks` table so the app shows the
//! current task list without requiring a chat session.

use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse},
};
use diesel::prelude::*;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::state::TasksState;

#[derive(Deserialize)]
pub struct UiListQuery {
    pub filter: Option<String>,
}

#[derive(Deserialize)]
pub struct UiStatsQuery {
    pub branch_id: Option<String>,
}

#[derive(diesel::QueryableByName)]
struct TaskRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    title: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    description: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Text)]
    status: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    priority: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Integer>)]
    progress: Option<i32>,
}

#[derive(diesel::QueryableByName)]
struct StatusCountRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    s: String,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    n: i64,
}

#[derive(diesel::QueryableByName)]
struct UuidRowNamed {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
}

fn resolve_branch(conn: &mut diesel::PgConnection, hint: Option<String>) -> Uuid {
    if let Some(h) = hint {
        if let Ok(id) = Uuid::parse_str(&h) {
            return id;
        }
    }
    // The suite resolves scope to the default bot's branch in admin mode.
    // Mirror the contacts/tickets resolution: the first bot flagged
    // is_default_for_branch owns the branch used by suite apps.
    let bid: Option<Uuid> = diesel::sql_query(
        "SELECT branch_id AS id FROM bots WHERE is_default_for_branch = true ORDER BY created_at ASC LIMIT 1",
    )
    .get_result::<UuidRowNamed>(conn)
    .ok()
    .map(|r| r.id);
    bid.unwrap_or_else(Uuid::nil)
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn status_class(status: &str) -> &'static str {
    match status {
        "todo" => "todo",
        "in_progress" | "in-progress" | "active" | "running" => "in-progress",
        "done" | "complete" | "completed" | "resolved" => "complete",
        "paused" => "paused",
        "blocked" | "awaiting" => "blocked",
        _ => "todo",
    }
}

pub async fn handle_ui_tasks_list(
    State(state): State<Arc<TasksState>>,
    Query(query): Query<UiListQuery>,
) -> impl IntoResponse {
    let pool = state.pool.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| e.to_string())?;
        let branch_id = resolve_branch(&mut conn, None);

        let filter = query.filter.as_deref().unwrap_or("all");
        let mut sql = String::from(
            "SELECT id, title, description, status, priority, progress \
             FROM tasks WHERE branch_id = $1",
        );
        match filter {
            "complete" | "completed" | "done" => sql.push_str(" AND status IN ('done','complete','completed','resolved')"),
            "active" | "running" | "in_progress" => sql.push_str(" AND status IN ('active','running','in_progress','in-progress')"),
            "awaiting" | "blocked" => sql.push_str(" AND status IN ('awaiting','blocked','pending_approval')"),
            "paused" => sql.push_str(" AND status = 'paused'"),
            _ => {}
        }
        sql.push_str(" ORDER BY created_at DESC LIMIT 100");

        let rows: Vec<TaskRow> = diesel::sql_query(&sql)
            .bind::<diesel::sql_types::Uuid, _>(branch_id)
            .load(&mut conn)
            .map_err(|e| e.to_string())?;

        Ok::<Vec<TaskRow>, String>(rows)
    })
    .await
    .ok()
    .and_then(Result::ok)
    .unwrap_or_default();

    if result.is_empty() {
        return Html(
            r#"<div class="tw-task-list-empty">No tasks yet. Type an intent above and press RUN to create one.</div>"#
                .to_string(),
        )
        .into_response();
    }

    let mut html = String::new();
    for task in &result {
        let cls = status_class(&task.status);
        let title = html_escape(&task.title);
        let desc = task
            .description
            .as_deref()
            .map(html_escape)
            .unwrap_or_default();
        let progress = task.progress.unwrap_or(0);
        let status_text = html_escape(&task.status);
        let priority = html_escape(&task.priority);
        html.push_str(&format!(
            r#"<div class="task-card {cls}" data-task-id="{id}" onclick="selectTask('{id}')">
  <div class="task-card-header">
    <span class="task-card-icon">📋</span>
    <span class="task-card-title">{title}</span>
  </div>
  <div class="task-card-meta">
    <span class="task-card-status {cls}">{status_text}</span>
    <span class="task-card-priority">{priority}</span>
  </div>
  <div class="task-card-progress">
    <div class="task-progress-bar"><div class="task-progress-fill" style="width:{progress}%"></div></div>
    <span class="task-progress-percent">{progress}%</span>
  </div>
</div>"#,
            id = task.id,
            cls = cls,
            title = title,
            status_text = status_text,
            priority = priority,
            progress = progress,
        ));
        if !desc.is_empty() {
            html.push_str(&format!(
                r#"<div class="task-card-subtitle" style="padding:0 12px 8px;color:var(--text-secondary);font-size:12px">{}</div>"#,
                desc
            ));
        }
    }

    Html(html).into_response()
}

pub async fn handle_ui_tasks_stats(
    State(state): State<Arc<TasksState>>,
) -> impl IntoResponse {
    let pool = state.pool.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| e.to_string())?;
        let branch_id = resolve_branch(&mut conn, None);

        let counts: Vec<StatusCountRow> = diesel::sql_query(
            "SELECT status AS s, count(*) AS n FROM tasks WHERE branch_id = $1 GROUP BY status",
        )
        .bind::<diesel::sql_types::Uuid, _>(branch_id)
        .load(&mut conn)
        .map_err(|e| e.to_string())?;

        let total: i64 = counts.iter().map(|r| r.n).sum();
        let completed: i64 = counts
            .iter()
            .filter(|r| matches!(r.s.as_str(), "done" | "complete" | "completed" | "resolved"))
            .map(|r| r.n)
            .sum();
        let active: i64 = counts
            .iter()
            .filter(|r| matches!(r.s.as_str(), "active" | "running" | "in_progress" | "in-progress"))
            .map(|r| r.n)
            .sum();

        Ok::<(i64, i64, i64), String>((total, completed, active))
    })
    .await
    .ok()
    .and_then(Result::ok)
    .unwrap_or((0, 0, 0));

    let (total, completed, active) = result;
    Html(format!(
        r#"<div class="task-stats">
  <span class="stat"><b>{total}</b> all</span>
  <span class="stat"><b>{active}</b> active</span>
  <span class="stat"><b>{completed}</b> done</span>
</div>"#
    ))
    .into_response()
}
