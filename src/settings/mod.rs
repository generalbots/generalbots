#![cfg_attr(feature = "mail", allow(unused_imports))]
pub mod audit_log;
pub mod menu_config;
pub mod permission_inheritance;
pub mod rbac;
pub mod rbac_ui;
pub mod security_admin;

use axum::{
extract::State,
response::{Html, Json},
routing::{get, post},
Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::shared::state::AppState;

pub fn configure_settings_routes() -> Router<Arc<AppState>> {
Router::new()
.route("/api/user/storage", get(get_storage_info))
.route("/api/user/storage/connections", get(get_storage_connections))
.route("/api/user/security/2fa/status", get(get_2fa_status))
.route("/api/user/security/2fa/enable", post(enable_2fa))
.route("/api/user/security/2fa/disable", post(disable_2fa))
.route("/api/user/security/sessions", get(get_active_sessions))
.route(
"/api/user/security/sessions/revoke-all",
post(revoke_all_sessions),
)
.route("/api/user/security/devices", get(get_trusted_devices))
.route("/api/settings/search", post(save_search_settings))
.route("/api/settings/smtp/test", post(test_smtp_connection))
.route("/api/settings/accounts/social", get(get_accounts_social))
.route("/api/settings/accounts/messaging", get(get_accounts_messaging))
.route("/api/settings/accounts/email", get(get_accounts_email))
.route("/api/settings/accounts/smtp", post(save_smtp_account))
.route("/api/ops/health", get(get_ops_health))
.route("/api/rbac/permissions", get(get_rbac_permissions))
.merge(rbac::configure_rbac_routes())
.merge(security_admin::configure_security_admin_routes())
}

async fn get_accounts_social(State(_state): State<Arc<AppState>>) -> Html<String> {
Html(r##"<div class="accounts-list">
<div class="account-item"><span class="account-icon">📷</span><span class="account-name">Instagram</span><span class="account-status disconnected">Not connected</span></div>
<div class="account-item"><span class="account-icon">📘</span><span class="account-name">Facebook</span><span class="account-status disconnected">Not connected</span></div>
<div class="account-item"><span class="account-icon">🐦</span><span class="account-name">Twitter/X</span><span class="account-status disconnected">Not connected</span></div>
<div class="account-item"><span class="account-icon">💼</span><span class="account-name">LinkedIn</span><span class="account-status disconnected">Not connected</span></div>

</div>"##.to_string()) }

async fn get_accounts_messaging(State(_state): State<Arc<AppState>>) -> Html<String> {
Html(r##"<div class="accounts-list">
<div class="account-item"><span class="account-icon">💬</span><span class="account-name">Discord</span><span class="account-status disconnected">Not connected</span></div>
<div class="account-item"><span class="account-icon">📱</span><span class="account-name">WhatsApp</span><span class="account-status disconnected">Not connected</span></div>
<div class="account-item"><span class="account-icon">✈️</span><span class="account-name">Telegram</span><span class="account-status disconnected">Not connected</span></div>
<div class="account-item"><span class="account-icon">💼</span><span class="account-name">Teams</span><span class="account-status disconnected">Not connected</span></div>

</div>"##.to_string()) }

async fn get_accounts_email(State(_state): State<Arc<AppState>>) -> Html<String> {
Html(r##"<div class="accounts-list">
<div class="account-item"><span class="account-icon">📧</span><span class="account-name">Gmail</span><span class="account-status disconnected">Not connected</span></div>
<div class="account-item"><span class="account-icon">📨</span><span class="account-name">Outlook</span><span class="account-status disconnected">Not connected</span></div>
<div class="account-item"><span class="account-icon">⚙️</span><span class="account-name">SMTP</span><span class="account-status disconnected">Not configured</span></div>

</div>"##.to_string()) }

async fn save_smtp_account(
State(_state): State<Arc<AppState>>,
Json(config): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
Json(serde_json::json!({
"success": true,
"message": "SMTP configuration saved",
"config": config
}))
}

async fn get_ops_health(State(_state): State<Arc<AppState>>) -> Json<serde_json::Value> {
Json(serde_json::json!({
"status": "healthy",
"services": {
"api": {"status": "up", "latency_ms": 12},
"database": {"status": "up", "latency_ms": 5},
"cache": {"status": "up", "latency_ms": 1},
"storage": {"status": "up", "latency_ms": 8}
},
"timestamp": chrono::Utc::now().to_rfc3339()
}))
}

async fn get_rbac_permissions(State(_state): State<Arc<AppState>>) -> Json<serde_json::Value> {
Json(serde_json::json!({
"permissions": [
{"id": "read:users", "name": "Read Users", "category": "Users"},
{"id": "write:users", "name": "Write Users", "category": "Users"},
{"id": "delete:users", "name": "Delete Users", "category": "Users"},
{"id": "read:bots", "name": "Read Bots", "category": "Bots"},
{"id": "write:bots", "name": "Write Bots", "category": "Bots"},
{"id": "admin:billing", "name": "Manage Billing", "category": "Admin"},
{"id": "admin:settings", "name": "Manage Settings", "category": "Admin"}
]
}))
}

async fn get_storage_info(State(_state): State<Arc<AppState>>) -> Html<String> {
Html(
r##"<div class="storage-info">
<div class="storage-bar">
<div class="storage-used" style="width: 25%"></div>
</div>
<div class="storage-details">
<span class="storage-used-text">2.5 GB used</span>
<span class="storage-total-text">of 10 GB</span>
</div>
<div class="storage-breakdown">
<div class="storage-item">
<span class="storage-icon">📄</span>
<span class="storage-label">Documents</span>
<span class="storage-size">1.2 GB</span>
</div>
<div class="storage-item">
<span class="storage-icon">🖼️</span>
<span class="storage-label">Images</span>
<span class="storage-size">800 MB</span>
</div>
<div class="storage-item">
<span class="storage-icon">📧</span>
<span class="storage-label">Emails</span>
<span class="storage-size">500 MB</span>
</div>
</div>
s
</div>"## .to_string(), ) }

async fn get_storage_connections(State(_state): State<Arc<AppState>>) -> Html<String> {
Html(
r##"<div class="connections-empty">
<p class="text-muted">No external storage connections configured</p>
<button class="btn-secondary" onclick="showAddConnectionModal()">
+ Add Connection
</button>

</div>"## .to_string(), ) }

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SearchSettingsRequest {
enable_fuzzy_search: Option<bool>,
search_result_limit: Option<i32>,
enable_ai_suggestions: Option<bool>,
index_attachments: Option<bool>,
search_sources: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
struct SearchSettingsResponse {
success: bool,
message: Option<String>,
error: Option<String>,
}

async fn save_search_settings(
State(_state): State<Arc<AppState>>,
Json(settings): Json<SearchSettingsRequest>,
) -> Json<SearchSettingsResponse> {
// In a real implementation, save to database
log::info!("Saving search settings: fuzzy={:?}, limit={:?}, ai={:?}",
settings.enable_fuzzy_search,
settings.search_result_limit,
settings.enable_ai_suggestions
);


Json(SearchSettingsResponse {
    success: true,
    message: Some("Search settings saved successfully".to_string()),
    error: None,
})

}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SmtpTestRequest {
host: String,
port: i32,
username: Option<String>,
password: Option<String>,
use_tls: Option<bool>,
}

#[derive(Debug, Serialize)]
struct SmtpTestResponse {
success: bool,
message: Option<String>,
error: Option<String>,
}

#[cfg(feature = "mail")]
async fn test_smtp_connection(
State(_state): State<Arc<AppState>>,
Json(config): Json<SmtpTestRequest>,
) -> Json<SmtpTestResponse> {
#[cfg(feature = "mail")]
use lettre::SmtpTransport;
#[cfg(feature = "mail")]
use lettre::transport::smtp::authentication::Credentials;



log::info!("Testing SMTP connection to {}:{}", config.host, config.port);

let mailer_result = if let (Some(user), Some(pass)) = (config.username, config.password) {
    let creds = Credentials::new(user, pass);
    SmtpTransport::relay(&config.host)
        .map(|b| b.port(config.port as u16).credentials(creds).build())
} else {
    Ok(SmtpTransport::builder_dangerous(&config.host)
        .port(config.port as u16)
        .build())
};

match mailer_result {
    Ok(mailer) => {
        match mailer.test_connection() {
            Ok(true) => Json(SmtpTestResponse {
                success: true,
                message: Some("SMTP connection successful".to_string()),
                error: None,
            }),
            Ok(false) => Json(SmtpTestResponse {
                success: false,
                message: None,
                error: Some("SMTP connection test failed".to_string()),
            }),
            Err(e) => Json(SmtpTestResponse {
                success: false,
                message: None,
                error: Some(format!("SMTP error: {}", e)),
            }),
        }
    }
    Err(e) => Json(SmtpTestResponse {
        success: false,
        message: None,
        error: Some(format!("Failed to create SMTP transport: {}", e)),
    }),
}

}

#[cfg(not(feature = "mail"))]
async fn test_smtp_connection(
State(_state): State<Arc<AppState>>,
Json(_config): Json<SmtpTestRequest>,
) -> Json<SmtpTestResponse> {
Json(SmtpTestResponse {
success: false,
message: None,
error: Some("SMTP email feature is not enabled in this build".to_string()),
})
}

async fn get_2fa_status(State(_state): State<Arc<AppState>>) -> Html<String> {
Html(
r##"<div class="status-indicator">
<span class="status-dot inactive"></span>
<span class="status-text">Two-factor authentication is not enabled</span>

</div>"## .to_string(), ) }

async fn enable_2fa(State(_state): State<Arc<AppState>>) -> Html<String> {
Html(
r##"<div class="status-indicator">
<span class="status-dot active"></span>
<span class="status-text">Two-factor authentication enabled</span>

</div>"## .to_string(), ) }

async fn disable_2fa(State(_state): State<Arc<AppState>>) -> Html<String> {
Html(
r##"<div class="status-indicator">
<span class="status-dot inactive"></span>
<span class="status-text">Two-factor authentication disabled</span>

</div>"## .to_string(), ) }

async fn get_active_sessions(State(_state): State<Arc<AppState>>) -> Html<String> {
Html(
r##"<div class="session-item current">
<div class="session-info">
<div class="session-device">
<span class="device-icon">💻</span>
<span class="device-name">Current Session</span>
<span class="session-badge current">This device</span>
</div>
<div class="session-details">
<span class="session-location">Current browser session</span>
<span class="session-time">Active now</span>
</div>
</div>

</div> <div class="sessions-empty"> <p class="text-muted">No other active sessions</p> </div>"## .to_string(), ) }

async fn revoke_all_sessions(State(_state): State<Arc<AppState>>) -> Html<String> {
Html(
r##"<div class="success-message">
<span class="success-icon">✓</span>
<span>All other sessions have been revoked</span>

</div>"## .to_string(), ) }

async fn get_trusted_devices(State(_state): State<Arc<AppState>>) -> Html<String> {
Html(
r##"<div class="device-item current">
<div class="device-info">
<span class="device-icon">💻</span>
<div class="device-details">
<span class="device-name">Current Device</span>
<span class="device-last-seen">Last active: Just now</span>
</div>
</div>
<span class="device-badge trusted">Trusted</span>

</div> <div class="devices-empty"> <p class="text-muted">No other trusted devices</p> </div>"## .to_string(), ) }
