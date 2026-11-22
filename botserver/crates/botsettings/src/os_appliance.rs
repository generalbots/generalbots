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

#[cfg(test)]
mod tests {
    use super::*;
    use botsecurity::auth::Role;
    use uuid::Uuid;

    fn admin() -> Option<Extension<AuthenticatedUser>> {
        let mut user = AuthenticatedUser::new(Uuid::new_v4(), "admin".to_string());
        user.roles.push(Role::Admin);
        Some(Extension(user))
    }

    fn loopback_headers() -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert("host", "localhost:8080".parse().unwrap());
        h
    }

    #[tokio::test]
    async fn appliance_endpoints_are_gated_and_return_honest_status() {
        {
            let mut config = botcore::product::PRODUCT_CONFIG.write().unwrap();
            config.shell_mode = true;
        }
        let cases = [
            network(admin(), loopback_headers()).await,
            display(admin(), loopback_headers()).await,
            sound(admin(), loopback_headers()).await,
            input(admin(), loopback_headers()).await,
            storage(admin(), loopback_headers()).await,
            users(admin(), loopback_headers()).await,
            diagnostics(admin(), loopback_headers()).await,
        ];
        for result in cases {
            let json = result.expect("gated appliance endpoint must succeed for loopback admin");
            assert!(json.get("status").is_some(), "missing status in {:?}", json);
            assert!(json.get("message").is_some(), "missing message in {:?}", json);
        }
        // Non-loopback requests are refused by the shared os.rs gate.
        let mut remote = HeaderMap::new();
        remote.insert("host", "example.com".parse().unwrap());
        assert!(network(admin(), remote).await.is_err());
        {
            let mut config = botcore::product::PRODUCT_CONFIG.write().unwrap();
            config.shell_mode = false;
        }
    }
}
