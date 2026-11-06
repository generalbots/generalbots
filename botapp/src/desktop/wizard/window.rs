//! Wizard window creation, sizing and the handoff to the suite window.

use log::{info, warn};
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

use super::session::{suite_url_with_token, wait_for_desktop_session};

/// Creates the wizard window and keeps the main window hidden.
pub fn open_window(app: &tauri::AppHandle) -> Result<(), String> {
    if let Some(existing) = app.get_webview_window("wizard") {
        let _ = existing.show();
        let _ = existing.set_focus();
        return Ok(());
    }

    let mut width = 1040.0_f64;
    let mut height = 700.0_f64;
    if let Ok(Some(monitor)) = app.primary_monitor() {
        let scale = if monitor.scale_factor() > 0.0 {
            monitor.scale_factor()
        } else {
            1.0
        };
        let available_width = f64::from(monitor.size().width) / scale;
        let available_height = f64::from(monitor.size().height) / scale;
        width = width.min(available_width - 80.0);
        height = height.min(available_height - 80.0);
    }

    // In development the wizard is served by botui (devUrl). WebKit may cache
    // the previous page between runs, so force a fresh document each start.
    let wizard_url = if cfg!(debug_assertions) {
        match &app.config().build.dev_url {
            Some(dev) => {
                let mut url = dev.clone();
                url.set_path("/wizard/index.html");
                let stamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                url.set_query(Some(&format!("run={stamp}")));
                WebviewUrl::External(url)
            }
            None => WebviewUrl::App("wizard/index.html".into()),
        }
    } else {
        WebviewUrl::App("wizard/index.html".into())
    };

    info!("wizard: opening {wizard_url:?}");
    let builder = WebviewWindowBuilder::new(app, "wizard", wizard_url)
        .title("Set up General Bots")
        .inner_size(width, height)
        .min_inner_size(900.0, 560.0)
        .resizable(true)
        .center()
        .visible(true);

    builder
        .build()
        .map(|_| ())
        .map_err(|e| format!("could not open the setup window: {e}"))
}

/// Default entry point for packaged desktop builds: the sidecar serves both the
/// suite and `/api`, so the page and the API share one origin.
fn sidecar_suite_url() -> Option<tauri::Url> {
    tauri::Url::parse("http://127.0.0.1:8080/suite/desktop.html").ok()
}

/// Re-navigates the hidden suite window to its entry point and reveals it.
///
/// The window is created at startup, before the API answers, so it may have
/// been redirected to the login page by the auth gate. Navigating again (rather
/// than reloading) guarantees the suite boots with the final product config and
/// a freshly minted session.
pub fn reveal_suite(app: &tauri::AppHandle) {
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let token = wait_for_desktop_session(15).await;
        reveal_suite_with_token(&handle, token.as_deref());
    });
}

fn reveal_suite_with_token(app: &tauri::AppHandle, token: Option<&str>) {
    let Some(main) = app.get_webview_window("main") else {
        warn!("main window is not available during wizard handoff");
        return;
    };
    fit_to_screen(&main);

    let target = std::env::var("GB_DESKTOP_SUITE_URL")
        .ok()
        .and_then(|value| tauri::Url::parse(value.trim()).ok())
        .or_else(|| {
            // Development runs are served by botui (devUrl); packaged builds
            // are served by the sidecar itself. Never follow devUrl in a
            // release binary — that would open localhost:3000, which does not
            // exist next to a release install.
            if !cfg!(debug_assertions) {
                return None;
            }
            app.config().build.dev_url.clone().map(|mut url| {
                url.set_path("/suite/desktop.html");
                url.set_query(None);
                url
            })
        })
        .or_else(sidecar_suite_url);

    let Some(url) = target else {
        warn!("no suite URL is configured");
        let _ = main.show();
        return;
    };

    let url = suite_url_with_token(url, token);
    if let Err(e) = main.navigate(url) {
        warn!("could not navigate the suite window: {e}");
    }
    let _ = main.show();
    let _ = main.set_focus();
}

/// Recreates the suite window after it has been destroyed (#1620).
///
/// A reply can outlive the window (the user closes it, or the window manager
/// destroys it). Instead of terminating the process and killing the sidecar
/// mid-generation, the shell recreates the window on the same URL so the
/// result is visible when it arrives.
pub fn ensure_suite_window(app: &tauri::AppHandle) {
    if app.get_webview_window("main").is_some() {
        reveal_suite(app);
        return;
    }
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let token = wait_for_desktop_session(5).await;
        let target = std::env::var("GB_DESKTOP_SUITE_URL")
            .ok()
            .and_then(|value| tauri::Url::parse(value.trim()).ok())
            .or_else(|| {
                if !cfg!(debug_assertions) {
                    return None;
                }
                handle.config().build.dev_url.clone().map(|mut url| {
                    url.set_path("/suite/desktop.html");
                    url.set_query(None);
                    url
                })
            })
            .or_else(sidecar_suite_url);
        let Some(url) = target else {
            warn!("could not recreate the suite window: no suite URL is configured");
            return;
        };
        match WebviewWindowBuilder::new(
            &handle,
            "main",
            WebviewUrl::External(suite_url_with_token(url, token.as_deref())),
        )
        .title("General Bots")
        .inner_size(1340.0, 860.0)
        .min_inner_size(900.0, 560.0)
        .resizable(true)
        .build()
        {
            Ok(_) => info!("suite window recreated after it was destroyed"),
            Err(e) => warn!("could not recreate the suite window: {e}"),
        }
    });
}

/// Keep the assistant window inside the display. The configured 1200x800 does
/// not fit a 1366x768 laptop once the panel and decorations are drawn, and a
/// window taller than the screen hides the composer and the send button.
fn fit_to_screen(window: &tauri::WebviewWindow) {
    // Prefer the primary display: current_monitor() reports the (smaller)
    // monitor a partially offscreen window sits on, which shrank the window a
    // little more on every launch.
    let monitor = match window.primary_monitor() {
        Ok(Some(monitor)) => monitor,
        _ => match window.current_monitor() {
            Ok(Some(monitor)) => monitor,
            _ => return,
        },
    };
    let scale = if monitor.scale_factor() > 0.0 {
        monitor.scale_factor()
    } else {
        1.0
    };
    let available_width = f64::from(monitor.size().width) / scale;
    let available_height = f64::from(monitor.size().height) / scale;
    let width = 1200.0_f64.min(available_width - 40.0).max(900.0);
    let height = 800.0_f64.min(available_height - 80.0).max(560.0);
    if width <= 0.0 || height <= 0.0 {
        return;
    }
    if let Err(e) = window.set_size(tauri::LogicalSize::new(width, height)) {
        warn!("could not fit the assistant window to the screen: {e}");
    }
    if let Err(e) = window.center() {
        warn!("could not centre the assistant window: {e}");
    }
}
