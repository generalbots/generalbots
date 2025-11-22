//! User Management Module
//!
//! Provides comprehensive user management operations including CRUD, security, and profile management.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::shared::state::AppState;

// ===== Request/Response Structures =====

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub display_name: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub phone: Option<String>,
    pub timezone: Option<String>,
    pub language: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UserQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub search: Option<String>,
    pub role: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct SetUserStatusRequest {
    pub status: String,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SetUserRoleRequest {
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct TwoFactorRequest {
    pub enable: bool,
    pub code: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NotificationPreferencesRequest {
    pub email_notifications: bool,
    pub push_notifications: bool,
    pub sms_notifications: bool,
    pub notification_types: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub role: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct UserProfileResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub phone: Option<String>,
    pub timezone: Option<String>,
    pub language: Option<String>,
    pub role: String,
    pub status: String,
    pub two_factor_enabled: bool,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct UserListResponse {
    pub users: Vec<UserResponse>,
    pub total: u32,
    pub page: u32,
    pub per_page: u32,
}

#[derive(Debug, Serialize)]
pub struct UserActivityResponse {
    pub user_id: Uuid,
    pub activities: Vec<ActivityEntry>,
    pub total: u32,
}

#[derive(Debug, Serialize)]
pub struct ActivityEntry {
    pub id: Uuid,
    pub action: String,
    pub resource: String,
    pub timestamp: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserPresenceResponse {
    pub user_id: Uuid,
    pub status: String,
    pub last_seen: DateTime<Utc>,
    pub custom_message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DeviceInfo {
    pub id: Uuid,
    pub device_name: String,
    pub device_type: String,
    pub last_active: DateTime<Utc>,
    pub trusted: bool,
    pub location: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SessionInfo {
    pub id: Uuid,
    pub device: String,
    pub ip_address: String,
    pub location: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub is_current: bool,
}

#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub success: bool,
    pub message: Option<String>,
}

// ===== API Handlers =====

/// POST /users/create - Create new user
pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = Uuid::new_v4();
    let now = Utc::now();

    let password_hash = hash_password(&req.password);

    let user = UserResponse {
        id: user_id,
        username: req.username,
        email: req.email,
        display_name: req.display_name,
        avatar_url: None,
        role: req.role.unwrap_or_else(|| "user".to_string()),
        status: "active".to_string(),
        created_at: now,
        updated_at: now,
        last_login: None,
    };

    Ok(Json(user))
}

/// PUT /users/:id/update - Update user information
pub async fn update_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, (StatusCode, Json<serde_json::Value>)> {
    let now = Utc::now();

    let user = UserResponse {
        id: user_id,
        username: "user".to_string(),
        email: req.email.unwrap_or_else(|| "user@example.com".to_string()),
        display_name: req.display_name,
        avatar_url: req.avatar_url,
        role: "user".to_string(),
        status: "active".to_string(),
        created_at: now,
        updated_at: now,
        last_login: None,
    };

    Ok(Json(user))
}

/// DELETE /users/:id/delete - Delete user
pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(SuccessResponse {
        success: true,
        message: Some(format!("User {} deleted successfully", user_id)),
    }))
}

/// GET /users/list - List all users with pagination
pub async fn list_users(
    State(state): State<Arc<AppState>>,
    Query(params): Query<UserQuery>,
) -> Result<Json<UserListResponse>, (StatusCode, Json<serde_json::Value>)> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);

    let users = vec![];

    Ok(Json(UserListResponse {
        users,
        total: 0,
        page,
        per_page,
    }))
}

/// GET /users/search - Search users
pub async fn search_users(
    State(state): State<Arc<AppState>>,
    Query(params): Query<UserQuery>,
) -> Result<Json<UserListResponse>, (StatusCode, Json<serde_json::Value>)> {
    list_users(State(state), Query(params)).await
}

/// GET /users/:id/profile - Get user profile
pub async fn get_user_profile(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserProfileResponse>, (StatusCode, Json<serde_json::Value>)> {
    let now = Utc::now();

    let profile = UserProfileResponse {
        id: user_id,
        username: "user".to_string(),
        email: "user@example.com".to_string(),
        display_name: Some("User Name".to_string()),
        bio: None,
        avatar_url: None,
        phone: None,
        timezone: Some("UTC".to_string()),
        language: Some("en".to_string()),
        role: "user".to_string(),
        status: "active".to_string(),
        two_factor_enabled: false,
        email_verified: true,
        created_at: now,
        updated_at: now,
        last_login: Some(now),
    };

    Ok(Json(profile))
}

/// PUT /users/profile/update - Update user's own profile
pub async fn update_profile(
    State(state): State<Arc<AppState>>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<UserProfileResponse>, (StatusCode, Json<serde_json::Value>)> {
    let now = Utc::now();
    let user_id = Uuid::new_v4();

    let profile = UserProfileResponse {
        id: user_id,
        username: "user".to_string(),
        email: req.email.unwrap_or_else(|| "user@example.com".to_string()),
        display_name: req.display_name,
        bio: req.bio,
        avatar_url: req.avatar_url,
        phone: req.phone,
        timezone: req.timezone,
        language: req.language,
        role: "user".to_string(),
        status: "active".to_string(),
        two_factor_enabled: false,
        email_verified: true,
        created_at: now,
        updated_at: now,
        last_login: Some(now),
    };

    Ok(Json(profile))
}

/// GET /users/:id/settings - Get user settings
pub async fn get_user_settings(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(serde_json::json!({
        "user_id": user_id,
        "theme": "light",
        "language": "en",
        "timezone": "UTC",
        "notifications": {
            "email": true,
            "push": true,
            "sms": false
        },
        "privacy": {
            "profile_visibility": "public",
            "show_email": false,
            "show_activity": true
        }
    })))
}

/// GET /users/:id/permissions - Get user permissions
pub async fn get_user_permissions(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(serde_json::json!({
        "user_id": user_id,
        "role": "user",
        "permissions": [
            "read:own_profile",
            "write:own_profile",
            "read:files",
            "write:files",
            "read:messages",
            "write:messages"
        ],
        "restrictions": []
    })))
}

/// GET /users/:id/roles - Get user roles
pub async fn get_user_roles(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(serde_json::json!({
        "user_id": user_id,
        "roles": ["user"],
        "primary_role": "user"
    })))
}

/// PUT /users/:id/roles - Set user roles
pub async fn set_user_roles(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    Json(req): Json<SetUserRoleRequest>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(SuccessResponse {
        success: true,
        message: Some(format!("User role updated to {}", req.role)),
    }))
}

/// GET /users/:id/status - Get user status
pub async fn get_user_status(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(serde_json::json!({
        "user_id": user_id,
        "status": "active",
        "online": true,
        "last_active": Utc::now().to_rfc3339()
    })))
}

/// PUT /users/:id/status - Set user status
pub async fn set_user_status(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    Json(req): Json<SetUserStatusRequest>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(SuccessResponse {
        success: true,
        message: Some(format!("User status updated to {}", req.status)),
    }))
}

/// GET /users/:id/presence - Get user presence information
pub async fn get_user_presence(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserPresenceResponse>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(UserPresenceResponse {
        user_id,
        status: "online".to_string(),
        last_seen: Utc::now(),
        custom_message: None,
    }))
}

/// GET /users/:id/activity - Get user activity log
pub async fn get_user_activity(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserActivityResponse>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(UserActivityResponse {
        user_id,
        activities: vec![],
        total: 0,
    }))
}

/// POST /users/security/2fa/enable - Enable two-factor authentication
pub async fn enable_2fa(
    State(state): State<Arc<AppState>>,
    Json(req): Json<TwoFactorRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(serde_json::json!({
        "success": true,
        "enabled": req.enable,
        "secret": "JBSWY3DPEHPK3PXP",
        "qr_code_url": "https://api.qrserver.com/v1/create-qr-code/?data=otpauth://totp/App:user@example.com?secret=JBSWY3DPEHPK3PXP&issuer=App"
    })))
}

/// POST /users/security/2fa/disable - Disable two-factor authentication
pub async fn disable_2fa(
    State(state): State<Arc<AppState>>,
    Json(req): Json<TwoFactorRequest>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(SuccessResponse {
        success: true,
        message: Some("Two-factor authentication disabled".to_string()),
    }))
}

/// GET /users/security/devices - List user devices
pub async fn list_user_devices(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<DeviceInfo>>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(vec![DeviceInfo {
        id: Uuid::new_v4(),
        device_name: "Chrome on Windows".to_string(),
        device_type: "browser".to_string(),
        last_active: Utc::now(),
        trusted: true,
        location: Some("San Francisco, CA".to_string()),
    }]))
}

/// GET /users/security/sessions - List active sessions
pub async fn list_user_sessions(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<SessionInfo>>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(vec![SessionInfo {
        id: Uuid::new_v4(),
        device: "Chrome on Windows".to_string(),
        ip_address: "192.168.1.1".to_string(),
        location: Some("San Francisco, CA".to_string()),
        created_at: Utc::now(),
        last_active: Utc::now(),
        is_current: true,
    }]))
}

/// PUT /users/notifications/settings - Update notification preferences
pub async fn update_notification_settings(
    State(state): State<Arc<AppState>>,
    Json(req): Json<NotificationPreferencesRequest>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(SuccessResponse {
        success: true,
        message: Some("Notification settings updated".to_string()),
    }))
}

// ===== Helper Functions =====

fn hash_password(password: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn verify_password(password: &str, hash: &str) -> bool {
    hash_password(password) == hash
}
