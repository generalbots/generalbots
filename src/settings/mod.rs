pub mod audit_log;
pub mod menu_config;
pub mod permission_inheritance;
pub mod rbac;
pub mod rbac_ui;

use axum::{
    extract::State,
    response::Html,
    routing::{get, post},
    Router,
};
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
        .merge(rbac::configure_rbac_routes())
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
</div>"##
            .to_string(),
    )
}

async fn get_storage_connections(State(_state): State<Arc<AppState>>) -> Html<String> {
    Html(
        r##"<div class="connections-empty">
    <p class="text-muted">No external storage connections configured</p>
    <button class="btn-secondary" onclick="showAddConnectionModal()">
        + Add Connection
    </button>
</div>"##
            .to_string(),
    )
}

async fn get_2fa_status(State(_state): State<Arc<AppState>>) -> Html<String> {
    Html(
        r##"<div class="status-indicator">
    <span class="status-dot inactive"></span>
    <span class="status-text">Two-factor authentication is not enabled</span>
</div>"##
            .to_string(),
    )
}

async fn enable_2fa(State(_state): State<Arc<AppState>>) -> Html<String> {
    Html(
        r##"<div class="status-indicator">
    <span class="status-dot active"></span>
    <span class="status-text">Two-factor authentication enabled</span>
</div>"##
            .to_string(),
    )
}

async fn disable_2fa(State(_state): State<Arc<AppState>>) -> Html<String> {
    Html(
        r##"<div class="status-indicator">
    <span class="status-dot inactive"></span>
    <span class="status-text">Two-factor authentication disabled</span>
</div>"##
            .to_string(),
    )
}

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
</div>
<div class="sessions-empty">
    <p class="text-muted">No other active sessions</p>
</div>"##
            .to_string(),
    )
}

async fn revoke_all_sessions(State(_state): State<Arc<AppState>>) -> Html<String> {
    Html(
        r##"<div class="success-message">
    <span class="success-icon">✓</span>
    <span>All other sessions have been revoked</span>
</div>"##
            .to_string(),
    )
}

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
</div>
<div class="devices-empty">
    <p class="text-muted">No other trusted devices</p>
</div>"##
            .to_string(),
    )
}
