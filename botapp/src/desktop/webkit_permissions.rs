//! Grants the local app access to the microphone and camera on WebKitGTK.
//!
//! wry keeps WebKitGTK's MediaStream/WebRTC settings disabled by default, and
//! WebKitGTK never shows a browser-style permission prompt. Without both the
//! settings and the permission-request handler, `getUserMedia()` rejects with
//! `NotAllowedError` and the chat mic/camera/live-voice buttons always fail
//! (#1898). A handler that never answers leaves the request pending forever,
//! which the chat camera reports as a black preview (#2106), so every decision
//! is logged: the desktop log then says whether the WebView even asked.
//!
//! #2504 — WebKitGTK 2.0 does not expose the requesting security origin on a
//! `UserMediaPermissionRequest`. In `webkit2gtk` 2.0.2 the request only
//! implements `PermissionRequest` (`allow`/`deny`) plus the
//! `is-for-audio-device` / `is-for-video-device` properties; the
//! `permission-request` signal carries just the WebView and the request. There
//! is no `security_origin()`, `uri()` or parent-frame getter on it. The
//! strictest available check is therefore the WebView's top-level URI
//! (`WebViewExt::uri()`). A foreign origin embedded in an iframe of an owned
//! page cannot be distinguished with this API: `Permissions-Policy` on the
//! served pages is the complementary defence.

use webkit2gtk::glib::prelude::Cast;
use webkit2gtk::{
    PermissionRequestExt, SettingsExt, UserMediaPermissionRequest, UserMediaPermissionRequestExt,
    WebViewExt,
};

/// Name of the plugin, also the prefix of its log lines.
const PLUGIN_NAME: &str = "generalbots-webkit-permissions";

/// The only origins the shell owns in a release build (#2504).
///
/// Tauri serves the bundled UI from `tauri://localhost` on Linux/Windows and
/// `https://tauri.localhost` on macOS. Everything else — mail bodies,
/// marketplace apps, LLM-rendered HTML, arbitrary links — is denied.
const OWNED_ORIGINS: &[&str] = &["tauri://localhost", "https://tauri.localhost"];

/// Tauri's dev server (`devUrl` in `tauri.conf.json`), trusted only in debug
/// builds; a release binary must never grant media to a plain-HTTP origin.
const DEV_ORIGIN: &str = "http://localhost:3000";

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

/// Whether an origin (as returned by [`origin_of`]) belongs to the shell.
///
/// A helper over a plain string keeps the decision unit-testable without a
/// live WebView. It takes an already-normalized origin, not a full page URI.
fn origin_allowed(origin: &str) -> bool {
    let owned = OWNED_ORIGINS
        .iter()
        .any(|allowed| origin.eq_ignore_ascii_case(allowed));
    owned || (cfg!(debug_assertions) && origin.eq_ignore_ascii_case(DEV_ORIGIN))
}

/// Replaces an IP-literal host with `<ip>` before it is written to the log
/// (AGENTS.md: no internal IPs in logs). Named hosts, including `localhost`,
/// are kept because they are the useful part of the audit trail.
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

/// Builds the plugin that enables MediaStream/WebRTC and answers the
/// user-media permission requests raised by the desktop WebView.
pub fn plugin<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    // #2502 — this hook is registered before Tauri creates the first WebView,
    // so scrubbing here means WebKit and all of its child processes inherit an
    // isolated session bus instead of the appliance's full user bus.
    super::dbus_isolation::isolate_webview_session_bus();

    tauri::plugin::Builder::new(PLUGIN_NAME)
        .on_webview_ready(|webview| {
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

                wk.connect_permission_request(|wv, request| {
                    let Some(media) = request.downcast_ref::<UserMediaPermissionRequest>() else {
                        // Leave geolocation, notifications and other requests
                        // to their own handlers.
                        return false;
                    };

                    let audio = media.is_for_audio_device();
                    let video = media.is_for_video_device();
                    if !audio && !video {
                        media.deny();
                        log::warn!(
                            "{PLUGIN_NAME}: AUDIT media-deny origin=<unknown> reason=no-device audio={audio} video={video}"
                        );
                        return true;
                    }

                    // #2504 — WebKit cannot tell us the request's own origin,
                    // so use the top-level page URI and never log the full URI
                    // (its query string may carry a token).
                    let page_uri = wv.uri().map(|value| value.to_string()).unwrap_or_default();
                    let Some(origin) = origin_of(&page_uri) else {
                        media.deny();
                        log::warn!(
                            "{PLUGIN_NAME}: AUDIT media-deny origin=<unknown> reason=unparseable audio={audio} video={video}"
                        );
                        return true;
                    };
                    if !origin_allowed(origin) {
                        media.deny();
                        log::warn!(
                            "{PLUGIN_NAME}: AUDIT media-deny origin={} reason=foreign audio={audio} video={video}",
                            redacted(origin)
                        );
                        return true;
                    }
                    media.allow();
                    log::info!(
                        "{PLUGIN_NAME}: AUDIT media-grant origin={origin} audio={audio} video={video}"
                    );
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
mod tests {
    use super::{origin_allowed, origin_of, redacted, DEV_ORIGIN, OWNED_ORIGINS};

    #[test]
    fn owned_origins_are_allowed() {
        for origin in OWNED_ORIGINS {
            assert!(
                origin_allowed(origin),
                "owned origin {origin} must be allowed"
            );
            assert!(
                origin_allowed(&origin.to_ascii_uppercase()),
                "origin comparison must be case-insensitive"
            );
        }
    }

    #[test]
    fn dev_origin_is_trusted_only_in_debug_builds() {
        assert_eq!(origin_allowed(DEV_ORIGIN), cfg!(debug_assertions));
    }

    #[test]
    fn foreign_origins_are_denied() {
        let foreign = [
            "https://mail.example.com",
            "https://evil.example",
            "https://tauri.localhost.evil.example",
            "tauri://evil.example",
            "http://localhost",
            "http://localhost:3001",
            "http://127.0.0.1:8080",
            "file://",
            "",
        ];
        for origin in foreign {
            assert!(
                !origin_allowed(origin),
                "foreign origin {origin} must be denied"
            );
        }
    }

    #[test]
    fn origin_of_normalizes_page_uris() {
        assert_eq!(
            origin_of("tauri://localhost/suite/desktop.html"),
            Some("tauri://localhost")
        );
        assert_eq!(
            origin_of("https://tauri.localhost/suite/desktop.html?q=x#frag"),
            Some("https://tauri.localhost")
        );
        assert_eq!(origin_of("http://user:secret@localhost:3000/"), None);
        assert_eq!(origin_of("not a uri"), None);
    }

    #[test]
    fn ip_literals_are_masked_in_logs() {
        assert_eq!(redacted("http://10.0.0.1:8080"), "http://<ip>");
        assert_eq!(redacted("https://[::1]:3000"), "https://<ip>");
        assert_eq!(
            redacted("https://mail.example.com"),
            "https://mail.example.com"
        );
    }
}
