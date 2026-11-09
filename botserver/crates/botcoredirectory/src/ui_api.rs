use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json, Router,
};
use diesel::{OptionalExtension, RunQueryDsl};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::api::DirectoryApiState;

mod group_handlers;
use group_handlers::{
    add_group_member_json, delete_group_json, get_group_members_json,
    remove_group_member_json, send_group_invite_json, update_group_json,
};

fn get_conn(state: &Arc<DirectoryApiState>) -> Option<diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>> {
    match state.conn.get() {
        Ok(c) => Some(c),
        Err(e) => {
            log::error!("Directory DB connection error: {e}");
            None
        }
    }
}

pub fn configure_directory_api_routes() -> Router<Arc<DirectoryApiState>> {
    Router::new()
        .route("/api/directory/users/:user_id/update", put(update_user_json))
        .route("/api/directory/users/:user_id/roles", get(get_user_roles_json))
        .route("/api/directory/organizations/list", get(list_organizations_json))
        .route("/api/directory/groups/:group_id/update", put(update_group_json))
        .route("/api/directory/groups/:group_id/delete", delete(delete_group_json))
        .route("/api/directory/groups/:group_id/members", get(get_group_members_json))
        .route("/api/directory/groups/:group_id/members/add", post(add_group_member_json))
        .route("/api/directory/groups/:group_id/members/remove", post(remove_group_member_json))
        .route("/api/directory/groups/:group_id/invites/send", post(send_group_invite_json))
}

#[derive(Debug, Deserialize)]
pub struct UserUpdateRequest {
    pub username: Option<String>,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub role: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub is_active: bool,
    pub is_admin: bool,
}

async fn update_user_json(
    State(state): State<Arc<DirectoryApiState>>,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<UserUpdateRequest>,
) -> Result<Json<UserResponse>, (StatusCode, String)> {
    let mut conn = get_conn(&state)
        .ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, "database unavailable".to_string()))?;

    if let Some(username) = &payload.username {
        diesel::sql_query("UPDATE users SET username = $1, updated_at = NOW() WHERE id = $2")
            .bind::<diesel::sql_types::Text, _>(username)
            .bind::<diesel::sql_types::Uuid, _>(user_id)
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(email) = &payload.email {
        let email = email.trim();
        // #1943 — `users.email` is UNIQUE. Only write it when no other row owns
        // the address, and answer 409 instead of surfacing a raw constraint
        // violation (`users_email_key`).
        let updated = diesel::sql_query(
            "UPDATE users SET email = $1, updated_at = NOW() \
             WHERE id = $2 AND NOT EXISTS ( \
               SELECT 1 FROM users other \
               WHERE other.id <> $2 AND lower(other.email) = lower($1))",
        )
        .bind::<diesel::sql_types::Text, _>(email)
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

        if updated == 0 {
            #[derive(diesel::QueryableByName)]
            #[diesel(check_for_backend(diesel::pg::Pg))]
            struct CurrentEmail {
                #[diesel(sql_type = diesel::sql_types::Text)]
                email: String,
            }
            let current = diesel::sql_query("SELECT email FROM users WHERE id = $1")
                .bind::<diesel::sql_types::Uuid, _>(user_id)
                .get_result::<CurrentEmail>(&mut conn)
                .optional()
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;
            match current {
                // Same address (differing only by case): the guarded update is
                // a no-op on purpose, not a conflict.
                Some(row) if row.email.eq_ignore_ascii_case(email) => {}
                Some(_) => {
                    return Err((
                        StatusCode::CONFLICT,
                        "Email already in use by another user".to_string(),
                    ))
                }
                None => return Err((StatusCode::NOT_FOUND, "User not found".to_string())),
            }
        }
    }

    if let Some(is_admin) = payload.role.as_deref().map(|r| r == "admin") {
        diesel::sql_query("UPDATE users SET is_admin = $1, updated_at = NOW() WHERE id = $2")
            .bind::<diesel::sql_types::Bool, _>(is_admin)
            .bind::<diesel::sql_types::Uuid, _>(user_id)
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    if let Some(is_active) = payload.is_active {
        diesel::sql_query("UPDATE users SET is_active = $1, updated_at = NOW() WHERE id = $2")
            .bind::<diesel::sql_types::Bool, _>(is_active)
            .bind::<diesel::sql_types::Uuid, _>(user_id)
            .execute(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    get_user_json(&state, user_id, &mut conn)
}

async fn get_user_roles_json(
    State(state): State<Arc<DirectoryApiState>>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = get_conn(&state)
        .ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, "database unavailable".to_string()))?;

    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct RoleRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        role_name: String,
    }

    let roles: Vec<RoleRow> = diesel::sql_query(
        "SELECT r.name AS role_name FROM rbac_user_roles ur \
         JOIN rbac_roles r ON r.id = ur.role_id WHERE ur.user_id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .load(&mut conn)
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(serde_json::json!({
        "user_id": user_id,
        "roles": roles.into_iter().map(|r| r.role_name).collect::<Vec<_>>(),
    })))
}

fn get_user_json(
    _state: &Arc<DirectoryApiState>,
    user_id: Uuid,
    conn: &mut diesel::PgConnection,
) -> Result<Json<UserResponse>, (StatusCode, String)> {
    use diesel::prelude::*;

    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct UserRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        username: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        email: String,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        is_active: bool,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        is_admin: bool,
    }

    let row: UserRow = diesel::sql_query(
        "SELECT id, username, email, is_active, is_admin FROM users WHERE id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .get_result(conn)
    .map_err(|e| (StatusCode::NOT_FOUND, format!("User not found: {e}")))?;

    Ok(Json(UserResponse {
        id: row.id,
        username: row.username,
        email: row.email,
        is_active: row.is_active,
        is_admin: row.is_admin,
    }))
}

#[derive(Debug, Deserialize)]
pub struct GroupUpdateRequest {
    pub name: Option<String>,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct GroupResponse {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub is_active: bool,
}

async fn list_organizations_json(
    State(state): State<Arc<DirectoryApiState>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = get_conn(&state)
        .ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, "database unavailable".to_string()))?;

    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct OrgRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        id: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
    }

    let rows: Vec<OrgRow> = diesel::sql_query(
        "SELECT org_id::text AS id, name FROM organizations WHERE org_id <> '00000000-0000-0000-0000-000000000000' ORDER BY name ASC",
    )
    .load(&mut conn)
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    let total = rows.len();
    Ok(Json(serde_json::json!({
        "organizations": rows.into_iter().map(|r| serde_json::json!({
            "id": r.id,
            "name": r.name,
            "primary_domain": null,
            "state": "active",
        })).collect::<Vec<_>>(),
        "total": total,
    })))
}
