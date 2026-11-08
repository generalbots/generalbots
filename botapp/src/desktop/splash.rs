//! Topmost splash window shown while the server stack boots.
//!
//! Follows the Tauri splash-screen pattern: a small, undecorated, always-on-top
//! window that is created at startup and closed once the suite (or the setup
//! wizard) is ready to take over.

use log::{info, warn};
use serde_json::json;
use std::time::{Duration, Instant};
use tauri::{Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

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

/// Keeps the splash above the freshly revealed suite window.
///
/// #1830 — the main window opens while the suite is still booting, so a plain
/// reveal would leave the main window covering (and hiding) the splash. This
/// re-asserts the always-on-top flag and pulls focus; a splash that was closed
/// earlier stays closed.
pub fn raise(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("splash") {
        let _ = window.show();
        let _ = window.set_always_on_top(true);
        let _ = window.set_focus();
    }
}

/// Fragment the suite stamps into its URL when boot is complete.
const BOOT_READY_FRAGMENT: &str = "gb-boot-ready";
const BOOT_WATCH_POLL: Duration = Duration::from_millis(400);
const BOOT_WATCH_DEADLINE: Duration = Duration::from_millis(45_000);

fn push_progress(splash: &WebviewWindow, value: u8, text: &str) {
    let script = format!(
        "if(typeof window.gbSplashProgress==='function')window.gbSplashProgress({},{});",
        value,
        json!(text)
    );
    let _ = splash.eval(&script);
}

/// Watches the suite window and closes the splash only when boot truly ends.
///
/// The suite page is served from its own origin and every Tauri IPC permission
/// is denied to it (the product ships with an empty capability set), so it
/// cannot invoke a command. Instead the suite marks the end of boot by
/// stamping the `gb-boot-ready` fragment into its URL (via
/// `history.replaceState`), which `url()` reports on every engine. The
/// deadline is the safety net for a broken boot: the splash must never trap
/// the user in front of a dead window.
pub fn watch_boot_ready(app: &tauri::AppHandle) {
    let handle = app.clone();
    std::thread::spawn(move || {
        let deadline = Instant::now() + BOOT_WATCH_DEADLINE;
        let mut saw_main = false;
        loop {
            let Some(splash) = handle.get_webview_window("splash") else {
                // The splash was closed somewhere else (wizard, restart flow).
                return;
            };
            if let Some(main) = handle.get_webview_window("main") {
                if !saw_main {
                    saw_main = true;
                    push_progress(&splash, 70, "Loading interface…");
                }
                if let Ok(url) = main.url() {
                    if url.as_str().contains(BOOT_READY_FRAGMENT) {
                        info!("splash: boot-ready fragment seen; closing");
                        push_progress(&splash, 100, "Ready");
                        std::thread::sleep(Duration::from_millis(120));
                        hide(&handle);
                        return;
                    }
                }
            } else if !saw_main {
                push_progress(&splash, 15, "Starting services…");
            }
            if Instant::now() >= deadline {
                warn!("splash: boot-ready fragment never arrived; hiding the splash");
                push_progress(&splash, 100, "Opening workspace…");
                std::thread::sleep(Duration::from_millis(120));
                hide(&handle);
                return;
            }
            std::thread::sleep(BOOT_WATCH_POLL);
        }
    });
}
