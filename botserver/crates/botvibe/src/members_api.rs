use axum::extract::{Extension, Path, Query};
use axum::http::StatusCode;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use botsecurity_auth::auth_api::types::AuthenticatedUser;

use crate::rbac::{ProjectMember, ProjectRbac, ProjectRole};

#[derive(Debug, Serialize)]
pub struct MembersResponse {
    pub success: bool,
    pub members: Option<Vec<ProjectMember>>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RoleRequest {
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct TransferRequest {
    pub user_id: Uuid,
}

type ApiResult = (StatusCode, Json<MembersResponse>);

fn ok_list(members: Vec<ProjectMember>) -> ApiResult {
    (
        StatusCode::OK,
        Json(MembersResponse {
            success: true,
            members: Some(members),
            error: None,
        }),
    )
}

fn ok_msg() -> ApiResult {
    (
        StatusCode::OK,
        Json(MembersResponse {
            success: true,
            members: None,
            error: None,
        }),
    )
}

fn err(msg: String) -> ApiResult {
    log::error!("Vibe members API error: {msg}");
    (
        StatusCode::OK,
        Json(MembersResponse {
            success: false,
            members: None,
            error: Some(msg),
        }),
    )
}

fn forbidden(msg: String) -> ApiResult {
    log::warn!("Vibe members API forbidden: {msg}");
    (
        StatusCode::FORBIDDEN,
        Json(MembersResponse {
            success: false,
            members: None,
            error: Some(msg),
        }),
    )
}

fn parse_role(req: &RoleRequest) -> Result<ProjectRole, ApiResult> {
    ProjectRole::parse(&req.role).ok_or_else(|| err(format!("invalid role '{}' (expected owner, admin, developer or viewer)", req.role)))
}

async fn list_members(
    Extension(rbac): Extension<ProjectRbac>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<Uuid>,
) -> ApiResult {
    match rbac.require_role(user.user_id, project_id, ProjectRole::Viewer) {
        Ok(_) => {}
        Err(e) => return forbidden(e),
    }
    match rbac.list_members(project_id) {
        Ok(members) => ok_list(members),
        Err(e) => err(e),
    }
}

async fn set_user_member(
    Extension(rbac): Extension<ProjectRbac>,
    Extension(user): Extension<AuthenticatedUser>,
    Path((project_id, user_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<RoleRequest>,
) -> ApiResult {
    let role = match parse_role(&req) {
        Ok(r) => r,
        Err(r) => return r,
    };
    match rbac.require_role(user.user_id, project_id, ProjectRole::Admin) {
        Ok(_) => {}
        Err(e) => return forbidden(e),
    }
    if role == ProjectRole::Owner {
        return err("use transfer-ownership to grant owner".into());
    }
    match rbac.set_user_role(project_id, user_id, role) {
        Ok(()) => ok_msg(),
        Err(e) => err(e),
    }
}

async fn remove_user_member(
    Extension(rbac): Extension<ProjectRbac>,
    Extension(user): Extension<AuthenticatedUser>,
    Path((project_id, user_id)): Path<(Uuid, Uuid)>,
) -> ApiResult {
    match rbac.require_role(user.user_id, project_id, ProjectRole::Admin) {
        Ok(_) => {}
        Err(e) => return forbidden(e),
    }
    if user_id == user.user_id {
        return err("removing your own membership is not allowed; transfer ownership first".into());
    }
    match rbac.remove_user(project_id, user_id) {
        Ok(true) => ok_msg(),
        Ok(false) => err(format!("user {user_id} is not a member of project {project_id}")),
        Err(e) => err(e),
    }
}

async fn set_group_member(
    Extension(rbac): Extension<ProjectRbac>,
    Extension(user): Extension<AuthenticatedUser>,
    Path((project_id, group_name)): Path<(Uuid, String)>,
    Json(req): Json<RoleRequest>,
) -> ApiResult {
    let role = match parse_role(&req) {
        Ok(r) => r,
        Err(r) => return r,
    };
    match rbac.require_role(user.user_id, project_id, ProjectRole::Admin) {
        Ok(_) => {}
        Err(e) => return forbidden(e),
    }
    match rbac.set_group_role(project_id, &group_name, role) {
        Ok(()) => ok_msg(),
        Err(e) => err(e),
    }
}

async fn remove_group_member(
    Extension(rbac): Extension<ProjectRbac>,
    Extension(user): Extension<AuthenticatedUser>,
    Path((project_id, group_name)): Path<(Uuid, String)>,
) -> ApiResult {
    match rbac.require_role(user.user_id, project_id, ProjectRole::Admin) {
        Ok(_) => {}
        Err(e) => return forbidden(e),
    }
    match rbac.remove_group(project_id, &group_name) {
        Ok(true) => ok_msg(),
        Ok(false) => err(format!("group '{group_name}' has no grant on project {project_id}")),
        Err(e) => err(e),
    }
}

#[derive(Debug, Deserialize)]
pub struct UserSearchQuery {
    pub q: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserSearchResponse {
    pub success: bool,
    pub users: Option<Vec<UserSearchHit>>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserSearchHit {
    pub id: Uuid,
    pub username: String,
    pub email: String,
}

async fn search_users(
    Extension(rbac): Extension<ProjectRbac>,
    Extension(_user): Extension<AuthenticatedUser>,
    Query(query): Query<UserSearchQuery>,
) -> Result<Json<UserSearchResponse>, (StatusCode, Json<UserSearchResponse>)> {
    let q = query.q.unwrap_or_default().trim().to_string();
    if q.len() < 2 {
        return Ok(Json(UserSearchResponse {
            success: true,
            users: Some(Vec::new()),
            error: None,
        }));
    }
    match rbac.search_users(&q, 20) {
        Ok(rows) => Ok(Json(UserSearchResponse {
            success: true,
            users: Some(
                rows.into_iter()
                    .map(|(id, username, email)| UserSearchHit { id, username, email })
                    .collect(),
            ),
            error: None,
        })),
        Err(e) => Ok(Json(UserSearchResponse {
            success: false,
            users: None,
            error: Some(e),
        })),
    }
}

async fn transfer_ownership(
    Extension(rbac): Extension<ProjectRbac>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(project_id): Path<Uuid>,
    Json(req): Json<TransferRequest>,
) -> ApiResult {
    match rbac.require_role(user.user_id, project_id, ProjectRole::Owner) {
        Ok(_) => {}
        Err(e) => return forbidden(e),
    }
    match rbac.transfer_ownership(project_id, req.user_id) {
        Ok(()) => ok_msg(),
        Err(e) => err(e),
    }
}

pub fn members_router(rbac: ProjectRbac) -> Router {
    Router::new()
        .route(
            "/api/vibe/users/search",
            get(search_users),
        )
        .route(
            "/api/vibe/projects/:project_id/members",
            get(list_members),
        )
        .route(
            "/api/vibe/projects/:project_id/members/:user_id",
            put(set_user_member).delete(remove_user_member),
        )
        .route(
            "/api/vibe/projects/:project_id/members/group/:group_name",
            put(set_group_member).delete(remove_group_member),
        )
        .route(
            "/api/vibe/projects/:project_id/members/transfer-ownership",
            post(transfer_ownership),
        )
        .layer(Extension(rbac))
}
