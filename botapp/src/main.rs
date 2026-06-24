#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use log::info;
use tauri::Manager;

mod desktop;

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

fn main() {
    botlib::logging::init_compact_logger("info");
    let version = env!("CARGO_PKG_VERSION");
    info!("BotApp {version} starting...");

    let tray_manager = TrayManager::with_mode(RunningMode::Desktop);
    let service_monitor = tokio::sync::Mutex::new(ServiceMonitor::new());

    let builder_result = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(tray_manager)
        .manage(service_monitor)
        .invoke_handler(tauri::generate_handler![
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
        ])
        .setup(|app| {
            let tray = app.state::<TrayManager>();
            let mode = tray.get_mode_string();
            info!("BotApp setup complete in {mode} mode");

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
        .run(tauri::generate_context!());

    if let Err(e) = builder_result {
        log::error!("Failed to run BotApp: {e}");
    }
}
