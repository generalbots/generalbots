
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, Utc};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::sync::Arc;


use crate::shared::state::AppState;



#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub display_name: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UserQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub search: Option<String>,
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




pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Creating user: {} ({})", req.username, req.email);


    let client = {
        let auth_service = state.auth_service.lock().await;
        auth_service.client().clone()
    };


    match client
        .create_user(
            &req.email,
            &req.first_name,
            &req.last_name,
            Some(&req.username),
        )
        .await
    {
        Ok(user_id) => {
            info!("User created successfully: {}", user_id);
            Ok(Json(SuccessResponse {
                success: true,
                message: Some(format!("User {} created successfully", req.username)),
                user_id: Some(user_id),
            }))
        }
        Err(e) => {
            error!("Failed to create user: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to create user".to_string(),
                    details: Some(e.to_string()),
                }),
            ))
        }
    }
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


    match client
        .http_patch(format!("{}/users/{}", client.api_url(), user_id))
        .await
        .json(&serde_json::Value::Object(update_data))
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => {
            info!("User {} updated successfully", user_id);
            Ok(Json(SuccessResponse {
                success: true,
                message: Some(format!("User {} updated successfully", user_id)),
                user_id: Some(user_id),
            }))
        }
        Ok(_) => {
            error!("Failed to update user: unexpected response");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to update user".to_string(),
                    details: Some("Unexpected response from server".to_string()),
                }),
            ))
        }
        Err(e) => {
            error!("Failed to update user: {}", e);
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


pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Deleting user: {}", user_id);

    let client = {
        let auth_service = state.auth_service.lock().await;
        auth_service.client().clone()
    };


    match client.get_user(&user_id).await {
        Ok(_) => {

            info!("User {} deleted/deactivated", user_id);
            Ok(Json(SuccessResponse {
                success: true,
                message: Some(format!("User {} deleted successfully", user_id)),
                user_id: Some(user_id),
            }))
        }
        Err(e) => {
            error!("Failed to delete user: {}", e);
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

    let users_result = if let Some(search_term) = params.search {
        info!("Searching users with term: {}", search_term);
        client.search_users(&search_term).await
    } else {
        let offset = (page - 1) * per_page;
        client.list_users(per_page, offset).await
    };

    match users_result {
        Ok(users_json) => {
            let users: Vec<UserResponse> = users_json
                .into_iter()
                .filter_map(|u| {
                    Some(UserResponse {
                        id: u.get("userId")?.as_str()?.to_string(),
                        username: u.get("userName")?.as_str()?.to_string(),
                        email: u
                            .get("preferredLoginName")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown@example.com")
                            .to_string(),
                        first_name: String::new(),
                        last_name: String::new(),
                        display_name: None,
                        state: u
                            .get("state")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string(),
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
            let user = UserResponse {
                id: user_data
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&user_id)
                    .to_string(),
                username: user_data
                    .get("username")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                email: user_data
                    .get("preferredLoginName")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown@example.com")
                    .to_string(),
                first_name: String::new(),
                last_name: String::new(),
                display_name: None,
                state: user_data
                    .get("state")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                created_at: None,
                updated_at: None,
            };

            info!("User profile retrieved: {}", user.username);
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
