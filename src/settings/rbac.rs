use crate::security::error_sanitizer::log_and_sanitize_str;
use crate::shared::models::{
    NewRbacGroup, NewRbacGroupRole, NewRbacRole, NewRbacUserGroup, NewRbacUserRole, RbacGroup,
    RbacGroupRole, RbacRole, RbacUserGroup, RbacUserRole, User,
};
use crate::shared::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use diesel::prelude::*;
use log::info;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

pub fn configure_rbac_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/rbac/roles", get(list_roles).post(create_role))
        .route("/api/rbac/roles/{role_id}", get(get_role).delete(delete_role))
        .route("/api/rbac/groups", get(list_groups).post(create_group))
        .route("/api/rbac/groups/{group_id}", get(get_group).delete(delete_group))
        .route("/api/rbac/users", get(list_users_with_roles))
        .route("/api/rbac/users/{user_id}/roles", get(get_user_roles))
        .route("/api/rbac/users/{user_id}/roles/{role_id}", post(assign_role_to_user).delete(remove_role_from_user))
        .route("/api/rbac/users/{user_id}/groups", get(get_user_groups))
        .route("/api/rbac/users/{user_id}/groups/{group_id}", post(add_user_to_group).delete(remove_user_from_group))
        .route("/api/rbac/groups/{group_id}/roles", get(get_group_roles))
        .route("/api/rbac/groups/{group_id}/roles/{role_id}", post(assign_role_to_group).delete(remove_role_from_group))
        .route("/api/rbac/users/{user_id}/permissions", get(get_effective_permissions))
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
pub struct PaginationParams {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
}

async fn list_roles(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let conn = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB error: {e}"))?;
        use crate::shared::models::schema::rbac_roles;
        rbac_roles::table
            .filter(rbac_roles::is_active.eq(true))
            .order(rbac_roles::display_name.asc())
            .load::<RbacRole>(&mut db_conn)
            .map_err(|e| format!("Query error: {e}"))
    })
    .await;

    match result {
        Ok(Ok(roles)) => Json(serde_json::json!({ "roles": roles })).into_response(),
        Ok(Err(e)) => {
            let sanitized = log_and_sanitize_str(&e, "list_roles", None);
            (StatusCode::INTERNAL_SERVER_ERROR, sanitized).into_response()
        }
        Err(e) => {
            let sanitized = log_and_sanitize_str(&e.to_string(), "list_roles", None);
            (StatusCode::INTERNAL_SERVER_ERROR, sanitized).into_response()
        }
    }
}

async fn get_role(State(state): State<Arc<AppState>>, Path(role_id): Path<Uuid>) -> impl IntoResponse {
    let conn = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB error: {e}"))?;
        use crate::shared::models::schema::rbac_roles;
        rbac_roles::table
            .find(role_id)
            .first::<RbacRole>(&mut db_conn)
            .map_err(|e| format!("Role not found: {e}"))
    })
    .await;

    match result {
        Ok(Ok(role)) => Json(role).into_response(),
        Ok(Err(e)) => {
            let sanitized = log_and_sanitize_str(&e, "get_role", None);
            (StatusCode::NOT_FOUND, sanitized).into_response()
        }
        Err(e) => {
            let sanitized = log_and_sanitize_str(&e.to_string(), "get_role", None);
            (StatusCode::INTERNAL_SERVER_ERROR, sanitized).into_response()
        }
    }
}

async fn create_role(State(state): State<Arc<AppState>>, Json(req): Json<CreateRoleRequest>) -> impl IntoResponse {
    let conn = state.conn.clone();
    let now = Utc::now();
    let new_role = NewRbacRole {
        id: Uuid::new_v4(),
        name: req.name.to_lowercase().replace(' ', "_"),
        display_name: req.display_name,
        description: req.description,
        is_system: false,
        is_active: true,
        created_by: None,
        created_at: now,
        updated_at: now,
    };

    let result = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB error: {e}"))?;
        use crate::shared::models::schema::rbac_roles;
        diesel::insert_into(rbac_roles::table)
            .values(&new_role)
            .get_result::<RbacRole>(&mut db_conn)
            .map_err(|e| format!("Insert error: {e}"))
    })
    .await;

    match result {
        Ok(Ok(role)) => {
            info!("Created role: {} ({})", role.display_name, role.id);
            (StatusCode::CREATED, Json(role)).into_response()
        }
        Ok(Err(e)) => {
            let sanitized = log_and_sanitize_str(&e, "create_role", None);
            (StatusCode::BAD_REQUEST, sanitized).into_response()
        }
        Err(e) => {
            let sanitized = log_and_sanitize_str(&e.to_string(), "create_role", None);
            (StatusCode::INTERNAL_SERVER_ERROR, sanitized).into_response()
        }
    }
}

async fn delete_role(State(state): State<Arc<AppState>>, Path(role_id): Path<Uuid>) -> impl IntoResponse {
    let conn = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB error: {e}"))?;
        use crate::shared::models::schema::rbac_roles;
        let role: RbacRole = rbac_roles::table.find(role_id).first(&mut db_conn).map_err(|e| format!("Role not found: {e}"))?;
        if role.is_system {
            return Err("Cannot delete system role".to_string());
        }
        diesel::update(rbac_roles::table.find(role_id))
            .set(rbac_roles::is_active.eq(false))
            .execute(&mut db_conn)
            .map_err(|e| format!("Delete error: {e}"))?;
        Ok(())
    })
    .await;

    match result {
        Ok(Ok(())) => StatusCode::NO_CONTENT.into_response(),
        Ok(Err(e)) => {
            let sanitized = log_and_sanitize_str(&e, "delete_role", None);
            (StatusCode::BAD_REQUEST, sanitized).into_response()
        }
        Err(e) => {
            let sanitized = log_and_sanitize_str(&e.to_string(), "delete_role", None);
            (StatusCode::INTERNAL_SERVER_ERROR, sanitized).into_response()
        }
    }
}

async fn list_groups(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let conn = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB error: {e}"))?;
        use crate::shared::models::schema::rbac_groups;
        rbac_groups::table
            .filter(rbac_groups::is_active.eq(true))
            .order(rbac_groups::display_name.asc())
            .load::<RbacGroup>(&mut db_conn)
            .map_err(|e| format!("Query error: {e}"))
    })
    .await;

    match result {
        Ok(Ok(groups)) => Json(serde_json::json!({ "groups": groups })).into_response(),
        Ok(Err(e)) => {
            let sanitized = log_and_sanitize_str(&e, "list_groups", None);
            (StatusCode::INTERNAL_SERVER_ERROR, sanitized).into_response()
        }
        Err(e) => {
            let sanitized = log_and_sanitize_str(&e.to_string(), "list_groups", None);
            (StatusCode::INTERNAL_SERVER_ERROR, sanitized).into_response()
        }
    }
}

async fn get_group(State(state): State<Arc<AppState>>, Path(group_id): Path<Uuid>) -> impl IntoResponse {
    let conn = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB error: {e}"))?;
        use crate::shared::models::schema::rbac_groups;
        rbac_groups::table
            .find(group_id)
            .first::<RbacGroup>(&mut db_conn)
            .map_err(|e| format!("Group not found: {e}"))
    })
    .await;

    match result {
        Ok(Ok(group)) => Json(group).into_response(),
        Ok(Err(e)) => {
            let sanitized = log_and_sanitize_str(&e, "get_group", None);
            (StatusCode::NOT_FOUND, sanitized).into_response()
        }
        Err(e) => {
            let sanitized = log_and_sanitize_str(&e.to_string(), "get_group", None);
            (StatusCode::INTERNAL_SERVER_ERROR, sanitized).into_response()
        }
    }
}

async fn create_group(State(state): State<Arc<AppState>>, Json(req): Json<CreateGroupRequest>) -> impl IntoResponse {
    let conn = state.conn.clone();
    let now = Utc::now();
    let new_group = NewRbacGroup {
        id: Uuid::new_v4(),
        name: req.name.to_lowercase().replace(' ', "_"),
        display_name: req.display_name,
        description: req.description,
        parent_group_id: req.parent_group_id,
        is_active: true,
        created_by: None,
        created_at: now,
        updated_at: now,
    };

    let result = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB error: {e}"))?;
        use crate::shared::models::schema::rbac_groups;
        diesel::insert_into(rbac_groups::table)
            .values(&new_group)
            .get_result::<RbacGroup>(&mut db_conn)
            .map_err(|e| format!("Insert error: {e}"))
    })
    .await;

    match result {
        Ok(Ok(group)) => {
            info!("Created group: {} ({})", group.display_name, group.id);
            (StatusCode::CREATED, Json(group)).into_response()
        }
        Ok(Err(e)) => {
            let sanitized = log_and_sanitize_str(&e, "create_group", None);
            (StatusCode::BAD_REQUEST, sanitized).into_response()
        }
        Err(e) => {
            let sanitized = log_and_sanitize_str(&e.to_string(), "create_group", None);
            (StatusCode::INTERNAL_SERVER_ERROR, sanitized).into_response()
        }
    }
}

async fn delete_group(State(state): State<Arc<AppState>>, Path(group_id): Path<Uuid>) -> impl IntoResponse {
    let conn = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || -> Result<(), String> {
        let mut db_conn = conn.get().map_err(|e| format!("DB error: {e}"))?;
        use crate::shared::models::schema::rbac_groups;
        diesel::update(rbac_groups::table.find(group_id))
            .set(rbac_groups::is_active.eq(false))
            .execute(&mut db_conn)
            .map_err(|e| format!("Delete error: {e}"))?;
        Ok(())
    })
    .await;

    match result {
        Ok(Ok(())) => StatusCode::NO_CONTENT.into_response(),
        Ok(Err(e)) => {
            let sanitized = log_and_sanitize_str(&e, "delete_group", None);
            (StatusCode::BAD_REQUEST, sanitized).into_response()
        }
        Err(e) => {
            let sanitized = log_and_sanitize_str(&e.to_string(), "delete_group", None);
            (StatusCode::INTERNAL_SERVER_ERROR, sanitized).into_response()
        }
    }
}

async fn list_users_with_roles(State(state): State<Arc<AppState>>, Query(params): Query<PaginationParams>) -> impl IntoResponse {
    let conn = state.conn.clone();
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;
    let search = params.search.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB error: {e}"))?;
        use crate::shared::models::schema::users;
        let mut query = users::table.filter(users::is_active.eq(true)).order(users::username.asc()).into_boxed();
        if let Some(ref s) = search {
            let pattern = format!("%{s}%");
            query = query.filter(users::username.ilike(pattern.clone()).or(users::email.ilike(pattern)));
        }
        query.offset(offset).limit(per_page).load::<User>(&mut db_conn).map_err(|e| format!("Query error: {e}"))
    })
    .await;

    match result {
        Ok(Ok(users)) => Json(serde_json::json!({ "users": users })).into_response(),
        Ok(Err(e)) => {
            let sanitized = log_and_sanitize_str(&e, "list_users_with_roles", None);
            (StatusCode::INTERNAL_SERVER_ERROR, sanitized).into_response()
        }
        Err(e) => {
            let sanitized = log_and_sanitize_str(&e.to_string(), "list_users_with_roles", None);
            (StatusCode::INTERNAL_SERVER_ERROR, sanitized).into_response()
        }
    }
}

async fn get_user_roles(State(state): State<Arc<AppState>>, Path(user_id): Path<Uuid>) -> impl IntoResponse {
    let conn = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB error: {e}"))?;
        use crate::shared::models::schema::{rbac_roles, rbac_user_roles};
        rbac_user_roles::table
            .inner_join(rbac_roles::table)
            .filter(rbac_user_roles::user_id.eq(user_id))
            .filter(rbac_roles::is_active.eq(true))
            .select(RbacRole::as_select())
            .load::<RbacRole>(&mut db_conn)
            .map_err(|e| format!("Query error: {e}"))
    })
    .await;

    match result {
        Ok(Ok(roles)) => Json(serde_json::json!({ "roles": roles })).into_response(),
        Ok(Err(e)) => {
            let sanitized = log_and_sanitize_str(&e, "get_user_roles", None);
            (StatusCode::INTERNAL_SERVER_ERROR, sanitized).into_response()
        }
        Err(e) => {
            let sanitized = log_and_sanitize_str(&e.to_string(), "get_user_roles", None);
            (StatusCode::INTERNAL_SERVER_ERROR, sanitized).into_response()
        }
    }
}

async fn assign_role_to_user(State(state): State<Arc<AppState>>, Path((user_id, role_id)): Path<(Uuid, Uuid)>, body: Option<Json<AssignRoleRequest>>) -> impl IntoResponse {
    let conn = state.conn.clone();
    let now = Utc::now();
    let expires_at = body.and_then(|b| b.expires_at);
    let new_assignment = NewRbacUserRole {
        id: Uuid::new_v4(),
        user_id,
        role_id,
        granted_by: None,
        granted_at: now,
        expires_at,
    };

    let result = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB error: {e}"))?;
        use crate::shared::models::schema::rbac_user_roles;
        let existing = rbac_user_roles::table
            .filter(rbac_user_roles::user_id.eq(user_id))
            .filter(rbac_user_roles::role_id.eq(role_id))
            .first::<RbacUserRole>(&mut db_conn)
            .optional()
            .map_err(|e| format!("Query error: {e}"))?;
        if existing.is_some() {
            return Err("Role already assigned to user".to_string());
        }
        diesel::insert_into(rbac_user_roles::table)
            .values(&new_assignment)
            .execute(&mut db_conn)
            .map_err(|e| format!("Insert error: {e}"))?;
        Ok(())
    })
    .await;

    match result {
        Ok(Ok(())) => {
            info!("Assigned role {role_id} to user {user_id}");
            StatusCode::CREATED.into_response()
        }
        Ok(Err(e)) => {
            let sanitized = log_and_sanitize_str(&e, "assign_role_to_user", None);
            (StatusCode::BAD_REQUEST, sanitized).into_response()
        }
        Err(e) => {
            let sanitized = log_and_sanitize_str(&e.to_string(), "assign_role_to_user", None);
            (StatusCode::INTERNAL_SERVER_ERROR, sanitized).into_response()
        }
    }
}

async fn remove_role_from_user(State(state): State<Arc<AppState>>, Path((user_id, role_id)): Path<(Uuid, Uuid)>) -> impl IntoResponse {
    let conn = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || -> Result<(), String> {
        let mut db_conn = conn.get().map_err(|e| format!("DB error: {e}"))?;
        use crate::shared::models::schema::rbac_user_roles;
        diesel::delete(rbac_user_roles::table.filter(rbac_user_roles::user_id.eq(user_id)).filter(rbac_user_roles::role_id.eq(role_id)))
            .execute(&mut db_conn)
            .map_err(|e| format!("Delete error: {e}"))?;
        Ok(())
    })
    .await;

    match result {
        Ok(Ok(())) => {
            info!("Removed role {role_id} from user {user_id}");
            StatusCode::NO_CONTENT.into_response()
        }
        Ok(Err(e)) => {
            let sanitized = log_and_sanitize_str(&e, "remove_role_from_user", None);
            (StatusCode::BAD_REQUEST, sanitized).into_response()
        }
        Err(e) => {
            let sanitized = log_and_sanitize_str(&e.to_string(), "remove_role_from_user", None);
            (StatusCode::INTERNAL_SERVER_ERROR, sanitized).into_response()
        }
    }
}

async fn get_user_groups(State(state): State<Arc<AppState>>, Path(user_id): Path<Uuid>) -> impl IntoResponse {
    let conn = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB error: {e}"))?;
        use crate::shared::models::schema::{rbac_groups, rbac_user_groups};
        rbac_user_groups::table
            .inner_join(rbac_groups::table)
            .filter(rbac_user_groups::user_id.eq(user_id))
            .filter(rbac_groups::is_active.eq(true))
            .select(RbacGroup::as_select())
            .load::<RbacGroup>(&mut db_conn)
            .map_err(|e| format!("Query error: {e}"))
    })
    .await;

    match result {
        Ok(Ok(groups)) => Json(serde_json::json!({ "groups": groups })).into_response(),
        Ok(Err(e)) => {
            let sanitized = log_and_sanitize_str(&e, "get_user_groups", None);
            (StatusCode::INTERNAL_SERVER_ERROR, sanitized).into_response()
        }
        Err(e) => {
            let sanitized = log_and_sanitize_str(&e.to_string(), "get_user_groups", None);
            (StatusCode::INTERNAL_SERVER_ERROR, sanitized).into_response()
        }
    }
}

async fn add_user_to_group(State(state): State<Arc<AppState>>, Path((user_id, group_id)): Path<(Uuid, Uuid)>) -> impl IntoResponse {
    let conn = state.conn.clone();
    let now = Utc::now();
    let new_membership = NewRbacUserGroup {
        id: Uuid::new_v4(),
        user_id,
        group_id,
        added_by: None,
        added_at: now,
    };

    let result = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB error: {e}"))?;
        use crate::shared::models::schema::rbac_user_groups;
        let existing = rbac_user_groups::table
            .filter(rbac_user_groups::user_id.eq(user_id))
            .filter(rbac_user_groups::group_id.eq(group_id))
            .first::<RbacUserGroup>(&mut db_conn)
            .optional()
            .map_err(|e| format!("Query error: {e}"))?;
        if existing.is_some() {
            return Err("User already in group".to_string());
        }
        diesel::insert_into(rbac_user_groups::table)
            .values(&new_membership)
            .execute(&mut db_conn)
            .map_err(|e| format!("Insert error: {e}"))?;
        Ok(())
    })
    .await;

    match result {
        Ok(Ok(())) => {
            info!("Added user {user_id} to group {group_id}");
            StatusCode::CREATED.into_response()
        }
        Ok(Err(e)) => {
            let sanitized = log_and_sanitize_str(&e, "add_user_to_group", None);
            (StatusCode::BAD_REQUEST, sanitized).into_response()
        }
        Err(e) => {
            let sanitized = log_and_sanitize_str(&e.to_string(), "add_user_to_group", None);
            (StatusCode::INTERNAL_SERVER_ERROR, sanitized).into_response()
        }
    }
}

async fn remove_user_from_group(State(state): State<Arc<AppState>>, Path((user_id, group_id)): Path<(Uuid, Uuid)>) -> impl IntoResponse {
    let conn = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || -> Result<(), String> {
        let mut db_conn = conn.get().map_err(|e| format!("DB error: {e}"))?;
        use crate::shared::models::schema::rbac_user_groups;
        diesel::delete(rbac_user_groups::table.filter(rbac_user_groups::user_id.eq(user_id)).filter(rbac_user_groups::group_id.eq(group_id)))
            .execute(&mut db_conn)
            .map_err(|e| format!("Delete error: {e}"))?;
        Ok(())
    })
    .await;

    match result {
        Ok(Ok(())) => {
            info!("Removed user {user_id} from group {group_id}");
            StatusCode::NO_CONTENT.into_response()
        }
        Ok(Err(e)) => {
            let sanitized = log_and_sanitize_str(&e, "remove_user_from_group", None);
            (StatusCode::BAD_REQUEST, sanitized).into_response()
        }
        Err(e) => {
            let sanitized = log_and_sanitize_str(&e.to_string(), "remove_user_from_group", None);
            (StatusCode::INTERNAL_SERVER_ERROR, sanitized).into_response()
        }
    }
}

async fn get_group_roles(State(state): State<Arc<AppState>>, Path(group_id): Path<Uuid>) -> impl IntoResponse {
    let conn = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB error: {e}"))?;
        use crate::shared::models::schema::{rbac_group_roles, rbac_roles};
        rbac_group_roles::table
            .inner_join(rbac_roles::table)
            .filter(rbac_group_roles::group_id.eq(group_id))
            .filter(rbac_roles::is_active.eq(true))
            .select(RbacRole::as_select())
            .load::<RbacRole>(&mut db_conn)
            .map_err(|e| format!("Query error: {e}"))
    })
    .await;

    match result {
        Ok(Ok(roles)) => Json(serde_json::json!({ "roles": roles })).into_response(),
        Ok(Err(e)) => {
            let sanitized = log_and_sanitize_str(&e, "get_group_roles", None);
            (StatusCode::INTERNAL_SERVER_ERROR, sanitized).into_response()
        }
        Err(e) => {
            let sanitized = log_and_sanitize_str(&e.to_string(), "get_group_roles", None);
            (StatusCode::INTERNAL_SERVER_ERROR, sanitized).into_response()
        }
    }
}

async fn assign_role_to_group(State(state): State<Arc<AppState>>, Path((group_id, role_id)): Path<(Uuid, Uuid)>) -> impl IntoResponse {
    let conn = state.conn.clone();
    let now = Utc::now();
    let new_assignment = NewRbacGroupRole {
        id: Uuid::new_v4(),
        group_id,
        role_id,
        granted_by: None,
        granted_at: now,
    };

    let result = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB error: {e}"))?;
        use crate::shared::models::schema::rbac_group_roles;
        let existing = rbac_group_roles::table
            .filter(rbac_group_roles::group_id.eq(group_id))
            .filter(rbac_group_roles::role_id.eq(role_id))
            .first::<RbacGroupRole>(&mut db_conn)
            .optional()
            .map_err(|e| format!("Query error: {e}"))?;
        if existing.is_some() {
            return Err("Role already assigned to group".to_string());
        }
        diesel::insert_into(rbac_group_roles::table)
            .values(&new_assignment)
            .execute(&mut db_conn)
            .map_err(|e| format!("Insert error: {e}"))?;
        Ok(())
    })
    .await;

    match result {
        Ok(Ok(())) => {
            info!("Assigned role {role_id} to group {group_id}");
            StatusCode::CREATED.into_response()
        }
        Ok(Err(e)) => {
            let sanitized = log_and_sanitize_str(&e, "assign_role_to_group", None);
            (StatusCode::BAD_REQUEST, sanitized).into_response()
        }
        Err(e) => {
            let sanitized = log_and_sanitize_str(&e.to_string(), "assign_role_to_group", None);
            (StatusCode::INTERNAL_SERVER_ERROR, sanitized).into_response()
        }
    }
}

async fn remove_role_from_group(State(state): State<Arc<AppState>>, Path((group_id, role_id)): Path<(Uuid, Uuid)>) -> impl IntoResponse {
    let conn = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || -> Result<(), String> {
        let mut db_conn = conn.get().map_err(|e| format!("DB error: {e}"))?;
        use crate::shared::models::schema::rbac_group_roles;
        diesel::delete(rbac_group_roles::table.filter(rbac_group_roles::group_id.eq(group_id)).filter(rbac_group_roles::role_id.eq(role_id)))
            .execute(&mut db_conn)
            .map_err(|e| format!("Delete error: {e}"))?;
        Ok(())
    })
    .await;

    match result {
        Ok(Ok(())) => {
            info!("Removed role {role_id} from group {group_id}");
            StatusCode::NO_CONTENT.into_response()
        }
        Ok(Err(e)) => {
            let sanitized = log_and_sanitize_str(&e, "remove_role_from_group", None);
            (StatusCode::BAD_REQUEST, sanitized).into_response()
        }
        Err(e) => {
            let sanitized = log_and_sanitize_str(&e.to_string(), "remove_role_from_group", None);
            (StatusCode::INTERNAL_SERVER_ERROR, sanitized).into_response()
        }
    }
}

async fn get_effective_permissions(State(state): State<Arc<AppState>>, Path(user_id): Path<Uuid>) -> impl IntoResponse {
    let conn = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB error: {e}"))?;
        use crate::shared::models::schema::{rbac_roles, rbac_user_roles, rbac_groups, rbac_user_groups, rbac_group_roles};

        let direct_roles: Vec<RbacRole> = rbac_user_roles::table
            .inner_join(rbac_roles::table)
            .filter(rbac_user_roles::user_id.eq(user_id))
            .filter(rbac_roles::is_active.eq(true))
            .select(RbacRole::as_select())
            .load(&mut db_conn)
            .map_err(|e| format!("Query error: {e}"))?;

        let user_group_ids: Vec<Uuid> = rbac_user_groups::table
            .filter(rbac_user_groups::user_id.eq(user_id))
            .select(rbac_user_groups::group_id)
            .load(&mut db_conn)
            .map_err(|e| format!("Query error: {e}"))?;

        let group_roles: Vec<RbacRole> = rbac_group_roles::table
            .inner_join(rbac_roles::table)
            .filter(rbac_group_roles::group_id.eq_any(&user_group_ids))
            .filter(rbac_roles::is_active.eq(true))
            .select(RbacRole::as_select())
            .load(&mut db_conn)
            .map_err(|e| format!("Query error: {e}"))?;

        let groups: Vec<RbacGroup> = rbac_user_groups::table
            .inner_join(rbac_groups::table)
            .filter(rbac_user_groups::user_id.eq(user_id))
            .filter(rbac_groups::is_active.eq(true))
            .select(RbacGroup::as_select())
            .load(&mut db_conn)
            .map_err(|e| format!("Query error: {e}"))?;

        Ok::<_, String>(serde_json::json!({
            "user_id": user_id,
            "direct_roles": direct_roles,
            "group_roles": group_roles,
            "groups": groups
        }))
    })
    .await;

    match result {
        Ok(Ok(data)) => Json(data).into_response(),
        Ok(Err(e)) => {
            let sanitized = log_and_sanitize_str(&e, "get_effective_permissions", None);
            (StatusCode::INTERNAL_SERVER_ERROR, sanitized).into_response()
        }
        Err(e) => {
            let sanitized = log_and_sanitize_str(&e.to_string(), "get_effective_permissions", None);
            (StatusCode::INTERNAL_SERVER_ERROR, sanitized).into_response()
        }
    }
}

pub use crate::settings::rbac_ui::{
    rbac_settings_page, rbac_users_list, rbac_roles_list, rbac_groups_list,
    user_assignment_panel, available_roles_for_user, assigned_roles_for_user,
    available_groups_for_user, assigned_groups_for_user,
};
