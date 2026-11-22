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

// #2481 — this module already exceeds the 450-line limit, so the strengthened
// tests for the pure `graphics_support_status()` classifier live in their own
// file. As a child module they still reach the private function via `super::`.
#[cfg(test)]
#[path = "os_appliance_tests.rs"]
mod os_appliance_tests;

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
        .route("/api/os/network/wifi", get(os_wifi_status).post(os_wifi_action))
        .route("/api/os/update/status", get(os_update_status))
        .route("/api/os/peripherals", get(os_peripherals_status).post(os_peripherals_set))
        .route("/api/os/session", get(os_session_status))
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

/// #2481 — classify graphics support from already-collected diagnostics text.
/// Pure: no I/O and no host probe, so the support bundle and the unit tests run
/// the same code. `gpu`, `vaapi` and `glx` are the redacted probe strings from
/// the bundle. llvmpipe/softpipe/swrast mean software compositing, not a GPU.
fn graphics_support_status(gpu: Option<&str>, vaapi: Option<&str>, glx: Option<&str>) -> Value {
    let software_markers = ["llvmpipe", "softpipe", "swrast", "software rasterizer"];
    let lower_glx = glx.unwrap_or("").to_ascii_lowercase();
    let software = software_markers.iter().any(|marker| lower_glx.contains(marker));
    let compositing = if glx.map_or(true, str::is_empty) {
        "unknown"
    } else if software {
        "software"
    } else {
        "hardware"
    };

    let mut decode: Vec<String> = Vec::new();
    let mut encode: Vec<String> = Vec::new();
    for line in vaapi.unwrap_or("").lines() {
        let Some((profile, entrypoint)) = line.split_once(':') else { continue };
        let profile = profile.trim();
        if !profile.starts_with("VAProfile") {
            continue;
        }
        let entry = entrypoint.trim().to_ascii_uppercase();
        if entry.contains("VLD") {
            decode.push(profile.to_string());
        } else if entry.contains("ENC") {
            encode.push(profile.to_string());
        }
    }
    decode.sort();
    decode.dedup();
    encode.sort();
    encode.dedup();

    json!({
        "compositing": compositing,
        "software_rendering": software,
        "gpu_detected": gpu.map_or(false, |text| text.lines().any(|line| {
            line.contains("VGA") || line.contains("Display") || line.contains("3D")
        })),
        "vaapi": {
            "available": !decode.is_empty() || !encode.is_empty(),
            "decode_profiles": decode,
            "encode_profiles": encode
        },
        "note": "Classification of probe text only; compositing throughput and decode speed are not measured offline."
    })
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
    // #2481 — collect the three probe strings once: raw text for support and a
    // pure classification of that same text for the graphics_support fields.
    let gpu = probe("lspci", &["-nn"]).map(|s| redact(&s.lines()
        .filter(|l| l.contains("VGA") || l.contains("Display") || l.contains("3D"))
        .take(8).collect::<Vec<_>>().join("\n")));
    let vaapi = probe("vainfo", &[]).map(|s| redact(&s.lines().take(20).collect::<Vec<_>>().join("\n")));
    let glx = probe("glxinfo", &["-B"]).map(|s| redact(&s.lines().take(12).collect::<Vec<_>>().join("\n")));
    let graphics_support = graphics_support_status(gpu.as_deref(), vaapi.as_deref(), glx.as_deref());
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
        // #2481 — GPU/VA-API strings for support reports, plus a pure classifier.
        // #2464 — thermal + battery telemetry for support reports.
        "thermal": probe("sensors", &[]).map(|s| redact(&s.lines().take(20).collect::<Vec<_>>().join("\n"))),
        "battery": json!(read_power_status()),
        "power_profile": probe("powerprofilesctl", &["get"]),
        // #2500 — WebKit bwrap sandbox status in the support report.
        "web_process_sandboxed": std::env::var("WEBKIT_FORCE_SANDBOX")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false),
        "graphics": json!({
            "gpu": gpu,
            "vaapi": vaapi,
            "glx": glx
        }),
        "graphics_support": graphics_support,
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

    #[test]
    fn graphics_support_classifies_compositing() {
        let sw = graphics_support_status(None, None, Some(
            "OpenGL renderer string: llvmpipe (LLVM 17.0, 256 bits)"));
        assert_eq!(sw["compositing"].as_str(), Some("software"));
        assert_eq!(sw["software_rendering"].as_bool(), Some(true));

        let hw = graphics_support_status(None, None, Some(
            "OpenGL renderer string: Mesa Intel(R) UHD Graphics 620 (KBL GT2)"));
        assert_eq!(hw["compositing"].as_str(), Some("hardware"));
        assert_eq!(hw["software_rendering"].as_bool(), Some(false));

        let unknown = graphics_support_status(None, None, None);
        assert_eq!(unknown["compositing"].as_str(), Some("unknown"));
        let empty = graphics_support_status(None, None, Some(""));
        assert_eq!(empty["compositing"].as_str(), Some("unknown"));
    }

    #[test]
    fn graphics_support_parses_vaapi_profiles() {
        let vainfo = "vainfo: Supported profile and entrypoints\n\
            VAProfileH264Main : VAEntrypointVLD\n\
            VAProfileH264High : VAEntrypointEncSlice\n\
            VAProfileH264Main : VAEntrypointVLD\n\
            VAProfileNone : VAEntrypointVideoProc";
        let status = graphics_support_status(
            Some("00:02.0 VGA compatible controller"), Some(vainfo), None);
        assert_eq!(status["vaapi"]["available"].as_bool(), Some(true));
        assert_eq!(status["gpu_detected"].as_bool(), Some(true));

        let decode = status["vaapi"]["decode_profiles"].as_array();
        assert!(decode.is_some(), "decode_profiles must be an array");
        if let Some(profiles) = decode {
            assert_eq!(profiles.len(), 1, "profiles must be de-duplicated");
            assert_eq!(profiles[0].as_str(), Some("VAProfileH264Main"));
        }
        let encode = status["vaapi"]["encode_profiles"].as_array();
        assert!(encode.is_some(), "encode_profiles must be an array");
        if let Some(profiles) = encode {
            assert_eq!(profiles.len(), 1);
            assert_eq!(profiles[0].as_str(), Some("VAProfileH264High"));
        }
    }

    #[test]
    fn graphics_support_handles_missing_probes() {
        let status = graphics_support_status(None, None, None);
        assert_eq!(status["vaapi"]["available"].as_bool(), Some(false));
        assert_eq!(status["gpu_detected"].as_bool(), Some(false));
        assert!(status["vaapi"]["decode_profiles"].as_array()
            .map_or(false, Vec::is_empty));
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

/// #2453 — Wi-Fi scan/join and wired network status via NetworkManager.
async fn os_wifi_status(
    user: Option<Extension<AuthenticatedUser>>,
    headers: HeaderMap,
) -> ApiResult {
    gated(user, headers)?;
    Ok(Json(json!({
        "status": "ok",
        "wifi_device": probe("nmcli", &["-t", "-f", "DEVICE,TYPE,STATE", "device", "status"]),
        "networks": probe("nmcli", &["-t", "-f", "IN-USE,SSID,SIGNAL,SECURITY", "device", "wifi", "list"]),
        "connection": probe("nmcli", &["-t", "-f", "NAME,TYPE,AUTOCONNECT", "connection", "show"]),
        "ip_dns": probe("nmcli", &["-t", "-f", "IP4-ADDRESS,IP4-GATEWAY,IP4-DNS", "device", "show", "ifname", "wlan0"])
    })))
}

#[derive(serde::Deserialize)]
struct WifiActionRequest { action: String, ssid: String, password: Option<String>, hidden: Option<bool>, priority: Option<i32> }

async fn os_wifi_action(
    user: Option<Extension<AuthenticatedUser>>,
    headers: HeaderMap,
    axum::Json(req): axum::Json<WifiActionRequest>,
) -> ApiResult {
    gated(user, headers)?;
    let ssid = req.ssid.clone();
    if !ssid.chars().all(|c| c.is_ascii_alphanumeric() || " -_.@#".contains(c)) || ssid.len() > 32 {
        return Ok(Json(json!({ "ok": false, "error": "invalid_ssid" })));
    }
    let ok = match req.action.as_str() {
        "join" => {
            let pwd = match req.password.clone() {
                Some(value) => value,
                None => {
                    log::warn!("OS appliance: password change request without a password");
                    String::new()
                }
            };
            let hidden = req.hidden.unwrap_or(false);
            if !pwd.is_empty() && (pwd.len() < 8 || pwd.len() > 63) {
                return Ok(Json(json!({ "ok": false, "error": "invalid_passphrase" })));
            }
            let mut args = vec!["device".to_string(), "wifi".to_string(), "connect".to_string(), ssid.clone()];
            if !pwd.is_empty() { args.push("password".to_string()); args.push(pwd); }
            if hidden { args.push("hidden".to_string()); args.push("yes".to_string()); }
            let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            run_display_command("nmcli", &refs).await
        }
        "forget" => {
            let a = vec!["connection".to_string(), "delete".to_string(), ssid.clone()];
            let refs: Vec<&str> = a.iter().map(|s| s.as_str()).collect();
            run_display_command("nmcli", &refs).await
        }
        "priority" => {
            let p = req.priority.unwrap_or(0).to_string();
            let a = vec!["connection".to_string(), "modify".to_string(), ssid.clone(),
                "connection.autoconnect-priority".to_string(), p];
            let refs: Vec<&str> = a.iter().map(|s| s.as_str()).collect();
            run_display_command("nmcli", &refs).await
        }
        _ => false,
    };
    Ok(Json(json!({ "ok": ok, "action": req.action, "ssid": ssid })))
}

/// #2443 — A/B slot update status, channel and offline media detection.
async fn os_update_status(
    user: Option<Extension<AuthenticatedUser>>,
    headers: HeaderMap,
) -> ApiResult {
    gated(user, headers)?;
    let active = std::fs::read_link("/opt/general-bots").ok()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "unversioned".to_string());
    let offline = std::path::Path::new("/media/gbdesk/gb-update.tar.zst").exists();
    Ok(Json(json!({
        "status": "ok",
        "active_slot": active,
        "slots": ["/var/lib/gbapp/a", "/var/lib/gbapp/b"],
        "channel": std::env::var("GB_UPDATE_CHANNEL").unwrap_or_else(|_| "stable".to_string()),
        "version": env!("CARGO_PKG_VERSION"),
        "build_channel": std::env::var("GB_OS_CHANNEL").unwrap_or_else(|_| "stable".to_string()),
        "offline_update_available": offline,
        "rollback_supported": active.starts_with("/var/lib/gbapp/")
    })))
}

/// #2462 — touchpad, Bluetooth and printers for the kiosk.
async fn os_peripherals_status(
    user: Option<Extension<AuthenticatedUser>>,
    headers: HeaderMap,
) -> ApiResult {
    gated(user, headers)?;
    Ok(Json(json!({
        "status": "ok",
        "touchpad": {
            "tap_to_click": probe("gsettings", &["get", "org.gnome.desktop.peripherals.touchpad", "tap-to-click"]),
            "natural_scroll": probe("gsettings", &["get", "org.gnome.desktop.peripherals.touchpad", "natural-scroll"]),
            "speed": probe("gsettings", &["get", "org.gnome.desktop.peripherals.touchpad", "speed"]),
            "disable_while_typing": probe("gsettings", &["get", "org.gnome.desktop.peripherals.touchpad", "disable-while-typing"])
        },
        "bluetooth": probe("bluetoothctl", &["show"]),
        "devices": probe("bluetoothctl", &["devices"]),
        "printers": probe("lpstat", &["-p"]),
        "discovery": probe("avahi-browse", &["-at", "-t", "_ipp._tcp"])
    })))
}

#[derive(serde::Deserialize)]
struct PeripheralRequest { setting: String, value: String }

async fn os_peripherals_set(
    user: Option<Extension<AuthenticatedUser>>,
    headers: HeaderMap,
    axum::Json(req): axum::Json<PeripheralRequest>,
) -> ApiResult {
    gated(user, headers)?;
    let ok = match req.setting.as_str() {
        "tap_to_click" | "natural_scroll" | "disable_while_typing"
            if req.value == "true" || req.value == "false" => {
            let key = req.setting.replace('_', "-");
            run_display_command("gsettings", &["set", "org.gnome.desktop.peripherals.touchpad", &key, &req.value]).await
        }
        "speed" if req.value.parse::<f64>().map(|v| (0.0..=1.0).contains(&v)).unwrap_or(false) =>
            run_display_command("gsettings", &["set", "org.gnome.desktop.peripherals.touchpad", "speed", &req.value]).await,
        "bluetooth" if req.value == "on" || req.value == "off" =>
            run_display_command("rfkill", &[if req.value == "on" { "unblock" } else { "block" }, "bluetooth"]).await,
        _ => false,
    };
    Ok(Json(json!({ "ok": ok, "setting": req.setting, "value": req.value })))
}

/// #2449 — real user-manager session status for the kiosk.
async fn os_session_status(
    user: Option<Extension<AuthenticatedUser>>,
    headers: HeaderMap,
) -> ApiResult {
    gated(user, headers)?;
    let unit = probe("systemctl", &["--user", "is-system-running"]);
    let runtime = match std::env::var("XDG_RUNTIME_DIR") {
        Ok(value) => value,
        Err(error) => {
            log::warn!("XDG_RUNTIME_DIR unavailable: {error}");
            String::new()
        }
    };
    let bus = match std::env::var("DBUS_SESSION_BUS_ADDRESS") {
        Ok(value) => value,
        Err(error) => {
            log::warn!("DBUS_SESSION_BUS_ADDRESS unavailable: {error}");
            String::new()
        }
    };
    let services = ["pipewire.socket", "wireplumber.service", "xdg-desktop-portal.service"]
        .iter().map(|name| {
            let active = probe("systemctl", &["--user", "is-active", name]);
            json!({ "unit": name, "active": active })
        }).collect::<Vec<_>>();
    Ok(Json(json!({
        "status": "ok",
        "user_manager": unit,
        "xdg_runtime_dir": runtime,
        "session_bus": redact(&bus),
        "user_services": services,
        "expected_runtime": "/run/user/<uid>".to_string()
    })))
}
