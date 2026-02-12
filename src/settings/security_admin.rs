use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::shared::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityOverview {
    pub tls_enabled: bool,
    pub mtls_enabled: bool,
    pub rate_limiting_enabled: bool,
    pub cors_configured: bool,
    pub api_keys_count: u32,
    pub active_sessions_count: u32,
    pub audit_log_enabled: bool,
    pub mfa_enabled_users: u32,
    pub total_users: u32,
    pub last_security_scan: Option<DateTime<Utc>>,
    pub security_score: u8,
    pub vulnerabilities: SecurityVulnerabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityVulnerabilities {
    pub critical: u32,
    pub high: u32,
    pub medium: u32,
    pub low: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsSettings {
    pub enabled: bool,
    pub cert_expiry: Option<DateTime<Utc>>,
    pub auto_renew: bool,
    pub min_version: String,
    pub cipher_suites: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitSettings {
    pub enabled: bool,
    pub requests_per_minute: u32,
    pub burst_size: u32,
    pub whitelist: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsSettings {
    pub enabled: bool,
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub max_age_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub resource: String,
    pub resource_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub success: bool,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyInfo {
    pub id: Uuid,
    pub name: String,
    pub prefix: String,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub scopes: Vec<String>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub scopes: Vec<String>,
    pub expires_in_days: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiKeyResponse {
    pub id: Uuid,
    pub name: String,
    pub key: String,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaSettings {
    pub require_mfa: bool,
    pub allowed_methods: Vec<String>,
    pub grace_period_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_email: String,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub is_current: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicy {
    pub min_length: u8,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_numbers: bool,
    pub require_special_chars: bool,
    pub max_age_days: Option<u32>,
    pub prevent_reuse_count: u8,
}

#[derive(Debug, Serialize)]
pub struct SecurityError {
    pub error: String,
    pub code: String,
}

impl IntoResponse for SecurityError {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": self.error, "code": self.code})),
        )
            .into_response()
    }
}

async fn get_security_overview(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<SecurityOverview>, SecurityError> {
    let overview = SecurityOverview {
        tls_enabled: true,
        mtls_enabled: false,
        rate_limiting_enabled: true,
        cors_configured: true,
        api_keys_count: 5,
        active_sessions_count: 12,
        audit_log_enabled: true,
        mfa_enabled_users: 8,
        total_users: 25,
        last_security_scan: Some(Utc::now()),
        security_score: 85,
        vulnerabilities: SecurityVulnerabilities {
            critical: 0,
            high: 1,
            medium: 3,
            low: 7,
        },
    };
    Ok(Json(overview))
}

async fn get_tls_settings(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<TlsSettings>, SecurityError> {
    let settings = TlsSettings {
        enabled: true,
        cert_expiry: Some(Utc::now() + chrono::Duration::days(90)),
        auto_renew: true,
        min_version: "TLS 1.2".to_string(),
        cipher_suites: vec![
            "TLS_AES_256_GCM_SHA384".to_string(),
            "TLS_CHACHA20_POLY1305_SHA256".to_string(),
            "TLS_AES_128_GCM_SHA256".to_string(),
        ],
    };
    Ok(Json(settings))
}

async fn update_tls_settings(
    State(_state): State<Arc<AppState>>,
    Json(_settings): Json<TlsSettings>,
) -> Result<Json<TlsSettings>, SecurityError> {
    let settings = TlsSettings {
        enabled: true,
        cert_expiry: Some(Utc::now() + chrono::Duration::days(90)),
        auto_renew: true,
        min_version: "TLS 1.2".to_string(),
        cipher_suites: vec![
            "TLS_AES_256_GCM_SHA384".to_string(),
            "TLS_CHACHA20_POLY1305_SHA256".to_string(),
        ],
    };
    Ok(Json(settings))
}

async fn get_rate_limit_settings(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<RateLimitSettings>, SecurityError> {
    let settings = RateLimitSettings {
        enabled: true,
        requests_per_minute: 60,
        burst_size: 100,
        whitelist: vec![],
    };
    Ok(Json(settings))
}

async fn update_rate_limit_settings(
    State(_state): State<Arc<AppState>>,
    Json(settings): Json<RateLimitSettings>,
) -> Result<Json<RateLimitSettings>, SecurityError> {
    Ok(Json(settings))
}

async fn get_cors_settings(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<CorsSettings>, SecurityError> {
    let settings = CorsSettings {
        enabled: true,
        allowed_origins: vec!["*".to_string()],
        allowed_methods: vec![
            "GET".to_string(),
            "POST".to_string(),
            "PUT".to_string(),
            "DELETE".to_string(),
        ],
        allowed_headers: vec!["Content-Type".to_string(), "Authorization".to_string()],
        max_age_seconds: 3600,
    };
    Ok(Json(settings))
}

async fn update_cors_settings(
    State(_state): State<Arc<AppState>>,
    Json(settings): Json<CorsSettings>,
) -> Result<Json<CorsSettings>, SecurityError> {
    Ok(Json(settings))
}

async fn list_audit_logs(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<Vec<AuditLogEntry>>, SecurityError> {
    let logs = vec![
        AuditLogEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            user_id: Some(Uuid::new_v4()),
            action: "login".to_string(),
            resource: "session".to_string(),
            resource_id: None,
            ip_address: Some("192.168.1.100".to_string()),
            user_agent: Some("Mozilla/5.0".to_string()),
            success: true,
            details: None,
        },
    ];
    Ok(Json(logs))
}

async fn list_api_keys(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<Vec<ApiKeyInfo>>, SecurityError> {
    let keys = vec![];
    Ok(Json(keys))
}

async fn create_api_key(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<CreateApiKeyRequest>,
) -> Result<Json<CreateApiKeyResponse>, SecurityError> {
    let response = CreateApiKeyResponse {
        id: Uuid::new_v4(),
        name: req.name,
        key: format!("gb_{}", Uuid::new_v4().to_string().replace('-', "")),
        expires_at: req.expires_in_days.map(|days| Utc::now() + chrono::Duration::days(i64::from(days))),
    };
    Ok(Json(response))
}

async fn revoke_api_key(
    State(_state): State<Arc<AppState>>,
    Path(_key_id): Path<Uuid>,
) -> Result<StatusCode, SecurityError> {
    Ok(StatusCode::NO_CONTENT)
}

async fn get_mfa_settings(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<MfaSettings>, SecurityError> {
    let settings = MfaSettings {
        require_mfa: false,
        allowed_methods: vec!["totp".to_string(), "webauthn".to_string()],
        grace_period_days: 7,
    };
    Ok(Json(settings))
}

async fn update_mfa_settings(
    State(_state): State<Arc<AppState>>,
    Json(settings): Json<MfaSettings>,
) -> Result<Json<MfaSettings>, SecurityError> {
    Ok(Json(settings))
}

async fn list_active_sessions(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<Vec<SessionInfo>>, SecurityError> {
    let sessions = vec![];
    Ok(Json(sessions))
}

async fn revoke_session(
    State(_state): State<Arc<AppState>>,
    Path(_session_id): Path<Uuid>,
) -> Result<StatusCode, SecurityError> {
    Ok(StatusCode::NO_CONTENT)
}

async fn revoke_all_user_sessions(
    State(_state): State<Arc<AppState>>,
    Path(_user_id): Path<Uuid>,
) -> Result<StatusCode, SecurityError> {
    Ok(StatusCode::NO_CONTENT)
}

async fn get_password_policy(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<PasswordPolicy>, SecurityError> {
    let policy = PasswordPolicy {
        min_length: 12,
        require_uppercase: true,
        require_lowercase: true,
        require_numbers: true,
        require_special_chars: true,
        max_age_days: Some(90),
        prevent_reuse_count: 5,
    };
    Ok(Json(policy))
}

async fn update_password_policy(
    State(_state): State<Arc<AppState>>,
    Json(policy): Json<PasswordPolicy>,
) -> Result<Json<PasswordPolicy>, SecurityError> {
    Ok(Json(policy))
}

async fn run_security_scan(
    State(state): State<Arc<AppState>>,
) -> Result<Json<SecurityOverview>, SecurityError> {
    get_security_overview(State(state)).await
}

pub fn configure_security_admin_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/settings/security/overview", get(get_security_overview))
        .route("/api/settings/security/scan", post(run_security_scan))
        .route(
            "/api/settings/security/tls",
            get(get_tls_settings).put(update_tls_settings),
        )
        .route(
            "/api/settings/security/rate-limit",
            get(get_rate_limit_settings).put(update_rate_limit_settings),
        )
        .route(
            "/api/settings/security/cors",
            get(get_cors_settings).put(update_cors_settings),
        )
        .route("/api/settings/security/audit", get(list_audit_logs))
        .route(
            "/api/settings/security/api-keys",
            get(list_api_keys).post(create_api_key),
        )
        .route("/api/settings/security/api-keys/:key_id", delete(revoke_api_key))
        .route(
            "/api/settings/security/mfa",
            get(get_mfa_settings).put(update_mfa_settings),
        )
        .route("/api/settings/security/sessions", get(list_active_sessions))
        .route("/api/settings/security/sessions/:session_id", delete(revoke_session))
        .route(
            "/api/settings/security/users/:user_id/sessions",
            delete(revoke_all_user_sessions),
        )
        .route(
            "/api/settings/security/password-policy",
            get(get_password_policy).put(update_password_policy),
        )
}
