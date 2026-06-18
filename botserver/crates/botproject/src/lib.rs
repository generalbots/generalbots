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
        .route("/projects/:project_id", get(get_project))
        .route("/projects/:project_id", delete(delete_project))
        .route("/projects/:project_id/tasks", post(create_task))
        .route("/projects/:project_id/tasks", get(get_tasks))
        .route("/projects/:project_id/gantt", get(get_gantt_chart))
        .route("/projects/:project_id/timeline", get(get_timeline))
        .route("/projects/:project_id/critical-path", get(get_critical_path))
        .route("/tasks/:task_id/progress", put(update_task_progress))
        .route("/tasks/:task_id/dependencies", post(add_dependency))
        .route("/tasks/:task_id", delete(delete_task))
        .with_state(service)
}
