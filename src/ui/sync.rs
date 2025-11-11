use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::process::{Command, Stdio};
use std::path::Path;
use std::fs::{OpenOptions, create_dir_all};
use std::io::Write;
use std::env;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RcloneConfig {
    name: String,
    remote_path: String,
    local_path: String,
    access_key: String,
    secret_key: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    name: String,
    status: String,
    transferred: String,
    bytes: String,
    errors: usize,
    last_updated: String,
}
pub(crate) struct AppState {
    pub sync_processes: Mutex<Vec<std::process::Child>>,
    pub sync_active: Mutex<bool>,
}
#[tauri::command]
pub fn save_config(config: RcloneConfig) -> Result<(), String> {
    let home_dir = env::var("HOME").map_err(|_| "HOME environment variable not set".to_string())?;
    let config_path = Path::new(&home_dir).join(".config/rclone/rclone.conf");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&config_path)
        .map_err(|e| format!("Failed to open config file: {}", e))?;
    writeln!(file, "[{}]", config.name)
        .and_then(|_| writeln!(file, "type = s3"))
        .and_then(|_| writeln!(file, "provider = Other"))
        .and_then(|_| writeln!(file, "access_key_id = {}", config.access_key))
        .and_then(|_| writeln!(file, "secret_access_key = {}", config.secret_key))
        .and_then(|_| writeln!(file, "endpoint = https:
        .and_then(|_| writeln!(file, "acl = private"))
        .map_err(|e| format!("Failed to write config: {}", e))
}
#[tauri::command]
pub fn start_sync(config: RcloneConfig, state: tauri::State<AppState>) -> Result<(), String> {
    let local_path = Path::new(&config.local_path);
    if !local_path.exists() {
        create_dir_all(local_path).map_err(|e| format!("Failed to create local path: {}", e))?;
    }
    let child = Command::new("rclone")
        .arg("sync")
        .arg(&config.remote_path)
        .arg(&config.local_path)
        .arg("--no-check-certificate")
        .arg("--verbose")
        .arg("--rc")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to start rclone: {}", e))?;
    state.sync_processes.lock().unwrap().push(child);
    *state.sync_active.lock().unwrap() = true;
    Ok(())
}
#[tauri::command]
pub fn stop_sync(state: tauri::State<AppState>) -> Result<(), String> {
    let mut processes = state.sync_processes.lock().unwrap();
    for child in processes.iter_mut() {
        child.kill().map_err(|e| format!("Failed to kill process: {}", e))?;
    }
    processes.clear();
    *state.sync_active.lock().unwrap() = false;
    Ok(())
}
#[tauri::command]
pub fn get_status(remote_name: String) -> Result<SyncStatus, String> {
    let output = Command::new("rclone")
        .arg("rc")
        .arg("core/stats")
        .arg("--json")
        .output()
        .map_err(|e| format!("Failed to execute rclone rc: {}", e))?;
    if !output.status.success() {
        return Err(format!("rclone rc failed: {}", String::from_utf8_lossy(&output.stderr)));
    }
    let json = String::from_utf8_lossy(&output.stdout);
    let value: serde_json::Value = serde_json::from_str(&json)
        .map_err(|e| format!("Failed to parse rclone status: {}", e))?;
    let transferred = value.get("bytes").and_then(|v| v.as_u64()).unwrap_or(0);
    let errors = value.get("errors").and_then(|v| v.as_u64()).unwrap_or(0);
    let speed = value.get("speed").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let status = if errors > 0 {
        "Error occurred".to_string()
    } else if speed > 0.0 {
        "Transferring".to_string()
    } else if transferred > 0 {
        "Completed".to_string()
    } else {
        "Initializing".to_string()
    };
    Ok(SyncStatus {
        name: remote_name,
        status,
        transferred: format_bytes(transferred),
        bytes: format!("{}/s", format_bytes(speed as u64)),
        errors: errors as usize,
        last_updated: chrono::Local::now().format("%H:%M:%S").to_string(),
    })
}
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
