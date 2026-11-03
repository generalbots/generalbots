#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use log::info;
use tauri::Manager;

mod desktop;

#[cfg(feature = "desktop-local")]
use desktop::sidecar::Sidecar;
#[cfg(feature = "desktop-local")]
use desktop::wizard::WizardState;
use desktop::tray::{RunningMode, ServiceMonitor, TrayEvent, TrayManager};

#[tauri::command]
async fn get_tray_status(tray: tauri::State<'_, TrayManager>) -> Result<bool, String> {
    Ok(tray.is_active().await)
}

#[tauri::command]
async fn start_tray(
    tray: tauri::State<'_, TrayManager>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    tray.start(&app).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn stop_tray(tray: tauri::State<'_, TrayManager>) -> Result<(), String> {
    tray.stop().await;
    Ok(())
}

#[tauri::command]
async fn show_notification(
    tray: tauri::State<'_, TrayManager>,
    title: String,
    body: String,
) -> Result<(), String> {
    tray.show_notification(&title, &body)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_tray_status(
    tray: tauri::State<'_, TrayManager>,
    status: String,
) -> Result<(), String> {
    tray.update_status(&status).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_tray_tooltip(
    tray: tauri::State<'_, TrayManager>,
    tooltip: String,
) -> Result<(), String> {
    tray.set_tooltip(&tooltip).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_tray_hostname(tray: tauri::State<'_, TrayManager>) -> Result<Option<String>, String> {
    Ok(tray.get_hostname().await)
}

#[tauri::command]
async fn set_tray_hostname(
    tray: tauri::State<'_, TrayManager>,
    hostname: String,
) -> Result<(), String> {
    tray.set_hostname(hostname).await;
    Ok(())
}

#[tauri::command]
fn handle_tray_event(tray: tauri::State<'_, TrayManager>, event: String) -> Result<(), String> {
    let tray_event = match event.as_str() {
        "open" => TrayEvent::Open,
        "settings" => TrayEvent::Settings,
        "about" => TrayEvent::About,
        "quit" => TrayEvent::Quit,
        _ => return Err(format!("Unknown event: {event}")),
    };
    tray.handle_event(tray_event);
    Ok(())
}

#[tauri::command]
async fn check_services(
    monitor: tauri::State<'_, tokio::sync::Mutex<ServiceMonitor>>,
) -> Result<Vec<desktop::tray::ServiceStatus>, String> {
    let mut guard = monitor.lock().await;
    let result = guard.check_services().await;
    drop(guard);
    Ok(result)
}

#[tauri::command]
async fn add_service(
    monitor: tauri::State<'_, tokio::sync::Mutex<ServiceMonitor>>,
    name: String,
    port: u16,
) -> Result<(), String> {
    let mut guard = monitor.lock().await;
    guard.add_service(&name, port);
    drop(guard);
    Ok(())
}

#[tauri::command]
async fn get_service(
    monitor: tauri::State<'_, tokio::sync::Mutex<ServiceMonitor>>,
    name: String,
) -> Result<Option<desktop::tray::ServiceStatus>, String> {
    let guard = monitor.lock().await;
    let result = guard.get_service(&name).cloned();
    drop(guard);
    Ok(result)
}

#[tauri::command]
async fn all_services_running(
    monitor: tauri::State<'_, tokio::sync::Mutex<ServiceMonitor>>,
) -> Result<bool, String> {
    let guard = monitor.lock().await;
    let result = guard.all_running();
    drop(guard);
    Ok(result)
}

#[tauri::command]
async fn any_service_running(
    monitor: tauri::State<'_, tokio::sync::Mutex<ServiceMonitor>>,
) -> Result<bool, String> {
    let guard = monitor.lock().await;
    let result = guard.any_running();
    drop(guard);
    Ok(result)
}

#[tauri::command]
fn get_tray_mode(tray: tauri::State<'_, TrayManager>) -> String {
    tray.get_mode_string()
}

#[tauri::command]
fn get_running_modes() -> Vec<&'static str> {
    vec!["Server", "Desktop", "Client"]
}

#[tauri::command]
fn create_tray_with_mode(mode: String) -> Result<String, String> {
    let running_mode = match mode.to_lowercase().as_str() {
        "server" => RunningMode::Server,
        "desktop" => RunningMode::Desktop,
        "client" => RunningMode::Client,
        _ => {
            return Err(format!(
                "Invalid mode: {mode}. Use Server, Desktop, or Client"
            ))
        }
    };
    let manager = TrayManager::with_mode(running_mode);
    Ok(manager.get_mode_string())
}

/// Starts the botserver sidecar and reveals the suite once the API answers.
#[cfg(feature = "desktop-local")]
fn start_sidecar_and_show(app: &tauri::App) {
    let resource_dir = app.path().resource_dir().ok();

    let handle = app.handle().clone();
    tauri::async_runtime::spawn(async move {
        // A previous run may have left the sidecar alive (for example when the
        // window was closed without a graceful exit). Reuse it instead of
        // starting a second listener on the same port.
        let already_running = Sidecar::wait_for_api("http://localhost:8080/api/product", 2).await;
        if !already_running {
            let sidecar = handle.state::<Sidecar>().inner().clone();
            if let Err(e) = sidecar.start(resource_dir.clone()) {
                log::error!("Failed to start botserver sidecar: {e}");
            }
        }
        let ready = Sidecar::wait_for_api("http://localhost:8080/api/product", 120).await;
        if !ready {
            log::error!("botserver did not respond within 120s");
        }
        desktop::wizard::reveal_suite(&handle);
    });
}

fn main() {
    if let Some(code) = desktop::payload::handle_cli() {
        std::process::exit(code);
    }
    botlib::logging::init_compact_logger("info");
    let version = env!("CARGO_PKG_VERSION");
    info!("BotApp {version} starting...");

    let tray_manager = TrayManager::with_mode(RunningMode::Desktop);
    let service_monitor = tokio::sync::Mutex::new(ServiceMonitor::new());

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(tray_manager)
        .manage(service_monitor);

    #[cfg(feature = "desktop-local")]
    let builder = builder
        .manage(Sidecar::new())
        .manage(WizardState::default());

    #[cfg(feature = "desktop-local")]
    let builder = builder.invoke_handler(tauri::generate_handler![
        desktop::drive::list_files,
        desktop::drive::upload_file,
        desktop::drive::create_folder,
        desktop::drive::delete_path,
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
        desktop::wizard::wizard_context,
        desktop::wizard::wizard_save_draft,
        desktop::wizard::wizard_load_draft,
        desktop::wizard::wizard_check_workspace,
        desktop::wizard::wizard_pick_folder,
        desktop::wizard::wizard_test_model,
        desktop::wizard::wizard_provision,
        desktop::wizard::wizard_cancel_provision,
        desktop::wizard::wizard_provision_status,
        desktop::wizard::wizard_reset_stack,
        desktop::wizard::wizard_open_folder,
        desktop::wizard::wizard_open_url,
        desktop::wizard::wizard_finish,
    ]);

    #[cfg(not(feature = "desktop-local"))]
    let builder = builder.invoke_handler(tauri::generate_handler![
        desktop::drive::list_files,
        desktop::drive::upload_file,
        desktop::drive::create_folder,
        desktop::drive::delete_path,
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
    ]);

    let app = builder
        .setup(|app| {
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
                    // Closing the wizard must not take the suite down with it:
                    // the main window is already answering, so keep the
                    // process (and the sidecar) alive until it is gone.
                    if app.get_webview_window("main").is_some() {
                        info!("keeping the app alive: the suite window still exists");
                        api.prevent_exit();
                    } else if desktop::activity::is_busy() {
                        // #1620 — a reply is still being generated. Killing the
                        // sidecar here truncated real answers (and the tool
                        // call that writes a PDF). Keep the process alive and
                        // bring the suite back so the result is visible.
                        info!("a reply is in progress: keeping the app alive and reopening the suite");
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
