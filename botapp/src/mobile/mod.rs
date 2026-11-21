//! Mobile / thin-client entry point.
//!
//! A WebView renders the existing suite from a validated remote origin. There
//! is no sidecar, wizard, tray, service monitor, splash, payload extraction or
//! host filesystem command in this module or anywhere it pulls in.

mod commands;
mod config;

use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

/// Builds and runs the mobile WebView. A missing/invalid origin is a hard
/// configuration error: the app never opens an unconfigured server.
pub fn run() {
    botlib::logging::init_compact_logger("info");

    let origin = match config::remote_origin() {
        Ok(origin) => origin,
        Err(error) => {
            log::error!("mobile thin client cannot start: {error}");
            std::process::exit(2);
        }
    };
    let suite_url = match config::suite_url(&origin, config::session_token().as_deref()) {
        Ok(url) => url,
        Err(error) => {
            log::error!("mobile thin client suite URL is invalid: {error}");
            std::process::exit(2);
        }
    };
    let init_script = config::initialization_script(&origin);

    let app = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![commands::open_external_url])
        .setup(move |app| {
            // The shared Tauri config declares a splash and a main window for
            // the desktop edition. Neither belongs to a thin client: drop both
            // and recreate "main" so the origin is installed by an init script
            // before the first suite script runs. The label stays "main" so
            // the existing capability still authorises the single command.
            if let Some(splash) = app.get_webview_window("splash") {
                let _ = splash.close();
            }
            if let Some(main) = app.get_webview_window("main") {
                let _ = main.close();
            }

            let allowed_origin = origin.clone();
            let window = WebviewWindowBuilder::new(&*app, "main", WebviewUrl::External(suite_url))
                .initialization_script(init_script.as_str())
                .on_navigation(move |url| url.origin().ascii_serialization() == allowed_origin)
                .build()?;
            if let Err(error) = window.show() {
                log::warn!("could not reveal the mobile suite window: {error}");
            }
            Ok(())
        })
        .build(tauri::generate_context!());

    match app {
        Ok(app) => app.run(|_, _| {}),
        Err(error) => log::error!("failed to build mobile BotApp: {error}"),
    }
}
