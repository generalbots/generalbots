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

                wk.connect_permission_request(|_wv, request| {
                    let Some(media) = request.downcast_ref::<UserMediaPermissionRequest>() else {
                        // Leave geolocation, notifications and other requests
                        // to their own handlers.
                        return false;
                    };

                    let audio = media.is_for_audio_device();
                    let video = media.is_for_video_device();
                    if audio || video {
                        media.allow();
                        log::info!("{PLUGIN_NAME}: allowed media (audio={audio}, video={video})");
                    } else {
                        media.deny();
                        log::warn!("{PLUGIN_NAME}: denied user media (no audio/video device)");
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
