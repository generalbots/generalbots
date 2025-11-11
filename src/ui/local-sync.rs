use dioxus::prelude::*;
use dioxus_desktop::{use_window, LogicalSize};
use std::env;
use std::fs::{File, OpenOptions, create_dir_all};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command as ProcCommand, Child, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use notify_rust::Notification;
use serde::{Deserialize, Serialize};
use serde_json::Value;
#[derive(Debug, Clone)]
struct AppState {
    name: String,
    access_key: String,
    secret_key: String,
    status_text: String,
    sync_processes: Arc<Mutex<Vec<Child>>>,
    sync_active: Arc<Mutex<bool>>,
    sync_statuses: Arc<Mutex<Vec<SyncStatus>>>,
    show_config_dialog: bool,
    show_about_dialog: bool,
    current_screen: Screen,
}
#[derive(Debug, Clone)]
enum Screen {
    Main,
    Status,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RcloneConfig {
    name: String,
    remote_path: String,
    local_path: String,
    access_key: String,
    secret_key: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SyncStatus {
    name: String,
    status: String,
    transferred: String,
    bytes: String,
    errors: usize,
    last_updated: String,
}
#[derive(Debug, Clone)]
enum Message {
    NameChanged(String),
    AccessKeyChanged(String),
    SecretKeyChanged(String),
    SaveConfig,
    StartSync,
    StopSync,
    UpdateStatus(Vec<SyncStatus>),
    ShowConfigDialog(bool),
    ShowAboutDialog(bool),
    ShowStatusScreen,
    BackToMain,
    None,
}
fn main() {
    dioxus_desktop::launch(app);
}
fn app(cx: Scope) -> Element {
    let window = use_window();
    window.set_inner_size(LogicalSize::new(800, 600));
    let state = use_ref(cx, || AppState {
        name: String::new(),
        access_key: String::new(),
        secret_key: String::new(),
        status_text: "Enter credentials to set up sync".to_string(),
        sync_processes: Arc::new(Mutex::new(Vec::new())),
        sync_active: Arc::new(Mutex::new(false)),
        sync_statuses: Arc::new(Mutex::new(Vec::new())),
        show_config_dialog: false,
        show_about_dialog: false,
        current_screen: Screen::Main,
    });
    use_future( async move {
        let state = state.clone();
        async move {
            let mut last_check = Instant::now();
            let check_interval = Duration::from_secs(5);
            loop {
                tokio::time::sleep(Duration::from_secs(1)).await;
                if !*state.read().sync_active.lock().unwrap() {
                    continue;
                }
                if last_check.elapsed() < check_interval {
                    continue;
                }
                last_check = Instant::now();
                match read_rclone_configs() {
                    Ok(configs) => {
                        let mut new_statuses = Vec::new();
                        for config in configs {
                            match get_rclone_status(&config.name) {
                                Ok(status) => new_statuses.push(status),
                                Err(e) => eprintln!("Failed to get status: {}", e),
                            }
                        }
                        *state.write().sync_statuses.lock().unwrap() = new_statuses.clone();
                        state.write().status_text = format!("Syncing {} repositories...", new_statuses.len());
                    }
                    Err(e) => eprintln!("Failed to read configs: {}", e),
                }
            }
        }
    });
    cx.render(rsx! {
        div {
            class: "app",
            div {
                class: "menu-bar",
                button {
                    onclick: move |_| state.write().show_config_dialog = true,
                    "Add Sync Configuration"
                }
                button {
                    onclick: move |_| state.write().show_about_dialog = true,
                    "About"
                }
            }
            {match state.read().current_screen {
                Screen::Main => rsx! {
                    div {
                        class: "main-screen",
                        h1 { "General Bots" }
                        p { "{state.read().status_text}" }
                        button {
                            onclick: move |_| start_sync(&state),
                            "Start Sync"
                        }
                        button {
                            onclick: move |_| stop_sync(&state),
                            "Stop Sync"
                        }
                        button {
                            onclick: move |_| state.write().current_screen = Screen::Status,
                            "Show Status"
                        }
                    }
                },
                Screen::Status => rsx! {
                    div {
                        class: "status-screen",
                        h1 { "Sync Status" }
                        div {
                            class: "status-list",
                            for status in state.read().sync_statuses.lock().unwrap().iter() {
                                div {
                                    class: "status-item",
                                    h2 { "{status.name}" }
                                    p { "Status: {status.status}" }
                                    p { "Transferred: {status.transferred}" }
                                    p { "Bytes: {status.bytes}" }
                                    p { "Errors: {status.errors}" }
                                    p { "Last Updated: {status.last_updated}" }
                                }
                            }
                        }
                        button {
                            onclick: move |_| state.write().current_screen = Screen::Main,
                            "Back"
                        }
                    }
                }
            }}
            if state.read().show_config_dialog {
                div {
                    class: "dialog",
                    h2 { "Add Sync Configuration" }
                    input {
                        value: "{state.read().name}",
                        oninput: move |e| state.write().name = e.value.clone(),
                        placeholder: "Enter sync name",
                    }
                    input {
                        value: "{state.read().access_key}",
                        oninput: move |e| state.write().access_key = e.value.clone(),
                        placeholder: "Enter access key",
                    }
                    input {
                        value: "{state.read().secret_key}",
                        oninput: move |e| state.write().secret_key = e.value.clone(),
                        placeholder: "Enter secret key",
                    }
                    button {
                        onclick: move |_| {
                            save_config(&state);
                            state.write().show_config_dialog = false;
                        },
                        "Save"
                    }
                    button {
                        onclick: move |_| state.write().show_config_dialog = false,
                        "Cancel"
                    }
                }
            }
            if state.read().show_about_dialog {
                div {
                    class: "dialog",
                    h2 { "About General Bots" }
                    p { "Version: 1.0.0" }
                    p { "A professional-grade sync tool for OneDrive/Dropbox-like functionality." }
                    button {
                        onclick: move |_| state.write().show_about_dialog = false,
                        "Close"
                    }
                }
            }
        }
    })
}
fn save_config(state: &UseRef<AppState>) {
    if state.read().name.is_empty() || state.read().access_key.is_empty() || state.read().secret_key.is_empty() {
        state.write_with(|state| state.status_text = "All fields are required!".to_string());
        return;
    }
    let new_config = RcloneConfig {
        name: state.read().name.clone(),
        remote_path: format!("s3:
        local_path: Path::new(&env::var("HOME").unwrap()).join("General Bots").join(&state.read().name).to_string_lossy().to_string(),
        access_key: state.read().access_key.clone(),
        secret_key: state.read().secret_key.clone(),
    };
    if let Err(e) = save_rclone_config(&new_config) {
        state.write_with(|state| state.status_text = format!("Failed to save config: {}", e));
    } else {
        state.write_with(|state| state.status_text = "New sync saved!".to_string());
    }
}
fn start_sync(state: &UseRef<AppState>) {
    let mut processes = state.write_with(|state| state.sync_processes.lock().unwrap());
    processes.clear();
    match read_rclone_configs() {
        Ok(configs) => {
            for config in configs {
                match run_sync(&config) {
                    Ok(child) => processes.push(child),
                    Err(e) => eprintln!("Failed to start sync: {}", e),
                }
            }
            state.write_with(|state| *state.sync_active.lock().unwrap() = true);
            state.write_with(|state| state.status_text = format!("Syncing with {} configurations.", processes.len()));
        }
        Err(e) => state.write_with(|state| state.status_text = format!("Failed to read configurations: {}", e)),
    }
}
fn stop_sync(state: &UseRef<AppState>) {
    let mut processes = state.write_with(|state| state.sync_processes.lock().unwrap());
    for child in processes.iter_mut() {
        let _ = child.kill();
    }
    processes.clear();
    state.write_with(|state| *state.sync_active.lock().unwrap() = false);
    state.write_with(|state| state.status_text = "Sync stopped.".to_string());
}
fn save_rclone_config(config: &RcloneConfig) -> Result<(), String> {
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
fn read_rclone_configs() -> Result<Vec<RcloneConfig>, String> {
    let home_dir = env::var("HOME").map_err(|_| "HOME environment variable not set".to_string())?;
    let config_path = Path::new(&home_dir).join(".config/rclone/rclone.conf");
    if !config_path.exists() {
        return Ok(Vec::new());
    }
    let file = File::open(&config_path).map_err(|e| format!("Failed to open config file: {}", e))?;
    let reader = BufReader::new(file);
    let mut configs = Vec::new();
    let mut current_config: Option<RcloneConfig> = None;
    for line in reader.lines() {
        let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            if let Some(config) = current_config.take() {
                configs.push(config);
            }
            let name = line[1..line.len()-1].to_string();
            current_config = Some(RcloneConfig {
                name: name.clone(),
                remote_path: format!("s3:
                local_path: Path::new(&home_dir).join("General Bots").join(&name).to_string_lossy().to_string(),
                access_key: String::new(),
                secret_key: String::new(),
            });
        } else if let Some(ref mut config) = current_config {
            if let Some(pos) = line.find('=') {
                let key = line[..pos].trim().to_string();
                let value = line[pos+1..].trim().to_string();
                match key.as_str() {
                    "access_key_id" => config.access_key = value,
                    "secret_access_key" => config.secret_key = value,
                    _ => {}
                }
            }
        }
    }
    if let Some(config) = current_config {
        configs.push(config);
    }
    Ok(configs)
}
fn run_sync(config: &RcloneConfig) -> Result<Child, std::io::Error> {
    let local_path = Path::new(&config.local_path);
    if !local_path.exists() {
        create_dir_all(local_path)?;
    }
    ProcCommand::new("rclone")
        .arg("sync")
        .arg(&config.remote_path)
        .arg(&config.local_path)
        .arg("--no-check-certificate")
        .arg("--verbose")
        .arg("--rc")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
}
fn get_rclone_status(remote_name: &str) -> Result<SyncStatus, String> {
    let output = ProcCommand::new("rclone")
        .arg("rc")
        .arg("core/stats")
        .arg("--json")
        .output()
        .map_err(|e| format!("Failed to execute rclone rc: {}", e))?;
    if !output.status.success() {
        return Err(format!("rclone rc failed: {}", String::from_utf8_lossy(&output.stderr)));
    }
    let json = String::from_utf8_lossy(&output.stdout);
    let parsed: Result<Value, _> = serde_json::from_str(&json);
    match parsed {
        Ok(value) => {
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
                name: remote_name.to_string(),
                status,
                transferred: format_bytes(transferred),
                bytes: format!("{}/s", format_bytes(speed as u64)),
                errors: errors as usize,
                last_updated: chrono::Local::now().format("%H:%M:%S").to_string(),
            })
        }
        Err(e) => Err(format!("Failed to parse rclone status: {}", e)),
    }
}
fn format_bytes(bytes: u64) -> String {
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