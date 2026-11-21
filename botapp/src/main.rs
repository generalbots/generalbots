#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(feature = "mobile")]
mod mobile;

#[cfg(not(feature = "mobile"))]
use log::{info, warn};
#[cfg(not(feature = "mobile"))]
use tauri::{Emitter, Manager};

#[cfg(not(feature = "mobile"))]
mod build_info;
#[cfg(not(feature = "mobile"))]
mod desktop;

#[cfg(all(feature = "desktop-local", not(feature = "mobile")))]
use desktop::sidecar::Sidecar;
#[cfg(not(feature = "mobile"))]
use desktop::tray::{RunningMode, ServiceMonitor, TrayManager};
#[cfg(all(feature = "desktop-local", not(feature = "mobile")))]
use desktop::wizard::WizardState;

#[cfg(feature = "mobile")]
fn main() {
    mobile::run();
}

#[cfg(not(feature = "mobile"))]
use crate::desktop::commands::{
    add_service, all_services_running, any_service_running, check_services, create_tray_with_mode,
    get_running_modes, get_service, get_tray_hostname, get_tray_mode, get_tray_status,
    handle_tray_event, set_tray_hostname, set_tray_tooltip, show_notification, start_tray,
    stop_tray, update_tray_status,
};

/// Starts the botserver sidecar and reveals the suite once the API answers.
#[cfg(all(feature = "desktop-local", not(feature = "mobile")))]
fn start_sidecar_and_show(app: &tauri::App) {
    let resource_dir = app.path().resource_dir().ok();

    let handle = app.handle().clone();
    tauri::async_runtime::spawn(async move {
        // A previous run may have left the sidecar alive (for example when the
        // window was closed without a graceful exit). Reuse it instead of
        // starting a second listener on the same port.
        let product_url = desktop::sidecar::probe::api_product_url();
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
            ready = Sidecar::wait_for_api(&desktop::sidecar::probe::api_product_url(), 120).await;
            if ready {
                break;
            }
            log::warn!("botserver has not answered after {}s; still waiting", round * 120);
        }
        if !ready {
            log::error!("botserver did not respond within 600s");
        }
        desktop::wizard::reveal_suite(&handle);
    });
}

#[cfg(not(feature = "mobile"))]
fn main() {
    if let Some(code) = desktop::payload::handle_cli() {
        std::process::exit(code);
    }
    botlib::logging::init_compact_logger("info");
    let version = build_info::version();
    info!("BotApp {version} starting...");

    // #1870 — carry the WebKit profile (localStorage, cache) from the legacy
    // bundle identifier's data folder before any webview opens, so users keep
    // their sessions and settings across the rename.
    desktop::legacy_data::migrate_legacy_data_dir();

    let tray_manager = TrayManager::with_mode(RunningMode::Desktop);
    let service_monitor = tokio::sync::Mutex::new(ServiceMonitor::new());

    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init());
    #[cfg(target_os = "linux")]
    { builder = builder.plugin(desktop::webkit_permissions::plugin()); }
    let builder = builder.manage(tray_manager).manage(service_monitor);

    #[cfg(feature = "desktop-local")]
    let builder = builder
        .manage(Sidecar::new())
        .manage(WizardState::default());

    #[cfg(feature = "desktop-local")]
    let builder = builder.invoke_handler(tauri::generate_handler![
        desktop::download::save_download,
        desktop::drive::list_files,
        desktop::drive::upload_file,
        desktop::drive::create_folder,
        desktop::drive::delete_path,
        desktop::drive::request_destructive_confirmation,
        desktop::drive::get_home_dir,
        desktop::sync::get_sync_status,
        desktop::sync::start_sync,
        desktop::sync::stop_sync,
        desktop::sync::configure_remote,
        desktop::sync::check_rclone_installed,
        desktop::sync::list_remotes,
        desktop::sync::get_sync_folder,
        desktop::sync::set_sync_folder,
        get_tray_status,
        start_tray,
        stop_tray,
        show_notification,
        update_tray_status,
        set_tray_tooltip,
        get_tray_hostname,
        set_tray_hostname,
        handle_tray_event,
        check_services,
        add_service,
        get_service,
        all_services_running,
        any_service_running,
        get_tray_mode,
        get_running_modes,
        create_tray_with_mode,
        desktop::update::check_release_update,
        desktop::update::open_release_url,
        desktop::wizard::wizard_context,
        desktop::wizard::wizard_save_draft,
        desktop::wizard::wizard_load_draft,
        desktop::wizard::wizard_check_workspace,
        desktop::wizard::wizard_pick_folder,
        desktop::wizard::wizard_host_check,
        desktop::wizard::wizard_media_capabilities,
        desktop::wizard::wizard_test_model,
        desktop::wizard::wizard_provision,
        desktop::wizard::wizard_cancel_provision,
        desktop::wizard::wizard_provision_status,
        desktop::wizard::wizard_reset_stack,
        desktop::wizard::wizard_open_folder,
        desktop::wizard::wizard_open_url,
        desktop::wizard::wizard_save_admin,
        desktop::wizard::wizard_ai_accounts,
        desktop::wizard::wizard_ai_connect,
        desktop::wizard::wizard_ai_status,
        desktop::wizard::wizard_ai_disconnect,
        desktop::wizard::wizard_ai_code,
        desktop::wizard::wizard_ai_use_chat,
        desktop::wizard::wizard_finish,
    ]);

    #[cfg(not(feature = "desktop-local"))]
    let builder = builder.invoke_handler(tauri::generate_handler![
        desktop::download::save_download,
        desktop::drive::list_files,
        desktop::drive::upload_file,
        desktop::drive::create_folder,
        desktop::drive::delete_path,
        desktop::drive::request_destructive_confirmation,
        desktop::drive::get_home_dir,
        desktop::sync::get_sync_status,
        desktop::sync::start_sync,
        desktop::sync::stop_sync,
        desktop::sync::configure_remote,
        desktop::sync::check_rclone_installed,
        desktop::sync::list_remotes,
        desktop::sync::get_sync_folder,
        desktop::sync::set_sync_folder,
        get_tray_status,
        start_tray,
        stop_tray,
        show_notification,
        update_tray_status,
        set_tray_tooltip,
        get_tray_hostname,
        set_tray_hostname,
        handle_tray_event,
        check_services,
        add_service,
        get_service,
        all_services_running,
        any_service_running,
        get_tray_mode,
        get_running_modes,
        create_tray_with_mode,
        desktop::update::check_release_update,
        desktop::update::open_release_url,
    ]);

    // #2450 — under cage the main window is the only client the compositor
    // has: a close request would end the session and come back as a black
    // screen. In kiosk mode the request is intercepted and the shell's power
    // menu is shown instead; the power API is the only path that really exits.
    let builder = builder.on_window_event(|window, event| {
        if !desktop::shell_mode() {
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
            desktop::splash::show(&app.handle());

            // GB_SPLASH_PREVIEW: show just the splash with simulated progress,
            // then exit. Does not start the sidecar, wizard or main window, and
            // does not touch the stack. Safe to run alongside the real app (no
            // single-instance guard exists).
            if let Some(seconds) = desktop::splash::splash_preview_seconds() {
                info!("splash preview mode: {seconds}s");
                desktop::splash::run_preview(&app.handle(), seconds);
                return Ok(());
            }

            // #1633 — first run of the self-extracting build provisions the
            // embedded stack before the wizard or the sidecar looks for it.
            if let Err(error) = desktop::payload::extract_if_present() {
                log::error!("embedded payload could not be extracted: {error}");
            }
            let tray = app.state::<TrayManager>();
            let mode = tray.get_mode_string();
            info!("BotApp setup complete in {mode} mode");

            // --- First run: the wizard owns the stack lifecycle ---
            #[cfg(feature = "desktop-local")]
            {
                if desktop::wizard::is_first_run() {
                    info!("First run detected: opening the setup wizard");
                    if let Err(e) = desktop::wizard::open_window(&app.handle()) {
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
                desktop::splash::hide(&app.handle());
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
                    if desktop::tray::quit_requested() {
                        info!("quitting from the tray: stopping the sidecar");
                        sidecar.stop();
                    } else if app.get_webview_window("main").is_some() {
                        // Closing the wizard must not take the suite down with it:
                        // the main window is already answering, so keep the
                        // process (and the sidecar) alive until it is gone.
                        info!("keeping the app alive: the suite window still exists");
                        api.prevent_exit();
                    } else if desktop::activity::is_busy() {
                        // #1620 — a reply is still being generated. Killing the
                        // sidecar here truncated real answers (and the tool
                        // call that writes a PDF). Keep the process alive and
                        // bring the suite back so the result is visible.
                        info!(
                            "a reply is in progress: keeping the app alive and reopening the suite"
                        );
                        api.prevent_exit();
                        desktop::wizard::ensure_suite_window(app);
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
