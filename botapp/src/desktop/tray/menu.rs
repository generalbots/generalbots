//! Tray menu actions.
//!
//! The suite is a single webview that manages its own app windows, so menu
//! actions run a small script in the already-loaded desktop shell instead of
//! opening new native windows. `openDeepLink` is the shell's public launcher
//! API (see `botui/ui/suite/js/window-manager/06_deeplinks.js`).

use log::{debug, info, warn};
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Manager};

#[cfg(feature = "desktop-tray")]
pub(super) const NEW_CHAT_ID: &str = "tray-new-chat";
#[cfg(feature = "desktop-tray")]
pub(super) const FILES_ID: &str = "tray-files";
#[cfg(feature = "desktop-tray")]
pub(super) const SETTINGS_ID: &str = "tray-settings";
#[cfg(feature = "desktop-tray")]
pub(super) const ABOUT_ID: &str = "tray-about";
#[cfg(feature = "desktop-tray")]
pub(super) const QUIT_ID: &str = "tray-quit";

/// Set when the user explicitly asks to quit. `main` reads it in
/// `RunEvent::ExitRequested` so the "keep the suite alive in the tray" rule
/// does not swallow a real quit (#1826).
static QUIT_REQUESTED: AtomicBool = AtomicBool::new(false);

pub fn quit_requested() -> bool {
    QUIT_REQUESTED.load(Ordering::SeqCst)
}

#[cfg(feature = "desktop-tray")]
pub(super) fn handle_menu_event(app: &AppHandle, id: &str) -> bool {
    match id {
        NEW_CHAT_ID => new_chat(app),
        FILES_ID => open_app(app, "drive", false),
        SETTINGS_ID => open_app(app, "settings", false),
        ABOUT_ID => open_app(app, "about", false),
        QUIT_ID => quit(app),
        _ => return false,
    }
    true
}

/// Opens the Chat app and starts a fresh conversation in it.
pub(super) fn new_chat(app: &AppHandle) {
    show_main_window(app);
    // A fresh conversation is a new tab in the Chat window (the same path the
    // sidebar's "New conversation" action takes). When Chat is not mounted
    // yet, focus/open it first and keep retrying until `GBSidebar` is ready.
    let script = r#"(function(){
        var attempts = 0;
        var focused = false;
        function fresh() {
            if (window.GBSidebar && typeof window.GBSidebar.newConversation === "function" &&
                document.getElementById("messageInput")) {
                window.GBSidebar.newConversation();
                return true;
            }
            return false;
        }
        function focusChat() {
            if (window.openDeepLink) {
                window.openDeepLink("chat", {}, {});
                return true;
            }
            return false;
        }
        focused = focusChat();
        if (fresh()) return;
        var timer = setInterval(function(){
            if (fresh() || ++attempts > 60) {
                clearInterval(timer);
                return;
            }
            if (!focused) focused = focusChat();
        }, 250);
    })();"#;
    eval_on_main(app, script);
}

/// Opens (or focuses) one of the suite apps.
pub(super) fn open_app(app: &AppHandle, app_id: &str, fresh_window: bool) {
    show_main_window(app);
    let opts = if fresh_window { "{new:true}" } else { "{}" };
    let script = format!(
        "(function(){{var n=0;var t=setInterval(function(){{if(window.openDeepLink){{clearInterval(t);window.openDeepLink({app_id:?},{{}},{opts});}}else if(++n>40){{clearInterval(t);}}}},250);}})();"
    );
    eval_on_main(app, &script);
}

/// Brings the suite (or the first-run wizard) to the front without navigating
/// away from the user's current app window.
pub(super) fn show_main_window(app: &AppHandle) {
    #[cfg(feature = "desktop-local")]
    if let Some(wizard) = app.get_webview_window("wizard") {
        let _ = wizard.show();
        let _ = wizard.unminimize();
        let _ = wizard.set_focus();
        return;
    }

    if let Some(window) = app.get_webview_window("main") {
        if let Err(e) = window.show() {
            warn!("tray: could not show the main window: {e}");
        }
        let _ = window.unminimize();
        if let Err(e) = window.set_focus() {
            warn!("tray: could not focus the main window: {e}");
        }
    } else {
        #[cfg(feature = "desktop-local")]
        crate::desktop::wizard::ensure_suite_window(app);
        #[cfg(not(feature = "desktop-local"))]
        warn!("tray: the main window is not available");
    }
}

/// Single left-click on the tray icon: hide a focused window, otherwise
/// restore and focus it.
#[cfg(feature = "desktop-tray")]
pub(super) fn toggle_main_window(app: &AppHandle) {
    #[cfg(feature = "desktop-local")]
    if let Some(wizard) = app.get_webview_window("wizard") {
        let _ = wizard.show();
        let _ = wizard.set_focus();
        return;
    }

    let Some(window) = app.get_webview_window("main") else {
        #[cfg(feature = "desktop-local")]
        crate::desktop::wizard::ensure_suite_window(app);
        return;
    };

    let visible = window.is_visible().unwrap_or(false);
    let focused = window.is_focused().unwrap_or(false);
    if visible && focused {
        if let Err(e) = window.hide() {
            warn!("tray: could not hide the main window: {e}");
        }
        return;
    }
    let _ = window.show();
    let _ = window.unminimize();
    let _ = window.set_focus();
}

/// Stops the sidecar and exits the process. The exit-request handler in
/// `main` checks [`quit_requested`] so it does not veto this request.
pub(super) fn quit(app: &AppHandle) {
    QUIT_REQUESTED.store(true, Ordering::SeqCst);
    info!("tray: quitting General Bots");

    #[cfg(feature = "desktop-local")]
    if let Some(sidecar) = app.try_state::<crate::desktop::sidecar::Sidecar>() {
        sidecar.stop();
    }

    app.exit(0);
}

fn eval_on_main(app: &AppHandle, script: &str) {
    match app.get_webview_window("main") {
        Some(window) => {
            debug!("tray: running a suite action in the main window");
            if let Err(e) = window.eval(script) {
                warn!("tray: could not run the suite action: {e}");
            }
        }
        None => warn!("tray: no suite window is available for this action"),
    }
}
