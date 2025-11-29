use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RcloneConfig {
    name: String,
    remote_path: String,
    local_path: String,
    access_key: String,
    secret_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotSyncConfig {
    bot_id: String,
    bot_name: String,
    bucket_name: String,
    sync_path: String,
    local_path: PathBuf,
    role: SyncRole,
    enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncRole {
    Admin,    // Full bucket access
    User,     // Home directory only
    ReadOnly, // Read-only access
}

impl BotSyncConfig {
    pub fn new(bot_name: &str, username: &str, role: SyncRole) -> Self {
        let bucket_name = format!("{}.gbdrive", bot_name);
        let (sync_path, local_path) = match role {
            SyncRole::Admin => (
                "/".to_string(),
                PathBuf::from(env::var("HOME").unwrap_or_default())
                    .join("BotSync")
                    .join(bot_name)
                    .join("admin"),
            ),
            SyncRole::User => (
                format!("/home/{}", username),
                PathBuf::from(env::var("HOME").unwrap_or_default())
                    .join("BotSync")
                    .join(bot_name)
                    .join(username),
            ),
            SyncRole::ReadOnly => (
                format!("/home/{}", username),
                PathBuf::from(env::var("HOME").unwrap_or_default())
                    .join("BotSync")
                    .join(bot_name)
                    .join(format!("{}-readonly", username)),
            ),
        };

        Self {
            bot_id: format!("{}-{}", bot_name, username),
            bot_name: bot_name.to_string(),
            bucket_name,
            sync_path,
            local_path,
            role,
            enabled: true,
        }
    }

    pub fn get_rclone_remote_name(&self) -> String {
        format!("{}_{}", self.bot_name, self.bot_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSyncProfile {
    username: String,
    bot_configs: Vec<BotSyncConfig>,
}

impl UserSyncProfile {
    pub fn new(username: String) -> Self {
        Self {
            username,
            bot_configs: Vec::new(),
        }
    }

    pub fn add_bot(&mut self, bot_name: &str, role: SyncRole) {
        let config = BotSyncConfig::new(bot_name, &self.username, role);
        self.bot_configs.push(config);
    }

    pub fn remove_bot(&mut self, bot_name: &str) {
        self.bot_configs.retain(|c| c.bot_name != bot_name);
    }

    pub fn get_active_configs(&self) -> Vec<&BotSyncConfig> {
        self.bot_configs.iter().filter(|c| c.enabled).collect()
    }
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
    pub sync_processes: Mutex<HashMap<String, std::process::Child>>,
    pub sync_active: Mutex<HashMap<String, bool>>,
    pub user_profile: Mutex<Option<UserSyncProfile>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            sync_processes: Mutex::new(HashMap::new()),
            sync_active: Mutex::new(HashMap::new()),
            user_profile: Mutex::new(None),
        }
    }
}
#[tauri::command]
pub fn load_user_profile(
    username: String,
    state: tauri::State<AppState>,
) -> Result<UserSyncProfile, String> {
    let config_path = PathBuf::from(env::var("HOME").unwrap_or_default())
        .join(".config")
        .join("botsync")
        .join(format!("{}.json", username));

    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read profile: {}", e))?;
        let profile: UserSyncProfile = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse profile: {}", e))?;

        let mut user_profile = state.user_profile.lock().unwrap();
        *user_profile = Some(profile.clone());
        Ok(profile)
    } else {
        let profile = UserSyncProfile::new(username);
        let mut user_profile = state.user_profile.lock().unwrap();
        *user_profile = Some(profile.clone());
        Ok(profile)
    }
}

#[tauri::command]
pub fn save_user_profile(
    profile: UserSyncProfile,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    let config_dir = PathBuf::from(env::var("HOME").unwrap_or_default())
        .join(".config")
        .join("botsync");

    create_dir_all(&config_dir).map_err(|e| format!("Failed to create config dir: {}", e))?;

    let config_path = config_dir.join(format!("{}.json", profile.username));
    let content = serde_json::to_string_pretty(&profile)
        .map_err(|e| format!("Failed to serialize profile: {}", e))?;

    std::fs::write(&config_path, content).map_err(|e| format!("Failed to save profile: {}", e))?;

    let mut user_profile = state.user_profile.lock().unwrap();
    *user_profile = Some(profile);

    Ok(())
}

#[tauri::command]
pub fn save_bot_config(
    bot_config: BotSyncConfig,
    credentials: HashMap<String, String>,
) -> Result<(), String> {
    let home_dir = env::var("HOME").map_err(|_| "HOME environment variable not set".to_string())?;
    let config_path = Path::new(&home_dir).join(".config/rclone/rclone.conf");

    create_dir_all(config_path.parent().unwrap())
        .map_err(|e| format!("Failed to create config directory: {}", e))?;

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&config_path)
        .map_err(|e| format!("Failed to open config file: {}", e))?;

    let remote_name = bot_config.get_rclone_remote_name();
    let endpoint = credentials
        .get("endpoint")
        .unwrap_or(&"https://localhost:9000".to_string());
    let access_key = credentials.get("access_key").unwrap_or(&"".to_string());
    let secret_key = credentials.get("secret_key").unwrap_or(&"".to_string());

    writeln!(file, "[{}]", remote_name)
        .and_then(|_| writeln!(file, "type = s3"))
        .and_then(|_| writeln!(file, "provider = Minio"))
        .and_then(|_| writeln!(file, "access_key_id = {}", access_key))
        .and_then(|_| writeln!(file, "secret_access_key = {}", secret_key))
        .and_then(|_| writeln!(file, "endpoint = {}", endpoint))
        .and_then(|_| writeln!(file, "region = us-east-1"))
        .and_then(|_| writeln!(file, "no_check_bucket = true"))
        .and_then(|_| writeln!(file, "force_path_style = true"))
        .map_err(|e| format!("Failed to write config: {}", e))
}
#[tauri::command]
pub fn start_bot_sync(
    bot_config: BotSyncConfig,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    if !bot_config.local_path.exists() {
        create_dir_all(&bot_config.local_path)
            .map_err(|e| format!("Failed to create local path: {}", e))?;
    }

    let remote_name = bot_config.get_rclone_remote_name();
    let remote_path = format!(
        "{}:{}{}",
        remote_name, bot_config.bucket_name, bot_config.sync_path
    );

    let mut cmd = Command::new("rclone");
    cmd.arg("sync")
        .arg(&remote_path)
        .arg(&bot_config.local_path)
        .arg("--no-check-certificate")
        .arg("--verbose")
        .arg("--rc");

    // Add read-only flag if needed
    if matches!(bot_config.role, SyncRole::ReadOnly) {
        cmd.arg("--read-only");
    }

    let child = cmd
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to start rclone: {}", e))?;

    let mut processes = state.sync_processes.lock().unwrap();
    processes.insert(bot_config.bot_id.clone(), child);

    let mut active = state.sync_active.lock().unwrap();
    active.insert(bot_config.bot_id.clone(), true);

    Ok(())
}

#[tauri::command]
pub fn start_all_syncs(state: tauri::State<AppState>) -> Result<(), String> {
    let profile = state
        .user_profile
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| "No user profile loaded".to_string())?;

    for config in profile.get_active_configs() {
        if let Err(e) = start_bot_sync(config.clone(), state.clone()) {
            log::error!("Failed to start sync for {}: {}", config.bot_name, e);
        }
    }

    Ok(())
}
#[tauri::command]
pub fn stop_bot_sync(bot_id: String, state: tauri::State<AppState>) -> Result<(), String> {
    let mut processes = state.sync_processes.lock().unwrap();
    if let Some(mut child) = processes.remove(&bot_id) {
        child
            .kill()
            .map_err(|e| format!("Failed to kill process: {}", e))?;
    }

    let mut active = state.sync_active.lock().unwrap();
    active.remove(&bot_id);

    Ok(())
}

#[tauri::command]
pub fn stop_all_syncs(state: tauri::State<AppState>) -> Result<(), String> {
    let mut processes = state.sync_processes.lock().unwrap();
    for (_, mut child) in processes.drain() {
        let _ = child.kill();
    }

    let mut active = state.sync_active.lock().unwrap();
    active.clear();

    Ok(())
}
#[tauri::command]
pub fn get_bot_sync_status(
    bot_id: String,
    state: tauri::State<AppState>,
) -> Result<SyncStatus, String> {
    let active = state.sync_active.lock().unwrap();
    if !active.contains_key(&bot_id) {
        return Err("Sync not active".to_string());
    }

    let output = Command::new("rclone")
        .arg("rc")
        .arg("core/stats")
        .arg("--json")
        .output()
        .map_err(|e| format!("Failed to execute rclone rc: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "rclone rc failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let json = String::from_utf8_lossy(&output.stdout);
    let value: serde_json::Value =
        serde_json::from_str(&json).map_err(|e| format!("Failed to parse rclone status: {}", e))?;

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
        name: bot_id,
        status,
        transferred: format_bytes(transferred),
        bytes: format!("{}/s", format_bytes(speed as u64)),
        errors: errors as usize,
        last_updated: chrono::Local::now().format("%H:%M:%S").to_string(),
    })
}

#[tauri::command]
pub fn get_all_sync_statuses(state: tauri::State<AppState>) -> Result<Vec<SyncStatus>, String> {
    let active = state.sync_active.lock().unwrap();
    let mut statuses = Vec::new();

    for bot_id in active.keys() {
        match get_bot_sync_status(bot_id.clone(), state.clone()) {
            Ok(status) => statuses.push(status),
            Err(e) => log::warn!("Failed to get status for {}: {}", bot_id, e),
        }
    }

    Ok(statuses)
}
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if bytes >= GB {
        format!("{:.2} GB ", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB ", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB ", bytes as f64 / KB as f64)
    } else {
        format!("{} B ", bytes)
    }
}
