use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Html,
    routing::get,
    Router,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use super::ProjectService;

#[derive(Deserialize)]
pub struct TaskFilterParams {
    status: Option<String>,
}

pub fn configure_project_ui_routes() -> Router<Arc<ProjectService>> {
    Router::new()
        .route("/api/ui/project/new", get(project_new_form))
        .route("/api/ui/project/task/new", get(task_new_form))
        .route("/api/ui/project/tasks", get(project_tasks_fragment))
        .route("/api/ui/project/gantt", get(project_gantt_fragment))
        .route("/api/ui/project/timeline", get(project_timeline_fragment))
        .route("/api/ui/project/tasks/list", get(project_tasks_list_fragment))
}

async fn project_new_form() -> Html<&'static str> {
    // The create_project handler expects a JSON body (name, description,
    // start_date, end_date), so this form submits via fetch instead of a
    // plain HTML form (which would send form-encoded data to the wrong URL).
    Html(r##"<div class="project-form"><h3>New Project</h3><div class="form-group"><label>Project Name</label><input type="text" id="new-project-name" class="form-input" required placeholder="Enter project name"></div><div class="form-group"><label>Description</label><textarea id="new-project-description" class="form-textarea" rows="3" placeholder="Project description..."></textarea></div><div class="form-group"><label>Start Date</label><input type="date" id="new-project-start" class="form-input"></div><div class="form-group"><label>End Date</label><input type="date" id="new-project-end" class="form-input"></div><div class="form-actions"><button type="button" class="form-btn secondary" onclick="closeProjectModal()">Cancel</button><button type="button" class="form-btn primary" onclick="createProjectFromForm()">Create Project</button></div></div>"##)
}

async fn task_new_form() -> Html<&'static str> {
    Html(r##"<div class="task-form"><h3>New Task</h3><form hx-post="/projects/{project_id}/tasks" hx-target="#gantt-table-body" hx-swap="innerHTML" hx-on::after-request="if(event.detail.successful){closeTaskModal();htmx.trigger('body','projectSelected')}"><div class="form-group"><label>Task Name</label><input type="text" name="name" class="form-input" required placeholder="Enter task name"></div><div class="form-row"><div class="form-group"><label>Start Date</label><input type="date" name="start_date" class="form-input"></div><div class="form-group"><label>End Date</label><input type="date" name="end_date" class="form-input"></div></div><div class="form-group"><label>Progress (%)</label><input type="number" name="percent_complete" class="form-input" value="0" min="0" max="100"></div><div class="form-actions"><button type="button" class="form-btn secondary" onclick="closeTaskModal()">Cancel</button><button type="submit" class="form-btn primary">Create Task</button></div></form></div>"##)
}

async fn project_tasks_fragment(
    State(service): State<Arc<ProjectService>>,
    Query(params): Query<TaskFilterParams>,
) -> Result<Html<String>, (StatusCode, String)> {
    let projects = service.get_projects_for_organization(Uuid::nil()).await;
    if projects.is_empty() {
        return Ok(Html("<tr><td colspan='6'>No projects available</td></tr>".to_string()));
    }
    let project_id = projects[0].id;
    let tasks = service.get_tasks_for_project(project_id).await;
    let rows: String = tasks.iter().map(|t| {
        let status_label = match t.percent_complete {
            0..=10 => "Not Started",
            11..=90 => "In Progress",
            _ => "Completed",
        };
        let status_filter = params.status.as_deref().unwrap_or("");
        if !status_filter.is_empty() && status_label != status_filter {
            return String::new();
        }
        let assignees = t
            .assigned_to
            .iter()
            .map(|u| u.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}%</td><td>{}</td><td><button class='card-btn' onclick='selectTask(\"{}\")' title='Details'><svg width='14' height='14' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2'><circle cx='12' cy='12' r='10'/><line x1='12' y1='16' x2='12' y2='12'/><line x1='12' y1='8' x2='12.01' y2='8'/></svg></button></td></tr>",
            t.name, t.start_date, t.end_date, t.percent_complete, assignees, t.id
        )
    }).collect();
    Ok(Html(rows))
}

async fn project_gantt_fragment(
    State(service): State<Arc<ProjectService>>,
) -> Result<Html<String>, (StatusCode, String)> {
    let projects = service.get_projects_for_organization(Uuid::nil()).await;
    if projects.is_empty() {
        return Ok(Html("<div class='gantt-empty'>Select a project to view Gantt chart</div>".to_string()));
    }
    let project_id = projects[0].id;
    match service.get_gantt_chart_data(project_id).await {
        Some(data) => {
            let bars: String = data.tasks.iter().map(|t| {
                let pct = t.percent_complete;
                format!(
                    "<div class='gantt-bar' style='width:{}%;background:var(--accent,#3b82f6);height:24px;border-radius:4px;margin:2px 0' title='{}: {}%'></div>",
                    pct, t.name, pct
                )
            }).collect();
            Ok(Html(format!("<div class='gantt-chart-data'>{}</div>", bars)))
        }
        None => Ok(Html("<div class='gantt-empty'>No Gantt data available</div>".to_string())),
    }
}

async fn project_timeline_fragment(
    State(service): State<Arc<ProjectService>>,
) -> Result<Html<String>, (StatusCode, String)> {
    let projects = service.get_projects_for_organization(Uuid::nil()).await;
    if projects.is_empty() {
        return Ok(Html("<div class='timeline-empty'>Select a project to view timeline</div>".to_string()));
    }
    let project_id = projects[0].id;
    match service.get_timeline_view(project_id).await {
        Some(view) => {
            let items: String = view.items.iter().map(|t| {
                format!(
                    "<div class='timeline-item'><span class='timeline-date'>{}</span><span class='timeline-title'>{}</span><span class='timeline-status'>{}</span></div>",
                    t.start_date,
                    t.name,
                    t.percent_complete
                )
            }).collect();
            Ok(Html(format!("<div class='timeline-view'>{}</div>", items)))
        }
        None => Ok(Html("<div class='timeline-empty'>No timeline data available</div>".to_string())),
    }
}

async fn project_tasks_list_fragment(
    State(service): State<Arc<ProjectService>>,
) -> Result<Html<String>, (StatusCode, String)> {
    let projects = service.get_projects_for_organization(Uuid::nil()).await;
    if projects.is_empty() {
        return Ok(Html("<div class='list-empty'>No tasks available</div>".to_string()));
    }
    let project_id = projects[0].id;
    let tasks = service.get_tasks_for_project(project_id).await;
    let items: String = tasks.iter().map(|t| {
        let assignees = t
            .assigned_to
            .iter()
            .map(|u| u.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "<div class='list-item'><span class='list-title'>{}</span><span class='list-progress'>{}%</span><span class='list-assignee'>{}</span></div>",
            t.name, t.percent_complete, assignees
        )
    }).collect();
    Ok(Html(format!("<div class='tasks-list'>{}</div>", items)))
}
