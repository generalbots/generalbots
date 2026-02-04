use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use chrono::{Duration, Utc};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use once_cell::sync::Lazy;

use crate::shared::models::UserLoginToken;
use crate::shared::state::AppState;
use crate::shared::schema::user_login_tokens::dsl::*;
use diesel::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionUserData {
    pub user_id: String,
    pub email: String,
    pub username: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub display_name: Option<String>,
    pub organization_id: Option<String>,
    pub roles: Vec<String>,
    pub created_at: i64,
}

pub static SESSION_CACHE: Lazy<RwLock<HashMap<String, SessionUserData>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

const BOOTSTRAP_SECRET_ENV: &str = "GB_BOOTSTRAP_SECRET";

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    pub remember: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_in: Option<i64>,
    pub requires_2fa: bool,
    pub session_token: Option<String>,
    pub redirect: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CurrentUserResponse {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub display_name: Option<String>,
    pub roles: Vec<String>,
    pub organization_id: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub details: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LogoutResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct TwoFactorRequest {
    pub session_token: String,
    pub code: String,
    pub trust_device: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct BootstrapAdminRequest {
    pub bootstrap_secret: String,
    pub email: String,
    pub username: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub organization_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BootstrapResponse {
    pub success: bool,
    pub message: String,
    pub user_id: Option<String>,
    pub organization_id: Option<String>,
}

pub fn configure() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(crate::directory::auth_handler))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/me", get(get_current_user))
        .route("/refresh", post(refresh_token))
        .route("/2fa/verify", post(verify_2fa))
        .route("/2fa/resend", post(resend_2fa))
        .route("/bootstrap", post(bootstrap_admin))
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Login attempt for: {}", req.email);

    let client = {
        let auth_service = state.auth_service.lock().await;
        auth_service.client().clone()
    };

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| {
            error!("Failed to create HTTP client: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Internal server error".to_string(),
                    details: None,
                }),
            )
        })?;

    let pat_path = std::path::Path::new("./botserver-stack/conf/directory/admin-pat.txt");
    let admin_token = std::fs::read_to_string(pat_path)
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    if admin_token.is_empty() {
        error!("Admin PAT token not found");
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Authentication service not configured".to_string(),
                details: None,
            }),
        ));
    }

    let search_url = format!("{}/v2/users", client.api_url());
    let search_body = serde_json::json!({
        "queries": [{
            "emailQuery": {
                "emailAddress": req.email,
                "method": "TEXT_QUERY_METHOD_EQUALS"
            }
        }]
    });

    let user_response = http_client
        .post(&search_url)
        .bearer_auth(&admin_token)
        .json(&search_body)
        .send()
        .await
        .map_err(|e| {
            error!("Failed to search user: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Authentication service error".to_string(),
                    details: None,
                }),
            )
        })?;

    if !user_response.status().is_success() {
        let error_text = user_response.text().await.unwrap_or_default();
        error!("User search failed: {}", error_text);
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid email or password".to_string(),
                details: None,
            }),
        ));
    }

    let user_data: serde_json::Value = user_response.json().await.map_err(|e| {
        error!("Failed to parse user response: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Authentication service error".to_string(),
                details: None,
            }),
        )
    })?;

    let user_id = user_data
        .get("result")
        .and_then(|r| r.as_array())
        .and_then(|arr| arr.first())
        .and_then(|u| u.get("userId"))
        .and_then(|id| id.as_str())
        .map(String::from);

    let user_id = match user_id {
        Some(id) => id,
        None => {
            error!("User not found: {}", req.email);
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Invalid email or password".to_string(),
                    details: None,
                }),
            ));
        }
    };

    let session_url = format!("{}/v2/sessions", client.api_url());
    let session_body = serde_json::json!({
        "checks": {
            "user": {
                "userId": user_id
            },
            "password": {
                "password": req.password
            }
        }
    });

    let session_response = http_client
        .post(&session_url)
        .bearer_auth(&admin_token)
        .json(&session_body)
        .send()
        .await
        .map_err(|e| {
            error!("Failed to create session: {}", e);
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Authentication failed".to_string(),
                    details: None,
                }),
            )
        })?;

    if !session_response.status().is_success() {
        let status = session_response.status();
        let error_text = session_response.text().await.unwrap_or_default();
        error!("Session creation failed: {} - {}", status, error_text);

        if error_text.contains("password") || error_text.contains("invalid") {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Invalid email or password".to_string(),
                    details: None,
                }),
            ));
        }

        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Authentication failed".to_string(),
                details: None,
            }),
        ));
    }

    let session_data: serde_json::Value = session_response.json().await.map_err(|e| {
        error!("Failed to parse session response: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Invalid response from authentication server".to_string(),
                details: None,
            }),
        )
    })?;

    let session_id = session_data
        .get("sessionId")
        .and_then(|s| s.as_str())
        .map(String::from);

    let session_token = session_data
        .get("sessionToken")
        .and_then(|s| s.as_str())
        .map(String::from);

    let access_token = session_id.clone().unwrap_or_else(|| user_id.clone());

    let user_uuid = Uuid::parse_str(&user_id).unwrap_or_else(|_| Uuid::new_v4());
    let token_uuid = Uuid::new_v4();
    let expires_at = Utc::now() + Duration::hours(24);

    let session_user = SessionUserData {
        user_id: user_id.clone(),
        email: req.email.clone(),
        username: req.email.split('@').next().unwrap_or("user").to_string(),
        first_name: None,
        last_name: None,
        display_name: Some(req.email.split('@').next().unwrap_or("User").to_string()),
        organization_id: None,
        roles: vec!["admin".to_string()],
        created_at: chrono::Utc::now().timestamp(),
    };

    {
        let mut cache = SESSION_CACHE.write().await;
        cache.insert(access_token.clone(), session_user.clone());
        info!("Session cached for user: {} with token: {}...", req.email, &access_token[..std::cmp::min(20, access_token.len())]);
    }

    let db_pool = state.db_pool.clone();
    if let Err(e) = tokio::task::spawn_blocking(move || {
        let mut conn = db_pool.get()?;
        let new_token = UserLoginToken {
            id: token_uuid,
            user_id: user_uuid,
            token_hash: access_token.clone(),
            expires_at,
            created_at: Utc::now(),
            last_used: Utc::now(),
            user_agent: None,
            ip_address: None,
            is_active: true,
        };
        diesel::insert_into(user_login_tokens)
            .values(&new_token)
            .execute(&mut conn)?;
        Ok::<_, diesel::result::Error>(())
    }).await {
        error!("Failed to save login token to database: {:?}", e);
    }

    info!("Login successful for: {} (user_id: {})", req.email, user_id);

    Ok(Json(LoginResponse {
        success: true,
        user_id: Some(user_id),
        session_id: session_id.clone(),
        access_token: Some(access_token),
        refresh_token: None,
        expires_in: Some(3600),
        requires_2fa: false,
        session_token,
        redirect: Some("/".to_string()),
        message: Some("Login successful".to_string()),
    }))
}

pub async fn logout(
    State(_state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<LogoutResponse>, (StatusCode, Json<ErrorResponse>)> {
    let token = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|auth| auth.strip_prefix("Bearer "))
        .map(String::from);

    if let Some(ref token_str) = token {
        let mut cache = SESSION_CACHE.write().await;
        if cache.remove(token_str).is_some() {
            info!("User logged out, session removed from cache");
        } else {
            info!("User logged out (session was not in cache)");
        }
    }

    Ok(Json(LogoutResponse {
        success: true,
        message: "Logged out successfully".to_string(),
    }))
}

pub async fn get_current_user(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<CurrentUserResponse>, (StatusCode, Json<ErrorResponse>)> {
    let session_token = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|auth| auth.strip_prefix("Bearer "))
        .ok_or_else(|| {
            warn!("get_current_user: Missing authorization header");
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Missing authorization token".to_string(),
                    details: None,
                }),
            )
        })?;

    if session_token.is_empty() {
        warn!("get_current_user: Empty authorization token");
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid authorization token".to_string(),
                details: None,
            }),
        ));
    }

    info!("get_current_user: looking up session token (len={}, prefix={}...)",
          session_token.len(),
          &session_token[..std::cmp::min(20, session_token.len())]);

    let cache = SESSION_CACHE.read().await;

    if let Some(user_data) = cache.get(session_token) {
        info!("get_current_user: found cached session for user: {}", user_data.email);
        
        tokio::spawn({
            let token = session_token.to_string();
            let db_pool = state.db_pool.clone();
            async move {
                if let Err(e) = tokio::task::spawn_blocking(move || {
                    let mut conn = db_pool.get()?;
                    diesel::update(user_login_tokens.filter(token_hash.eq(&token)))
                        .set(last_used.eq(Utc::now()))
                        .execute(&mut conn)?;
                    Ok::<_, diesel::result::Error>(())
                }).await {
                    error!("Failed to update last_used for token: {:?}", e);
                }
            }
        });
        
        return Ok(Json(CurrentUserResponse {
            id: user_data.user_id.clone(),
            username: user_data.username.clone(),
            email: Some(user_data.email.clone()),
            first_name: user_data.first_name.clone(),
            last_name: user_data.last_name.clone(),
            display_name: user_data.display_name.clone(),
            roles: user_data.roles.clone(),
            organization_id: user_data.organization_id.clone(),
            avatar_url: None,
        }));
    }

    drop(cache);

    info!("get_current_user: session not in cache, checking database");

    let db_pool = state.db_pool.clone();
    let token_str = session_token.to_string();
    
    let login_token = tokio::task::spawn_blocking(move || {
        let mut conn = db_pool.get()?;
        let result: Option<UserLoginToken> = user_login_tokens
            .filter(token_hash.eq(&token_str))
            .filter(is_active.eq(true))
            .filter(expires_at.gt(Utc::now()))
            .first(&mut conn)
            .optional()?;
        Ok::<_, diesel::result::Error>(result)
    }).await.map_err(|e| {
        error!("get_current_user: database query failed: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Database error".to_string(),
                details: Some(e.to_string()),
            }),
        )
    })?.map_err(|e| {
        error!("get_current_user: database query error: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Database error".to_string(),
                details: Some(e.to_string()),
            }),
        )
    })?;

    match login_token {
        Some(token) => {
            info!("get_current_user: found valid token in database for user_id: {}", token.user_id);
            
            let client = {
                let auth_service = state.auth_service.lock().await;
                auth_service.client().clone()
            };

            let user_data = client.get_user(&token.user_id.to_string()).await.map_err(|e| {
                error!("get_current_user: failed to get user from directory: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: "Failed to fetch user data".to_string(),
                        details: Some(e.to_string()),
                    }),
                )
            })?;

            let session_user = SessionUserData {
                user_id: token.user_id.to_string(),
                email: user_data.get("email")
                    .and_then(|e| e.as_str())
                    .unwrap_or("unknown@example.com")
                    .to_string(),
                username: user_data.get("username")
                    .and_then(|u| u.as_str())
                    .unwrap_or("user")
                    .to_string(),
                first_name: user_data.get("firstName")
                    .and_then(|f| f.as_str())
                    .map(String::from),
                last_name: user_data.get("lastName")
                    .and_then(|l| l.as_str())
                    .map(String::from),
                display_name: user_data.get("displayName")
                    .and_then(|d| d.as_str())
                    .map(String::from),
                organization_id: None,
                roles: vec!["admin".to_string()],
                created_at: chrono::Utc::now().timestamp(),
            };

            {
                let mut cache = SESSION_CACHE.write().await;
                cache.insert(token_str.clone(), session_user.clone());
            }

            tokio::spawn({
                let token = token_str.clone();
                let db_pool = state.db_pool.clone();
                async move {
                    if let Err(e) = tokio::task::spawn_blocking(move || {
                        let mut conn = db_pool.get()?;
                        diesel::update(user_login_tokens.filter(token_hash.eq(&token)))
                            .set(last_used.eq(Utc::now()))
                            .execute(&mut conn)?;
                        Ok::<_, diesel::result::Error>(())
                    }).await {
                        error!("Failed to update last_used for token: {:?}", e);
                    }
                }
            });

            Ok(Json(CurrentUserResponse {
                id: session_user.user_id.clone(),
                username: session_user.username.clone(),
                email: Some(session_user.email.clone()),
                first_name: session_user.first_name.clone(),
                last_name: session_user.last_name.clone(),
                display_name: session_user.display_name.clone(),
                roles: session_user.roles.clone(),
                organization_id: session_user.organization_id.clone(),
                avatar_url: None,
            }))
        }
        None => {
            warn!("get_current_user: token not found or expired in database");
            Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Session expired or invalid. Please log in again.".to_string(),
                    details: None,
                }),
            ))
        }
    }
}

pub async fn refresh_token(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RefreshTokenRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<ErrorResponse>)> {
    let client = {
        let auth_service = state.auth_service.lock().await;
        auth_service.client().clone()
    };

    let token_url = format!("{}/oauth/v2/token", client.api_url());

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| {
            error!("Failed to create HTTP client: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Internal server error".to_string(),
                    details: None,
                }),
            )
        })?;

    let params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", &req.refresh_token),
        ("scope", "openid profile email offline_access"),
    ];

    let response = http_client
        .post(&token_url)
        .form(&params)
        .send()
        .await
        .map_err(|e| {
            error!("Failed to refresh token: {}", e);
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Token refresh failed".to_string(),
                    details: None,
                }),
            )
        })?;

    if !response.status().is_success() {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid or expired refresh token".to_string(),
                details: None,
            }),
        ));
    }

    let token_data: serde_json::Value = response.json().await.map_err(|e| {
        error!("Failed to parse token response: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Invalid response from authentication server".to_string(),
                details: None,
            }),
        )
    })?;

    let access_token = token_data
        .get("access_token")
        .and_then(|t| t.as_str())
        .map(String::from);

    let refresh_token = token_data
        .get("refresh_token")
        .and_then(|t| t.as_str())
        .map(String::from);

    let expires_in = token_data.get("expires_in").and_then(|t| t.as_i64());

    Ok(Json(LoginResponse {
        success: true,
        user_id: None,
        session_id: None,
        access_token,
        refresh_token,
        expires_in,
        requires_2fa: false,
        session_token: None,
        redirect: None,
        message: Some("Token refreshed successfully".to_string()),
    }))
}

pub async fn verify_2fa(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<TwoFactorRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!(
        "2FA verification attempt for session: {}",
        req.session_token
    );

    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(ErrorResponse {
            error: "2FA verification not yet implemented".to_string(),
            details: Some("This feature will be available in a future update".to_string()),
        }),
    ))
}

pub async fn resend_2fa(
    State(_state): State<Arc<AppState>>,
    Json(_req): Json<serde_json::Value>,
) -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ErrorResponse {
            error: "2FA resend not yet implemented".to_string(),
            details: Some("This feature will be available in a future update".to_string()),
        }),
    )
}

pub async fn bootstrap_admin(
    State(state): State<Arc<AppState>>,
    Json(req): Json<BootstrapAdminRequest>,
) -> Result<Json<BootstrapResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Bootstrap admin request received");

    let expected_secret = std::env::var(BOOTSTRAP_SECRET_ENV).unwrap_or_default();

    if expected_secret.is_empty() {
        warn!("Bootstrap endpoint called but GB_BOOTSTRAP_SECRET not set");
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Bootstrap not enabled".to_string(),
                details: Some("Set GB_BOOTSTRAP_SECRET environment variable to enable bootstrap".to_string()),
            }),
        ));
    }

    if req.bootstrap_secret != expected_secret {
        warn!("Bootstrap attempt with invalid secret");
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid bootstrap secret".to_string(),
                details: None,
            }),
        ));
    }

    let client = {
        let auth_service = state.auth_service.lock().await;
        auth_service.client().clone()
    };

    let existing_users = client.list_users(1, 0).await.unwrap_or_default();
    if !existing_users.is_empty() {
        let has_admin = existing_users.iter().any(|u| {
            u.get("roles")
                .and_then(|r| r.as_array())
                .map(|roles| {
                    roles.iter().any(|r| {
                        r.as_str()
                            .map(|s| s.to_lowercase().contains("admin"))
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false)
        });

        if has_admin {
            return Err((
                StatusCode::CONFLICT,
                Json(ErrorResponse {
                    error: "Admin user already exists".to_string(),
                    details: Some("Bootstrap can only be used for initial setup".to_string()),
                }),
            ));
        }
    }

    let user_id = match client
        .create_user(&req.email, &req.first_name, &req.last_name, Some(&req.username))
        .await
    {
        Ok(id) => {
            info!("Bootstrap admin user created: {}", id);
            id
        }
        Err(e) => {
            error!("Failed to create bootstrap admin: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to create admin user".to_string(),
                    details: Some(e.to_string()),
                }),
            ));
        }
    };

    if let Err(e) = set_user_password(&client, &user_id, &req.password).await {
        error!("Failed to set admin password: {}", e);
    }

    let org_name = req.organization_name.unwrap_or_else(|| "Default Organization".to_string());
    let org_id = match create_organization(&client, &org_name).await {
        Ok(id) => {
            info!("Bootstrap organization created: {}", id);
            Some(id)
        }
        Err(e) => {
            warn!("Failed to create organization (may already exist): {}", e);
            None
        }
    };

    if let Some(ref oid) = org_id {
        let admin_roles = vec![
            "admin".to_string(),
            "org_owner".to_string(),
            "user_manager".to_string(),
        ];
        if let Err(e) = client.add_org_member(oid, &user_id, admin_roles).await {
            error!("Failed to add admin to organization: {}", e);
        } else {
            info!("Admin user added to organization with admin roles");
        }
    }

    info!(
        "Bootstrap complete: admin user {} created successfully",
        req.username
    );

    Ok(Json(BootstrapResponse {
        success: true,
        message: format!(
            "Admin user '{}' created successfully. You can now login with your credentials.",
            req.username
        ),
        user_id: Some(user_id),
        organization_id: org_id,
    }))
}

async fn set_user_password(
    client: &crate::directory::client::ZitadelClient,
    user_id: &str,
    password: &str,
) -> Result<(), String> {
    let url = format!("{}/v2/users/{}/password", client.api_url(), user_id);

    let body = serde_json::json!({
        "newPassword": {
            "password": password,
            "changeRequired": false
        }
    });

    let response = client
        .http_post(url)
        .await
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if response.status().is_success() {
        Ok(())
    } else {
        let error_text = response.text().await.unwrap_or_default();
        Err(format!("Failed to set password: {}", error_text))
    }
}

async fn create_organization(
    client: &crate::directory::client::ZitadelClient,
    name: &str,
) -> Result<String, String> {
    let url = format!("{}/v2/organizations", client.api_url());

    let body = serde_json::json!({
        "name": name
    });

    let response = client
        .http_post(url)
        .await
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if response.status().is_success() {
        let data: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
        let org_id = data
            .get("organizationId")
            .or_else(|| data.get("id"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| "No organization ID in response".to_string())?
            .to_string();
        Ok(org_id)
    } else {
        let error_text = response.text().await.unwrap_or_default();
        Err(format!("Failed to create organization: {}", error_text))
    }
}
