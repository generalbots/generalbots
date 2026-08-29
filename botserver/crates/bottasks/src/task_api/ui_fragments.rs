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
use chrono::{DateTime, Duration, Utc};
use diesel::prelude::*;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::state::TasksState;

#[derive(Deserialize)]
pub struct UiListQuery {
    pub filter: Option<String>,
    pub stage: Option<String>,
    pub priority: Option<String>,
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
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
    parent_id: Option<Uuid>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
    due_date: Option<DateTime<Utc>>,
}

#[derive(diesel::QueryableByName)]
struct StatusCountRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    s: String,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    n: i64,
}

#[derive(diesel::QueryableByName)]
struct AtRiskRow {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    overdue: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    due_soon: i64,
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

/// Reorders a flat task list so parents come first and their subtasks
/// (rows with `parent_id`) follow immediately beneath them (issue #878).
/// Subtasks whose parent is not present in the loaded page are appended at
/// the end so they are never dropped.
fn order_tasks_with_subtasks(rows: Vec<TaskRow>) -> Vec<TaskRow> {
    use std::collections::HashMap;

    let mut children: HashMap<Uuid, Vec<TaskRow>> = HashMap::new();
    let mut roots: Vec<TaskRow> = Vec::new();
    for row in rows {
        match row.parent_id {
            Some(pid) => children.entry(pid).or_default().push(row),
            None => roots.push(row),
        }
    }

    let child_count: usize = children.values().map(Vec::len).sum();
    let mut ordered: Vec<TaskRow> = Vec::with_capacity(roots.len() + child_count);
    for root in roots {
        let root_id = root.id;
        ordered.push(root);
        if let Some(kids) = children.remove(&root_id) {
            ordered.extend(kids);
        }
    }

    // Orphaned subtasks (parent outside the loaded page).
    let mut leftover: Vec<TaskRow> = children.into_values().flatten().collect();
    ordered.append(&mut leftover);
    ordered
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
        let not_done = "status NOT IN ('done','complete','completed','resolved')";
        let mut sql = String::from(
            "SELECT id, title, description, status, priority, progress, parent_id, due_date \
             FROM tasks WHERE branch_id = $1",
        );
        match filter {
            "complete" | "completed" | "done" => sql.push_str(" AND status IN ('done','complete','completed','resolved')"),
            "active" | "running" | "in_progress" => sql.push_str(" AND status IN ('active','running','in_progress','in-progress')"),
            "awaiting" | "blocked" => sql.push_str(" AND status IN ('awaiting','blocked','pending_approval')"),
            "paused" => sql.push_str(" AND status = 'paused'"),
            "overdue" => sql.push_str(&format!(" AND {not_done} AND due_date IS NOT NULL AND due_date < NOW()")),
            "due-soon" | "due_soon" | "due soon" => {
                sql.push_str(&format!(" AND {not_done} AND due_date IS NOT NULL AND due_date >= NOW() AND due_date <= NOW() + INTERVAL '24 hours'"))
            }
            _ => {}
        }
        // Stage filter (pipeline tabs: plan/build/review/deploy/monitor).
        if let Some(stage) = query.stage.as_deref() {
            let validated = match stage {
                "plan" | "build" | "review" | "deploy" | "monitor" => stage,
                _ => "",
            };
            if !validated.is_empty() {
                sql.push_str(&format!(" AND stage = '{validated}'"));
            }
        }
        // Priority filter (low/medium/high/urgent).
        if let Some(priority) = query.priority.as_deref() {
            let validated = match priority {
                "low" | "medium" | "high" | "urgent" => priority,
                _ => "",
            };
            if !validated.is_empty() {
                sql.push_str(&format!(" AND priority = '{validated}'"));
            }
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

    // Order parents first, subtasks indented beneath them (issue #878).
    let result = order_tasks_with_subtasks(result);

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
        let done = matches!(
            task.status.as_str(),
            "done" | "complete" | "completed" | "resolved"
        );
        let now = Utc::now();
        let overdue = !done && task.due_date.map(|d| d < now).unwrap_or(false);
        let due_soon = !done
            && !overdue
            && task
                .due_date
                .map(|d| d <= now + Duration::hours(24))
                .unwrap_or(false);
        let is_subtask = task.parent_id.is_some();
        let mut card_class = if is_subtask {
            format!("task-card task-card-subtask {cls}")
        } else {
            format!("task-card {cls}")
        };
        if overdue {
            card_class.push_str(" overdue");
        } else if due_soon {
            card_class.push_str(" due-soon");
        }
        let due_badge = if overdue {
            "<span class=\"task-due-badge overdue\">Overdue</span>".to_string()
        } else if due_soon {
            "<span class=\"task-due-badge due-soon\">Due soon</span>".to_string()
        } else {
            String::new()
        };
        let parent_attr = task
            .parent_id
            .map(|pid| format!(" data-parent-id=\"{pid}\""))
            .unwrap_or_default();
        let complete_action = if done {
            format!(
                r#"<button class="task-action-btn" title="Reopen" onclick="event.stopPropagation(); reopenTask('{id}')">↩</button>"#,
                id = task.id
            )
        } else {
            format!(
                r#"<button class="task-action-btn" title="Complete" onclick="event.stopPropagation(); completeTask('{id}')">✓</button>"#,
                id = task.id
            )
        };
        html.push_str(&format!(
            r#"<div class="{card_class}" data-task-id="{id}"{parent_attr} onclick="selectTask('{id}')">
  <div class="task-card-header">
    <span class="task-card-icon">📋</span>
    <span class="task-card-title">{title}</span>
  </div>
  <div class="task-card-meta">
    <span class="task-card-status {cls}">{status_text}</span>
    <span class="task-card-priority">{priority}</span>
    {due_badge}
  </div>
  <div class="task-card-progress">
    <div class="task-progress-bar"><div class="task-progress-fill" style="width:{progress}%"></div></div>
    <span class="task-progress-percent">{progress}%</span>
  </div>
  <div class="task-card-actions">
    <button class="task-action-btn" title="Edit" onclick="event.stopPropagation(); editTask('{id}')">✏️</button>
    {complete_action}
    <button class="task-action-btn" title="Delete" onclick="event.stopPropagation(); deleteTask('{id}')">🗑</button>
  </div>
</div>"#,
            id = task.id,
            card_class = card_class,
            parent_attr = parent_attr,
            cls = cls,
            title = title,
            status_text = status_text,
            priority = priority,
            due_badge = due_badge,
            progress = progress,
            complete_action = complete_action,
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

        let at_risk = diesel::sql_query(
            "SELECT \
               count(*) FILTER (WHERE status NOT IN ('done','complete','completed','resolved') AND due_date IS NOT NULL AND due_date < NOW()) AS overdue, \
               count(*) FILTER (WHERE status NOT IN ('done','complete','completed','resolved') AND due_date IS NOT NULL AND due_date >= NOW() AND due_date <= NOW() + INTERVAL '24 hours') AS due_soon \
             FROM tasks WHERE branch_id = $1",
        )
        .bind::<diesel::sql_types::Uuid, _>(branch_id)
        .get_result::<AtRiskRow>(&mut conn)
        .map_err(|e| e.to_string())?;

        Ok::<(i64, i64, i64, i64, i64), String>((
            total,
            completed,
            active,
            at_risk.overdue,
            at_risk.due_soon,
        ))
    })
    .await
    .ok()
    .and_then(Result::ok)
    .unwrap_or((0, 0, 0, 0, 0));

    let (total, completed, active, overdue, due_soon) = result;
    Html(format!(
        r#"<div class="task-stats">
  <span class="stat"><b>{total}</b> all</span>
  <span class="stat"><b>{active}</b> active</span>
  <span class="stat"><b>{completed}</b> done</span>
  <span class="stat stat-overdue"><b>{overdue}</b> overdue</span>
  <span class="stat stat-due-soon"><b>{due_soon}</b> due soon</span>
</div>"#
    ))
    .into_response()
}
