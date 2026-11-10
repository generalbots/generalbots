//! Shell-mode Operating System API (#1852).
//!
//! Settings exposes an "Operating System" panel only when the deployment runs
//! as a shell (Tauri desktop app, desktop-local build or the kiosk image).
//! These routes are the server-side half of that gate:
//!
//! * the product file must set `shell_mode=true`;
//! * the caller must be an administrator (the single-user desktop identity
//!   counts as the local owner);
//! * the request must arrive through a loopback host header.
//!
//! The panel is read-only except for an allow-listed `systemctl restart` tool,
//! which requires a typed confirmation and is written to the audit log.

use axum::{
    extract::{Extension, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use botcore::shared::state::AppState;
use botsecurity::auth::AuthenticatedUser;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::os_data::{
    expected_log_path, journal_tail, log_file_for, log_file_infos, read_disks, read_hostname,
    read_kernel, read_os, read_services, read_timezone, read_uptime, tail, DiskInfo, LogFileInfo,
    ServiceInfo, SERVICES,
};

const DEFAULT_LOG_LINES: usize = 200;
const MAX_LOG_LINES: usize = 1000;

/// #2102 — shown instead of an empty pane when neither a file nor the journal
/// has anything for the service. Only the log location is exposed, never the
/// rest of the host layout.
fn missing_log_message(service: &str) -> String {
    let path = expected_log_path(service);
    format!(
        "No log content for {service} yet.\n\nExpected file: {}\n\nA desktop install creates it when the sidecar starts; a systemd install writes to the journal instead (journalctl -u {service}).",
        path.display()
    )
}

pub fn configure_os_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/system/os", get(os_status))
        .route("/api/system/os/services/restart", post(restart_service))
        .route("/api/system/os/logs", get(os_logs))
        .route("/api/system/os/logs/download", get(os_logs_download))
}

#[derive(Serialize)]
struct OsStatus {
    shell_mode: bool,
    hostname: String,
    os: String,
    kernel: String,
    arch: String,
    uptime_secs: u64,
    timezone: String,
    disks: Vec<DiskInfo>,
    services: Vec<ServiceInfo>,
    log_files: Vec<LogFileInfo>,
}

#[derive(Deserialize)]
struct LogQuery {
    service: Option<String>,
    lines: Option<usize>,
}

#[derive(Deserialize)]
struct RestartRequest {
    service: String,
    confirmation: String,
}

fn deny(message: &str) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::FORBIDDEN,
        Json(serde_json::json!({ "error": message })),
    )
}

/// Whether the request was addressed to the local host.
///
/// Only the real `Host` header is consulted: `X-Forwarded-Host` is
/// caller-supplied, so `X-Forwarded-Host: localhost` used to satisfy the gate
/// from any peer (#2122). A reverse proxy forwards the external host in `Host`,
/// which correctly keeps the panel unreachable through it.
fn request_is_loopback(headers: &HeaderMap) -> bool {
    let raw = headers
        .get("host")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .trim();
    let host = if let Some(rest) = raw.strip_prefix('[') {
        rest.split(']').next().unwrap_or(rest)
    } else if raw.matches(':').count() == 1 {
        raw.split(':').next().unwrap_or(raw)
    } else {
        raw
    };
    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }
    // Tauri serves the WebView from tauri.localhost; RFC 6761 reserves the
    // whole .localhost tree for the loopback interface.
    if host.ends_with(".localhost") {
        return true;
    }
    host.parse::<std::net::IpAddr>()
        .map(|ip| ip.is_loopback())
        .unwrap_or(false)
}

fn authorize(
    user: &AuthenticatedUser,
    headers: &HeaderMap,
) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    if !botcore::product::is_shell_mode() {
        return Err(deny(
            "Operating System panel is unavailable outside shell mode",
        ));
    }
    let local_owner = user.metadata.contains_key("desktop_local");
    if !user.is_admin() && !user.is_super_admin() && !local_owner {
        return Err(deny("Administrator access required"));
    }
    if !request_is_loopback(headers) {
        return Err(deny(
            "Operating System panel is only reachable from the local host",
        ));
    }
    Ok(())
}

pub async fn os_status(
    Extension(user): Extension<AuthenticatedUser>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(response) = authorize(&user, &headers) {
        return response.into_response();
    }

    Json(OsStatus {
        shell_mode: true,
        hostname: read_hostname(),
        os: read_os(),
        kernel: read_kernel(),
        arch: std::env::consts::ARCH.to_string(),
        uptime_secs: read_uptime(),
        timezone: read_timezone(),
        disks: read_disks(),
        services: read_services(),
        log_files: log_file_infos(),
    })
    .into_response()
}

pub async fn os_logs(
    Extension(user): Extension<AuthenticatedUser>,
    headers: HeaderMap,
    Query(query): Query<LogQuery>,
) -> impl IntoResponse {
    if let Err(response) = authorize(&user, &headers) {
        return response.into_response();
    }

    let service = query.service.unwrap_or_else(|| "botserver".to_string());
    let allowed = service == "botserver"
        || SERVICES
            .iter()
            .any(|(name, _unit, _process)| *name == service.as_str());
    if !allowed {
        return (
            StatusCode::BAD_REQUEST,
            [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
            "Unknown service".to_string(),
        )
            .into_response();
    }

    let lines = query
        .lines
        .unwrap_or(DEFAULT_LOG_LINES)
        .clamp(1, MAX_LOG_LINES);
    let body = match log_file_for(&service) {
        Some(path) => tail(&path, lines),
        // #2102 — production logs to journald only.
        None => journal_tail(&service, lines).unwrap_or_else(|| missing_log_message(&service)),
    };
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        body,
    )
        .into_response()
}

/// #1874 — downloads the whole log file (the tail endpoint stays for the
/// viewer). Same allow-list and authorization as `/api/system/os/logs`.
pub async fn os_logs_download(
    Extension(user): Extension<AuthenticatedUser>,
    headers: HeaderMap,
    Query(query): Query<LogQuery>,
) -> impl IntoResponse {
    if let Err(response) = authorize(&user, &headers) {
        return response.into_response();
    }

    let service = query.service.unwrap_or_else(|| "botserver".to_string());
    let allowed = service == "botserver"
        || SERVICES
            .iter()
            .any(|(name, _unit, _process)| *name == service.as_str());
    if !allowed {
        return (
            StatusCode::BAD_REQUEST,
            [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
            "Unknown service".to_string(),
        )
            .into_response();
    }

    // #2102 — file first, then the journal (production), then a documented
    // 404 that still tells the panel where the file would be.
    let body = match log_file_for(&service) {
        Some(path) => match std::fs::read(&path) {
            Ok(bytes) => bytes,
            Err(e) => {
                log::error!("os_logs_download could not read {service}: {e}");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": "log_file_unreadable" })),
                )
                    .into_response();
            }
        },
        None => match journal_tail(&service, MAX_LOG_LINES) {
            Some(text) => text.into_bytes(),
            None => {
                let expected = expected_log_path(&service).to_string_lossy().to_string();
                return (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({
                        "error": "log_file_not_found",
                        "expected_path": expected,
                    })),
                )
                    .into_response();
            }
        },
    };

    let filename = format!("attachment; filename=\"{service}.log\"");
    let disposition = match axum::http::HeaderValue::from_str(&filename) {
        Ok(value) => value,
        Err(e) => {
            log::error!("os_logs_download invalid filename for {service}: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "log_file_unreadable" })),
            )
                .into_response();
        }
    };
    (
        StatusCode::OK,
        [
            (
                header::CONTENT_TYPE,
                axum::http::HeaderValue::from_static("text/plain; charset=utf-8"),
            ),
            (header::CONTENT_DISPOSITION, disposition),
        ],
        body,
    )
        .into_response()
}

pub async fn restart_service(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    headers: HeaderMap,
    Json(request): Json<RestartRequest>,
) -> impl IntoResponse {
    if let Err(response) = authorize(&user, &headers) {
        return response.into_response();
    }

    if request.confirmation != request.service {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "confirmation_mismatch" })),
        )
            .into_response();
    }

    let Some((name, unit, _process)) = SERVICES
        .iter()
        .find(|(name, _unit, _process)| *name == request.service.as_str())
    else {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "service_not_allowed" })),
        )
            .into_response();
    };

    let script = format!("systemctl restart {unit}");
    let result = tokio::task::spawn_blocking(move || {
        botsecurity::command_guard::SafeCommand::new("sh")
            .and_then(|command| command.arg("-c"))
            .and_then(|command| command.trusted_shell_script_arg(&script))
            .and_then(|command| command.execute())
    })
    .await;

    let success = matches!(
        &result,
        Ok(Ok(output)) if output.status.success()
    );

    crate::audit_log::record_audit_event(crate::audit_log::AuditEvent {
        state: &state,
        event_type: "os_service_restart",
        actor_id: user.user_id,
        action: "restart_service",
        target_type: Some("service"),
        target_id: None,
        success,
        details: Some(*name),
    });

    if success {
        (
            StatusCode::OK,
            Json(serde_json::json!({ "ok": true, "service": name })),
        )
            .into_response()
    } else {
        log::error!("restart_service failed for {name}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "service_restart_failed" })),
        )
            .into_response()
    }
}
