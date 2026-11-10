//! Wizard window creation, sizing and the handoff to the suite window.

use log::{info, warn};
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

use super::session::{suite_url_with_token, wait_for_desktop_session};

/// Whether the Tauri `devUrl` (the dev server, usually port 3000) should be
/// used instead of the embedded frontend.
///
/// Only a debug build that does **not** embed the frontend may follow `devUrl`.
/// A `custom-protocol` build carries the bundled assets, so even a debug binary
/// built with it must serve them directly — otherwise it opens whatever
/// unrelated server happens to hold port 3000 on the developer's machine.
fn use_dev_server() -> bool {
    cfg!(debug_assertions) && !cfg!(feature = "custom-protocol")
}

/// Design size of the wizard window and the margin kept around it.
const WIZARD_WIDTH: f64 = 1040.0;
const WIZARD_HEIGHT: f64 = 700.0;
const WIZARD_MARGIN: f64 = 80.0;
const WIZARD_MIN_WIDTH: f64 = 640.0;
const WIZARD_MIN_HEIGHT: f64 = 480.0;

/// Logical size of a display described in physical pixels.
fn logical_size(width: u32, height: u32, scale: f64) -> (f64, f64) {
    let scale = if scale > 0.0 { scale } else { 1.0 };
    (f64::from(width) / scale, f64::from(height) / scale)
}

/// Wizard window size: the design size, shrunk to leave a margin on a smaller
/// display. A display smaller than the minimum keeps the minimum so the
/// builder never receives a zero or negative size.
fn wizard_size(available_width: f64, available_height: f64) -> (f64, f64) {
    if !available_width.is_finite() || !available_height.is_finite() {
        return (WIZARD_WIDTH, WIZARD_HEIGHT);
    }
    (
        (available_width - WIZARD_MARGIN).clamp(WIZARD_MIN_WIDTH, WIZARD_WIDTH),
        (available_height - WIZARD_MARGIN).clamp(WIZARD_MIN_HEIGHT, WIZARD_HEIGHT),
    )
}

/// The assistant window size for a display of `available_width` x
/// `available_height` (logical pixels). The configured 1200x800 does not fit a
/// 1366x768 laptop once the panel and decorations are drawn, and a window
/// taller than the screen hides the composer and the send button.
fn fitted_size(available_width: f64, available_height: f64) -> (f64, f64) {
    if !available_width.is_finite() || !available_height.is_finite() {
        return (1200.0, 800.0);
    }
    (
        1200.0_f64.min(available_width - 40.0).max(900.0),
        800.0_f64.min(available_height - 80.0).max(560.0),
    )
}

/// Creates the wizard window and keeps the main window hidden.
pub fn open_window(app: &tauri::AppHandle) -> Result<(), String> {
    if let Some(existing) = app.get_webview_window("wizard") {
        let _ = existing.show();
        let _ = existing.set_focus();
        return Ok(());
    }

    let (mut width, mut height) = (WIZARD_WIDTH, WIZARD_HEIGHT);
    if let Ok(Some(monitor)) = app.primary_monitor() {
        let (available_width, available_height) = logical_size(
            monitor.size().width,
            monitor.size().height,
            monitor.scale_factor(),
        );
        (width, height) = wizard_size(available_width, available_height);
    }

    // In development the wizard is served by botui (devUrl). WebKit may cache
    // the previous page between runs, so force a fresh document each start.
    // Embedded (`custom-protocol`) builds always use the bundled asset.
    let wizard_url = if use_dev_server() {
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

    let result = builder
        .build()
        .map(|_| ())
        .map_err(|e| format!("could not open the setup window: {e}"));
    if result.is_ok() {
        crate::desktop::splash::hide(app);
    }
    result
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
            // release or embedded build — that would open localhost:3000,
            // which does not belong to this install.
            if !use_dev_server() {
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
        crate::desktop::splash::hide(app);
        return;
    };

    // #1919 — WebKitGTK does not realise a hidden webview, so navigate() on
    // a window created with visible:false never changes the URL. Reveal the
    // window first (the always-on-top splash stays above it) before navigating.
    let _ = main.show();
    let url = suite_url_with_token(url, token);
    if let Err(e) = main.navigate(url.clone()) {
        warn!("could not navigate the suite window: {e}");
        let _ = main.show();
        let _ = main.set_focus();
        crate::desktop::splash::raise(app);
        crate::desktop::splash::watch_boot_ready(&app.clone());
        return;
    }

    // #1890 — on a cold boot the window still holds the bundled
    // `tauri://localhost` document while the navigation is in flight, and that
    // page fires relative `/api/*` requests that resolve against the asset
    // protocol (`error manager:asset not found: api/...`). Showing the window
    // right away put the user in front of that half-booted page. Reveal it only
    // once the webview actually reports the sidecar/dev origin, with a deadline
    // so a wedged navigation can never trap the user behind the splash.
    let splash_app = app.clone();
    let window = main.clone();
    let expected = url.clone();
    tauri::async_runtime::spawn(async move {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(8);
        loop {
            let landed = window
                .url()
                .map(|current| {
                    current.host_str() == expected.host_str()
                        && current.port_or_known_default() == expected.port_or_known_default()
                })
                .unwrap_or(false);
            if landed || std::time::Instant::now() >= deadline {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(150)).await;
        }
        let _ = window.show();
        let _ = window.set_focus();
        // #1830 — the suite is still booting (product config, catalog, default
        // app). Keep the splash on top of it and let the `gb-boot-ready`
        // fragment decide when the splash closes.
        crate::desktop::splash::raise(&splash_app);
        crate::desktop::splash::watch_boot_ready(&splash_app.clone());
    });
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
                if !use_dev_server() {
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
            Ok(_) => {
                info!("suite window recreated after it was destroyed");
                // #1830 — same handoff as reveal_suite: the recreated window
                // has to boot before anything is on screen.
                crate::desktop::splash::raise(&handle);
                crate::desktop::splash::watch_boot_ready(&handle.clone());
            }
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
    let (available_width, available_height) = logical_size(
        monitor.size().width,
        monitor.size().height,
        monitor.scale_factor(),
    );
    let (width, height) = fitted_size(available_width, available_height);
    if let Err(e) = window.set_size(tauri::LogicalSize::new(width, height)) {
        warn!("could not fit the assistant window to the screen: {e}");
    }
    if let Err(e) = window.center() {
        warn!("could not centre the assistant window: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::{fitted_size, logical_size, wizard_size};

    #[test]
    fn physical_pixels_are_divided_by_the_scale_factor() {
        assert_eq!(logical_size(1920, 1080, 2.0), (960.0, 540.0));
        assert_eq!(logical_size(1920, 1080, 1.0), (1920.0, 1080.0));
        // A zero or negative factor is not a usable scale; treat it as 1:1.
        assert_eq!(logical_size(1920, 1080, 0.0), (1920.0, 1080.0));
        assert_eq!(logical_size(1920, 1080, -1.0), (1920.0, 1080.0));
    }

    #[test]
    fn the_wizard_keeps_its_design_size_on_a_large_display() {
        assert_eq!(wizard_size(2560.0, 1440.0), (1040.0, 700.0));
    }

    #[test]
    fn the_wizard_shrinks_to_leave_a_margin_but_never_below_the_minimum() {
        // 1366x768: 80px margin leaves 1286x688, so only the height shrinks.
        assert_eq!(wizard_size(1366.0, 768.0), (1040.0, 688.0));
        // A tiny display keeps the minimum instead of a negative size.
        assert_eq!(wizard_size(320.0, 240.0), (640.0, 480.0));
        assert_eq!(wizard_size(0.0, 0.0), (640.0, 480.0));
        assert_eq!(wizard_size(f64::NAN, 800.0), (1040.0, 700.0));
    }

    #[test]
    fn the_assistant_window_fits_the_display() {
        assert_eq!(fitted_size(3840.0, 2160.0), (1200.0, 800.0));
        // 1366x768 laptop: the panel leaves 688 logical pixels of height.
        assert_eq!(fitted_size(1366.0, 768.0), (1200.0, 688.0));
        // 1024x600 netbook: both dimensions are clamped to the minimum.
        assert_eq!(fitted_size(1024.0, 600.0), (984.0, 560.0));
        assert_eq!(fitted_size(f64::NAN, f64::INFINITY), (1200.0, 800.0));
    }
}
