//! Grants the local app access to the microphone and camera on WebKitGTK.
//!
//! wry keeps WebKitGTK's MediaStream/WebRTC settings disabled by default, and
//! WebKitGTK never shows a browser-style permission prompt. Without both the
//! settings and the permission-request handler, `getUserMedia()` rejects with
//! `NotAllowedError` and the chat mic/camera/live-voice buttons always fail
//! (#1898). A handler that never answers leaves the request pending forever,
//! which the chat camera reports as a black preview (#2106), so every decision
//! is logged: the desktop log then says whether the WebView even asked.

use webkit2gtk::glib::prelude::Cast;
use webkit2gtk::{
    PermissionRequestExt, SettingsExt, UserMediaPermissionRequest, UserMediaPermissionRequestExt,
    WebViewExt,
};

/// Name of the plugin, also the prefix of its log lines.
const PLUGIN_NAME: &str = "generalbots-webkit-permissions";

/// Hosts the shell owns and may therefore capture from (#2504).
///
/// WebKitGTK has no browser-style permission prompt, so the grant has to be
/// decided here. Before this list existed every origin got the microphone and
/// camera silently: a mail body, a marketplace app or LLM-rendered HTML could
/// open the laptop's mic with no indicator. The shell's own pages are served
/// from loopback in both the bundled and the sidecar deployments; anything
/// else is denied unless the operator names it in `GB_MEDIA_ORIGINS`
/// (comma-separated hosts, for a hosted suite on a LAN).
const OWNED_HOSTS: &[&str] = &[
    "localhost",
    "127.0.0.1",
    "[::1]",
    "::1",
    "tauri.localhost",
];

/// Extracts the host from a page URI.
fn host_of(uri: &str) -> Option<&str> {
    let rest = uri.split_once("://")?.1;
    let authority = rest.split(['/', '?', '#']).next().unwrap_or(rest);
    let without_userinfo = authority.rsplit('@').next().unwrap_or(authority);
    Some(without_userinfo)
}

/// Whether `uri` is an origin the shell owns.
fn origin_allowed(uri: &str) -> bool {
    let Some(authority) = host_of(uri) else {
        return false;
    };
    // The bundled UI runs from `tauri://localhost` and `http://tauri.localhost`,
    // where the authority is a bare host; a loopback page carries a port.
    let host = if let Some(rest) = authority.strip_prefix('[') {
        rest.split(']').next().unwrap_or(rest)
    } else if authority.matches(':').count() == 1 {
        authority.split(':').next().unwrap_or(authority)
    } else {
        authority
    };
    let host = host.trim();
    if OWNED_HOSTS
        .iter()
        .any(|owned| host.eq_ignore_ascii_case(owned.trim_matches(['[', ']'])))
    {
        return true;
    }
    // RFC 6761 reserves the whole .localhost tree for the loopback interface.
    if host.to_ascii_lowercase().ends_with(".localhost") {
        return true;
    }
    std::env::var("GB_MEDIA_ORIGINS")
        .map(|list| {
            list.split(',').any(|entry| {
                let entry = entry.trim();
                !entry.is_empty() && (host.eq_ignore_ascii_case(entry) || authority.eq_ignore_ascii_case(entry))
            })
        })
        .unwrap_or(false)
}

/// Builds the plugin that enables MediaStream/WebRTC and answers the
/// user-media permission requests raised by the desktop WebView.
pub fn plugin<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
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
                        log::warn!("{PLUGIN_NAME}: denied user media (no audio/video device)");
                        return true;
                    }

                    // #2504 — the grant is per origin, not global. The URI of
                    // the page that asked decides; an unknown page is denied
                    // and the origin is logged so the audit trail shows who
                    // tried to open the microphone.
                    let uri = wv.uri().map(|value| value.to_string()).unwrap_or_default();
                    if !origin_allowed(&uri) {
                        media.deny();
                        log::warn!(
                            "{PLUGIN_NAME}: denied media from untrusted origin {uri:?} (audio={audio}, video={video})"
                        );
                        return true;
                    }
                    media.allow();
                    log::info!(
                        "{PLUGIN_NAME}: allowed media for {uri} (audio={audio}, video={video})"
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
    use super::{host_of, origin_allowed};

    #[test]
    fn bundled_and_loopback_origins_are_owned() {
        assert!(origin_allowed("tauri://localhost/index.html"));
        assert!(origin_allowed("http://tauri.localhost/suite/desktop.html"));
        assert!(origin_allowed("http://localhost:3000/chat"));
        assert!(origin_allowed("http://127.0.0.1:8080/api/product"));
        assert!(origin_allowed("http://[::1]:3000/"));
    }

    #[test]
    fn remote_origins_are_refused() {
        assert!(!origin_allowed("https://mail.example.com/inbox"));
        assert!(!origin_allowed("https://evil.example/"));
        assert!(!origin_allowed(""));
        assert!(!origin_allowed("file:///var/lib/gbdesk/index.html"));
    }

    #[test]
    fn host_parse_ignores_port_and_userinfo() {
        assert_eq!(host_of("http://user@127.0.0.1:3000/x"), Some("127.0.0.1:3000"));
        assert_eq!(host_of("not a uri"), None);
    }
}
