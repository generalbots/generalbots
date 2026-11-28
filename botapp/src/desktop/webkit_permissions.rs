//! Routes WebKitGTK user-media permission requests on the desktop shell (#2504).
//!
//! wry keeps WebKitGTK's MediaStream/WebRTC settings disabled by default, and
//! WebKitGTK never shows a browser-style permission prompt. Without both the
//! settings and the permission-request handler, `getUserMedia()` rejects with
//! `NotAllowedError` and the chat mic/camera/live-voice buttons always fail
//! (#1898). A handler that never answers leaves the request pending forever,
//! which the chat camera reports as a black preview (#2106), so every decision
//! is logged: the desktop log then says whether the WebView even asked.
//!
//! #2504 hardening: a request is granted only when the WebView's top-level URI
//! resolves to a first-party origin — the shell's bundled origins, localhost
//! (debug plain HTTP, release HTTPS), the product domains documented in
//! AGENTS.md over HTTPS, or an exact origin explicitly named in
//! `GB_MEDIA_ORIGINS`. Every other origin is denied and logged. WebKitGTK 2.0
//! does not expose the requesting frame's security origin on a
//! `UserMediaPermissionRequest`, so a foreign iframe inside an owned page
//! cannot be distinguished with this API; `Permissions-Policy` on the served
//! pages is the complementary defence. There is no WebKitGTK prompt or
//! capture-indicator API either: each grant is published as a structured Tauri
//! event for the shell UI to render, in addition to the audit log. The tray
//! LED convention, Settings → Privacy revocation and the UI listener are
//! separate surfaces and are not implemented by this handler.

use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{Emitter, Manager};
use webkit2gtk::glib::prelude::Cast;
use webkit2gtk::{
    GeolocationPermissionRequest, NotificationPermissionRequest, PermissionRequest,
    PermissionRequestExt, SettingsExt, UserMediaPermissionRequest, UserMediaPermissionRequestExt,
    WebViewExt,
};

/// Name of the plugin, also the prefix of its log lines.
const PLUGIN_NAME: &str = "generalbots-webkit-permissions";

/// Origins of the bundled UI: Tauri's custom scheme on Linux/Windows and its
/// HTTPS alias on macOS. Exact matches only; no subdomains or ports.
const BUNDLED_ORIGINS: &[&str] = &["tauri://localhost", "https://tauri.localhost"];

/// First-party product hosts from AGENTS.md. Remote content (mail bodies,
/// marketplace apps, LLM-generated HTML) lives on other hosts and is denied.
const PRODUCT_HOSTS: &[&str] = &[
    "pragmatismo.com.br",
    "cloud.pragmatismo.com.br",
    "chat.pragmatismo.com.br",
    "login.pragmatismo.com.br",
    "login.generalbots.org",
    "generalbots.org",
];

/// Bot workspaces are hosted as `https://{bot}.generalbots.org/…`. Only one
/// DNS label is accepted below the suffix, so `a.b.generalbots.org` and
/// `generalbots.org.evil.example` never match.
const BOT_SUBDOMAIN_SUFFIX: &str = ".generalbots.org";

/// Local suite/cloud/login/API ports documented in AGENTS.md. Plain HTTP is
/// trusted only in debug builds; a release binary never grants media to it.
const LOCAL_PORTS: &[u16] = &[3000, 4000, 5000, 8080];

/// Tauri's dev server (`devUrl` in `tauri.conf.json`).
const DEV_ORIGIN: &str = "http://localhost:3000";

/// Optional, operator-controlled extension of the allowlist: a comma-separated
/// list of exact `scheme://authority` origins. Never a wildcard or an IP.
const MEDIA_ORIGINS_ENV: &str = "GB_MEDIA_ORIGINS";

/// Structured, local-only event carrying each media grant to the shell UI.
/// It contains no UI text: rendering and localisation belong to the consumer.
const MEDIA_GRANT_EVENT: &str = "gb-media-grant";

/// #2504 — Settings → Privacy can revoke a device at OS level: while the
/// flag is set the permission handler denies every matching request from
/// every origin, regardless of the allowlist. Atomic because permission
/// requests and the settings command run on different threads.
static AUDIO_REVOKED: AtomicBool = AtomicBool::new(false);
static VIDEO_REVOKED: AtomicBool = AtomicBool::new(false);

fn revoked(audio: bool, video: bool) -> bool {
    (audio && AUDIO_REVOKED.load(Ordering::Relaxed))
        || (video && VIDEO_REVOKED.load(Ordering::Relaxed))
}

/// Sets the OS-level permission for a device kind. `kind` is `audio` or
/// `video`; `enabled = false` denies all future requests for that device.
pub fn set_device_permission(kind: &str, enabled: bool) -> Result<(), String> {
    let flag = match kind {
        "audio" => &AUDIO_REVOKED,
        "video" => &VIDEO_REVOKED,
        other => return Err(format!("unknown media device: {other}")),
    };
    flag.store(!enabled, Ordering::Relaxed);
    log::info!("{PLUGIN_NAME}: device {kind} enabled={enabled}");
    Ok(())
}

#[tauri::command]
pub fn set_media_permission(kind: String, enabled: bool) -> Result<(), String> {
    set_device_permission(&kind, enabled)
}

/// The outcome of a user-media request, computed without touching WebKit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MediaDecision {
    /// Neither audio nor video was requested: not a media permission at all.
    DenyNoDevice,
    /// The top-level page URI is missing or not an absolute hierarchical URI.
    DenyUnparseable,
    /// The page origin is not a first-party origin.
    DenyForeign,
    /// The page origin is first-party and audio/video was requested.
    Grant,
}

/// Returns the `scheme://authority` prefix of a page URI, or `None` when the
/// string is not an absolute hierarchical URI. Path, query and fragment are
/// dropped because they are not part of an origin. Userinfo is rejected rather
/// than returned so credentials can never reach the audit log.
fn origin_of(uri: &str) -> Option<&str> {
    let (scheme, rest) = uri.split_once("://")?;
    let valid_scheme = !scheme.is_empty()
        && scheme
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'-' | b'.'));
    if !valid_scheme {
        return None;
    }
    let authority = rest.split(['/', '?', '#']).next().unwrap_or(rest);
    if authority.is_empty() || authority.contains('@') {
        return None;
    }
    let end = scheme.len() + 3 + authority.len();
    uri.get(..end)
}

/// Splits an authority into host and explicit port. An unparsable port makes
/// the whole authority the host, so the string cannot match an allowlist entry.
fn split_authority(authority: &str) -> (&str, Option<u16>) {
    if let Some(rest) = authority.strip_prefix('[') {
        if let Some((host, tail)) = rest.split_once(']') {
            let port = tail.strip_prefix(':').and_then(|value| value.parse().ok());
            return (host, port);
        }
        return (rest, None);
    }
    if let Some((host, port)) = authority.rsplit_once(':') {
        if let Ok(port) = port.parse::<u16>() {
            return (host, Some(port));
        }
    }
    (authority, None)
}

/// Whether a host is a documented product host or a single-label bot workspace
/// under `generalbots.org`. Matching is case-insensitive and exact: a suffix
/// attack like `evil.pragmatismo.com.br` never grants media.
fn product_host_allowed(host: &str) -> bool {
    let lower = host.to_ascii_lowercase();
    if PRODUCT_HOSTS
        .iter()
        .any(|allowed| lower.as_str() == *allowed)
    {
        return true;
    }
    match lower.strip_suffix(BOT_SUBDOMAIN_SUFFIX) {
        Some(label) => {
            !label.is_empty()
                && !label.contains('.')
                && label
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        }
        None => false,
    }
}

/// Pure allowlist decision over a normalised origin. `extra` is the parsed
/// `GB_MEDIA_ORIGINS` list; `debug` makes the plain-HTTP dev origin explicit so
/// the policy is testable in either build profile.
fn origin_allowed(origin: &str, extra: &[String], debug: bool) -> bool {
    if extra
        .iter()
        .any(|allowed| allowed.eq_ignore_ascii_case(origin))
        || BUNDLED_ORIGINS
            .iter()
            .any(|allowed| origin.eq_ignore_ascii_case(allowed))
    {
        return true;
    }
    let Some((scheme, authority)) = origin.split_once("://") else {
        return false;
    };
    let (host, port) = split_authority(authority);
    let local = host.eq_ignore_ascii_case("localhost");
    match (scheme, local) {
        ("https", true) => port.is_none() || port.is_some_and(|value| LOCAL_PORTS.contains(&value)),
        ("http", true) => debug && port.is_some_and(|value| LOCAL_PORTS.contains(&value)),
        ("https", false) => product_host_allowed(host) && (port.is_none() || port == Some(443)),
        _ => false,
    }
}

/// The whole permission decision, independent of WebKit so it can be tested
/// offline. Device type is checked first: a harmless permission never grants.
fn decide(
    page_uri: Option<&str>,
    audio: bool,
    video: bool,
    extra: &[String],
    debug: bool,
) -> MediaDecision {
    if !audio && !video {
        return MediaDecision::DenyNoDevice;
    }
    let Some(page_uri) = page_uri else {
        return MediaDecision::DenyUnparseable;
    };
    let Some(origin) = origin_of(page_uri) else {
        return MediaDecision::DenyUnparseable;
    };
    if origin_allowed(origin, extra, debug) {
        MediaDecision::Grant
    } else {
        MediaDecision::DenyForeign
    }
}

/// #2698 — geolocation and notifications are sensitive permissions. They are
/// granted only to the shell's own trusted local UI: the bundled custom-scheme
/// origins, plus the Tauri dev server in debug builds. The loopback sidecar
/// (`http(s)://localhost:8080` / `127.0.0.1:8080`) serves remote, possibly
/// model-generated content, so it is explicitly untrusted here even though it
/// is local on the network. Unlike media, the `GB_MEDIA_ORIGINS` override never
/// widens this allowlist.
fn sensitive_permission_allowed(page_uri: Option<&str>, debug: bool) -> bool {
    let Some(origin) = page_uri.and_then(origin_of) else {
        return false;
    };
    if BUNDLED_ORIGINS
        .iter()
        .any(|allowed| origin.eq_ignore_ascii_case(allowed))
    {
        return true;
    }
    debug && origin.eq_ignore_ascii_case(DEV_ORIGIN)
}

/// Parses the optional operator override into exact normalised origins. An
/// empty, malformed or IP-literal entry is simply ignored (fail-closed).
fn configured_extra_origins() -> Vec<String> {
    let Ok(raw) = std::env::var(MEDIA_ORIGINS_ENV) else {
        return Vec::new();
    };
    raw.split(',')
        .filter_map(|entry| {
            let origin = origin_of(entry.trim())?;
            let (_, authority) = origin.split_once("://")?;
            let (host, _) = split_authority(authority);
            if host.parse::<std::net::IpAddr>().is_ok() {
                return None;
            }
            Some(origin.to_string())
        })
        .collect()
}

/// Replaces an IP-literal host with `<ip>` before it is written to the log or
/// the indicator event (AGENTS.md: no internal IPs in logs). Named hosts,
/// including `localhost`, are kept because they are the useful audit trail.
fn redacted(origin: &str) -> String {
    let Some((scheme, authority)) = origin.split_once("://") else {
        return "<unknown>".to_string();
    };
    let host = if let Some(rest) = authority.strip_prefix('[') {
        rest.split(']').next().unwrap_or(rest)
    } else {
        authority.split(':').next().unwrap_or(authority)
    };
    if host.parse::<std::net::IpAddr>().is_ok() {
        format!("{scheme}://<ip>")
    } else {
        origin.to_string()
    }
}

/// Publishes a grant to the shell UI without adding any localised text to the
/// WebKit handler. The consumer owns the visible badge and its translations.
fn publish_grant<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    origin: &str,
    audio: bool,
    video: bool,
) {
    let payload = json!({ "origin": origin, "audio": audio, "video": video });
    if let Err(e) = app.emit(MEDIA_GRANT_EVENT, payload) {
        log::warn!("{PLUGIN_NAME}: could not publish the media grant indicator: {e}");
    }
}

/// Answers the sensitive (geolocation / notification) permission requests the
/// WebView raises. Returns `true` only when this handler actually decided the
/// request; unknown request kinds return `false` so WebKit keeps its own
/// handlers. The decision itself is the pure `sensitive_permission_allowed`.
fn handle_sensitive_permission(page_uri: Option<&str>, request: &PermissionRequest) -> bool {
    let kind = if request
        .downcast_ref::<GeolocationPermissionRequest>()
        .is_some()
    {
        "geolocation"
    } else if request
        .downcast_ref::<NotificationPermissionRequest>()
        .is_some()
    {
        "notification"
    } else {
        return false;
    };

    let origin = page_uri.and_then(origin_of).unwrap_or("<unknown>");
    if sensitive_permission_allowed(page_uri, cfg!(debug_assertions)) {
        request.allow();
        log::info!(
            "{PLUGIN_NAME}: AUDIT sensitive-allow kind={kind} origin={}",
            redacted(origin)
        );
    } else {
        request.deny();
        log::warn!(
            "{PLUGIN_NAME}: AUDIT sensitive-deny kind={kind} origin={} reason=untrusted",
            redacted(origin)
        );
    }
    true
}

/// Builds the plugin that enables MediaStream/WebRTC and answers the
/// user-media and sensitive permission requests raised by the desktop WebView.
pub fn plugin<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    // #2502 — this hook is registered before Tauri creates the first WebView,
    // so scrubbing here means WebKit and all of its child processes inherit an
    // isolated session bus instead of the appliance's full user bus.
    super::dbus_isolation::isolate_webview_session_bus();

    tauri::plugin::Builder::new(PLUGIN_NAME)
        .on_webview_ready(|webview| {
            let app = webview.app_handle().clone();
            let result = webview.with_webview(|platform| {
                let wk = platform.inner();

                match wk.settings() {
                    Some(settings) => {
                        settings.set_enable_media_stream(true);
                        // Enabling WebRTC also enables MediaStream; keep both
                        // explicit so live voice and Meet keep working even if
                        // WebKit changes one of the defaults.
                        settings.set_enable_webrtc(true);
                        log::info!("{PLUGIN_NAME}: media-stream and webrtc enabled");
                    }
                    None => log::error!(
                        "{PLUGIN_NAME}: no WebView settings — camera and microphone will fail"
                    ),
                }

                wk.connect_permission_request(move |wv, request| {
                    let page_uri = wv.uri();

                    let Some(media) = request.downcast_ref::<UserMediaPermissionRequest>() else {
                        return handle_sensitive_permission(page_uri.as_deref(), request);
                    };

                    let audio = media.is_for_audio_device();
                    let video = media.is_for_video_device();
                    let extra = configured_extra_origins();
                    let decision = decide(
                        page_uri.as_deref(),
                        audio,
                        video,
                        &extra,
                        cfg!(debug_assertions),
                    );
                    let origin = page_uri
                        .as_deref()
                        .and_then(origin_of)
                        .unwrap_or("<unknown>");

                    match decision {
                        MediaDecision::DenyNoDevice => {
                            media.deny();
                            log::warn!(
                                "{PLUGIN_NAME}: AUDIT media-deny origin={origin} reason=no-device audio={audio} video={video}"
                            );
                        }
                        MediaDecision::DenyUnparseable => {
                            media.deny();
                            log::warn!(
                                "{PLUGIN_NAME}: AUDIT media-deny origin=<unknown> reason=unparseable audio={audio} video={video}"
                            );
                        }
                        MediaDecision::DenyForeign => {
                            media.deny();
                            log::warn!(
                                "{PLUGIN_NAME}: AUDIT media-deny origin={} reason=foreign audio={audio} video={video}",
                                redacted(origin)
                            );
                        }
                        MediaDecision::Grant => {
                            // #2504 — a Settings → Privacy revoke wins over
                            // the allowlist: no origin gets the device.
                            if revoked(audio, video) {
                                media.deny();
                                log::warn!(
                                    "{PLUGIN_NAME}: AUDIT media-deny origin={} reason=revoked audio={audio} video={video}",
                                    redacted(origin)
                                );
                            } else {
                                media.allow();
                                let safe_origin = redacted(origin);
                                log::info!(
                                    "{PLUGIN_NAME}: AUDIT media-grant origin={safe_origin} audio={audio} video={video}"
                                );
                                publish_grant(&app, &safe_origin, audio, video);
                            }
                        }
                    }
                    true
                });
            });

            if let Err(e) = result {
                log::error!("{PLUGIN_NAME}: could not configure the WebView: {e}");
            }
        })
        .build()
}

#[cfg(test)]
mod tests;
