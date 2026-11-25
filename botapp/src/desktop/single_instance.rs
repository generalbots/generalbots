//! One running copy per person. A second launch exits at once and brings the
//! running window (splash, setup wizard or suite) to the front: two copies
//! would start two servers on the same data stack and ports.

use log::info;
use tauri::{AppHandle, Builder, Manager, Runtime, WebviewWindow};

/// Whether this process takes part in the guard. The splash preview and the
/// hidden test rig (its own WebView profile and server) run beside the
/// person's own copy, so they skip it.
fn enabled() -> bool {
    !super::splash::is_test_rig() && super::splash::splash_preview_seconds().is_none()
}

/// Adds the guard to the builder. It must be the first plugin registered, so
/// a second copy exits before anything else starts.
pub fn register<R: Runtime>(builder: Builder<R>) -> Builder<R> {
    if !enabled() {
        info!("single-instance guard skipped (test rig or splash preview)");
        return builder;
    }
    builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
        info!("second launch: bringing the running window to the front");
        focus_running_window(app);
    }))
}

fn bring_forward<R: Runtime>(window: &WebviewWindow<R>) {
    let _ = window.show();
    let _ = window.unminimize();
    let _ = window.set_focus();
}

/// The wizard owns the first run; while the server boots the splash is the
/// window the person is waiting on; afterwards the suite, even when it was
/// hidden in the tray or minimised.
fn focus_running_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(wizard) = app.get_webview_window("wizard") {
        bring_forward(&wizard);
        return;
    }
    if let Some(splash) = app.get_webview_window("splash") {
        if splash.is_visible().unwrap_or(false) {
            bring_forward(&splash);
            return;
        }
    }
    if let Some(main) = app.get_webview_window("main") {
        bring_forward(&main);
    }
}
