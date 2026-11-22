//! Topmost splash window shown while the server stack boots.
//!
//! Follows the Tauri splash-screen pattern: a small, undecorated, always-on-top
//! window that is created at startup and closed once the suite (or the setup
//! wizard) is ready to take over.

use log::{info, warn};
use serde_json::json;
use std::time::{Duration, Instant};
use super::i18n;
use tauri::{Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

/// Whether the splash stays above every other window and takes the focus.
/// The government edition can spend minutes starting its services, and a
/// topmost window that cannot be closed gets in the way of everything else
/// on the computer, so there it is an ordinary window.
const TOPMOST: bool = !cfg!(feature = "gbgov");

/// Splash dimensions: the edition uses a wide animated hero (760x428);
/// the standard build keeps the square logo (420x420).
const SPLASH_WIDTH: f64 = if cfg!(feature = "gbgov") { 760.0 } else { 420.0 };
const SPLASH_HEIGHT: f64 = if cfg!(feature = "gbgov") { 428.0 } else { 420.0 };

/// Minimum time the splash stays on screen (milliseconds). The edition's hero
/// animation needs at least 6 s; overridable with `GB_SPLASH_MIN_MS`.
const DEFAULT_SPLASH_MIN_MS: u64 = if cfg!(feature = "gbgov") { 10000 } else { 0 };

/// Whether the app is running under the test rig. Windows are positioned far
/// off-screen so the WebView2 engine still renders (DevTools screenshots
/// capture the render tree) but nothing appears on the owner's desktop.
pub(crate) fn is_test_rig() -> bool {
    std::env::var("GB_TEST_RIG").as_deref() == Ok("1")
}

/// Moves a window far off-screen and hides it from the taskbar, so it renders
/// content without being visible.
pub(crate) fn park_offscreen(window: &WebviewWindow) {
    let _ = window.set_position(tauri::LogicalPosition::new(-10000.0, -10000.0));
    let _ = window.set_skip_taskbar(true);
    let _ = window.set_always_on_top(false);
}

/// Reads the minimum splash duration from the environment or the compile-time
/// default.
fn splash_min_duration() -> Duration {
    let ms = std::env::var("GB_SPLASH_MIN_MS")
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
        .unwrap_or(DEFAULT_SPLASH_MIN_MS);
    Duration::from_millis(ms)
}

/// Shows the splash window, creating it on demand when the configured window is
/// not already present (for example in a development run).
pub fn show(app: &tauri::AppHandle) {
    let window_created_at = Instant::now();
    let rig = is_test_rig();
    if let Some(window) = app.get_webview_window("splash") {
        // Test rig: park before showing, so the splash never appears on screen.
        if rig {
            park_offscreen(&window);
        }
        let _ = window.show();
        if !rig {
            if TOPMOST {
                let _ = window.set_focus();
            }
            let _ = window.set_always_on_top(TOPMOST);
        }
        // Resize to the edition dimensions (the conf declares 420x420).
        let _ = window.set_size(tauri::LogicalSize::new(SPLASH_WIDTH, SPLASH_HEIGHT));
        if !rig {
            let _ = window.center();
        }
        push_version(&window);
        watch_splash_painted(app, window_created_at);
        return;
    }

    let mut builder = WebviewWindowBuilder::new(app, "splash", WebviewUrl::App("splash.html".into()))
        .title("General Bots")
        .inner_size(SPLASH_WIDTH, SPLASH_HEIGHT)
        .min_inner_size(320.0, 320.0)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .skip_taskbar(true)
        .visible(true);
    if rig {
        builder = builder
            .always_on_top(false)
            .position(-10000.0, -10000.0);
    } else {
        builder = builder
            .always_on_top(TOPMOST)
            .center();
    }

    match builder.build() {
        Ok(win) => {
            push_version(&win);
            info!("splash: window created");
            watch_splash_painted(app, window_created_at);
        }
        Err(error) => warn!("splash: could not create the window: {error}"),
    }
}

/// Records when the splash page has actually painted (not when the window
/// object was created). The minimum-duration gate in `hide()` measures from
/// this instant so that the 10 s minimum counts from visible content, not
/// from process start.
static SPLASH_SHOWN_AT: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

/// Fragment the splash page stamps into its URL when it has finished the
/// first paint. The page sets `location.hash = '#splash-painted'` in its
/// `requestAnimationFrame` callback.
const SPLASH_PAINTED_FRAGMENT: &str = "splash-painted";

/// Polls the splash window URL for the painted fragment and records the
/// paint instant in `SPLASH_SHOWN_AT`. Falls back to a deadline: if the
/// page never signals (broken load), the timer starts after 3 s regardless.
fn watch_splash_painted(app: &tauri::AppHandle, created_at: Instant) {
    let handle = app.clone();
    std::thread::spawn(move || {
        let deadline = created_at + Duration::from_secs(3);
        loop {
            if SPLASH_SHOWN_AT.get().is_some() {
                return;
            }
            if let Some(splash) = handle.get_webview_window("splash") {
                if let Ok(url) = splash.url() {
                    if url.as_str().contains(SPLASH_PAINTED_FRAGMENT) {
                        let now = Instant::now();
                        SPLASH_SHOWN_AT.get_or_init(|| now);
                        info!(
                            "splash: page painted ({:?} after window creation)",
                            now.duration_since(created_at)
                        );
                        return;
                    }
                }
            } else {
                // Splash was closed before it painted.
                return;
            }
            if Instant::now() >= deadline {
                let now = Instant::now();
                SPLASH_SHOWN_AT.get_or_init(|| now);
                info!(
                    "splash: paint signal deadline reached ({:?} after creation); \
                     starting the minimum timer now",
                    now.duration_since(created_at)
                );
                return;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    });
}

/// Hides (closes) the splash window once the real UI is ready, respecting the
/// minimum display duration so the edition's hero animation completes.
pub fn hide(app: &tauri::AppHandle) {
    // Callers include the setup code on the main thread, so the minimum
    // display time is waited on a thread of its own: sleeping here froze the
    // event loop, and every window with it, for the whole remainder.
    let min = splash_min_duration();
    let remaining = SPLASH_SHOWN_AT
        .get()
        .map(|shown| {
            let elapsed = shown.elapsed();
            info!("splash: ready; painted {elapsed:?} ago, minimum {min:?}");
            min.saturating_sub(elapsed)
        })
        .unwrap_or_else(|| {
            info!("splash: ready before the page painted; closing immediately");
            Duration::ZERO
        });
    if remaining.is_zero() {
        close(app);
        return;
    }
    info!("splash: closing in {remaining:?} (minimum display time)");
    let handle = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(remaining);
        close(&handle);
    });
}

fn close(app: &tauri::AppHandle) {
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
        if is_test_rig() {
            park_offscreen(&window);
        }
        let _ = window.show();
        if !is_test_rig() {
            let _ = window.set_always_on_top(TOPMOST);
            if TOPMOST {
                let _ = window.set_focus();
            }
        }
    }
}

/// Fragment the suite stamps into its URL when boot is complete.
const BOOT_READY_FRAGMENT: &str = "gb-boot-ready";
const BOOT_WATCH_POLL: Duration = Duration::from_millis(400);
const BOOT_WATCH_DEADLINE: Duration = Duration::from_millis(45_000);

fn push_version(splash: &WebviewWindow) {
    let version = env!("CARGO_PKG_VERSION");
    // The page detects the locale itself; pass the bare number and let JS format.
    let script = format!(
        "if(typeof window.gbSplashVersion==='function'){{var pt=navigator.language&&navigator.language.toLowerCase().startsWith('pt');window.gbSplashVersion((pt?'Vers\\u00e3o ':'Version ')+{});}}",
        json!(version)
    );
    let _ = splash.eval(&script);
}

/// Progress with a catalogue key. The page receives the key next to the
/// text and shows its own wording for it, in the language of the page: this
/// process has no locale on Windows (`LANG` is unset there), so its text
/// would be English under a Portuguese page.
fn push_progress_key(splash: &WebviewWindow, value: u8, key: &str) {
    let script = format!(
        "if(typeof window.gbSplashProgress==='function')window.gbSplashProgress({},{},{});",
        value,
        json!(i18n::t(key)),
        json!(key)
    );
    let _ = splash.eval(&script);
}

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
                    push_progress_key(&splash, 70, "splash-loading-interface");
                }
                if let Ok(url) = main.url() {
                    if url.as_str().contains(BOOT_READY_FRAGMENT) {
                        info!("splash: boot-ready fragment seen; closing");
                        push_progress_key(&splash, 100, "splash-ready");
                        std::thread::sleep(Duration::from_millis(120));
                        hide(&handle);
                        return;
                    }
                }
            } else if !saw_main {
                push_progress_key(&splash, 15, "splash-starting-services");
            }
            if Instant::now() >= deadline {
                warn!("splash: boot-ready fragment never arrived; hiding the splash");
                push_progress_key(&splash, 100, "splash-opening-workspace");
                std::thread::sleep(Duration::from_millis(120));
                hide(&handle);
                return;
            }
            std::thread::sleep(BOOT_WATCH_POLL);
        }
    });
}

/// Parses `GB_SPLASH_PREVIEW`: a number of seconds, or absent/empty.
pub fn splash_preview_seconds() -> Option<u64> {
    std::env::var("GB_SPLASH_PREVIEW")
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
        .filter(|s| *s > 0)
}

/// Runs the splash preview loop on an already-shown splash window, feeding
/// plausible progress every ~700 ms. Does NOT start the sidecar, wizard or
/// main window. Exits the app when done.
pub fn run_preview(app: &tauri::AppHandle, seconds: u64) {
    let handle = app.clone();
    std::thread::spawn(move || {
        let total = Duration::from_secs(seconds);
        let step = Duration::from_millis(700);
        let start = Instant::now();
        let labels = [
            "Starting services...",
            "Loading database...",
            "Connecting to cache...",
            "Loading drive...",
            "Starting identity...",
            "Loading assistant...",
            "Preparing workspace...",
            "Almost ready...",
        ];
        let mut idx = 0usize;
        loop {
            let elapsed = start.elapsed();
            if elapsed >= total {
                break;
            }
            let progress = ((elapsed.as_millis() as f64 / total.as_millis() as f64) * 95.0) as u8;
            let label = labels[idx % labels.len()];
            if let Some(splash) = handle.get_webview_window("splash") {
                push_progress(&splash, progress.min(95), label);
            }
            idx += 1;
            std::thread::sleep(step);
        }
        if let Some(splash) = handle.get_webview_window("splash") {
            push_progress(&splash, 100, "Ready");
        }
        std::thread::sleep(Duration::from_millis(300));
        std::process::exit(0);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_splash_min_matches_feature() {
        if cfg!(feature = "gbgov") {
            assert_eq!(DEFAULT_SPLASH_MIN_MS, 10000);
        } else {
            assert_eq!(DEFAULT_SPLASH_MIN_MS, 0);
        }
    }

    /// Tests the parsing logic that `splash_min_duration` and
    /// `splash_preview_seconds` share, without touching env vars (which
    /// race when tests run in parallel).
    #[test]
    fn parse_ms_from_string() {
        let parse = |s: &str| -> Option<u64> { s.trim().parse::<u64>().ok() };
        assert_eq!(parse("1234"), Some(1234));
        assert_eq!(parse("  0  "), Some(0));
        assert_eq!(parse("not-a-number"), None);
        assert_eq!(parse(""), None);
    }

    #[test]
    fn parse_preview_seconds() {
        let parse = |s: &str| -> Option<u64> {
            s.trim().parse::<u64>().ok().filter(|v| *v > 0)
        };
        assert_eq!(parse("5"), Some(5));
        assert_eq!(parse("0"), None);
        assert_eq!(parse("abc"), None);
        assert_eq!(parse(""), None);
    }

    /// #2561 — the minimum timer must start from the page-painted instant,
    /// not from window creation. This test verifies the pure timing rule:
    /// given a `SPLASH_SHOWN_AT` set N ms ago and a 10 s minimum, the
    /// remaining wait is `max(0, 10000 - N)`.
    #[test]
    fn minimum_duration_measured_from_painted_instant() {
        let min = Duration::from_millis(10_000);
        // Painted 2 s ago: 8 s remain.
        let painted_2s_ago = Instant::now() - Duration::from_millis(2_000);
        let remaining = min.saturating_sub(painted_2s_ago.elapsed());
        assert!(remaining >= Duration::from_millis(7_500), "expected ~8s remaining, got {remaining:?}");
        assert!(remaining <= Duration::from_millis(8_500));

        // Painted 12 s ago: 0 s remain.
        let painted_12s_ago = Instant::now() - Duration::from_millis(12_000);
        let remaining = min.saturating_sub(painted_12s_ago.elapsed());
        assert!(remaining.is_zero(), "expected zero remaining, got {remaining:?}");
    }

    #[test]
    fn splash_dimensions_match_edition() {
        if cfg!(feature = "gbgov") {
            assert_eq!((SPLASH_WIDTH, SPLASH_HEIGHT), (760.0, 428.0));
        } else {
            assert_eq!((SPLASH_WIDTH, SPLASH_HEIGHT), (420.0, 420.0));
        }
    }
}
