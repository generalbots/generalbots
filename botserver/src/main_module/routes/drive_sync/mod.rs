//! #1955 — local Drive sync manager (desktop/shell edition).
//!
//! Drive mirrors a local folder with a folder inside the desktop Drive root
//! using `rsync`, and the server owns the jobs: the REST API stores them, a
//! background tick runs the ones that are due, and every run records status,
//! duration, a transferred summary and — when both sides changed the same file
//! — a conflict list the Sync dialog can resolve.
//!
//! Safety: paths must be absolute, `..`-free directories that exist; `rsync`
//! runs through `SafeCommand` (never a shell) with an explicit argument list;
//! one run per job at a time; every failure comes back with its reason.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    path::{Component, PathBuf},
    sync::Once,
    time::Duration,
};


/// How often the server checks for jobs that are due.
mod handlers;
mod rsync_engine;

pub use handlers::*;

const TICK_SECONDS: u64 = 30;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncJob {
    pub id: String,
    pub name: String,
    pub local_path: String,
    pub drive_path: String,
    /// `push` (local → Drive), `pull` (Drive → local) or `two-way`.
    pub direction: String,
    /// 0 means "manual only".
    pub interval_minutes: u64,
    pub enabled: bool,
    pub last_run: Option<String>,
    pub last_status: String,
    pub last_error: Option<String>,
    pub last_summary: Option<String>,
    pub last_duration_ms: Option<u64>,
    #[serde(default)]
    pub conflicts: Vec<SyncConflict>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncConflict {
    pub path: String,
    pub detail: String,
}

#[derive(Debug, Deserialize)]
pub struct JobInput {
    pub name: String,
    pub local_path: String,
    pub drive_path: String,
    #[serde(default)]
    pub direction: Option<String>,
    #[serde(default)]
    pub interval_minutes: Option<u64>,
    #[serde(default)]
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ResolveInput {
    pub path: String,
    /// `local`, `remote` or `both`.
    pub resolution: String,
}

fn store_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("GB_DRIVE_SYNC_DIR") {
        if !dir.trim().is_empty() {
            return PathBuf::from(dir);
        }
    }
    match std::env::var("HOME") {
        Ok(home) if !home.trim().is_empty() => {
            PathBuf::from(home).join(".local/share/generalbots/drive-sync")
        }
        _ => std::env::temp_dir().join("gb-drive-sync"),
    }
}

fn store_file() -> PathBuf {
    store_dir().join("jobs.json")
}

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn due(job: &SyncJob) -> bool {
    if job.interval_minutes == 0 {
        return false;
    }
    let Some(last) = job.last_run.as_deref() else {
        return true;
    };
    match chrono::DateTime::parse_from_rfc3339(last) {
        Ok(parsed) => {
            let elapsed = chrono::Utc::now().signed_duration_since(parsed.with_timezone(&chrono::Utc));
            elapsed.num_minutes() as u64 >= job.interval_minutes
        }
        Err(_) => true,
    }
}


fn load() -> Vec<SyncJob> {
    let path = store_file();
    match std::fs::read_to_string(&path) {
        Ok(raw) => match serde_json::from_str(&raw) {
            Ok(jobs) => jobs,
            Err(error) => {
                log::warn!("drive sync: {} is unreadable ({error}); starting empty", path.display());
                Vec::new()
            }
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(error) => {
            log::warn!("drive sync: cannot read {}: {error}", path.display());
            Vec::new()
        }
    }
}

fn save(jobs: &[SyncJob]) -> Result<(), String> {
    let dir = store_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("create {}: {e}", dir.display()))?;
    let raw = serde_json::to_string_pretty(jobs).map_err(|e| format!("serialize jobs: {e}"))?;
    let tmp = dir.join("jobs.json.tmp");
    std::fs::write(&tmp, raw).map_err(|e| format!("write {}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, store_file()).map_err(|e| format!("replace the job store: {e}"))
}

/// A sync folder must be an absolute, existing directory with no `..`.
fn clean_dir(input: &str, field: &str) -> Result<PathBuf, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(format!("{field} is required"));
    }
    let path = PathBuf::from(trimmed);
    if !path.is_absolute() {
        return Err(format!("{field} must be an absolute path"));
    }
    if path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(format!("{field} must not contain '..'"));
    }
    if !path.is_dir() {
        return Err(format!(
            "{field} is not a folder on this machine: {}",
            path.display()
        ));
    }
    Ok(path)
}

fn run_job_blocking(id: &str) -> Result<SyncJob, String> {
    let mut jobs = load();
    let index = jobs
        .iter()
        .position(|job| job.id == id)
        .ok_or_else(|| format!("sync job {id} not found"))?;
    let mut job = jobs[index].clone();
    let outcome = rsync_engine::perform(&mut job);
    if let Err(error) = &outcome {
        job.last_status = "error".to_string();
        job.last_error = Some(error.clone());
        job.last_run = Some(now());
    }
    jobs[index] = job.clone();
    save(&jobs)?;
    outcome.map(|()| job)
}

fn resolve_blocking(id: &str, path: &str, resolution: &str) -> Result<SyncJob, String> {
    let mut jobs = load();
    let index = jobs
        .iter()
        .position(|job| job.id == id)
        .ok_or_else(|| format!("sync job {id} not found"))?;
    let mut job = jobs[index].clone();
    let local = clean_dir(&job.local_path, "local folder")?;
    let drive = clean_dir(&job.drive_path, "Drive folder")?;
    let relative = path.trim_start_matches("./").trim_start_matches('/');
    if relative.is_empty() {
        return Err("a conflict path is required".to_string());
    }
    let local_target = local.join(relative);
    let drive_target = drive.join(relative);
    if !local_target.starts_with(&local) || !drive_target.starts_with(&drive) {
        return Err("the conflict path escapes its sync folder".to_string());
    }
    match resolution {
        "local" => {
            let parent = drive_target.parent().ok_or("conflict path has no parent")?;
            std::fs::create_dir_all(parent).map_err(|e| format!("prepare {}: {e}", parent.display()))?;
            std::fs::copy(&local_target, &drive_target)
                .map_err(|e| format!("copy local copy into Drive: {e}"))?;
        }
        "remote" => {
            let parent = local_target.parent().ok_or("conflict path has no parent")?;
            std::fs::create_dir_all(parent).map_err(|e| format!("prepare {}: {e}", parent.display()))?;
            std::fs::copy(&drive_target, &local_target)
                .map_err(|e| format!("copy Drive copy into the local folder: {e}"))?;
        }
        "both" => {
            let stamp = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();
            let keep = local.join(format!("{relative}.local-{stamp}"));
            std::fs::copy(&local_target, &keep)
                .map_err(|e| format!("keep the local copy: {e}"))?;
            std::fs::copy(&drive_target, &local_target)
                .map_err(|e| format!("copy Drive copy into the local folder: {e}"))?;
        }
        other => return Err(format!("unknown resolution '{other}'")),
    }
    job.conflicts.retain(|conflict| conflict.path != path);
    job.last_status = if job.conflicts.is_empty() { "ok".to_string() } else { "conflict".to_string() };
    job.last_summary = Some(match resolution {
        "local" => "kept the local copy".to_string(),
        "remote" => "kept the Drive copy".to_string(),
        _ => "kept both copies".to_string(),
    });
    jobs[index] = job.clone();
    save(&jobs)?;
    Ok(job)
}

fn status_snapshot() -> Value {
    let jobs = load();
    let syncing = jobs.iter().filter(|job| job.last_status == "syncing").count();
    let errors = jobs.iter().filter(|job| job.last_status == "error").count();
    let conflicts: usize = jobs.iter().map(|job| job.conflicts.len()).sum();
    let state = if errors > 0 {
        "error"
    } else if conflicts > 0 {
        "conflict"
    } else if syncing > 0 {
        "syncing"
    } else {
        "idle"
    };
    json!({
        "success": true,
        "state": state,
        "jobs": jobs.len(),
        "syncing": syncing,
        "errors": errors,
        "conflicts": conflicts,
        "store": store_file().display().to_string(),
    })
}

/// One background check for due jobs, started once by the router setup so the
/// schedule is owned by the server rather than by the dialog.
pub fn start_scheduler() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(TICK_SECONDS)).await;
                let jobs = load();
                for job in jobs {
                    if !job.enabled || !due(&job) {
                        continue;
                    }
                    let id = job.id.clone();
                    let name = job.name.clone();
                    let joined = tokio::task::spawn_blocking(move || run_job_blocking(&id)).await;
                    match joined {
                        Ok(Ok(done)) => log::info!(
                            "drive sync: scheduled run of '{}' finished ({})",
                            name,
                            done.last_status
                        ),
                        Ok(Err(error)) => {
                            log::warn!("drive sync: scheduled run of '{name}' failed: {error}")
                        }
                        Err(error) => log::warn!("drive sync: scheduled run of '{name}' panicked: {error}"),
                    }
                }
            }
        });
    });
}
