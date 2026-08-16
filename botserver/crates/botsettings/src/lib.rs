pub mod admin_dashboard;
pub mod audit_log;
pub mod menu_config;
pub mod settings_billing;
pub mod settings_credentials;
pub mod settings_oauth;
pub mod settings_webhooks;
pub mod webhook_delivery;
pub mod settings_profile;
pub mod ops;
pub mod permission_inheritance;
pub mod rbac;
#[cfg(feature = "rbac")]
pub mod rbac_kb;
pub mod rbac_ui;
pub mod security_admin;
pub mod settings_api;
pub mod settings_ui;

use axum::{
extract::State,
response::Json,
routing::{get, post},
Router,
};
use diesel::RunQueryDsl;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use botcore::shared::state::AppState;

pub fn configure_settings_routes() -> Router<Arc<AppState>> {
Router::new()
.route("/api/ui/user/storage", get(settings_ui::get_storage_info))
.route("/api/ui/user/storage/connections", get(settings_ui::get_storage_connections))
.route("/api/ui/user/security/2fa/status", get(settings_ui::get_2fa_status))
.route("/api/ui/user/security/2fa/enable", post(settings_ui::enable_2fa))
.route("/api/ui/user/security/2fa/disable", post(settings_ui::disable_2fa))
.route("/api/ui/user/security/sessions", get(settings_ui::get_active_sessions))
.route(
"/api/ui/user/security/sessions/revoke-all",
post(settings_ui::revoke_all_sessions),
)
.route("/api/ui/user/security/devices", get(settings_ui::get_trusted_devices))
.route("/api/settings/search", post(settings_ui::save_search_settings))
.route("/api/settings/smtp/test", post(test_smtp_connection))
.route("/api/oauth/accounts", get(settings_oauth::oauth_accounts_list))
.route("/api/oauth/:provider/unlink", post(settings_oauth::oauth_unlink))
.route("/api/oauth/:provider/callback", get(settings_oauth::oauth_callback))
.route("/api/ui/settings/accounts/social", get(settings_ui::get_accounts_social))
.route("/api/ui/settings/accounts/messaging", get(settings_ui::get_accounts_messaging))
.route("/api/ui/settings/accounts/email", get(settings_ui::get_accounts_email))
.route("/api/settings/accounts/smtp", post(settings_ui::save_smtp_account))
.route("/api/ops/health", get(get_ops_health))
.merge(rbac::configure_rbac_routes())
.merge(security_admin::configure_security_admin_routes())
.merge(admin_dashboard::configure_admin_dashboard_routes())
.merge(settings_api::configure_settings_api_routes())
.merge(ops::configure_ops_routes())
}

async fn get_ops_health(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    use std::time::Instant;

    let db_ok = state
        .conn
        .get()
        .ok()
        .and_then(|mut c| {
            #[derive(diesel::QueryableByName)]
            #[diesel(check_for_backend(diesel::pg::Pg))]
            struct DbCheck {
                #[diesel(sql_type = diesel::sql_types::Int4)]
                result: i32,
            }
            diesel::sql_query("SELECT 1 as result")
                .get_result::<DbCheck>(&mut c)
                .ok()
                .map(|r| r.result == 1)
        })
        .unwrap_or(false);

    let cache_ok = state
        .cache
        .as_ref()
        .map(|c| {
            c.get_connection_with_timeout(std::time::Duration::from_millis(750))
                .and_then(|mut conn| redis::cmd("PING").query::<String>(&mut conn))
                .map(|reply| reply.to_uppercase() == "PONG")
                .unwrap_or(false)
        })
        .unwrap_or(false);

    let drive_ok = state.drive.is_some();

    let db_latency = {
        let start = Instant::now();
        let _ = state.conn.get();
        start.elapsed().as_millis()
    };

    Json(serde_json::json!({
        "status": if db_ok && cache_ok { "healthy" } else { "degraded" },
        "services": {
            "api": {"status": "up", "latency_ms": db_latency},
            "database": {"status": if db_ok { "up" } else { "down" }, "latency_ms": db_latency},
            "cache": {"status": if cache_ok { "up" } else { "down" }, "latency_ms": 0},
            "storage": {"status": if drive_ok { "up" } else { "down" }, "latency_ms": 0}
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}



#[derive(Debug, Serialize)]
struct SmtpTestResponse {
success: bool,
message: Option<String>,
error: Option<String>,
}

#[cfg(feature = "mail")]
#[derive(Debug, Deserialize)]
struct SmtpTestRequest {
    host: String,
    port: i32,
    username: Option<String>,
    password: Option<String>,
    _use_tls: Option<bool>,
}

#[cfg(not(feature = "mail"))]
#[derive(Debug, Deserialize)]
struct SmtpTestRequest {
_host: String,
_port: i32,
_username: Option<String>,
_password: Option<String>,
_use_tls: Option<bool>,
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
    if config.port == 465 {
        SmtpTransport::relay(&config.host)
            .map(|b| b.port(config.port as u16).credentials(creds).build())
    } else {
        SmtpTransport::starttls_relay(&config.host)
            .map(|b| b.port(config.port as u16).credentials(creds).build())
    }
} else {
    SmtpTransport::builder_dangerous(&config.host)
        .port(config.port as u16)
        .build()
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


