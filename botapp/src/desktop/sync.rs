use super::safe_command::SafeCommand;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::{Child, Stdio};
use std::sync::Mutex;
use tauri::{Emitter, Window};

static RCLONE_PROCESS: Mutex<Option<Child>> = Mutex::new(None);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub status: String,
    pub is_running: bool,
    pub last_sync: Option<String>,
    pub files_synced: u64,
    pub bytes_transferred: u64,
    pub current_file: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub local_path: String,
    pub remote_name: String,
    pub remote_path: String,
    pub sync_mode: SyncMode,
    pub exclude_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncMode {
    Push,
    Pull,
    Bisync,
}

impl Default for SyncConfig {
    fn default() -> Self {
        let local_path = dirs::home_dir().map_or_else(
            || "~/GeneralBots".to_string(),
            |p| p.join("GeneralBots").to_string_lossy().to_string(),
        );
        Self {
            local_path,
            remote_name: "gbdrive".to_string(),
            remote_path: "/".to_string(),
            sync_mode: SyncMode::Bisync,
            exclude_patterns: vec![
                ".DS_Store".to_string(),
                "Thumbs.db".to_string(),
                "*.tmp".to_string(),
                ".git/**".to_string(),
            ],
        }
    }
}

#[tauri::command]
#[must_use]
pub fn get_sync_status() -> SyncStatus {
    let process_guard = RCLONE_PROCESS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let is_running = process_guard.is_some();
    drop(process_guard);

    SyncStatus {
        status: if is_running {
            "syncing".to_string()
        } else {
            "idle".to_string()
        },
        is_running,
        last_sync: None,
        files_synced: 0,
        bytes_transferred: 0,
        current_file: None,
        error: None,
    }
}

#[tauri::command]
pub fn start_sync(window: Window, config: Option<SyncConfig>) -> Result<SyncStatus, String> {
    let config = config.unwrap_or_default();

    {
        let process_guard = RCLONE_PROCESS
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if process_guard.is_some() {
            return Err("Sync already running".to_string());
        }
    }

    let local_path = PathBuf::from(&config.local_path);
    if !local_path.exists() {
        std::fs::create_dir_all(&local_path)
            .map_err(|e| format!("Failed to create local directory: {e}"))?;
    }

    let remote_spec = format!("{}:{}", config.remote_name, config.remote_path);

    let cmd_result = match config.sync_mode {
        SyncMode::Push => SafeCommand::new("rclone")
            .and_then(|c| c.arg("sync"))
            .and_then(|c| c.arg(&config.local_path))
            .and_then(|c| c.arg(&remote_spec)),
        SyncMode::Pull => SafeCommand::new("rclone")
            .and_then(|c| c.arg("sync"))
            .and_then(|c| c.arg(&remote_spec))
            .and_then(|c| c.arg(&config.local_path)),
        SyncMode::Bisync => SafeCommand::new("rclone")
            .and_then(|c| c.arg("bisync"))
            .and_then(|c| c.arg(&config.local_path))
            .and_then(|c| c.arg(&remote_spec))
            .and_then(|c| c.arg("--resync")),
    };

    let mut cmd_builder = cmd_result
        .and_then(|c| c.arg("--progress"))
        .and_then(|c| c.arg("--verbose"))
        .and_then(|c| c.arg("--checksum"))
        .map_err(|e| format!("Failed to build rclone command: {e}"))?;

    for pattern in &config.exclude_patterns {
        cmd_builder = cmd_builder
            .arg("--exclude")
            .and_then(|c| c.arg(pattern))
            .map_err(|e| format!("Invalid exclude pattern: {e}"))?;
    }

    let child = cmd_builder
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            let err_str = e.to_string();
            if err_str.contains("NotFound") || err_str.contains("not found") {
                "rclone not found. Please install rclone: https://rclone.org/install/".to_string()
            } else {
                format!("Failed to start rclone: {e}")
            }
        })?;

    {
        let mut process_guard = RCLONE_PROCESS
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *process_guard = Some(child);
    }

    let _ = window.emit("sync_started", ());

    std::thread::spawn(move || {
        monitor_sync_process(&window);
    });

    Ok(SyncStatus {
        status: "syncing".to_string(),
        is_running: true,
        last_sync: None,
        files_synced: 0,
        bytes_transferred: 0,
        current_file: None,
        error: None,
    })
}

#[tauri::command]
pub fn stop_sync() -> Result<SyncStatus, String> {
    let mut process_guard = RCLONE_PROCESS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    process_guard
        .take()
        .ok_or_else(|| "No sync process running".to_string())
        .map(|mut child| {
            let _ = child.kill();
            std::thread::sleep(std::time::Duration::from_millis(500));
            let _ = child.wait();

            SyncStatus {
                status: "stopped".to_string(),
                is_running: false,
                last_sync: Some(chrono::Utc::now().to_rfc3339()),
                files_synced: 0,
                bytes_transferred: 0,
                current_file: None,
                error: None,
            }
        })
}

#[tauri::command]
pub fn configure_remote(
    remote_name: &str,
    endpoint: &str,
    access_key: &str,
    secret_key: &str,
    bucket: &str,
) -> Result<(), String> {
    let output = SafeCommand::new("rclone")
        .and_then(|c| c.arg("config"))
        .and_then(|c| c.arg("create"))
        .and_then(|c| c.arg(remote_name))
        .and_then(|c| c.arg("s3"))
        .and_then(|c| c.arg("provider"))
        .and_then(|c| c.arg("Minio"))
        .and_then(|c| c.arg("endpoint"))
        .and_then(|c| c.arg(endpoint))
        .and_then(|c| c.arg("access_key_id"))
        .and_then(|c| c.arg(access_key))
        .and_then(|c| c.arg("secret_access_key"))
        .and_then(|c| c.arg(secret_key))
        .and_then(|c| c.arg("acl"))
        .and_then(|c| c.arg("private"))
        .map_err(|e| format!("Failed to build rclone command: {e}"))?
        .output()
        .map_err(|e| format!("Failed to configure rclone: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("rclone config failed: {stderr}"));
    }

    let _ = SafeCommand::new("rclone")
        .and_then(|c| c.arg("config"))
        .and_then(|c| c.arg("update"))
        .and_then(|c| c.arg(remote_name))
        .and_then(|c| c.arg("bucket"))
        .and_then(|c| c.arg(bucket))
        .and_then(|c| c.output());

    Ok(())
}

#[tauri::command]
pub fn check_rclone_installed() -> Result<String, String> {
    let output = SafeCommand::new("rclone")
        .and_then(|c| c.arg("version"))
        .map_err(|e| format!("Failed to build rclone command: {e}"))?
        .output()
        .map_err(|e| {
            let err_str = e.to_string();
            if err_str.contains("NotFound") || err_str.contains("not found") {
                "rclone not installed".to_string()
            } else {
                format!("Error checking rclone: {e}")
            }
        })?;

    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout);
        let first_line = version.lines().next().unwrap_or("unknown");
        Ok(first_line.to_string())
    } else {
        Err("rclone check failed".to_string())
    }
}

#[tauri::command]
pub fn list_remotes() -> Result<Vec<String>, String> {
    let output = SafeCommand::new("rclone")
        .and_then(|c| c.arg("listremotes"))
        .map_err(|e| format!("Failed to build rclone command: {e}"))?
        .output()
        .map_err(|e| format!("Failed to list remotes: {e}"))?;

    if output.status.success() {
        let remotes = String::from_utf8_lossy(&output.stdout);
        Ok(remotes
            .lines()
            .map(|s| s.trim_end_matches(':').to_string())
            .filter(|s| !s.is_empty())
            .collect())
    } else {
        Err("Failed to list rclone remotes".to_string())
    }
}

#[tauri::command]
#[must_use]
pub fn get_sync_folder() -> String {
    dirs::home_dir().map_or_else(
        || "~/GeneralBots".to_string(),
        |p| p.join("GeneralBots").to_string_lossy().to_string(),
    )
}

#[tauri::command]
pub fn set_sync_folder(path: &str) -> Result<(), String> {
    let path = PathBuf::from(path);

    if !path.exists() {
        std::fs::create_dir_all(&path).map_err(|e| format!("Failed to create directory: {e}"))?;
    }

    if !path.is_dir() {
        return Err("Path is not a directory".to_string());
    }

    Ok(())
}

fn monitor_sync_process(window: &Window) {
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));

        let mut process_guard = RCLONE_PROCESS
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let status_opt = if let Some(ref mut child) = *process_guard {
            match child.try_wait() {
                Ok(Some(exit_status)) => {
                    let success = exit_status.success();
                    *process_guard = None;
                    drop(process_guard);

                    let status = SyncStatus {
                        status: if success {
                            "completed".to_string()
                        } else {
                            "error".to_string()
                        },
                        is_running: false,
                        last_sync: Some(chrono::Utc::now().to_rfc3339()),
                        files_synced: 0,
                        bytes_transferred: 0,
                        current_file: None,
                        error: if success {
                            None
                        } else {
                            Some(format!("Exit code: {:?}", exit_status.code()))
                        },
                    };

                    if success {
                        let _ = window.emit("sync_completed", &status);
                    } else {
                        let _ = window.emit("sync_error", &status);
                    }
                    return;
                }
                Ok(None) => {
                    drop(process_guard);
                    Some(SyncStatus {
                        status: "syncing".to_string(),
                        is_running: true,
                        last_sync: None,
                        files_synced: 0,
                        bytes_transferred: 0,
                        current_file: None,
                        error: None,
                    })
                }
                Err(e) => {
                    *process_guard = None;
                    drop(process_guard);

                    let status = SyncStatus {
                        status: "error".to_string(),
                        is_running: false,
                        last_sync: Some(chrono::Utc::now().to_rfc3339()),
                        files_synced: 0,
                        bytes_transferred: 0,
                        current_file: None,
                        error: Some(format!("Process error: {e}")),
                    };
                    let _ = window.emit("sync_error", &status);
                    return;
                }
            }
        } else {
            drop(process_guard);
            return;
        };

        if let Some(status) = status_opt {
            let _ = window.emit("sync_progress", &status);
        }
    }
}
