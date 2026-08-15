use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use botcore::shared::state::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub display_name: Option<String>,
    pub role: Option<String>,
    pub organization_id: Option<String>,
    pub roles: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub organization_id: Option<String>,
    pub roles: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UserQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub search: Option<String>,
    pub organization_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub display_name: Option<String>,
    pub state: String,
    pub organization_id: Option<String>,
    pub roles: Vec<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct UserListResponse {
    pub users: Vec<UserResponse>,
    pub total: usize,
    pub page: u32,
    pub per_page: u32,
}

#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub success: bool,
    pub message: Option<String>,
    pub user_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub details: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AssignOrganizationRequest {
    pub organization_id: String,
    pub roles: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRolesRequest {
    pub roles: Vec<String>,
}

pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Creating user: {} ({})", req.username, req.email);

    let auth_service = state.auth_service.as_ref().ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "No auth service".to_string(), details: None })))?.lock().await;

    let user_id = match auth_service
        .create_user(&req.email, &req.first_name, &req.last_name, Some(&req.username))
        .await
    {
        Ok(id) => id,
        Err(e) => {
            log::error!("Failed to create user in Zitadel: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to create user".to_string(),
                    details: Some(e),
                }),
            ));
        }
    };

    if let Some(password) = &req.password {
        if let Err(e) = auth_service.set_user_password(&user_id, password).await {
            error!("Failed to set initial password for user {}: {}", user_id, e);
        } else {
            info!("Initial password set for user {}", user_id);
        }
    }

    if let Some(ref org_id) = req.organization_id {
        let roles = req.roles.clone().unwrap_or_else(|| vec!["user".to_string()]);

        if let Err(e) = auth_service.add_org_member(org_id, &user_id, roles.clone()).await {
            error!(
                "Failed to add user {} to organization {}: {}",
                user_id, org_id, e
            );
        } else {
            info!(
                "User {} added to organization {} with roles {:?}",
                user_id, org_id, roles
            );
        }
    }

    info!("User created successfully: {}", user_id);
    Ok(Json(SuccessResponse {
        success: true,
        message: Some(format!("User {} created successfully", req.username)),
        user_id: Some(user_id),
    }))
}

pub async fn update_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Updating user: {}", user_id);

    let auth_service = state.auth_service.as_ref().ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "No auth service".to_string(), details: None })))?.lock().await;

    let mut update_data = serde_json::Map::new();
    if let Some(username) = &req.username {
        update_data.insert("userName".to_string(), serde_json::json!(username));
    }
    if let Some(email) = &req.email {
        update_data.insert("email".to_string(), serde_json::json!(email));
    }
    if let Some(first_name) = &req.first_name {
        update_data.insert("firstName".to_string(), serde_json::json!(first_name));
    }
    if let Some(last_name) = &req.last_name {
        update_data.insert("lastName".to_string(), serde_json::json!(last_name));
    }
    if let Some(display_name) = &req.display_name {
        update_data.insert("displayName".to_string(), serde_json::json!(display_name));
    }
    if let Some(phone) = &req.phone {
        update_data.insert("phone".to_string(), serde_json::json!(phone));
    }

    if !update_data.is_empty() {
        match auth_service
            .http_patch(format!("{}/users/{}", auth_service.api_url(), user_id), serde_json::Value::Object(update_data))
            .await
        {
            Ok(_) => {
                info!("User {} profile updated successfully", user_id);
            }
            Err(e) => {
                log::error!("Failed to update user profile: {}", e);
            }
        }
    }

    if let Some(ref org_id) = req.organization_id {
        let roles = req.roles.clone().unwrap_or_else(|| vec!["user".to_string()]);

        if let Err(e) = auth_service.add_org_member(org_id, &user_id, roles.clone()).await {
            error!(
                "Failed to update user {} organization membership: {}",
                user_id, e
            );
        } else {
            info!(
                "User {} organization membership updated to {} with roles {:?}",
                user_id, org_id, roles
            );
        }
    }

    Ok(Json(SuccessResponse {
        success: true,
        message: Some(format!("User {} updated successfully", user_id)),
        user_id: Some(user_id),
    }))
}

pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Deleting user: {}", user_id);

    let auth_service = state.auth_service.as_ref().ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "No auth service".to_string(), details: None })))?.lock().await;

    match auth_service
        .http_delete(format!("{}/v2/users/{}", auth_service.api_url(), user_id))
        .await
    {
        Ok(_) => {
            info!("User {} deleted successfully", user_id);
            Ok(Json(SuccessResponse {
                success: true,
                message: Some(format!("User {} deleted successfully", user_id)),
                user_id: Some(user_id),
            }))
        }
        Err(e) => {
            log::error!("Failed to delete user: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to delete user".to_string(),
                    details: Some(e),
                }),
            ))
        }
    }
}

pub async fn list_users(
    State(state): State<Arc<AppState>>,
    Query(params): Query<UserQuery>,
) -> Result<Json<UserListResponse>, (StatusCode, Json<ErrorResponse>)> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);

    info!("Listing users (page: {}, per_page: {})", page, per_page);

    let auth_service = state.auth_service.as_ref().ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "No auth service".to_string(), details: None })))?.lock().await;

    let users_result = if let Some(ref org_id) = params.organization_id {
        info!("Filtering users by organization: {}", org_id);
        auth_service.get_org_members(org_id).await
    } else if let Some(ref search_term) = params.search {
        info!("Searching users with term: {}", search_term);
        auth_service.search_users(search_term).await
    } else {
        let offset = (page - 1) * per_page;
        auth_service.list_users(per_page as i64, offset as i64).await.map(|v| v.as_array().cloned().unwrap_or_default())
    };

    match users_result {
        Ok(users_json) => {
            let users: Vec<UserResponse> = users_json
                .into_iter()
                .filter_map(|u| {
                    let id = u.get("userId").and_then(|v| v.as_str()).map(String::from)
                        .or_else(|| u.get("user_id").and_then(|v| v.as_str()).map(String::from))?;

                    let username = u.get("userName").and_then(|v| v.as_str())
                        .or_else(|| u.get("username").and_then(|v| v.as_str()))
                        .unwrap_or("unknown")
                        .to_string();

                    let email = u.get("preferredLoginName").and_then(|v| v.as_str())
                        .or_else(|| u.get("email").and_then(|v| v.as_str()))
                        .unwrap_or("unknown@example.com")
                        .to_string();

                    let first_name = u.get("profile")
                        .and_then(|p| p.get("givenName"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    let last_name = u.get("profile")
                        .and_then(|p| p.get("familyName"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    let display_name = u.get("profile")
                        .and_then(|p| p.get("displayName"))
                        .and_then(|v| v.as_str())
                        .map(String::from);

                    let state = u.get("state").and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();

                    let organization_id = u.get("orgId").and_then(|v| v.as_str())
                        .or_else(|| u.get("organization_id").and_then(|v| v.as_str()))
                        .map(String::from);

                    let roles = u.get("roles")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|r| r.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default();

                    Some(UserResponse {
                        id,
                        username,
                        email,
                        first_name,
                        last_name,
                        display_name,
                        state,
                        organization_id,
                        roles,
                        created_at: None,
                        updated_at: None,
                    })
                })
                .collect();

            let total = users.len();
            info!("Found {} users", total);

            Ok(Json(UserListResponse {
                users,
                total,
                page,
                per_page,
            }))
        }
        Err(e) => {
            log::error!("Failed to list users: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to list users".to_string(),
                    details: Some(e.to_string()),
                }),
            ))
        }
    }
}

pub async fn get_user_profile(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> Result<Json<UserResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Getting profile for user: {}", user_id);

    let auth_service = state.auth_service.as_ref().ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "No auth service".to_string(), details: None })))?.lock().await;

    match auth_service.get_user(&user_id).await {
        Ok(user_data) => {
            let id = user_data.get("id").and_then(|v| v.as_str())
                .unwrap_or(&user_id)
                .to_string();

            let username = user_data.get("username").and_then(|v| v.as_str())
                .or_else(|| user_data.get("userName").and_then(|v| v.as_str()))
                .unwrap_or("unknown")
                .to_string();

            let email = user_data.get("preferredLoginName").and_then(|v| v.as_str())
                .or_else(|| user_data.get("email").and_then(|v| v.as_str()))
                .unwrap_or("unknown@example.com")
                .to_string();

            let first_name = user_data.get("profile")
                .and_then(|p| p.get("givenName"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let last_name = user_data.get("profile")
                .and_then(|p| p.get("familyName"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let display_name = user_data.get("profile")
                .and_then(|p| p.get("displayName"))
                .and_then(|v| v.as_str())
                .map(String::from);

            let state = user_data.get("state").and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            let organization_id = user_data.get("orgId").and_then(|v| v.as_str())
                .map(String::from);

            let roles = user_data.get("roles")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|r| r.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();

            let user = UserResponse {
                id,
                username: username.clone(),
                email,
                first_name,
                last_name,
                display_name,
                state,
                organization_id,
                roles,
                created_at: None,
                updated_at: None,
            };

            info!("User profile retrieved: {}", username);
            Ok(Json(user))
        }
        Err(e) => {
            log::error!("Failed to get user profile: {}", e);
            Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "User not found".to_string(),
                    details: Some(e.to_string()),
                }),
            ))
        }
    }
}

pub async fn assign_organization(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
    Json(req): Json<AssignOrganizationRequest>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!(
        "Assigning user {} to organization {}",
        user_id, req.organization_id
    );

    let auth_service = state.auth_service.as_ref().ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "No auth service".to_string(), details: None })))?.lock().await;

    let roles = req.roles.unwrap_or_else(|| vec!["user".to_string()]);

    match auth_service
        .add_org_member(&req.organization_id, &user_id, roles.clone())
        .await
    {
        Ok(()) => {
            info!(
                "User {} assigned to organization {} with roles {:?}",
                user_id, req.organization_id, roles
            );
            Ok(Json(SuccessResponse {
                success: true,
                message: Some(format!(
                    "User assigned to organization {} with roles {:?}",
                    req.organization_id, roles
                )),
                user_id: Some(user_id),
            }))
        }
        Err(e) => {
            log::error!("Failed to assign user to organization: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to assign user to organization".to_string(),
                    details: Some(e.to_string()),
                }),
            ))
        }
    }
}

pub async fn remove_from_organization(
    State(state): State<Arc<AppState>>,
    Path((user_id, org_id)): Path<(String, String)>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Removing user {} from organization {}", user_id, org_id);

    let auth_service = state.auth_service.as_ref().ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "No auth service".to_string(), details: None })))?.lock().await;

    match auth_service.remove_org_member(&org_id, &user_id).await {
        Ok(()) => {
            info!("User {} removed from organization {}", user_id, org_id);
            Ok(Json(SuccessResponse {
                success: true,
                message: Some(format!("User removed from organization {}", org_id)),
                user_id: Some(user_id),
            }))
        }
        Err(e) => {
            log::error!("Failed to remove user from organization: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to remove user from organization".to_string(),
                    details: Some(e.to_string()),
                }),
            ))
        }
    }
}

pub async fn get_user_memberships(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    info!("Getting memberships for user: {}", user_id);

    let auth_service = state.auth_service.as_ref().ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "No auth service".to_string(), details: None })))?.lock().await;

    match auth_service.get_user_memberships(&user_id, 0, 100).await {
        Ok(memberships) => {
            info!("Retrieved memberships for user {}", user_id);
            Ok(Json(memberships))
        }
        Err(e) => {
            log::error!("Failed to get user memberships: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to get user memberships".to_string(),
                    details: Some(e.to_string()),
                }),
            ))
        }
    }
}

pub async fn update_user_roles(
    State(state): State<Arc<AppState>>,
    Path((user_id, org_id)): Path<(String, String)>,
    Json(req): Json<UpdateRolesRequest>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!(
        "Updating roles for user {} in organization {}: {:?}",
        user_id, org_id, req.roles
    );

    let auth_service = state.auth_service.as_ref().ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "No auth service".to_string(), details: None })))?.lock().await;

    if let Err(e) = auth_service.remove_org_member(&org_id, &user_id).await {
        log::error!("Failed to remove existing membership: {}", e);
    }

    match auth_service
        .add_org_member(&org_id, &user_id, req.roles.clone())
        .await
    {
        Ok(()) => {
            info!(
                "User {} roles updated in organization {}: {:?}",
                user_id, org_id, req.roles
            );
            Ok(Json(SuccessResponse {
                success: true,
                message: Some(format!("User roles updated to {:?}", req.roles)),
                user_id: Some(user_id),
            }))
        }
        Err(e) => {
            log::error!("Failed to update user roles: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to update user roles".to_string(),
                    details: Some(e.to_string()),
                }),
            ))
        }
    }
}


#[derive(Debug, Serialize)]
pub struct UserPermissionsResponse {
    pub user_id: String,
    pub permissions: Vec<String>,
    pub roles: Vec<String>,
}

pub async fn get_user_permissions(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> Result<Json<UserPermissionsResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Getting permissions for user: {}", user_id);

    let auth_service = state.auth_service.as_ref().ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "No auth service".to_string(), details: None })))?.lock().await;

    let roles = match auth_service.get_user_memberships(&user_id, 0, 100).await {
        Ok(data) => {
            data.get("result")
                .and_then(|r| r.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|m| {
                            m.get("roles")
                                .and_then(|r| r.as_array())
                                .map(|roles| {
                                    roles.iter()
                                        .filter_map(|r| r.as_str().map(String::from))
                                        .collect::<Vec<String>>()
                                })
                        })
                        .flatten()
                        .collect::<Vec<String>>()
                })
                .unwrap_or_default()
        }
        Err(_) => vec![],
    };

    Ok(Json(UserPermissionsResponse {
        user_id: user_id.clone(),
        permissions: vec![],
        roles,
    }))
}


pub async fn get_user_presence(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    info!("Getting presence for user: {}", user_id);

    let user_uuid = user_id.parse::<uuid::Uuid>().map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid user id".to_string(),
                details: None,
            }),
        )
    })?;
    let mut conn = state.conn.get().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("DB error: {e}"),
                details: None,
            }),
        )
    })?;

    #[derive(diesel::QueryableByName)]
    struct LastActivityRow {
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
        last_activity: Option<DateTime<Utc>>,
    }

    let row: Option<LastActivityRow> = diesel::sql_query(
        "SELECT last_activity FROM user_sessions WHERE user_id = $1 ORDER BY last_activity DESC LIMIT 1",
    )
    .bind::<diesel::sql_types::Uuid, _>(user_uuid)
    .get_result(&mut conn)
    .ok();

    let last_seen = row.as_ref().and_then(|r| r.last_activity);
    let online = last_seen
        .map(|t| Utc::now().signed_duration_since(t) < chrono::Duration::minutes(5))
        .unwrap_or(false);

    Ok(Json(serde_json::json!({
        "user_id": user_id,
        "status": if online { "online" } else { "offline" },
        "last_seen": last_seen.map(|t| t.to_rfc3339()),
        "online": online
    })))
}


pub async fn get_user_activity(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    info!("Getting activity for user: {}", user_id);

    let user_uuid = user_id.parse::<uuid::Uuid>().map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid user id".to_string(),
                details: None,
            }),
        )
    })?;
    let mut conn = state.conn.get().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("DB error: {e}"),
                details: None,
            }),
        )
    })?;

    #[derive(diesel::QueryableByName)]
    struct ActivityRow {
        #[diesel(sql_type = diesel::sql_types::Varchar)]
        title: String,
        #[diesel(sql_type = diesel::sql_types::Int4)]
        message_count: i32,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        updated_at: DateTime<Utc>,
    }

    let rows: Vec<ActivityRow> = diesel::sql_query(
        "SELECT title, message_count, updated_at FROM user_sessions WHERE user_id = $1 ORDER BY updated_at DESC LIMIT 10",
    )
    .bind::<diesel::sql_types::Uuid, _>(user_uuid)
    .load(&mut conn)
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to load activity: {e}"),
                details: None,
            }),
        )
    })?;

    let recent_activity: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "type": "session",
                "title": r.title,
                "message_count": r.message_count,
                "timestamp": r.updated_at.to_rfc3339(),
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "user_id": user_id,
        "recent_activity": recent_activity,
        "total_sessions": rows.len()
    })))
}


#[derive(Debug, Deserialize)]
pub struct Enable2faRequest {
    pub method: Option<String>,
}

fn set_mfa_flag(
    state: &Arc<AppState>,
    user_id: &str,
    enabled: bool,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let user_uuid = user_id.parse::<uuid::Uuid>().map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid user id".to_string(),
                details: None,
            }),
        )
    })?;
    let mut conn = state.conn.get().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("DB error: {e}"),
                details: None,
            }),
        )
    })?;
    let value = if enabled { "true" } else { "false" };
    diesel::sql_query(
        "INSERT INTO user_preferences (id, user_id, preference_key, preference_value, created_at, updated_at) \
         VALUES ($1, $2, 'mfa_enabled', $3::jsonb, NOW(), NOW()) \
         ON CONFLICT (user_id, preference_key) \
         DO UPDATE SET preference_value = EXCLUDED.preference_value, updated_at = NOW()",
    )
    .bind::<diesel::sql_types::Uuid, _>(uuid::Uuid::new_v4())
    .bind::<diesel::sql_types::Uuid, _>(user_uuid)
    .bind::<diesel::sql_types::Text, _>(value)
    .execute(&mut conn)
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to update 2FA: {e}"),
                details: None,
            }),
        )
    })?;
    Ok(())
}

pub async fn enable_2fa(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
    Json(_req): Json<Enable2faRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    info!("Enabling 2FA for user: {}", user_id);

    set_mfa_flag(&state, &user_id, true)?;

    Ok(Json(serde_json::json!({
        "success": true,
        "user_id": user_id,
        "message": "2FA enabled successfully"
    })))
}


pub async fn disable_2fa(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    info!("Disabling 2FA for user: {}", user_id);

    set_mfa_flag(&state, &user_id, false)?;

    Ok(Json(serde_json::json!({
        "success": true,
        "user_id": user_id,
        "message": "2FA disabled successfully"
    })))
}


pub async fn get_user_devices(
    State(_state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    info!("Getting devices for user: {}", user_id);

    Ok(Json(serde_json::json!({
        "user_id": user_id,
        "devices": []
    })))
}


pub async fn get_user_sessions(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    info!("Getting sessions for user: {}", user_id);

    let user_uuid = user_id.parse::<uuid::Uuid>().map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid user id".to_string(),
                details: None,
            }),
        )
    })?;
    let mut conn = state.conn.get().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("DB error: {e}"),
                details: None,
            }),
        )
    })?;

    #[derive(diesel::QueryableByName)]
    struct SessionRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: uuid::Uuid,
        #[diesel(sql_type = diesel::sql_types::Varchar)]
        title: String,
        #[diesel(sql_type = diesel::sql_types::Int4)]
        message_count: i32,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        last_activity: DateTime<Utc>,
    }

    let sessions: Vec<SessionRow> = diesel::sql_query(
        "SELECT id, title, message_count, last_activity FROM user_sessions WHERE user_id = $1 ORDER BY last_activity DESC LIMIT 50",
    )
    .bind::<diesel::sql_types::Uuid, _>(user_uuid)
    .load(&mut conn)
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to load sessions: {e}"),
                details: None,
            }),
        )
    })?;

    let sessions_json: Vec<serde_json::Value> = sessions
        .iter()
        .map(|s| {
            serde_json::json!({
                "id": s.id,
                "title": s.title,
                "message_count": s.message_count,
                "last_activity": s.last_activity.to_rfc3339(),
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "user_id": user_id,
        "sessions": sessions_json
    })))
}


#[derive(Debug, Deserialize)]
pub struct UpdateNotificationPreferencesRequest {
    pub email: Option<bool>,
    pub push: Option<bool>,
    pub sms: Option<bool>,
}

pub async fn update_notification_preferences(
    State(_state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
    Json(req): Json<UpdateNotificationPreferencesRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    info!("Updating notification preferences for user: {}: {:?}", user_id, req);

    Ok(Json(serde_json::json!({
        "success": true,
        "user_id": user_id,
        "preferences": {
            "email": req.email.unwrap_or(true),
            "push": req.push.unwrap_or(true),
            "sms": req.sms.unwrap_or(false)
        }
    })))
}
