//! Topmost splash window shown while the server stack boots.
//!
//! Follows the Tauri splash-screen pattern: a small, undecorated, always-on-top
//! window that is created at startup and closed once the suite (or the setup
//! wizard) is ready to take over.

use log::{info, warn};
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

/// Shows the splash window, creating it on demand when the configured window is
/// not already present (for example in a development run).
pub fn show(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("splash") {
        let _ = window.show();
        let _ = window.set_focus();
        let _ = window.set_always_on_top(true);
        return;
    }

    let builder = WebviewWindowBuilder::new(app, "splash", WebviewUrl::App("splash.html".into()))
        .title("General Bots")
        .inner_size(420.0, 420.0)
        .min_inner_size(320.0, 320.0)
        .max_inner_size(480.0, 480.0)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .center()
        .visible(true);

    match builder.build() {
        Ok(_) => info!("splash: window created"),
        Err(error) => warn!("splash: could not create the window: {error}"),
    }
}

/// Hides (closes) the splash window once the real UI is ready.
pub fn hide(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("splash") {
        if let Err(error) = window.close() {
            warn!("splash: could not close the window: {error}");
        } else {
            info!("splash: window closed");
        }
    }
}
