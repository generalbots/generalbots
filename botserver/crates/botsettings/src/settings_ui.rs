//! Settings page HTMX fragment handlers (accounts, storage, search, security).
//! Split out of `lib.rs` to keep that file within the size limit.

use axum::{extract::State, response::{Html, Json}};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use botcore::shared::state::AppState;

pub async fn get_accounts_social(State(_state): State<Arc<AppState>>) -> Html<String> {
Html(r##"<div class="accounts-list">
<div class="account-item"><span class="account-icon">📷</span><span class="account-name">Instagram</span><span class="account-status disconnected">Not connected</span></div>
<div class="account-item"><span class="account-icon">📘</span><span class="account-name">Facebook</span><span class="account-status disconnected">Not connected</span></div>
<div class="account-item"><span class="account-icon">🐦</span><span class="account-name">Twitter/X</span><span class="account-status disconnected">Not connected</span></div>
<div class="account-item"><span class="account-icon">💼</span><span class="account-name">LinkedIn</span><span class="account-status disconnected">Not connected</span></div>

</div>"##.to_string()) }

pub async fn get_accounts_messaging(State(_state): State<Arc<AppState>>) -> Html<String> {
Html(r##"<div class="accounts-list">
<div class="account-item"><span class="account-icon">💬</span><span class="account-name">Discord</span><span class="account-status disconnected">Not connected</span></div>
<div class="account-item"><span class="account-icon">📱</span><span class="account-name">WhatsApp</span><span class="account-status disconnected">Not connected</span></div>
<div class="account-item"><span class="account-icon">✈️</span><span class="account-name">Telegram</span><span class="account-status disconnected">Not connected</span></div>
<div class="account-item"><span class="account-icon">💼</span><span class="account-name">Teams</span><span class="account-status disconnected">Not connected</span></div>

</div>"##.to_string()) }

pub async fn get_accounts_email(State(state): State<Arc<AppState>>) -> Html<String> {
    let smtp_configured = {
        let Some(mut conn) = crate::settings_api::get_conn(&state) else {
            return Html(String::new());
        };
        let Some(user_id) = crate::settings_api::first_user(&mut conn) else {
            return Html(String::new());
        };
        !crate::settings_api::read_pref(&mut conn, user_id, "smtp_account").is_null()
    };

    let smtp_status = if smtp_configured {
        "<span class=\"account-status connected\">Configured</span>"
    } else {
        "<span class=\"account-status disconnected\">Not configured</span>"
    };

    Html(format!(
        r##"<div class="accounts-list">
<div class="account-item"><span class="account-icon">📧</span><span class="account-name">Gmail</span><span class="account-status disconnected">Not connected</span></div>
<div class="account-item"><span class="account-icon">📨</span><span class="account-name">Outlook</span><span class="account-status disconnected">Not connected</span></div>
<div class="account-item"><span class="account-icon">⚙️</span><span class="account-name">SMTP</span>{smtp_status}</div>

</div>"##
    ))
}

pub async fn save_smtp_account(
State(state): State<Arc<AppState>>,
Json(config): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let mut conn = match crate::settings_api::get_conn(&state) {
        Some(c) => c,
        None => {
            return Json(serde_json::json!({
                "success": false,
                "error": "Database unavailable"
            }));
        }
    };
    let Some(user_id) = crate::settings_api::first_user(&mut conn) else {
        return Json(serde_json::json!({
            "success": false,
            "error": "No user found"
        }));
    };

    crate::settings_api::write_pref(&mut conn, user_id, "smtp_account", &config);

    Json(serde_json::json!({
        "success": true,
        "message": "SMTP configuration saved",
        "config": config
    }))
}

pub async fn get_storage_info(State(state): State<Arc<AppState>>) -> Html<String> {
    let mut conn = match state.conn.get() {
        Ok(c) => c,
        Err(_) => return Html(String::new()),
    };

    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct SizeRow {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        size: i64,
    }
    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct CountRow {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }

    let db_size: i64 = diesel::sql_query(
        "SELECT pg_database_size(current_database())::bigint AS size",
    )
    .get_result::<SizeRow>(&mut conn)
    .map(|r| r.size)
    .unwrap_or(0);

    let bot_count: i64 = diesel::sql_query("SELECT COUNT(*)::bigint AS count FROM bots")
        .get_result::<CountRow>(&mut conn)
        .map(|r| r.count)
        .unwrap_or(0);

    let message_count: i64 = diesel::sql_query("SELECT COUNT(*)::bigint AS count FROM message_history")
        .get_result::<CountRow>(&mut conn)
        .map(|r| r.count)
        .unwrap_or(0);

    let source_count: i64 = diesel::sql_query("SELECT COUNT(*)::bigint AS count FROM knowledge_sources")
        .get_result::<CountRow>(&mut conn)
        .map(|r| r.count)
        .unwrap_or(0);

    let used_gb = db_size as f64 / (1024.0 * 1024.0 * 1024.0);

    Html(format!(
        r##"<div class="storage-info">
<div class="storage-details">
<span class="storage-used-text">{used_gb:.2} GB</span>
<span class="storage-total-text">database storage used</span>
</div>
<div class="storage-breakdown">
<div class="storage-item">
<span class="storage-icon">🤖</span>
<span class="storage-label">Bots</span>
<span class="storage-size">{bot_count}</span>
</div>
<div class="storage-item">
<span class="storage-icon">💬</span>
<span class="storage-label">Messages</span>
<span class="storage-size">{message_count}</span>
</div>
<div class="storage-item">
<span class="storage-icon">📄</span>
<span class="storage-label">KB Sources</span>
<span class="storage-size">{source_count}</span>
</div>
</div>
</div>"##
    ))
}

pub async fn get_storage_connections(State(_state): State<Arc<AppState>>) -> Html<String> {
Html(
r##"<div class="connections-empty">
<p class="text-muted">No external storage connections configured</p>
<button class="btn-secondary" onclick="showAddConnectionModal()">
+ Add Connection
</button>

</div>"## .to_string(), ) }

#[derive(Debug, Deserialize)]
pub struct SearchSettingsRequest {
enable_fuzzy_search: Option<bool>,
search_result_limit: Option<i32>,
enable_ai_suggestions: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct SearchSettingsResponse {
success: bool,
message: Option<String>,
error: Option<String>,
}

pub async fn save_search_settings(
State(state): State<Arc<AppState>>,
Json(settings): Json<SearchSettingsRequest>,
) -> Json<SearchSettingsResponse> {
    let mut conn = match crate::settings_api::get_conn(&state) {
        Some(c) => c,
        None => {
            return Json(SearchSettingsResponse {
                success: false,
                message: None,
                error: Some("Database unavailable".to_string()),
            });
        }
    };
    let Some(user_id) = crate::settings_api::first_user(&mut conn) else {
        return Json(SearchSettingsResponse {
            success: false,
            message: None,
            error: Some("No user found".to_string()),
        });
    };

    crate::settings_api::write_pref(
        &mut conn,
        user_id,
        "search_settings",
        &serde_json::json!({
            "enable_fuzzy_search": settings.enable_fuzzy_search,
            "search_result_limit": settings.search_result_limit,
            "enable_ai_suggestions": settings.enable_ai_suggestions,
        }),
    );

    Json(SearchSettingsResponse {
        success: true,
        message: Some("Search settings saved successfully".to_string()),
        error: None,
    })
}

fn read_mfa_enabled(state: &Arc<AppState>) -> bool {
    let mut conn = match crate::settings_api::get_conn(state) {
        Some(c) => c,
        None => return false,
    };
    let Some(user_id) = crate::settings_api::first_user(&mut conn) else {
        return false;
    };
    crate::settings_api::read_pref(&mut conn, user_id, "mfa_enabled")
        .as_bool()
        .unwrap_or(false)
}

fn write_mfa_enabled(state: &Arc<AppState>, enabled: bool) {
    let mut conn = match crate::settings_api::get_conn(state) {
        Some(c) => c,
        None => return,
    };
    let Some(user_id) = crate::settings_api::first_user(&mut conn) else {
        return;
    };
    crate::settings_api::write_pref(&mut conn, user_id, "mfa_enabled", &serde_json::json!(enabled));
}

pub async fn get_2fa_status(State(state): State<Arc<AppState>>) -> Html<String> {
    if read_mfa_enabled(&state) {
        Html(
            r##"<div class="status-indicator">
<span class="status-dot active"></span>
<span class="status-text">Two-factor authentication enabled</span>
</div>"##
            .to_string(),
        )
    } else {
        Html(
            r##"<div class="status-indicator">
<span class="status-dot inactive"></span>
<span class="status-text">Two-factor authentication is not enabled</span>
</div>"##
            .to_string(),
        )
    }
}

pub async fn enable_2fa(State(state): State<Arc<AppState>>) -> Html<String> {
    write_mfa_enabled(&state, true);
    Html(
        r##"<div class="status-indicator">
<span class="status-dot active"></span>
<span class="status-text">Two-factor authentication enabled</span>
</div>"##
        .to_string(),
    )
}

pub async fn disable_2fa(State(state): State<Arc<AppState>>) -> Html<String> {
    write_mfa_enabled(&state, false);
    Html(
        r##"<div class="status-indicator">
<span class="status-dot inactive"></span>
<span class="status-text">Two-factor authentication disabled</span>
</div>"##
        .to_string(),
    )
}

pub async fn get_active_sessions(State(state): State<Arc<AppState>>) -> Html<String> {
    let mut conn = match crate::settings_api::get_conn(&state) {
        Some(c) => c,
        None => return Html(String::new()),
    };
    let Some(user_id) = crate::settings_api::first_user(&mut conn) else {
        return Html(String::new());
    };

    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct SessionRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        user_data: String,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        created_at: chrono::DateTime<chrono::Utc>,
    }

    let rows: Vec<SessionRow> = diesel::sql_query(
        "SELECT user_data, created_at FROM login_sessions ORDER BY created_at DESC LIMIT 20",
    )
    .load::<SessionRow>(&mut conn)
    .unwrap_or_default();

    let now = chrono::Utc::now();
    let mut items = String::new();
    let mut count = 0;
    for r in rows {
        let parsed: serde_json::Value = serde_json::from_str(&r.user_data).unwrap_or_default();
        let uid = parsed
            .get("user_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let Ok(parsed_uid) = uuid::Uuid::parse_str(uid) else {
            continue;
        };
        if parsed_uid != user_id {
            continue;
        }
        count += 1;
        let age = now.signed_duration_since(r.created_at);
        let time_str = if age.num_minutes() < 1 {
            "just now".to_string()
        } else if age.num_hours() < 1 {
            format!("{} min ago", age.num_minutes())
        } else if age.num_days() < 1 {
            format!("{} hrs ago", age.num_hours())
        } else {
            format!("{} days ago", age.num_days())
        };
        items.push_str(&format!(
            r##"<div class="session-item">
<div class="session-info">
<div class="session-device">
<span class="device-icon">💻</span>
<span class="device-name">Active Session</span>
<span class="session-badge current">Session {count}</span>
</div>
<div class="session-details">
<span class="session-location">Browser session</span>
<span class="session-time">{time}</span>
</div>
</div>
</div>"##,
            time = time_str,
        ));
    }

    if items.is_empty() {
        items = r##"<div class="sessions-empty"><p class="text-muted">No active sessions</p></div>"##.to_string();
    }

    Html(items)
}

pub async fn revoke_all_sessions(State(state): State<Arc<AppState>>) -> Html<String> {
    let mut conn = match crate::settings_api::get_conn(&state) {
        Some(c) => c,
        None => return Html(String::new()),
    };
    let Some(user_id) = crate::settings_api::first_user(&mut conn) else {
        return Html(String::new());
    };

    // Delete persisted sessions belonging to this user from login_sessions so
    // the revocation is real — not just a preference flag.
    let rows: Vec<String> = {
        #[derive(diesel::QueryableByName)]
        #[diesel(check_for_backend(diesel::pg::Pg))]
        struct SessionRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            token: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            user_data: String,
        }
        diesel::sql_query("SELECT token, user_data FROM login_sessions")
            .load::<SessionRow>(&mut conn)
            .unwrap_or_default()
            .into_iter()
            .filter(|r| {
                serde_json::from_str::<serde_json::Value>(&r.user_data)
                    .ok()
                    .and_then(|v| v.get("user_id").and_then(|u| u.as_str()).map(String::from))
                    .and_then(|u| uuid::Uuid::parse_str(&u).ok())
                    == Some(user_id)
            })
            .map(|r| r.token)
            .collect()
    };

    let mut revoked = 0usize;
    for token in &rows {
        let result = diesel::sql_query("DELETE FROM login_sessions WHERE token = $1")
            .bind::<diesel::sql_types::Text, _>(token)
            .execute(&mut conn);
        if result.is_ok() {
            revoked += 1;
        }
        // Also evict from the in-memory suite session cache.
        if let Ok(mut cache) = botcoredirectory::auth_routes::SESSION_CACHE.try_write() {
            cache.remove(token);
        }
    }

    if revoked == 0 {
        return Html(
            r##"<div class="info-message"><span>No other sessions to revoke</span></div>"##
                .to_string(),
        );
    }

    Html(format!(
        r##"<div class="success-message">
<span class="success-icon">✓</span>
<span>{revoked} session(s) revoked</span>
</div>"##
    ))
}

pub async fn get_trusted_devices(State(_state): State<Arc<AppState>>) -> Html<String> {
    // Trusted-device tracking is not persisted; report the current device only
    // without claiming any device is trusted.
    Html(
        r##"<div class="device-item current">
<div class="device-info">
<span class="device-icon">💻</span>
<div class="device-details">
<span class="device-name">Current Device</span>
<span class="device-last-seen">Active now</span>
</div>
</div>
</div>
<div class="devices-empty"><p class="text-muted">Trusted-device tracking is not enabled</p></div>"##
            .to_string(),
    )
}
