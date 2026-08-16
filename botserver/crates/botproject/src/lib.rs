pub mod import;
mod types;
mod service;
mod analysis;
mod handlers;

pub use types::*;
pub use service::ProjectService;

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;

use crate::handlers::*;

pub fn configure(service: Arc<ProjectService>) -> Router<Arc<ProjectService>> {
    Router::new()
        .route("/projects", get(list_projects).post(create_project))
        .route(
            "/projects/:project_id",
            get(get_project).put(update_project),
        )
        .route("/projects/:project_id/status", post(update_project_status))
        .route("/projects/:project_id", delete(delete_project))
        .route("/projects/:project_id/tasks", post(create_task))
        .route("/projects/:project_id/tasks", get(get_tasks))
        .route("/projects/:project_id/gantt", get(get_gantt_chart))
        .route("/projects/:project_id/timeline", get(get_timeline))
        .route("/projects/:project_id/critical-path", get(get_critical_path))
        .route("/tasks/:task_id/progress", put(update_task_progress))
        .route("/tasks/:task_id/dependencies", post(add_dependency))
        .route(
            "/tasks/:task_id/dependencies/remove",
            post(remove_dependency),
        )
        .route("/tasks/:task_id/assign", post(assign_resource))
        .route("/tasks/:task_id/assignments", get(list_assignments))
        .route("/tasks/:task_id", delete(delete_task))
        .route("/tasks/:task_id", put(update_task))
        .route("/projects/:project_id/resources", post(create_resource))
        .route("/projects/:project_id/resources", get(list_resources))
        .route("/resources/:resource_id", delete(delete_resource))
        // `/api`-prefixed aliases so the frontend (proxied through botui
        // which routes `/api/*` to this server) can reach the same handlers.
        .route("/api/projects", get(list_projects).post(create_project))
        .route(
            "/api/projects/:project_id",
            get(get_project).put(update_project),
        )
        .route("/api/projects/:project_id/status", post(update_project_status))
        .route("/api/projects/:project_id", delete(delete_project))
        .route("/api/projects/:project_id/tasks", post(create_task))
        .route("/api/projects/:project_id/tasks", get(get_tasks))
        .route("/api/projects/:project_id/gantt", get(get_gantt_chart))
        .route("/api/projects/:project_id/timeline", get(get_timeline))
        .route("/api/projects/:project_id/critical-path", get(get_critical_path))
        .route("/api/tasks/:task_id/progress", put(update_task_progress))
        .route("/api/tasks/:task_id/dependencies", post(add_dependency))
        .route(
            "/api/tasks/:task_id/dependencies/remove",
            post(remove_dependency),
        )
        .route("/api/tasks/:task_id/assign", post(assign_resource))
        .route("/api/tasks/:task_id/assignments", get(list_assignments))
        .route("/api/tasks/:task_id", delete(delete_task))
        .route("/api/tasks/:task_id", put(update_task))
        .route("/api/projects/:project_id/resources", post(create_resource))
        .route("/api/projects/:project_id/resources", get(list_resources))
        .route("/api/resources/:resource_id", delete(delete_resource))
        .with_state(service)
}
