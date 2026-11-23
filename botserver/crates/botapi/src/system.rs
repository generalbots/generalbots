use axum::{
    extract::State,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use botcore::shared::state::AppState;

#[path = "system_setup.rs"]
mod system_setup;

pub use system_setup::{
    configure_setup, get_setup_status, ConfigureSetupData, ConfigureSetupRequest,
    ConfigureSetupResponse, SetupStatusResponse,
};

pub fn configure_system_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/system/versions", get(get_versions))
        .route("/api/system/check-updates", post(check_updates))
        .route("/api/system/usage", get(get_system_usage))
        .route("/api/setup/status", get(get_setup_status))
        .route("/api/setup/configure", post(configure_setup))
}

/// Live host resource usage for the desktop Resources widget (#1150).
///
/// Values are best-effort: any unreadable `/proc` entry degrades to `0`
/// instead of failing the request, so the widget keeps working on systems
/// where the platform data is unavailable (containers, Windows, sandboxes).
#[derive(Serialize)]
pub struct SystemUsage {
    /// CPU usage percentage (0-100), sampled across two short reads.
    pub cpu: u8,
    /// Physical memory used as a percentage of total.
    pub memory_percent: u8,
    /// Logical CPU count.
    pub cores: u32,
    /// Host uptime in seconds.
    pub uptime_secs: u64,
}

fn read_proc(path: &str) -> Option<String> {
    std::fs::read_to_string(Path::new(path)).ok()
}

/// Splits the first `/proc/stat` line into `(busy, total)` jiffies.
/// Columns 4 (idle) and 5 (iowait) count as idle time; any parse failure or
/// missing file yields `None`.
fn parse_proc_stat_busy_total() -> Option<(u64, u64)> {
    let text = read_proc("/proc/stat")?;
    let line = text.lines().next()?;
    let mut fields = line.split_whitespace();
    if fields.next() != Some("cpu") {
        return None;
    }
    let mut total: u64 = 0;
    let mut idle: u64 = 0;
    for (index, field) in fields.enumerate() {
        let value: u64 = field.parse().ok()?;
        total = total.saturating_add(value);
        if index == 3 || index == 4 {
            idle = idle.saturating_add(value);
        }
    }
    Some((total.saturating_sub(idle), total))
}

/// CPU usage percentage across two `/proc/stat` samples ~120 ms apart.
fn cpu_percent() -> u8 {
    let Some((start_busy, start_total)) = parse_proc_stat_busy_total() else {
        return 0;
    };
    std::thread::sleep(std::time::Duration::from_millis(120));
    let Some((end_busy, end_total)) = parse_proc_stat_busy_total() else {
        return 0;
    };
    let busy_delta = end_busy.saturating_sub(start_busy);
    let total_delta = end_total.saturating_sub(start_total);
    if total_delta == 0 {
        return 0;
    }
    ((busy_delta * 100) / total_delta).min(100) as u8
}

fn parse_meminfo_kb(line: &str, key: &str) -> Option<u64> {
    let rest = line.strip_prefix(key)?;
    let rest = rest.trim_start_matches(':').trim_start();
    let rest = rest.strip_suffix(" kB")?;
    rest.trim().parse().ok()
}

/// Memory used as a percentage of total (`MemTotal` minus `MemAvailable`,
/// falling back to `MemFree`). Zero when `/proc/meminfo` is unavailable.
fn memory_percent() -> u8 {
    let Some(text) = read_proc("/proc/meminfo") else {
        return 0;
    };
    let mut total_kb: u64 = 0;
    let mut free_kb: u64 = 0;
    let mut available_kb: Option<u64> = None;
    for line in text.lines() {
        if let Some(value) = parse_meminfo_kb(line, "MemTotal") {
            total_kb = value;
        } else if let Some(value) = parse_meminfo_kb(line, "MemAvailable") {
            available_kb = Some(value);
        } else if let Some(value) = parse_meminfo_kb(line, "MemFree") {
            free_kb = value;
        }
    }
    if total_kb == 0 {
        return 0;
    }
    let used_kb = total_kb.saturating_sub(available_kb.unwrap_or(free_kb));
    ((used_kb * 100) / total_kb).min(100) as u8
}

pub async fn get_system_usage() -> Json<SystemUsage> {
    let cores = std::thread::available_parallelism()
        .map(|v| v.get() as u32)
        .unwrap_or(0);
    let uptime_secs = read_proc("/proc/uptime")
        .and_then(|text| {
            text.split_whitespace()
                .next()
                .and_then(|v| v.parse::<f64>().ok())
        })
        .map(|secs| secs as u64)
        .unwrap_or(0);
    Json(SystemUsage {
        cpu: cpu_percent(),
        memory_percent: memory_percent(),
        cores,
        uptime_secs,
    })
}

#[derive(Serialize)]
pub struct SystemVersionsResponse {
    pub botserver: String,
    pub botui: String,
    pub rust: String,
    pub postgresql: String,
    pub valkey: String,
    pub minio: String,
    pub qdrant: String,
    pub vault: String,
    pub zitadel: String,
}

pub async fn get_versions(
    State(_state): State<Arc<AppState>>,
) -> Json<SystemVersionsResponse> {
    Json(SystemVersionsResponse {
        botserver: env!("CARGO_PKG_VERSION").to_string(),
        botui: env!("CARGO_PKG_VERSION").to_string(),
        rust: "1.75.0".to_string(),
        postgresql: "16.1".to_string(),
        valkey: "8.0.2".to_string(),
        minio: "2026.03.15".to_string(),
        qdrant: "1.7.0".to_string(),
        vault: "1.15.0".to_string(),
        zitadel: "2.45.0".to_string(),
    })
}

#[derive(Deserialize)]
pub struct CheckUpdateRequest {
    pub component: String,
}

#[derive(Serialize)]
pub struct CheckUpdateResponse {
    pub component: String,
    pub current: String,
    pub latest: String,
    pub update_available: bool,
}

pub async fn check_updates(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<CheckUpdateRequest>,
) -> Json<CheckUpdateResponse> {
    let current_ver = env!("CARGO_PKG_VERSION").to_string();
    Json(CheckUpdateResponse {
        component: payload.component,
        current: current_ver.clone(),
        latest: current_ver,
        update_available: false,
    })
}

#[cfg(test)]
#[path = "system_tests.rs"]
mod system_tests;
