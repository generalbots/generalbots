//! OS appliance status endpoints for the Settings → System pages (#2466).
//!
//! Each route uses the same shell-mode + admin + loopback gate as `os.rs` and
//! reports honest host state through `SafeCommand`. When a host tool is not
//! installed the response says so explicitly — the UI never shows a fake
//! value or a localStorage stub.

use axum::{
    extract::Extension,
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::get,
    Router,
};
use botcore::shared::state::AppState;
use botsecurity::{auth::AuthenticatedUser, command_guard::SafeCommand};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::os::{authorize, deny};
use crate::os_data::{read_disks, read_hostname, read_kernel, read_os, read_uptime};

pub(crate) fn configure_appliance_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/os/network", get(network))
        .route("/api/os/display", get(display))
        .route("/api/os/sound", get(sound))
        .route("/api/os/input", get(input))
        .route("/api/os/storage", get(storage))
        .route("/api/os/users", get(users))
        .route("/api/os/diagnostics", get(diagnostics))
}

fn probe(program: &str, args: &[&str]) -> Option<String> {
    let mut command = SafeCommand::new(program).ok()?;
    for arg in args {
        command = command.arg(*arg).ok()?;
    }
    let output = command.execute().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text.chars().take(2000).collect())
    }
}

fn unavailable(tool: &str) -> Value {
    json!({
        "status": "unavailable",
        "message": format!("{tool} is not installed in this build; this page is read-only.")
    })
}

type ApiResult = Result<Json<Value>, (StatusCode, Json<Value>)>;

fn gated(
    user: Option<Extension<AuthenticatedUser>>,
    headers: HeaderMap,
) -> Result<(), (StatusCode, Json<Value>)> {
    let Extension(user) = user.ok_or_else(|| deny("Authentication required"))?;
    authorize(&user, &headers)
}

async fn network(
    user: Option<Extension<AuthenticatedUser>>,
    headers: HeaderMap,
) -> ApiResult {
    gated(user, headers)?;
    Ok(Json(match probe("nmcli", &["-t", "-f", "DEVICE,TYPE,STATE", "device", "status"]) {
        Some(status) => json!({ "status": "ok", "message": status }),
        None => unavailable("NetworkManager (nmcli)"),
    }))
}

async fn display(
    user: Option<Extension<AuthenticatedUser>>,
    headers: HeaderMap,
) -> ApiResult {
    gated(user, headers)?;
    let backlight = std::fs::read_dir("/sys/class/backlight")
        .ok()
        .map(|entries| entries.filter_map(Result::ok).count())
        .unwrap_or(0);
    let outputs = probe("xrandr", &["--query"]);
    Ok(Json(match outputs {
        Some(list) => json!({ "status": "ok", "message": list, "backlight_devices": backlight }),
        None => json!({
            "status": "unavailable",
            "message": "No display probe is available in this build; this page is read-only.",
            "backlight_devices": backlight
        }),
    }))
}

async fn sound(
    user: Option<Extension<AuthenticatedUser>>,
    headers: HeaderMap,
) -> ApiResult {
    gated(user, headers)?;
    Ok(Json(match probe("pactl", &["info"]) {
        Some(info) => json!({ "status": "ok", "message": info }),
        None => unavailable("PipeWire/PulseAudio (pactl)"),
    }))
}

async fn input(
    user: Option<Extension<AuthenticatedUser>>,
    headers: HeaderMap,
) -> ApiResult {
    gated(user, headers)?;
    Ok(Json(match probe("localectl", &["status"]) {
        Some(info) => json!({ "status": "ok", "message": info }),
        None => unavailable("systemd-localed (localectl)"),
    }))
}

async fn storage(
    user: Option<Extension<AuthenticatedUser>>,
    headers: HeaderMap,
) -> ApiResult {
    gated(user, headers)?;
    let disks = read_disks();
    Ok(Json(json!({
        "status": "ok",
        "message": format!("{} block device(s) detected", disks.len()),
        "count": disks.len()
    })))
}

async fn users(
    user: Option<Extension<AuthenticatedUser>>,
    headers: HeaderMap,
) -> ApiResult {
    gated(user, headers)?;
    Ok(Json(json!({
        "status": "ok",
        "message": "Local accounts are managed by the directory service; this page is read-only in this build."
    })))
}

async fn diagnostics(
    user: Option<Extension<AuthenticatedUser>>,
    headers: HeaderMap,
) -> ApiResult {
    gated(user, headers)?;
    Ok(Json(json!({
        "status": "ok",
        "message": format!(
            "{} · {} · {} · uptime {}s",
            read_hostname(), read_os(), read_kernel(), read_uptime()
        )
    })))
}
