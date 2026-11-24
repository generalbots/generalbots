use crate::{BotAccess, UserAccount, UserProvisioningService, UserRole};
use anyhow::Result;
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

pub use botlib::db_pool::DbPool;

pub struct DirectoryApiState {
    pub conn: DbPool,
    pub base_url: String,
}

// SECURITY (#2850): these are the *wire* types for POST /api/directory/users/provision.
// Privilege-bearing fields (`is_admin` and per-bot `role`) are deliberately absent.
// `deny_unknown_fields` turns a forged field into a 422 instead of silently
// escalating the caller: admin status is never accepted from the request body.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub organization: String,
    pub bots: Vec<BotAccessRequest>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BotAccessRequest {
    pub bot_id: String,
    pub bot_name: String,
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

pub fn configure_user_routes() -> Router<Arc<DirectoryApiState>> {
    Router::new()
        .route("/api/directory/users", get(list_users_handler))
        .route("/api/directory/users/:id", get(get_user_handler))
        .route("/api/directory/users/provision", post(provision_user_handler))
        .route("/api/directory/users/:id/deprovision", delete(deprovision_user_handler))
        .route("/api/directory/services/status", get(check_services_status))
        .route(
            "/api/directory/users/create",
            axum::routing::post(crate::ui_fragments::create_user),
        )
        .route(
            "/api/directory/users/:id",
            axum::routing::put(crate::ui_fragments::update_user),
        )
        .merge(crate::ui_fragments::configure_directory_ui_routes())
        .merge(crate::ui_api::configure_directory_api_routes())
}

/// Builds the internal provisioning account from a validated wire request.
///
/// SECURITY (#2850): privilege is NEVER copied from the client. Every
/// provisioned account starts as a non-admin with plain `User` access to its
/// bots; elevation must go through a verified, admin-gated server-side path
/// (RBAC group/role assignment or the dedicated admin-promotion endpoints).
fn build_provisioning_account(request: &CreateUserRequest) -> UserAccount {
    let mut account = UserAccount {
        username: request.username.clone(),
        email: request.email.clone(),
        first_name: request.first_name.clone(),
        last_name: request.last_name.clone(),
        organization: request.organization.clone(),
        bots: Vec::new(),
    };

    for bot_req in &request.bots {
        account.bots.push(BotAccess {
            bot_id: bot_req.bot_id.clone(),
            bot_name: bot_req.bot_name.clone(),
            role: UserRole::User,
            home_path: format!("/home/{}", request.username),
        });
    }

    account
}

pub async fn provision_user_handler(
    State(state): State<Arc<DirectoryApiState>>,
    Json(request): Json<CreateUserRequest>,
) -> impl IntoResponse {
    let account = build_provisioning_account(&request);

    let provisioning = UserProvisioningService::new(state.conn.clone(), None, state.base_url.clone());

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

pub async fn deprovision_user_handler(
    State(state): State<Arc<DirectoryApiState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let provisioning = UserProvisioningService::new(state.conn.clone(), None, state.base_url.clone());

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

pub async fn get_user_handler(
    State(state): State<Arc<DirectoryApiState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    use diesel::prelude::*;

    let mut conn = match state.conn.get() {
        Ok(conn) => conn,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": botsecurity::error_sanitizer::log_and_sanitize_json(&e, "botcoredirectory::api", None)})),
            );
        }
    };

    let Ok(user_uuid) = Uuid::parse_str(&id) else {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Invalid user ID format"})));
    };

    let user_result: Result<(Uuid, String, String, bool), _> = schema::users::table
        .filter(schema::users::id.eq(user_uuid))
        .select((schema::users::id, schema::users::username, schema::users::email, schema::users::is_admin))
        .first(&mut conn);

    match user_result {
        Ok((user_id, username, email, is_admin)) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "id": user_id.to_string(),
                "username": username,
                "email": email,
                "is_admin": is_admin
            })),
        ),
        Err(_) => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "User not found"}))),
    }
}

pub async fn list_users_handler(State(state): State<Arc<DirectoryApiState>>) -> impl IntoResponse {
    use diesel::prelude::*;

    let mut conn = match state.conn.get() {
        Ok(conn) => conn,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": botsecurity::error_sanitizer::log_and_sanitize_json(&e, "botcoredirectory::api", None)})),
            );
        }
    };

    let users_result: Result<Vec<(Uuid, String, String, bool)>, _> = schema::users::table
        .select((schema::users::id, schema::users::username, schema::users::email, schema::users::is_admin))
        .load(&mut conn);

    match users_result {
        Ok(users_list) => {
            let users_json: Vec<_> = users_list
                .into_iter()
                .map(|(user_id, username, email, is_admin)| {
                    serde_json::json!({
                        "id": user_id.to_string(),
                        "username": username,
                        "email": email,
                        "is_admin": is_admin
                    })
                })
                .collect();
            (StatusCode::OK, Json(serde_json::json!({"users": users_json})))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": botsecurity::error_sanitizer::log_and_sanitize_json(&e, "botcoredirectory::api", None)})),
        ),
    }
}

pub async fn check_services_status(State(state): State<Arc<DirectoryApiState>>) -> impl IntoResponse {
    let mut status = ServiceStatusResponse {
        directory: false,
        database: false,
        drive: false,
        email: false,
        git: false,
    };

    status.database = state.conn.get().is_ok();

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            log::warn!("Failed to build status-check HTTP client: {:?}", e);
            reqwest::Client::new()
        }
    };

    // Server-side request forgery guard: the configured directory base URL
    // must be an http(s) URL with an explicit host before it is requested.
    let directory_ok = url::Url::parse(&state.base_url).is_ok_and(|parsed| {
        matches!(parsed.scheme(), "http" | "https") && parsed.host_str().is_some()
    });
    if directory_ok {
        if let Ok(response) = client.get(format!("{}/healthz", state.base_url)).send().await {
            status.directory = response.status().is_success();
        }
    }

    if let Ok(response) = client.get("https://localhost:8025/health").send().await {
        status.email = response.status().is_success();
    }

    if let Ok(response) = client.get("https://localhost:3000/api/version").send().await {
        status.git = response.status().is_success();
    }

    (StatusCode::OK, Json(status))
}

mod schema {
  // Canonical definition lives in `botschema` (#1497); the local copy had a
  // ghost `is_admin` and omitted the account columns.
  pub use botschema::users;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_payload() -> serde_json::Value {
        serde_json::json!({
            "username": "jdoe",
            "email": "jdoe@example.com",
            "first_name": "John",
            "last_name": "Doe",
            "organization": "org",
            "bots": [{ "bot_id": "bot-1", "bot_name": "default" }]
        })
    }

    #[test]
    fn forged_is_admin_true_is_rejected() {
        let mut payload = valid_payload();
        payload["is_admin"] = serde_json::json!(true);
        assert!(
            serde_json::from_value::<CreateUserRequest>(payload).is_err(),
            "a client-supplied is_admin must not deserialize into the provisioning request"
        );
    }

    #[test]
    fn forged_bot_role_admin_is_rejected() {
        let mut payload = valid_payload();
        payload["bots"][0]["role"] = serde_json::json!("admin");
        assert!(
            serde_json::from_value::<CreateUserRequest>(payload).is_err(),
            "a client-supplied per-bot role must not deserialize into the provisioning request"
        );
    }

    #[test]
    fn valid_request_grants_no_elevated_bot_role() {
        let request: CreateUserRequest =
            serde_json::from_value(valid_payload()).expect("valid provisioning payload");
        let account = build_provisioning_account(&request);
        assert!(
            account.bots.iter()
                .all(|access| access.role == UserRole::User),
            "provisioning must never grant an elevated bot role"
        );
    }
}
