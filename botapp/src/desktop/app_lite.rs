//! Desktop entry point for the Lite edition (`--features gblite`).
//!
//! Compiled only when the `gblite` feature is active. The platform desktop
//! uses `app.rs`; this module adds the edition-specific setup: single-instance
//! guard, edition migration, boot supervisor with retry, agent runtime managed
//! state, the `open_download` and `local_password_reset` commands, and the
//! `WindowEvent::Destroyed` / `ExitRequested` handling that the first-run
//! wizard needs.

#[cfg(not(feature = "mobile"))]
use log::info;
#[cfg(not(feature = "mobile"))]
use tauri::Manager;

#[cfg(all(feature = "desktop-local", not(feature = "mobile")))]
use crate::desktop::sidecar::Sidecar;
#[cfg(not(feature = "mobile"))]
use crate::desktop::tray::{RunningMode, ServiceMonitor, TrayManager};
#[cfg(all(feature = "desktop-local", not(feature = "mobile")))]
use crate::desktop::wizard::WizardState;

#[cfg(not(feature = "mobile"))]
pub fn run() {
    if let Some(code) = crate::desktop::payload::handle_cli() {
        std::process::exit(code);
    }
    botlib::logging::init_compact_logger("info");
    let version = crate::build_info::version();
    info!("BotApp {version} ({}, built {}) starting (Lite edition)...",
        crate::build_info::commit(),
        crate::build_info::build_date()
    );

    let context: tauri::Context<tauri::Wry> = tauri::generate_context!();
    let running_identifier = context.config().identifier.clone();
    crate::desktop::legacy_data::migrate_legacy_data_dir(&running_identifier);
    crate::desktop::edition_migration::migrate(&running_identifier);

    let tray_manager = TrayManager::with_mode(RunningMode::Desktop);
    let service_monitor = tokio::sync::Mutex::new(ServiceMonitor::new());

    let builder = crate::desktop::single_instance::register(tauri::Builder::default())
        .plugin(tauri_plugin_dialog::init());

    #[cfg(feature = "desktop-local")]
    let builder = builder.plugin(crate::desktop::wizard::navigation_guard());

    #[cfg(target_os = "linux")]
    let builder = builder.plugin(crate::desktop::webkit_permissions::plugin());

    let builder = builder.manage(tray_manager).manage(service_monitor);

    #[cfg(feature = "desktop-local")]
    let builder = builder
        .manage(Sidecar::new())
        .manage(WizardState::default())
        .manage(crate::desktop::agent_runtime::AgentRuntimeManager::default());

    #[cfg(feature = "desktop-local")]
    let builder = builder.invoke_handler(tauri::generate_handler![
        crate::desktop::download::save_download,
        crate::desktop::open_download::open_download,
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
        crate::desktop::recovery::local_password_reset,
        crate::desktop::agent_runtime::commands::agent_runtime_status,
        crate::desktop::agent_runtime::commands::agent_runtime_install,
        crate::desktop::agent_runtime::commands::agent_runtime_cancel,
        crate::desktop::agent_runtime::commands::agent_runtime_remove,
        crate::desktop::agent_runtime::commands::agent_runtime_check_update,
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

    let app = builder
        .setup(|app| {
            for window in app.webview_windows().values() {
                if let Err(e) = window.set_title(&app.package_info().name) {
                    log::warn!("could not set the title of window {}: {e}", window.label());
                }
            }
            crate::desktop::splash::show(&app.handle());

            if let Some(seconds) = crate::desktop::splash::splash_preview_seconds() {
                info!("splash preview mode: {seconds}s");
                crate::desktop::splash::run_preview(&app.handle(), seconds);
                return Ok(());
            }

            if let Err(error) = crate::desktop::payload::extract_if_present() {
                log::error!("embedded payload could not be extracted: {error}");
            }
            let tray = app.state::<TrayManager>();
            let mode = tray.get_mode_string();
            info!("BotApp setup complete in {mode} mode");

            #[cfg(feature = "desktop-local")]
            {
                if crate::desktop::wizard::is_first_run() {
                    info!("First run detected: opening the setup wizard");
                    if let Err(e) = crate::desktop::wizard::open_window(&app.handle()) {
                        log::error!("Could not open the setup wizard: {e}");
                        crate::desktop::boot_supervisor::start_sidecar_and_show(app);
                    }
                } else {
                    crate::desktop::boot_supervisor::start_sidecar_and_show(app);
                }
            }

            #[cfg(not(feature = "desktop-local"))]
            {
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.show();
                }
                crate::desktop::splash::hide(&app.handle());
            }

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
        .build(context);

    match app {
        Ok(app) => {
            #[cfg(feature = "desktop-local")]
            let sidecar = app.state::<Sidecar>().inner().clone();

            app.run(move |app, event| {
                #[cfg(feature = "desktop-local")]
                match &event {
                    tauri::RunEvent::WindowEvent {
                        label,
                        event: tauri::WindowEvent::Destroyed,
                        ..
                    } if label == "wizard" || label == "main" => {
                        use crate::desktop::exit_decision::{on_window_destroyed, WindowAction};
                        let ws = app.state::<WizardState>();
                        let action = on_window_destroyed(
                            label,
                            ws.handoff_complete
                                .load(std::sync::atomic::Ordering::SeqCst),
                            crate::desktop::wizard::is_first_run(),
                        );
                        if action == WindowAction::AbortFirstRun {
                            ws.handoff_complete
                                .store(true, std::sync::atomic::Ordering::SeqCst);
                            info!(
                                "window '{label}' destroyed during first run: \
                                 cancelling setup and exiting"
                            );
                            crate::desktop::wizard::cancel_provision(&ws);
                            sidecar.stop();
                            crate::desktop::stack_stop::stop_stack();
                            for (name, window) in app.webview_windows() {
                                if name != *label {
                                    let _ = window.destroy();
                                }
                            }
                            app.exit(0);
                        }
                    }

                    tauri::RunEvent::ExitRequested { api, .. } => {
                        use crate::desktop::exit_decision::{on_exit_requested, ExitAction};
                        let main_visible = app
                            .get_webview_window("main")
                            .and_then(|w| w.is_visible().ok())
                            .unwrap_or(false);
                        let action = on_exit_requested(
                            crate::desktop::tray::quit_requested(),
                            main_visible,
                            crate::desktop::activity::is_busy(),
                        );
                        match action {
                            ExitAction::Exit => {
                                info!("quitting: stopping the sidecar");
                                sidecar.stop();
                                crate::desktop::stack_stop::stop_stack();
                            }
                            ExitAction::KeepAlive => {
                                info!(
                                    "keeping the app alive: the suite window still exists"
                                );
                                api.prevent_exit();
                            }
                            ExitAction::KeepAliveAndRecreate => {
                                info!(
                                    "a reply is in progress: keeping the app alive \
                                     and reopening the suite"
                                );
                                api.prevent_exit();
                                crate::desktop::wizard::ensure_suite_window(app);
                            }
                            ExitAction::CancelAndExit => {
                                info!(
                                    "no visible window remains: cancelling setup and exiting"
                                );
                                crate::desktop::wizard::cancel_provision(
                                    &app.state::<WizardState>(),
                                );
                                sidecar.stop();
                                crate::desktop::stack_stop::stop_stack();
                            }
                        }
                    }
                    _ => {}
                }
                let _ = (&app, &event);
            });
        }
        Err(e) => {
            log::error!("Failed to build BotApp: {e}");
        }
    }
}
