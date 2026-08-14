//! #743 — REST surface for the Vibe project registry. Handlers are thin:
//! validation + delegation to `ProjectRegistry`; errors are sanitized.
//! #768 — every mutation is gated by per-project RBAC: create grants the
//! creator `owner`; update requires `developer+`; delete requires `owner`.

use std::sync::Arc;

use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use botsecurity_auth::auth_api::types::AuthenticatedUser;

use crate::harness;
use crate::metering::VMetering;
use crate::projects::{
    CreateProjectRequest, ListProjectsQuery, Project, ProjectRegistryRef, UpdateProjectRequest,
};
use crate::rbac::{ProjectRbac, ProjectRole};
use crate::vm_lifecycle::VmLifecycle;

#[derive(Debug, Serialize)]
pub struct ProjectResponse {
    pub success: bool,
    pub project: Option<Project>,
    pub projects: Option<Vec<Project>>,
    pub error: Option<String>,
}

type ApiResult = (StatusCode, Json<ProjectResponse>);

fn ok_project(p: Project) -> ApiResult {
    (
        StatusCode::OK,
        Json(ProjectResponse {
            success: true,
            project: Some(p),
            projects: None,
            error: None,
        }),
    )
}

fn ok_projects(list: Vec<Project>) -> ApiResult {
    (
        StatusCode::OK,
        Json(ProjectResponse {
            success: true,
            project: None,
            projects: Some(list),
            error: None,
        }),
    )
}

fn err_response(msg: String) -> ApiResult {
    log::error!("Vibe projects API error: {msg}");
    (
        StatusCode::OK,
        Json(ProjectResponse {
            success: false,
            project: None,
            projects: None,
            error: Some(msg),
        }),
    )
}

fn forbidden(msg: String) -> ApiResult {
    log::warn!("Vibe projects API forbidden: {msg}");
    (
        StatusCode::FORBIDDEN,
        Json(ProjectResponse {
            success: false,
            project: None,
            projects: None,
            error: Some(msg),
        }),
    )
}

fn deleted() -> ApiResult {
    (
        StatusCode::OK,
        Json(ProjectResponse {
            success: true,
            project: None,
            projects: None,
            error: None,
        }),
    )
}

async fn create_project(
    Extension(registry): Extension<ProjectRegistryRef>,
    Extension(rbac): Extension<ProjectRbac>,
    Extension(metering): Extension<Arc<VMetering>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<CreateProjectRequest>,
) -> ApiResult {
    if user.user_id.is_nil() {
        return forbidden("forbidden: anonymous users cannot create projects".into());
    }
    if req.name.trim().is_empty() {
        return err_response("project name must not be empty".into());
    }
    let org_id = req.org_id.unwrap_or_else(Uuid::nil);
    let branch_id = req.branch_id.unwrap_or_else(Uuid::nil);
    if let Err(e) = metering.enforce_project_creation(
        org_id,
        branch_id,
        req.project_type.as_deref().unwrap_or("bot"),
    ) {
        return forbidden(e);
    }
    match registry.create(&req) {
        Ok(p) => {
            if let Err(e) = rbac.set_user_role(p.id, user.user_id, ProjectRole::Owner) {
                log::error!("grant owner on project {} failed: {e}", p.id);
            }
            ok_project(p)
        }
        Err(e) => err_response(e),
    }
}

async fn delete_project(
    Extension(registry): Extension<ProjectRegistryRef>,
    Extension(rbac): Extension<ProjectRbac>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> ApiResult {
    match rbac.require_role(user.user_id, id, ProjectRole::Owner) {
        Ok(_) => {}
        Err(e) => return forbidden(e),
    }
    match registry.delete(id) {
        Ok(true) => deleted(),
        Ok(false) => err_response(format!("project {id} not found")),
        Err(e) => err_response(e),
    }
}

async fn update_project(
    Extension(registry): Extension<ProjectRegistryRef>,
    Extension(rbac): Extension<ProjectRbac>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateProjectRequest>,
) -> ApiResult {
    match rbac.require_role(user.user_id, id, ProjectRole::Developer) {
        Ok(_) => {}
        Err(e) => return forbidden(e),
    }
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
    Extension(registry): Extension<ProjectRegistryRef>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(query): Query<ListProjectsQuery>,
) -> ApiResult {
    if user.user_id.is_nil() {
        return forbidden("forbidden: anonymous users cannot list projects".into());
    }
    match registry.list(&query) {
        Ok(list) => ok_projects(list),
        Err(e) => err_response(e),
    }
}

async fn get_project(
    Extension(registry): Extension<ProjectRegistryRef>,
    Extension(rbac): Extension<ProjectRbac>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> ApiResult {
    match rbac.require_role(user.user_id, id, ProjectRole::Viewer) {
        Ok(_) => {}
        Err(e) => return forbidden(e),
    }
    match registry.get(id) {
        Ok(Some(p)) => ok_project(p),
        Ok(None) => err_response(format!("project {id} not found")),
        Err(e) => err_response(e),
    }
}

pub fn projects_router(
    registry: ProjectRegistryRef,
    rbac: ProjectRbac,
    metering: Arc<VMetering>,
) -> axum::Router {
    use axum::routing::{delete, get, post, put};
    axum::Router::new()
        .route("/api/vibe/projects", post(create_project))
        .route("/api/vibe/projects", get(list_projects))
        .route("/api/vibe/projects/:project_id", get(get_project))
        .route("/api/vibe/projects/:project_id", put(update_project))
        .route("/api/vibe/projects/:project_id", delete(delete_project))
        .route("/api/vibe/projects/:project_id/files", get(list_project_files))
        .route("/api/vibe/projects/:project_id/files/content", get(read_project_file))
        .route("/api/vibe/projects/:project_id/files", post(write_project_file))
        .layer(Extension(registry))
        .layer(Extension(rbac))
        .layer(Extension(metering))
}

// ── Workspace file browser (load a project's actual agent output) ────────────
// The code editor used to read `/api/editor/files` (the bot's global editor
// workspace), which is unrelated to the per-project Vibe workspace. These
// endpoints expose the real `VIBE_WORKSPACE_ROOT/{slug}/` files so selecting
// a project in the sidebar loads its source instead of nothing.

#[derive(Debug, Deserialize)]
pub struct WorkspaceFileQuery {
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct WriteWorkspaceFileRequest {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct WorkspaceFilesResponse {
    pub success: bool,
    pub project_id: Option<Uuid>,
    pub workspace: Option<String>,
    pub files: Option<Vec<String>>,
    pub path: Option<String>,
    pub content: Option<String>,
    pub bytes: Option<usize>,
    pub error: Option<String>,
}

/// Canonical workspace directory key for a project: the ALM repo slug, which
/// is the same key `collect_workspace_files` (publish) and the agent's
/// `file/*` tools use for `VIBE_WORKSPACE_ROOT/{key}/`.
fn workspace_key(project: &Project) -> String {
    VmLifecycle::alm_repo(&project.name)
}

fn ws_ok(project_id: Uuid, key: String) -> WorkspaceFilesResponse {
    WorkspaceFilesResponse {
        success: true,
        project_id: Some(project_id),
        workspace: Some(key),
        files: None,
        path: None,
        content: None,
        bytes: None,
        error: None,
    }
}

fn ws_err(project_id: Option<Uuid>, key: Option<String>, msg: String) -> (StatusCode, Json<WorkspaceFilesResponse>) {
    log::error!("Vibe workspace files API error: {msg}");
    (
        StatusCode::OK,
        Json(WorkspaceFilesResponse {
            success: false,
            project_id,
            workspace: key,
            files: None,
            path: None,
            content: None,
            bytes: None,
            error: Some(msg),
        }),
    )
}

fn ws_forbidden(msg: String) -> (StatusCode, Json<WorkspaceFilesResponse>) {
    log::warn!("Vibe workspace files API forbidden: {msg}");
    (
        StatusCode::FORBIDDEN,
        Json(WorkspaceFilesResponse {
            success: false,
            project_id: None,
            workspace: None,
            files: None,
            path: None,
            content: None,
            bytes: None,
            error: Some(msg),
        }),
    )
}

async fn list_project_files(
    Extension(registry): Extension<ProjectRegistryRef>,
    Extension(rbac): Extension<ProjectRbac>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
) -> (StatusCode, Json<WorkspaceFilesResponse>) {
    if let Err(e) = rbac.require_role(user.user_id, id, ProjectRole::Viewer) {
        return ws_forbidden(e);
    }
    let project = match registry.get(id) {
        Ok(Some(p)) => p,
        Ok(None) => return ws_err(Some(id), None, format!("project {id} not found")),
        Err(e) => return ws_err(Some(id), None, e),
    };
    let key = workspace_key(&project);
    match harness::list_rel(&key, "", 0) {
        Ok(mut files) => {
            files.sort();
            let mut resp = ws_ok(id, key);
            resp.files = Some(files);
            (StatusCode::OK, Json(resp))
        }
        Err(e) => ws_err(Some(id), Some(key), e),
    }
}

async fn read_project_file(
    Extension(registry): Extension<ProjectRegistryRef>,
    Extension(rbac): Extension<ProjectRbac>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
    Query(query): Query<WorkspaceFileQuery>,
) -> (StatusCode, Json<WorkspaceFilesResponse>) {
    if let Err(e) = rbac.require_role(user.user_id, id, ProjectRole::Viewer) {
        return ws_forbidden(e);
    }
    let project = match registry.get(id) {
        Ok(Some(p)) => p,
        Ok(None) => return ws_err(Some(id), None, format!("project {id} not found")),
        Err(e) => return ws_err(Some(id), None, e),
    };
    let key = workspace_key(&project);
    match harness::read_rel_file(&key, &query.path, 4 * 1024 * 1024) {
        Ok(bytes) => {
            let content = String::from_utf8_lossy(&bytes).into_owned();
            let mut resp = ws_ok(id, key);
            resp.path = Some(query.path.clone());
            resp.content = Some(content);
            resp.bytes = Some(bytes.len());
            (StatusCode::OK, Json(resp))
        }
        Err(e) => {
            let mut resp = ws_err(Some(id), Some(key), e);
            resp.1 .0.path = Some(query.path.clone());
            resp
        }
    }
}

async fn write_project_file(
    Extension(registry): Extension<ProjectRegistryRef>,
    Extension(rbac): Extension<ProjectRbac>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<Uuid>,
    Json(req): Json<WriteWorkspaceFileRequest>,
) -> (StatusCode, Json<WorkspaceFilesResponse>) {
    if let Err(e) = rbac.require_role(user.user_id, id, ProjectRole::Developer) {
        return ws_forbidden(e);
    }
    if req.path.trim().is_empty() {
        return ws_err(Some(id), None, "path must not be empty".to_string());
    }
    let project = match registry.get(id) {
        Ok(Some(p)) => p,
        Ok(None) => return ws_err(Some(id), None, format!("project {id} not found")),
        Err(e) => return ws_err(Some(id), None, e),
    };
    let key = workspace_key(&project);
    match harness::write_rel_file(&key, &req.path, req.content.as_bytes()) {
        Ok(()) => {
            let mut resp = ws_ok(id, key);
            resp.path = Some(req.path.clone());
            resp.bytes = Some(req.content.len());
            (StatusCode::OK, Json(resp))
        }
        Err(e) => ws_err(Some(id), Some(key), e),
    }
}
