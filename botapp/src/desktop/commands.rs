//! Tray and service commands shared by the desktop shells.
//!
//! This module is compiled only for non-mobile builds. The mobile thin client
//! exposes a single `open_external_url` command and never reaches this file.

use crate::desktop::tray::{RunningMode, ServiceMonitor, ServiceStatus, TrayEvent, TrayManager};

#[tauri::command]
pub async fn get_tray_status(tray: tauri::State<'_, TrayManager>) -> Result<bool, String> {
    Ok(tray.is_active().await)
}

#[tauri::command]
pub async fn start_tray(
    tray: tauri::State<'_, TrayManager>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    tray.start(&app).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_tray(tray: tauri::State<'_, TrayManager>) -> Result<(), String> {
    tray.stop().await;
    Ok(())
}

#[tauri::command]
pub async fn show_notification(
    tray: tauri::State<'_, TrayManager>,
    title: String,
    body: String,
) -> Result<(), String> {
    tray.show_notification(&title, &body)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_tray_status(
    tray: tauri::State<'_, TrayManager>,
    status: String,
) -> Result<(), String> {
    tray.update_status(&status).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_tray_tooltip(
    tray: tauri::State<'_, TrayManager>,
    tooltip: String,
) -> Result<(), String> {
    tray.set_tooltip(&tooltip).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_tray_hostname(tray: tauri::State<'_, TrayManager>) -> Result<Option<String>, String> {
    Ok(tray.get_hostname().await)
}

#[tauri::command]
pub async fn set_tray_hostname(
    tray: tauri::State<'_, TrayManager>,
    hostname: String,
) -> Result<(), String> {
    tray.set_hostname(hostname).await;
    Ok(())
}

#[tauri::command]
pub fn handle_tray_event(
    app: tauri::AppHandle,
    tray: tauri::State<'_, TrayManager>,
    event: String,
) -> Result<(), String> {
    let tray_event = match event.as_str() {
        "new_chat" | "new-chat" | "newchat" => TrayEvent::NewChat,
        "files" => TrayEvent::Files,
        "open" => TrayEvent::Open,
        "settings" => TrayEvent::Settings,
        "about" => TrayEvent::About,
        "quit" => TrayEvent::Quit,
        _ => return Err(format!("Unknown event: {event}")),
    };
    tray.handle_event(&app, tray_event);
    Ok(())
}

#[tauri::command]
pub async fn check_services(
    monitor: tauri::State<'_, tokio::sync::Mutex<ServiceMonitor>>,
) -> Result<Vec<ServiceStatus>, String> {
    let mut guard = monitor.lock().await;
    let result = guard.check_services().await;
    drop(guard);
    Ok(result)
}

#[tauri::command]
pub async fn add_service(
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
pub async fn get_service(
    monitor: tauri::State<'_, tokio::sync::Mutex<ServiceMonitor>>,
    name: String,
) -> Result<Option<ServiceStatus>, String> {
    let guard = monitor.lock().await;
    let result = guard.get_service(&name).cloned();
    drop(guard);
    Ok(result)
}

#[tauri::command]
pub async fn all_services_running(
    monitor: tauri::State<'_, tokio::sync::Mutex<ServiceMonitor>>,
) -> Result<bool, String> {
    let guard = monitor.lock().await;
    let result = guard.all_running();
    drop(guard);
    Ok(result)
}

#[tauri::command]
pub async fn any_service_running(
    monitor: tauri::State<'_, tokio::sync::Mutex<ServiceMonitor>>,
) -> Result<bool, String> {
    let guard = monitor.lock().await;
    let result = guard.any_running();
    drop(guard);
    Ok(result)
}

#[tauri::command]
pub fn get_tray_mode(tray: tauri::State<'_, TrayManager>) -> String {
    tray.get_mode_string()
}

#[tauri::command]
pub fn get_running_modes() -> Vec<&'static str> {
    vec!["Server", "Desktop", "Client"]
}

#[tauri::command]
pub fn create_tray_with_mode(mode: String) -> Result<String, String> {
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
