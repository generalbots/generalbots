pub mod handlers;
pub mod utils;

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use botcore::shared::state::AppState;

pub fn configure_rbac_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/rbac/roles", get(handlers::list_roles).post(handlers::create_role))
        .route(
            "/api/rbac/roles/{role_id}",
            get(handlers::get_role)
                .put(handlers::update_role)
                .delete(handlers::delete_role),
        )
        .route(
            "/api/rbac/roles/{role_id}/permissions",
            get(handlers::get_role_permissions).post(handlers::update_role_permissions),
        )
        .route("/api/rbac/permissions", get(handlers::list_permissions))
        .route("/api/rbac/groups", get(handlers::list_groups).post(handlers::create_group))
        .route(
            "/api/rbac/groups/{group_id}",
            get(handlers::get_group)
                .put(handlers::update_group)
                .delete(handlers::delete_group),
        )
        .route("/api/rbac/users", get(handlers::list_users_with_roles))
        .route("/api/rbac/users/{user_id}/roles", get(handlers::get_user_roles))
        .route(
            "/api/rbac/users/{user_id}/roles/{role_id}",
            post(handlers::assign_role_to_user).delete(handlers::remove_role_from_user),
        )
        .route("/api/rbac/users/{user_id}/groups", get(handlers::get_user_groups))
        .route(
            "/api/rbac/users/{user_id}/groups/{group_id}",
            post(handlers::add_user_to_group).delete(handlers::remove_user_from_group),
        )
        .route("/api/rbac/groups/{group_id}/roles", get(handlers::get_group_roles))
        .route(
            "/api/rbac/groups/{group_id}/roles/{role_id}",
            post(handlers::assign_role_to_group).delete(handlers::remove_role_from_group),
        )
        .route("/api/rbac/users/{user_id}/permissions", get(handlers::get_effective_permissions))
        .route("/api/rbac/check", post(handlers::check_permission))
        .route("/api/rbac/my-permissions", get(handlers::my_permissions))
        .route("/settings/rbac", get(rbac_settings_page))
        .route("/settings/rbac/users", get(rbac_users_list))
        .route("/settings/rbac/roles", get(rbac_roles_list))
        .route("/settings/rbac/groups", get(rbac_groups_list))
        .route("/settings/rbac/users/{user_id}/assignment", get(user_assignment_panel))
        .route("/settings/rbac/users/{user_id}/available-roles", get(available_roles_for_user))
        .route("/settings/rbac/users/{user_id}/assigned-roles", get(assigned_roles_for_user))
        .route("/settings/rbac/users/{user_id}/available-groups", get(available_groups_for_user))
        .route("/settings/rbac/users/{user_id}/assigned-groups", get(assigned_groups_for_user))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRoleRequest {
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateGroupRequest {
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub parent_group_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssignRoleRequest {
    pub expires_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRolePermissionsRequest {
    pub permission_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CheckPermissionRequest {
    pub permission: String,
}

#[derive(Debug, Serialize)]
pub struct CheckPermissionResponse {
    pub granted: bool,
    pub permission: String,
    pub source: String,
}

pub use crate::rbac_ui::{
    rbac_settings_page, rbac_users_list, rbac_roles_list, rbac_groups_list,
    user_assignment_panel, available_roles_for_user, assigned_roles_for_user,
    available_groups_for_user, assigned_groups_for_user,
};
