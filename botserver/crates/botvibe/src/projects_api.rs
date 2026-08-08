//! #743 — REST surface for the Vibe project registry. Handlers are thin:
//! validation + delegation to `ProjectRegistry`; errors are sanitized.

use crate::projects::{
    CreateProjectRequest, ListProjectsQuery, Project, ProjectRegistryRef, UpdateProjectRequest,
};
use axum::{
    extract::{Path, Query},
    Json,
};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct ProjectResponse {
    pub success: bool,
    pub project: Option<Project>,
    pub projects: Option<Vec<Project>>,
    pub error: Option<String>,
}

fn ok_project(p: Project) -> Json<ProjectResponse> {
    Json(ProjectResponse {
        success: true,
        project: Some(p),
        projects: None,
        error: None,
    })
}

fn ok_projects(list: Vec<Project>) -> Json<ProjectResponse> {
    Json(ProjectResponse {
        success: true,
        project: None,
        projects: Some(list),
        error: None,
    })
}

fn err_response(msg: String) -> Json<ProjectResponse> {
    log::error!("Vibe projects API error: {msg}");
    Json(ProjectResponse {
        success: false,
        project: None,
        projects: None,
        error: Some(msg),
    })
}

async fn create_project(
    axum::extract::Extension(registry): axum::extract::Extension<ProjectRegistryRef>,
    Json(req): Json<CreateProjectRequest>,
) -> Json<ProjectResponse> {
    if req.name.trim().is_empty() {
        return err_response("project name must not be empty".into());
    }
    match registry.create(&req) {
        Ok(p) => ok_project(p),
        Err(e) => err_response(e),
    }
}

async fn delete_project(
    axum::extract::Extension(registry): axum::extract::Extension<ProjectRegistryRef>,
    Path(id): Path<Uuid>,
) -> Json<ProjectResponse> {
    match registry.delete(id) {
        Ok(true) => Json(ProjectResponse {
            success: true,
            project: None,
            projects: None,
            error: None,
        }),
        Ok(false) => err_response(format!("project {id} not found")),
        Err(e) => err_response(e),
    }
}

async fn update_project(
    axum::extract::Extension(registry): axum::extract::Extension<ProjectRegistryRef>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateProjectRequest>,
) -> Json<ProjectResponse> {
    match registry.update(id, &req) {
        Ok(true) => match registry.get(id) {
            Ok(Some(p)) => ok_project(p),
            Ok(None) => err_response(format!("project {id} not found")),
            Err(e) => err_response(e),
        },
        Ok(false) => err_response(format!("project {id} not found or no changes")),
        Err(e) => err_response(e),
    }
}

async fn list_projects(
    axum::extract::Extension(registry): axum::extract::Extension<ProjectRegistryRef>,
    Query(query): Query<ListProjectsQuery>,
) -> Json<ProjectResponse> {
    match registry.list(&query) {
        Ok(list) => ok_projects(list),
        Err(e) => err_response(e),
    }
}

async fn get_project(
    axum::extract::Extension(registry): axum::extract::Extension<ProjectRegistryRef>,
    Path(id): Path<Uuid>,
) -> Json<ProjectResponse> {
    match registry.get(id) {
        Ok(Some(p)) => ok_project(p),
        Ok(None) => err_response(format!("project {id} not found")),
        Err(e) => err_response(e),
    }
}

pub fn projects_router(registry: ProjectRegistryRef) -> axum::Router {
    use axum::routing::{delete, get, post, put};
    axum::Router::new()
        .route("/api/vibe/projects", post(create_project))
        .route("/api/vibe/projects", get(list_projects))
        .route("/api/vibe/projects/:project_id", get(get_project))
        .route("/api/vibe/projects/:project_id", put(update_project))
        .route("/api/vibe/projects/:project_id", delete(delete_project))
        .layer(axum::Extension(registry))
}