use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, Utc};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::core::shared::state::AppState;

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

    let client = {
        let auth_service = state.auth_service.lock().await;
        auth_service.client().clone()
    };

    let user_id = match client
        .create_user(&req.email, &req.first_name, &req.last_name, Some(&req.username))
        .await
    {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to create user in Zitadel: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to create user".to_string(),
                    details: Some(e.to_string()),
                }),
            ));
        }
    };

    if let Some(ref org_id) = req.organization_id {
        let roles = req.roles.clone().unwrap_or_else(|| vec!["user".to_string()]);

        if let Err(e) = client.add_org_member(org_id, &user_id, roles.clone()).await {
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

    let client = {
        let auth_service = state.auth_service.lock().await;
        auth_service.client().clone()
    };

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
        match client
            .http_patch(format!("{}/users/{}", client.api_url(), user_id))
            .await
            .json(&serde_json::Value::Object(update_data))
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                info!("User {} profile updated successfully", user_id);
            }
            Ok(response) => {
                let status = response.status();
                error!("Failed to update user profile: {}", status);
            }
            Err(e) => {
                error!("Failed to update user profile: {}", e);
            }
        }
    }

    if let Some(ref org_id) = req.organization_id {
        let roles = req.roles.clone().unwrap_or_else(|| vec!["user".to_string()]);

        if let Err(e) = client.add_org_member(org_id, &user_id, roles.clone()).await {
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

    let client = {
        let auth_service = state.auth_service.lock().await;
        auth_service.client().clone()
    };

    match client
        .http_delete(format!("{}/v2/users/{}", client.api_url(), user_id))
        .await
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => {
            info!("User {} deleted successfully", user_id);
            Ok(Json(SuccessResponse {
                success: true,
                message: Some(format!("User {} deleted successfully", user_id)),
                user_id: Some(user_id),
            }))
        }
        Ok(response) => {
            let status = response.status();
            error!("Failed to delete user: {}", status);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to delete user".to_string(),
                    details: Some(format!("Server returned {}", status)),
                }),
            ))
        }
        Err(e) => {
            error!("Failed to delete user: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to delete user".to_string(),
                    details: Some(e.to_string()),
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

    let client = {
        let auth_service = state.auth_service.lock().await;
        auth_service.client().clone()
    };

    let users_result = if let Some(ref org_id) = params.organization_id {
        info!("Filtering users by organization: {}", org_id);
        client.get_org_members(org_id).await
    } else if let Some(ref search_term) = params.search {
        info!("Searching users with term: {}", search_term);
        client.search_users(search_term).await
    } else {
        let offset = (page - 1) * per_page;
        client.list_users(per_page, offset).await
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
            error!("Failed to list users: {}", e);
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

    let client = {
        let auth_service = state.auth_service.lock().await;
        auth_service.client().clone()
    };

    match client.get_user(&user_id).await {
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
            error!("Failed to get user profile: {}", e);
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

    let client = {
        let auth_service = state.auth_service.lock().await;
        auth_service.client().clone()
    };

    let roles = req.roles.unwrap_or_else(|| vec!["user".to_string()]);

    match client
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
            error!("Failed to assign user to organization: {}", e);
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

    let client = {
        let auth_service = state.auth_service.lock().await;
        auth_service.client().clone()
    };

    match client.remove_org_member(&org_id, &user_id).await {
        Ok(()) => {
            info!("User {} removed from organization {}", user_id, org_id);
            Ok(Json(SuccessResponse {
                success: true,
                message: Some(format!("User removed from organization {}", org_id)),
                user_id: Some(user_id),
            }))
        }
        Err(e) => {
            error!("Failed to remove user from organization: {}", e);
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

    let client = {
        let auth_service = state.auth_service.lock().await;
        auth_service.client().clone()
    };

    match client.get_user_memberships(&user_id, 0, 100).await {
        Ok(memberships) => {
            info!("Retrieved memberships for user {}", user_id);
            Ok(Json(memberships))
        }
        Err(e) => {
            error!("Failed to get user memberships: {}", e);
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

    let client = {
        let auth_service = state.auth_service.lock().await;
        auth_service.client().clone()
    };

    if let Err(e) = client.remove_org_member(&org_id, &user_id).await {
        error!("Failed to remove existing membership: {}", e);
    }

    match client
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
            error!("Failed to update user roles: {}", e);
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
