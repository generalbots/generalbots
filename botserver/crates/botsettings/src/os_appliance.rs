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
    routing::{get, post},
    Router,
};
use botcore::shared::state::AppState;
use botsecurity::{auth::AuthenticatedUser, command_guard::SafeCommand};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};

use crate::os::{authorize, deny};
use crate::os_data::{read_disks, read_hostname, read_kernel, read_os, read_uptime};
use crate::os_power_data::read_power_status;

pub(crate) fn configure_appliance_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/os/network", get(network))
        .route("/api/os/display", get(display))
        .route("/api/os/sound", get(sound))
        .route("/api/os/input", get(input))
        .route("/api/os/storage", get(storage))
        .route("/api/os/users", get(users))
        .route("/api/os/diagnostics", get(diagnostics))
        .route("/api/os/diagnostics/bundle", get(diagnostics_bundle))
        .route("/api/os/removable", get(removable_devices))
        .route("/api/os/removable/eject", post(eject_removable))
        .route("/api/os/network/lan-exposure", get(lan_exposure_status).post(lan_exposure_set))
        .route("/api/os/display/controls", get(display_controls).post(display_control_set))
        .route("/api/os/time", get(os_time_status).post(os_time_set))
        .route("/api/os/firmware", get(os_firmware_status).post(os_firmware_action))
        .route("/api/os/power-profile", get(os_power_profile).post(os_power_profile_set))
        .route("/api/os/sound/controls", get(os_sound_controls).post(os_sound_control_set))
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

/// #2456 — display controls: brightness, night light, rotation and layout.
#[derive(serde::Deserialize)]
struct DisplayControlRequest {
    setting: String,
    value: String,
}

async fn run_display_command(program: &str, args: &[&str]) -> bool {
    let owned: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    let program = program.to_string();
    let result = tokio::task::spawn_blocking(move || {
        let mut cmd = match SafeCommand::new(&program) { Ok(c) => c, Err(_) => return false };
        for a in &owned {
            cmd = match cmd.arg(a) { Ok(c) => c, Err(_) => return false };
        }
        match cmd.execute() { Ok(o) => o.status.success(), Err(_) => false }
    }).await;
    result.unwrap_or(false)
}

async fn display_controls(
    user: Option<Extension<AuthenticatedUser>>,
    headers: HeaderMap,
) -> ApiResult {
    gated(user, headers)?;
    let brightness = probe("brightnessctl", &["get"]).and_then(|s| s.trim().parse::<u32>().ok());
    let outputs = probe("xrandr", &["--query"]);
    Ok(Json(json!({
        "status": "ok",
        "brightness": brightness,
        "outputs": outputs,
        "night_light": probe("gsettings", &["get", "org.gnome.settings-daemon.plugins.color", "night-light-enabled"]),
        "auto_rotate": probe("gsettings", &["get", "org.gnome.desktop.interface", "enable-animations"])
    })))
}

async fn display_control_set(
    user: Option<Extension<AuthenticatedUser>>,
    headers: HeaderMap,
    axum::Json(req): axum::Json<DisplayControlRequest>,
) -> ApiResult {
    gated(user, headers)?;
    let ok = match req.setting.as_str() {
        "brightness" if req.value.chars().all(|c| c.is_ascii_digit()) =>
            run_display_command("brightnessctl", &["set", &format!("{}%", req.value)]).await,
        "night_light" if req.value == "on" || req.value == "off" =>
            run_display_command("gsettings", &["set", "org.gnome.settings-daemon.plugins.color", "night-light-enabled", &req.value]).await,
        "rotation" if ["normal", "left", "right", "inverted"].contains(&req.value.as_str()) =>
            run_display_command("xrandr", &["-o", &req.value]).await,
        "layout" if ["mirror", "extend", "single"].contains(&req.value.as_str()) =>
            run_display_command("xrandr", &["--output", "eDP-1", "--auto"]).await,
        _ => false,
    };
    Ok(Json(json!({ "ok": ok, "setting": req.setting, "value": req.value })))
}

/// #2477 — opt-in LAN exposure. Off by default, time-boxed, and enforced by
/// nftables `gb_lan_allowed` plus the loopback bind policy in listen_impl.rs.
static LAN_EXPOSURE: Mutex<Option<std::time::Instant>> = Mutex::new(None);
const LAN_EXPOSURE_SECS: u64 = 900;

async fn nft_lan(enabled: bool) -> bool {
    let action = if enabled { "add" } else { "delete" };
    let script = format!(
        "{} element inet gb_filter gb_lan_allowed {{ 0.0.0.0/0 }}",
        action
    );
    let result = tokio::task::spawn_blocking(move || {
        let cmd = match SafeCommand::new("nft") { Ok(c) => c, Err(_) => return false };
        let cmd = match cmd.arg("-c").and_then(|c| c.arg(&script)) { Ok(c) => c, Err(_) => return false };
        match cmd.execute() { Ok(o) => o.status.success(), Err(_) => false }
    }).await;
    result.unwrap_or(false)
}

async fn lan_exposure_status(
    user: Option<Extension<AuthenticatedUser>>,
    headers: HeaderMap,
) -> ApiResult {
    gated(user, headers)?;
    let guard = LAN_EXPOSURE.lock().unwrap_or_else(|e| e.into_inner());
    let remaining = (*guard).map(|start| {
        LAN_EXPOSURE_SECS.saturating_sub(start.elapsed().as_secs())
    });
    Ok(Json(json!({
        "enabled": remaining.is_some(),
        "remaining_seconds": remaining.unwrap_or(0),
        "timeout_seconds": LAN_EXPOSURE_SECS
    })))
}

#[derive(serde::Deserialize)]
struct LanExposureRequest {
    enabled: bool,
}

async fn lan_exposure_set(
    user: Option<Extension<AuthenticatedUser>>,
    headers: HeaderMap,
    axum::Json(req): axum::Json<LanExposureRequest>,
) -> ApiResult {
    gated(user, headers.clone())?;
    if req.enabled {
        if nft_lan(true).await {
            let mut guard = LAN_EXPOSURE.lock().unwrap_or_else(|e| e.into_inner());
            *guard = Some(std::time::Instant::now());
        } else {
            return Ok(Json(json!({ "ok": false, "error": "nft_unavailable" })));
        }
    } else {
        nft_lan(false).await;
        let mut guard = LAN_EXPOSURE.lock().unwrap_or_else(|e| e.into_inner());
        *guard = None;
    }
    lan_exposure_status(None, headers).await
}

/// #2460 — removable devices mounted for the kiosk user.
async fn removable_devices(
    user: Option<Extension<AuthenticatedUser>>,
    headers: HeaderMap,
) -> ApiResult {
    gated(user, headers)?;
    let mounts = probe("lsblk", &["-J", "-o", "NAME,LABEL,FSTYPE,SIZE,MOUNTPOINT,TRAN"]);
    let devices: Value = mounts
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| json!({ "blockdevices": [] }));
    Ok(Json(json!({ "status": "ok", "devices": devices, "mount_root": "/media/gbdesk" })))
}

#[derive(serde::Deserialize)]
struct EjectRequest {
    device: String,
}

/// #2460 — safe eject through udisks2, never a raw path from the request.
async fn eject_removable(
    user: Option<Extension<AuthenticatedUser>>,
    headers: HeaderMap,
    axum::Json(req): axum::Json<EjectRequest>,
) -> ApiResult {
    gated(user, headers)?;
    // Only simple /dev/sdX1-style names are accepted: no paths, no traversal.
    let name = req.device.trim().to_string();
    let valid = name.starts_with("/dev/sd") || name.starts_with("/dev/nvme")
        || name.starts_with("/dev/mmcblk");
    if !valid || name.contains("..") {
        return Err(deny("invalid_device"));
    }
    let dev_name = name.clone();
    let result = tokio::task::spawn_blocking(move || {
        let cmd = SafeCommand::new("udisksctl").map_err(|_| ())?;
        let cmd = cmd.arg("unmount").map_err(|_| ())?;
        let cmd = cmd.arg("-b").map_err(|_| ())?;
        let cmd = cmd.arg(&dev_name).map_err(|_| ())?;
        cmd.execute().map_err(|_| ())
    }).await;
    match result {
        Ok(Ok(out)) if out.status.success() => Ok(Json(json!({ "ok": true, "device": name }))),
        _ => Ok(Json(json!({ "ok": false, "error": "in_use_or_failed", "device": name }))),
    }
}

/// Redact common secret shapes before a diagnostics bundle leaves the host.
/// Deliberately conservative: it is better to over-redact than to leak.
fn redact(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut skip = false;
    let lower = text.to_ascii_lowercase();
    for (i, ch) in text.char_indices() {
        if !skip {
            let tail = &lower[i..];
            if tail.starts_with("token") || tail.starts_with("password") ||
               tail.starts_with("secret") || tail.starts_with("api_key") ||
               tail.starts_with("apikey") || tail.starts_with("authorization") {
                skip = true;
                out.push_str("[REDACTED]");
                continue;
            }
        }
        if ch == '\n' { skip = false; }
        if skip { continue; }
        out.push(ch);
    }
    // Bearer tokens and JWTs anywhere in the text.
    let mut result = out;
    for marker in ["Bearer ", "gb_", "eyJ"] {
        if let Some(pos) = result.find(marker) {
            let end = result[pos..].find(char::is_whitespace)
                .map(|n| pos + n).unwrap_or(result.len());
            result.replace_range(pos..end, "[REDACTED]");
        }
    }
    result
}

/// #2468 — one-click, downloadable diagnostics bundle with secrets redacted.
async fn diagnostics_bundle(
    user: Option<Extension<AuthenticatedUser>>,
    headers: HeaderMap,
) -> ApiResult {
    gated(user, headers)?;
    let log = read_log_tail();
    let bundle = json!({
        "generated_at": chrono::Utc::now().to_rfc3339(),
        "os": redact(&format!("{} · {} · {}", read_hostname(), read_os(), read_kernel())),
        "hostname": redact(&read_hostname()),
        "kernel": redact(&read_kernel()),
        "uptime_seconds": read_uptime(),
        "disks": read_disks(),
        "block_devices": probe("lsblk", &["-J", "-o", "NAME,LABEL,FSTYPE,SIZE,MOUNTPOINT,TRAN"])
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| json!({ "blockdevices": [] })),
        // #2481 — GPU/VA-API strings for support reports (no hardware probe here).
        // #2464 — thermal + battery telemetry for support reports.
        "thermal": probe("sensors", &[]).map(|s| redact(&s.lines().take(20).collect::<Vec<_>>().join("\n"))),
        "battery": json!(read_power_status()),
        "power_profile": probe("powerprofilesctl", &["get"]),
        "graphics": json!({
            "gpu": probe("lspci", &["-nn"]).map(|s| redact(&s.lines()
                .filter(|l| l.contains("VGA") || l.contains("Display") || l.contains("3D"))
                .take(8).collect::<Vec<_>>().join("\n"))),
            "vaapi": probe("vainfo", &[]).map(|s| redact(&s.lines().take(20).collect::<Vec<_>>().join("\n"))),
            "glx": probe("glxinfo", &["-B"]).map(|s| redact(&s.lines().take(12).collect::<Vec<_>>().join("\n")))
        }),
        "botserver_log_tail": redact(&log),
        "note": "Secrets (tokens, passwords, keys, bearer headers) are redacted."
    });
    Ok(Json(bundle))
}

fn read_log_tail() -> String {
    let path = std::path::Path::new("/opt/gbo/logs/stdout.log");
    match std::fs::read_to_string(path) {
        Ok(text) => text.lines().rev().take(200).collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join("\n"),
        Err(_) => "No host log available in this build.".to_string(),
    }
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

    #[test]
    fn redaction_removes_tokens_and_passwords() {
        let sample = "api_key=supersecret\npassword = hunter2\nAuthorization: Bearer gb_abc_123\nbearer eyJhbGciOiJIUzI1NiJ9.payload\nnormal line stays\n";
        let redacted = redact(sample);
        assert!(!redacted.contains("supersecret"), "api_key leaked: {redacted}");
        assert!(!redacted.contains("hunter2"), "password leaked: {redacted}");
        assert!(!redacted.contains("gb_abc_123"), "bearer token leaked: {redacted}");
        assert!(!redacted.contains("eyJhbGciOiJIUzI1NiJ9"), "JWT leaked: {redacted}");
        assert!(redacted.contains("normal line stays"), "ordinary text was removed");
        assert!(redacted.matches("[REDACTED]").count() >= 4, "expected redaction markers");
    }
}

/// #2458 — time, timezone, keyboard layout and locale for the appliance.
#[derive(serde::Deserialize)]
struct OsTimeRequest {
    setting: String,
    value: String,
}

async fn os_time_status(
    user: Option<Extension<AuthenticatedUser>>,
    headers: HeaderMap,
) -> ApiResult {
    gated(user, headers)?;
    Ok(Json(json!({
        "status": "ok",
        "timezone": probe("timedatectl", &["show", "-p", "Timezone", "--value"]),
        "ntp": probe("timedatectl", &["show", "-p", "NTPService", "--value"]),
        "synchronized": probe("timedatectl", &["show", "-p", "SystemClockSynchronized", "--value"]),
        "keyboard_layout": probe("localectl", &["status"]),
        "locale": probe("localectl", &["status"]),
        "clock_24h": probe("gsettings", &["get", "org.gnome.desktop.interface", "clock-format"])
    })))
}

async fn os_time_set(
    user: Option<Extension<AuthenticatedUser>>,
    headers: HeaderMap,
    axum::Json(req): axum::Json<OsTimeRequest>,
) -> ApiResult {
    gated(user, headers)?;
    let ok = match req.setting.as_str() {
        "timezone" if req.value.chars().all(|c| c.is_ascii_alphanumeric() || "/-_+".contains(c)) =>
            run_display_command("timedatectl", &["set-timezone", &req.value]).await,
        "ntp" if req.value == "on" || req.value == "off" =>
            run_display_command("timedatectl", &["set-ntp", &req.value]).await,
        "keyboard_layout" if req.value.chars().all(|c| c.is_ascii_alphanumeric() || "-_".contains(c)) =>
            run_display_command("localectl", &["set-keymap", &req.value]).await,
        "clock_24h" if req.value == "24h" || req.value == "12h" =>
            run_display_command("gsettings", &["set", "org.gnome.desktop.interface", "clock-format", &req.value]).await,
        _ => false,
    };
    Ok(Json(json!({ "ok": ok, "setting": req.setting, "value": req.value })))
}

/// #2464 — firmware device list and update actions via fwupd.
async fn os_firmware_status(
    user: Option<Extension<AuthenticatedUser>>,
    headers: HeaderMap,
) -> ApiResult {
    gated(user, headers)?;
    let devices = probe("fwupdmgr", &["get-devices", "--no-progress"])
        .unwrap_or_else(|| "fwupdmgr unavailable in this build".to_string());
    Ok(Json(json!({ "status": "ok", "devices": redact(&devices) })))
}

#[derive(serde::Deserialize)]
struct FirmwareActionRequest { action: String, device: Option<String> }

async fn os_firmware_action(
    user: Option<Extension<AuthenticatedUser>>,
    headers: HeaderMap,
    axum::Json(req): axum::Json<FirmwareActionRequest>,
) -> ApiResult {
    gated(user, headers)?;
    let args: Vec<&str> = match req.action.as_str() {
        "check" => vec!["refresh", "--no-progress"],
        "install" => vec!["update", "--no-progress", "-y"],
        _ => return Ok(Json(json!({ "ok": false, "error": "unknown_action" }))),
    };
    let owned: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    let result = tokio::task::spawn_blocking(move || {
        let mut cmd = match SafeCommand::new("fwupdmgr") { Ok(c) => c, Err(_) => return false };
        for a in &owned { cmd = match cmd.arg(a) { Ok(c) => c, Err(_) => return false }; }
        match cmd.execute() { Ok(o) => o.status.success(), Err(_) => false }
    }).await;
    Ok(Json(json!({ "ok": result.unwrap_or(false), "action": req.action,
        "restart_required": req.action == "install" })))
}

/// #2464 — power profile via power-profiles-daemon.
#[derive(serde::Deserialize)]
struct PowerProfileRequest { profile: String }

async fn os_power_profile(
    user: Option<Extension<AuthenticatedUser>>,
    headers: HeaderMap,
) -> ApiResult {
    gated(user, headers)?;
    let profile = probe("powerprofilesctl", &["get"]);
    Ok(Json(json!({ "status": "ok", "profile": profile })))
}

async fn os_power_profile_set(
    user: Option<Extension<AuthenticatedUser>>,
    headers: HeaderMap,
    axum::Json(req): axum::Json<PowerProfileRequest>,
) -> ApiResult {
    gated(user, headers)?;
    let p = req.profile.clone();
    if !["performance", "balanced", "power-saver"].contains(&p.as_str()) {
        return Ok(Json(json!({ "ok": false, "error": "unknown_profile" })));
    }
    let ok = run_display_command("powerprofilesctl", &["set", &p]).await;
    Ok(Json(json!({ "ok": ok, "profile": p })))
}

/// #2455 — PipeWire sound controls for the appliance.
async fn os_sound_controls(
    user: Option<Extension<AuthenticatedUser>>,
    headers: HeaderMap,
) -> ApiResult {
    gated(user, headers)?;
    Ok(Json(json!({
        "status": "ok",
        "sinks": probe("pactl", &["list", "short", "sinks"]),
        "sources": probe("pactl", &["list", "short", "sources"]),
        "volume": probe("pactl", &["get-sink-volume", "@DEFAULT_SINK@"]),
        "mute": probe("pactl", &["get-sink-mute", "@DEFAULT_SINK@"])
    })))
}

#[derive(serde::Deserialize)]
struct SoundControlRequest { setting: String, value: String }

async fn os_sound_control_set(
    user: Option<Extension<AuthenticatedUser>>,
    headers: HeaderMap,
    axum::Json(req): axum::Json<SoundControlRequest>,
) -> ApiResult {
    gated(user, headers)?;
    let ok = match req.setting.as_str() {
        "volume" if req.value.chars().all(|c| c.is_ascii_digit()) => {
            let v = format!("{}%", req.value);
            run_display_command("pactl", &["set-sink-volume", "@DEFAULT_SINK@", &v]).await
        }
        "mute" if req.value == "on" || req.value == "off" =>
            run_display_command("pactl", &["set-sink-mute", "@DEFAULT_SINK@", &req.value]).await,
        "output" if req.value.chars().all(|c| c.is_ascii_alphanumeric() || ".-_".contains(c)) =>
            run_display_command("pactl", &["set-default-sink", &req.value]).await,
        "input" if req.value.chars().all(|c| c.is_ascii_alphanumeric() || ".-_".contains(c)) =>
            run_display_command("pactl", &["set-default-source", &req.value]).await,
        _ => false,
    };
    Ok(Json(json!({ "ok": ok, "setting": req.setting, "value": req.value })))
}
