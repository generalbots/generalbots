use crate::core::directory::{BotAccess, UserAccount, UserProvisioningService, UserRole};
use crate::core::urls::ApiUrls;
use crate::shared::state::AppState;
use anyhow::Result;
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub organization: String,
    pub is_admin: bool,
    pub bots: Vec<BotAccessRequest>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BotAccessRequest {
    pub bot_id: String,
    pub bot_name: String,
    pub role: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub success: bool,
    pub message: String,
    pub user_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ServiceStatusResponse {
    pub directory: bool,
    pub database: bool,
    pub drive: bool,
    pub email: bool,
    pub git: bool,
}

/// POST /api/users/provision - Create user with full provisioning across all services
pub async fn provision_user_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateUserRequest>,
) -> impl IntoResponse {
    // Convert request to UserAccount
    let mut account = UserAccount {
        username: request.username.clone(),
        email: request.email,
        first_name: request.first_name,
        last_name: request.last_name,
        organization: request.organization,
        is_admin: request.is_admin,
        bots: Vec::new(),
    };

    // Convert bot access requests
    for bot_req in request.bots {
        let role = match bot_req.role.to_lowercase().as_str() {
            "admin" => UserRole::Admin,
            "readonly" | "read_only" => UserRole::ReadOnly,
            _ => UserRole::User,
        };

        account.bots.push(BotAccess {
            bot_id: bot_req.bot_id,
            bot_name: bot_req.bot_name.clone(),
            role,
            home_path: format!("/home/{}", request.username),
        });
    }

    // Get provisioning service
    let db_conn = match state.conn.get() {
        Ok(conn) => Arc::new(conn),
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(UserResponse {
                    success: false,
                    message: format!("Database connection failed: {}", e),
                    user_id: None,
                }),
            );
        }
    };

    let provisioning = UserProvisioningService::new(
        db_conn,
        state.drive.clone(),
        state.config.server.base_url.clone(),
    );

    // Provision the user
    match provisioning.provision_user(&account).await {
        Ok(_) => (
            StatusCode::CREATED,
            Json(UserResponse {
                success: true,
                message: format!("User {} created successfully", account.username),
                user_id: Some(account.username),
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(UserResponse {
                success: false,
                message: format!("Failed to provision user: {}", e),
                user_id: None,
            }),
        ),
    }
}

/// DELETE /api/users/:id/deprovision - Delete user and remove from all services
pub async fn deprovision_user_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let db_conn = match state.conn.get() {
        Ok(conn) => Arc::new(conn),
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(UserResponse {
                    success: false,
                    message: format!("Database connection failed: {}", e),
                    user_id: None,
                }),
            );
        }
    };

    let provisioning = UserProvisioningService::new(
        db_conn,
        state.drive.clone(),
        state.config.server.base_url.clone(),
    );

    match provisioning.deprovision_user(&id).await {
        Ok(_) => (
            StatusCode::OK,
            Json(UserResponse {
                success: true,
                message: format!("User {} deleted successfully", id),
                user_id: None,
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(UserResponse {
                success: false,
                message: format!("Failed to deprovision user: {}", e),
                user_id: None,
            }),
        ),
    }
}

/// GET /api/users/:id - Get user information
pub async fn get_user_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    use crate::shared::models::schema::users;
    use diesel::prelude::*;

    let conn = match state.conn.get() {
        Ok(conn) => conn,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Database connection failed: {}", e)
                })),
            );
        }
    };

    let user_result: Result<(String, String, String, bool), _> = users::table
        .filter(users::id.eq(&id))
        .select((users::id, users::username, users::email, users::is_admin))
        .first(&conn);

    match user_result {
        Ok((id, username, email, is_admin)) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "id": id,
                "username": username,
                "email": email,
                "is_admin": is_admin
            })),
        ),
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "User not found"
            })),
        ),
    }
}

/// GET /api/users - List all users
pub async fn list_users_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    use crate::shared::models::schema::users;
    use diesel::prelude::*;

    let conn = match state.conn.get() {
        Ok(conn) => conn,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Database connection failed: {}", e)
                })),
            );
        }
    };

    let users_result: Result<Vec<(String, String, String, bool)>, _> = users::table
        .select((users::id, users::username, users::email, users::is_admin))
        .load(&conn);

    match users_result {
        Ok(users_list) => {
            let users_json: Vec<_> = users_list
                .into_iter()
                .map(|(id, username, email, is_admin)| {
                    serde_json::json!({
                        "id": id,
                        "username": username,
                        "email": email,
                        "is_admin": is_admin
                    })
                })
                .collect();

            (
                StatusCode::OK,
                Json(serde_json::json!({ "users": users_json })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": format!("Failed to list users: {}", e)
            })),
        ),
    }
}

/// GET /api/services/status - Check all integrated services status
pub async fn check_services_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut status = ServiceStatusResponse {
        directory: false,
        database: false,
        drive: false,
        email: false,
        git: false,
    };

    // Check database
    status.database = state.conn.get().is_ok();

    // Check S3/MinIO
    if let Ok(result) = state.drive.list_buckets().send().await {
        status.drive = result.buckets.is_some();
    }

    // Check Directory (Zitadel)
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .unwrap();

    if let Ok(response) = client.get("https://localhost:8080/healthz").send().await {
        status.directory = response.status().is_success();
    }

    // Check Email (Stalwart)
    if let Ok(response) = client.get("https://localhost:8025/health").send().await {
        status.email = response.status().is_success();
    }

    // Check Git (Forgejo)
    if let Ok(response) = client
        .get("https://localhost:3000/api/v1/version")
        .send()
        .await
    {
        status.git = response.status().is_success();
    }

    (StatusCode::OK, Json(status))
}

/// Configure user and provisioning routes
pub fn configure_user_routes() -> Router<Arc<AppState>> {
    Router::new()
        // User management
        .route(ApiUrls::USERS, get(list_users_handler))
        .route(ApiUrls::USER_BY_ID, get(get_user_handler))
        .route(ApiUrls::USER_PROVISION, post(provision_user_handler))
        .route(ApiUrls::USER_DEPROVISION, delete(deprovision_user_handler))
        // Service status
        .route(ApiUrls::SERVICES_STATUS, get(check_services_status))
}
