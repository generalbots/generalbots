//! Desktop application entry point.
//!
//! Compiled only for non-mobile builds. The mobile thin client has its own
//! `crate::mobile::run()` and never reaches this module.

#[cfg(not(feature = "mobile"))]
use log::{info, warn};
#[cfg(not(feature = "mobile"))]
use tauri::{Emitter, Manager};

#[cfg(all(feature = "desktop-local", not(feature = "mobile")))]
use crate::desktop::sidecar::Sidecar;
#[cfg(not(feature = "mobile"))]
use crate::desktop::tray::{RunningMode, ServiceMonitor, TrayManager};
#[cfg(all(feature = "desktop-local", not(feature = "mobile")))]
use crate::desktop::wizard::WizardState;

/// Starts the botserver sidecar and reveals the suite once the API answers.
#[cfg(all(feature = "desktop-local", not(feature = "mobile")))]
fn start_sidecar_and_show(app: &tauri::App) {
    let resource_dir = app.path().resource_dir().ok();

    let handle = app.handle().clone();
    tauri::async_runtime::spawn(async move {
        // A previous run may have left the sidecar alive (for example when the
        // window was closed without a graceful exit). Reuse it instead of
        // starting a second listener on the same port.
        let product_url = crate::desktop::sidecar::probe::api_product_url();
        let already_running = Sidecar::wait_for_api(&product_url, 5).await;
        if !already_running {
            let sidecar = handle.state::<Sidecar>().inner().clone();
            if let Err(e) = sidecar.start(resource_dir.clone()) {
                log::error!("Failed to start botserver sidecar: {e}");
            }
        }
        // A cold start (services stopped, first run after a reboot) can take
        // several minutes. Revealing the window before the server answers
        // showed the bundled page, which cannot reach the API; keep waiting
        // behind the splash instead.
        let mut ready = false;
        for round in 1..=5u32 {
            ready = Sidecar::wait_for_api(&crate::desktop::sidecar::probe::api_product_url(), 120).await;
            if ready {
                break;
            }
            log::warn!("botserver has not answered after {}s; still waiting", round * 120);
        }
        if !ready {
            log::error!("botserver did not respond within 600s");
        }
        crate::desktop::wizard::reveal_suite(&handle);
    });
}

#[cfg(not(feature = "mobile"))]
pub fn run() {
    if let Some(code) = crate::desktop::payload::handle_cli() {
        std::process::exit(code);
    }
    botlib::logging::init_compact_logger("info");
    let version = crate::build_info::version();
    info!("BotApp {version} starting...");

    // #1870 — carry the WebKit profile (localStorage, cache) from the legacy
    // bundle identifier's data folder before any webview opens, so users keep
    // their sessions and settings across the rename.
    crate::desktop::legacy_data::migrate_legacy_data_dir();

    let tray_manager = TrayManager::with_mode(RunningMode::Desktop);
    let service_monitor = tokio::sync::Mutex::new(ServiceMonitor::new());

    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init());
    // #2697 — guard the privileged main WebView against redirects to external
    // origins. Registered before any window is created, so it also covers the
    // config-created `main` window (the builder-level hook only covers windows
    // we create ourselves).
    #[cfg(feature = "desktop-local")]
    {
        builder = builder.plugin(crate::desktop::wizard::navigation_guard());
    }
    #[cfg(target_os = "linux")]
    { builder = builder.plugin(crate::desktop::webkit_permissions::plugin()); }
    let builder = builder.manage(tray_manager).manage(service_monitor);

    #[cfg(feature = "desktop-local")]
    let builder = builder
        .manage(Sidecar::new())
        .manage(WizardState::default());

    #[cfg(feature = "desktop-local")]
    let builder = builder.invoke_handler(tauri::generate_handler![
        crate::desktop::download::save_download,
        crate::desktop::drive::list_files,
        crate::desktop::drive::upload_file,
        crate::desktop::drive::create_folder_from_window,
        crate::desktop::drive::delete_path_from_window,
        crate::desktop::drive::request_destructive_confirmation,
        crate::desktop::webkit_permissions::set_media_permission,
        crate::desktop::drive::get_home_dir,
        crate::desktop::sync::get_sync_status,
        crate::desktop::sync::start_sync,
        crate::desktop::sync::stop_sync,
        crate::desktop::sync::configure_remote,
        crate::desktop::sync::check_rclone_installed,
        crate::desktop::sync::list_remotes,
        crate::desktop::sync::get_sync_folder,
        crate::desktop::sync::set_sync_folder,
        crate::desktop::commands::get_tray_status,
        crate::desktop::commands::start_tray,
        crate::desktop::commands::stop_tray,
        crate::desktop::commands::show_notification,
        crate::desktop::commands::update_tray_status,
        crate::desktop::commands::set_tray_tooltip,
        crate::desktop::commands::get_tray_hostname,
        crate::desktop::commands::set_tray_hostname,
        crate::desktop::commands::handle_tray_event,
        crate::desktop::commands::check_services,
        crate::desktop::commands::add_service,
        crate::desktop::commands::get_service,
        crate::desktop::commands::all_services_running,
        crate::desktop::commands::any_service_running,
        crate::desktop::commands::get_tray_mode,
        crate::desktop::commands::get_running_modes,
        crate::desktop::commands::create_tray_with_mode,
        crate::desktop::update::check_release_update,
        crate::desktop::update::open_release_url,
        crate::desktop::wizard::wizard_context,
        crate::desktop::wizard::wizard_save_draft,
        crate::desktop::wizard::wizard_load_draft,
        crate::desktop::wizard::wizard_check_workspace,
        crate::desktop::wizard::wizard_pick_folder,
        crate::desktop::wizard::wizard_host_check,
        crate::desktop::wizard::wizard_media_capabilities,
        crate::desktop::wizard::wizard_test_model,
        crate::desktop::wizard::wizard_provision,
        crate::desktop::wizard::wizard_cancel_provision,
        crate::desktop::wizard::wizard_provision_status,
        crate::desktop::wizard::wizard_reset_stack,
        crate::desktop::wizard::request_reset_stack_confirmation,
        crate::desktop::wizard::wizard_open_folder,
        crate::desktop::wizard::wizard_open_url,
        crate::desktop::wizard::wizard_save_admin,
        crate::desktop::wizard::wizard_ai_accounts,
        crate::desktop::wizard::wizard_ai_connect,
        crate::desktop::wizard::wizard_ai_status,
        crate::desktop::wizard::wizard_ai_disconnect,
        crate::desktop::wizard::wizard_ai_code,
        crate::desktop::wizard::wizard_ai_use_chat,
        crate::desktop::wizard::wizard_finish,
    ]);

    #[cfg(not(feature = "desktop-local"))]
    let builder = builder.invoke_handler(tauri::generate_handler![
        crate::desktop::download::save_download,
        crate::desktop::drive::list_files,
        crate::desktop::drive::upload_file,
        crate::desktop::drive::create_folder_from_window,
        crate::desktop::drive::delete_path_from_window,
        crate::desktop::drive::request_destructive_confirmation,
        crate::desktop::webkit_permissions::set_media_permission,
        crate::desktop::drive::get_home_dir,
        crate::desktop::sync::get_sync_status,
        crate::desktop::sync::start_sync,
        crate::desktop::sync::stop_sync,
        crate::desktop::sync::configure_remote,
        crate::desktop::sync::check_rclone_installed,
        crate::desktop::sync::list_remotes,
        crate::desktop::sync::get_sync_folder,
        crate::desktop::sync::set_sync_folder,
        crate::desktop::commands::get_tray_status,
        crate::desktop::commands::start_tray,
        crate::desktop::commands::stop_tray,
        crate::desktop::commands::show_notification,
        crate::desktop::commands::update_tray_status,
        crate::desktop::commands::set_tray_tooltip,
        crate::desktop::commands::get_tray_hostname,
        crate::desktop::commands::set_tray_hostname,
        crate::desktop::commands::handle_tray_event,
        crate::desktop::commands::check_services,
        crate::desktop::commands::add_service,
        crate::desktop::commands::get_service,
        crate::desktop::commands::all_services_running,
        crate::desktop::commands::any_service_running,
        crate::desktop::commands::get_tray_mode,
        crate::desktop::commands::get_running_modes,
        crate::desktop::commands::create_tray_with_mode,
        crate::desktop::update::check_release_update,
        crate::desktop::update::open_release_url,
    ]);

    // #2450 — under cage the main window is the only client the compositor
    // has: a close request would end the session and come back as a black
    // screen. In kiosk mode the request is intercepted and the shell's power
    // menu is shown instead; the power API is the only path that really exits.
    let builder = builder.on_window_event(|window, event| {
        if !crate::desktop::shell_mode() {
            return;
        }
        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            if let Err(e) = window.emit("gb-power-menu", ()) {
                warn!("could not ask the shell for its power menu: {e}");
            }
        }
    });

    let app = builder
        .setup(|app| {
            // #1827 — show the topmost splash while the (possibly heavy) payload
            // extraction and the server stack boot run.
            crate::desktop::splash::show(&app.handle());

            // GB_SPLASH_PREVIEW: show just the splash with simulated progress,
            // then exit. Does not start the sidecar, wizard or main window, and
            // does not touch the stack. Safe to run alongside the real app (no
            // single-instance guard exists).
            if let Some(seconds) = crate::desktop::splash::splash_preview_seconds() {
                info!("splash preview mode: {seconds}s");
                crate::desktop::splash::run_preview(&app.handle(), seconds);
                return Ok(());
            }

            // #1633 — first run of the self-extracting build provisions the
            // embedded stack before the wizard or the sidecar looks for it.
            if let Err(error) = crate::desktop::payload::extract_if_present() {
                log::error!("embedded payload could not be extracted: {error}");
            }
            let tray = app.state::<TrayManager>();
            let mode = tray.get_mode_string();
            info!("BotApp setup complete in {mode} mode");

            // --- First run: the wizard owns the stack lifecycle ---
            #[cfg(feature = "desktop-local")]
            {
                if crate::desktop::wizard::is_first_run() {
                    info!("First run detected: opening the setup wizard");
                    if let Err(e) = crate::desktop::wizard::open_window(&app.handle()) {
                        log::error!("Could not open the setup wizard: {e}");
                        start_sidecar_and_show(app);
                    }
                } else {
                    start_sidecar_and_show(app);
                }
            }

            // --- Non-sidecar: show window immediately ---
            #[cfg(not(feature = "desktop-local"))]
            {
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.show();
                }
                crate::desktop::splash::hide(&app.handle());
            }

            // --- Tray icon ---
            let tray_clone = tray.inner().clone();
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                let rt = match tokio::runtime::Runtime::new() {
                    Ok(rt) => rt,
                    Err(e) => {
                        log::error!("Failed to create runtime: {e}");
                        return;
                    }
                };
                rt.block_on(async {
                    if let Err(e) = tray_clone.start(&app_handle).await {
                        log::error!("Failed to start tray: {e}");
                    }
                });
            });

            Ok(())
        })
        .build(tauri::generate_context!());

    match app {
        Ok(app) => {
            #[cfg(feature = "desktop-local")]
            let sidecar = app.state::<Sidecar>().inner().clone();

            app.run(move |app, event| {
                #[cfg(feature = "desktop-local")]
                if let tauri::RunEvent::ExitRequested { api, .. } = &event {
                    // #1826 — an explicit Quit from the tray must terminate the
                    // process even when the suite window still exists. The
                    // tray handler stops the sidecar and sets this flag before
                    // requesting exit.
                    if crate::desktop::tray::quit_requested() {
                        info!("quitting from the tray: stopping the sidecar");
                        sidecar.stop();
                    } else if app.get_webview_window("main").is_some() {
                        // Closing the wizard must not take the suite down with it:
                        // the main window is already answering, so keep the
                        // process (and the sidecar) alive until it is gone.
                        info!("keeping the app alive: the suite window still exists");
                        api.prevent_exit();
                    } else if crate::desktop::activity::is_busy() {
                        // #1620 — a reply is still being generated. Killing the
                        // sidecar here truncated real answers (and the tool
                        // call that writes a PDF). Keep the process alive and
                        // bring the suite back so the result is visible.
                        info!(
                            "a reply is in progress: keeping the app alive and reopening the suite"
                        );
                        api.prevent_exit();
                        crate::desktop::wizard::ensure_suite_window(app);
                    } else {
                        sidecar.stop();
                    }
                }
                let _ = (&app, &event);
            });
        }
        Err(e) => {
            log::error!("Failed to build BotApp: {e}");
        }
    }
}
