//! Local inference runtime: install, status (#2561).
//!
//! The `llm` component (llama.cpp) is installed by the package manager the same
//! way the wizard does it.  The endpoints here let a Settings panel trigger the
//! install without re-running the wizard.  Once installed and a model is
//! activated, the person restarts the app and `ensure_llama_servers_running`
//! starts the servers automatically.

use axum::{extract::{Extension, Path}, http::StatusCode, response::Json};
use botsecurity::auth::AuthenticatedUser;
use serde::Serialize;
use serde_json::json;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Instant;

use botcore::package_manager::{InstallMode, PackageManager};

use super::{api_error, require_admin, local_hardware, ApiResult};

// ── runtime status ───────────────────────────────────────────────────────

/// The llama-server binary path inside the stack.
fn llama_server_dir() -> PathBuf {
    PathBuf::from(botcore::shared::utils::get_stack_path())
        .join("bin")
        .join("llm")
        .join("bin")
}

fn llama_server_binary() -> PathBuf {
    let dir = llama_server_dir();
    if cfg!(windows) {
        dir.join("llama-server.exe")
    } else {
        dir.join("llama-server")
    }
}

#[derive(Serialize)]
pub(super) struct RuntimeStatus {
    /// The llama-server binary exists on disk.
    pub installed: bool,
    /// The chat-completions port (8081) answers TCP.
    pub runtime_reachable: bool,
    /// A download/install job is running right now.
    pub installing: bool,
    /// Progress of the running install job, if any.
    pub install_job: Option<JobSnapshot>,
    /// Human-readable reason when the install would fail immediately.
    pub install_blocked: Option<String>,
    /// What would be installed: the download source and estimated size.
    pub install_info: InstallInfo,
}

#[derive(Serialize)]
pub(super) struct InstallInfo {
    /// Display name of the runtime component.
    pub component: &'static str,
    /// Where the archive would be downloaded from (empty when bundled).
    pub download_host: &'static str,
    /// Approximate download size in bytes (0 when bundled/unknown).
    pub download_size_bytes: u64,
}

pub(super) async fn runtime_status() -> RuntimeStatus {
    let binary = llama_server_binary();
    let installed = binary.exists();

    let reachable = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        tokio::net::TcpStream::connect("127.0.0.1:8081"),
    )
    .await
    .map(|r| r.is_ok())
    .unwrap_or(false);

    let (installing, install_job) = current_install_job();

    let install_blocked = if installing {
        Some("An install is already running.".to_string())
    } else if installed {
        None
    } else {
        // Check disk space.
        let dir = local_hardware::models_dir();
        if let Some(free) = local_hardware::free_bytes_for(&dir) {
            // llama-server archive is ~100 MB; require at least 500 MB.
            if free < 500_000_000 {
                Some("Insufficient disk space (need at least 500 MB).".to_string())
            } else {
                None
            }
        } else {
            None
        }
    };

    RuntimeStatus {
        installed,
        runtime_reachable: reachable,
        installing,
        install_job,
        install_blocked,
        install_info: InstallInfo {
            component: "llama.cpp (inference engine)",
            download_host: "github.com",
            download_size_bytes: 100_000_000,
        },
    }
}

// ── install job ──────────────────────────────────────────────────────────

static INSTALL_JOB: LazyLock<Mutex<Option<Arc<InstallJob>>>> =
    LazyLock::new(|| Mutex::new(None));

struct InstallJob {
    id: String,
    state: Mutex<&'static str>,
    started: Instant,
    message: Mutex<String>,
    bytes_done: AtomicU64,
    bytes_total: AtomicU64,
}

const STATE_RUNNING: &str = "running";
const STATE_DONE: &str = "done";
const STATE_FAILED: &str = "failed";

#[derive(Debug, Clone, Serialize)]
pub(super) struct JobSnapshot {
    pub job_id: String,
    pub state: &'static str,
    pub message: String,
    pub elapsed_ms: u64,
    pub bytes_done: u64,
    pub bytes_total: u64,
}

fn current_install_job() -> (bool, Option<JobSnapshot>) {
    let guard = match INSTALL_JOB.lock() {
        Ok(g) => g,
        Err(_) => return (false, None),
    };
    match guard.as_ref() {
        Some(job) => {
            let state = job.state.lock().map(|g| *g).unwrap_or(STATE_FAILED);
            let installing = state == STATE_RUNNING;
            let snapshot = JobSnapshot {
                job_id: job.id.clone(),
                state,
                message: job.message.lock().ok().map(|g| g.clone()).unwrap_or_default(),
                elapsed_ms: job.started.elapsed().as_millis() as u64,
                bytes_done: job.bytes_done.load(Ordering::Relaxed),
                bytes_total: job.bytes_total.load(Ordering::Relaxed),
            };
            (installing, Some(snapshot))
        }
        None => (false, None),
    }
}

// ── POST /api/llm/runtime/install ────────────────────────────────────────

pub(super) async fn handle_install(
    Extension(user): Extension<AuthenticatedUser>,
) -> ApiResult {
    require_admin(&user)?;

    // Refuse if already installed.
    if llama_server_binary().exists() {
        return Err(api_error(StatusCode::CONFLICT, "already_installed"));
    }
    // Refuse if a job is already running.
    {
        let guard = INSTALL_JOB.lock().map_err(|_| {
            api_error(StatusCode::INTERNAL_SERVER_ERROR, "lock_failed")
        })?;
        if let Some(job) = guard.as_ref() {
            let state = job.state.lock().map(|g| *g).unwrap_or(STATE_FAILED);
            if state == STATE_RUNNING {
                return Ok(Json(json!({ "job_id": job.id, "already_running": true })));
            }
        }
    }

    let id = uuid::Uuid::new_v4().to_string();
    let job = Arc::new(InstallJob {
        id: id.clone(),
        state: Mutex::new(STATE_RUNNING),
        started: Instant::now(),
        message: Mutex::new("Starting install".to_string()),
        bytes_done: AtomicU64::new(0),
        bytes_total: AtomicU64::new(0),
    });

    {
        let mut guard = INSTALL_JOB.lock().map_err(|_| {
            api_error(StatusCode::INTERNAL_SERVER_ERROR, "lock_failed")
        })?;
        *guard = Some(Arc::clone(&job));
    }

    tokio::spawn(async move {
        run_install(job).await;
    });

    Ok(Json(json!({ "job_id": id })))
}

async fn run_install(job: Arc<InstallJob>) {
    fn set_message(job: &InstallJob, msg: &str) {
        if let Ok(mut guard) = job.message.lock() {
            *guard = msg.to_string();
        }
    }
    fn set_state(job: &InstallJob, state: &'static str) {
        if let Ok(mut guard) = job.state.lock() {
            *guard = state;
        }
    }

    set_message(&job, "Creating package manager");
    let pm = match PackageManager::new(InstallMode::Local, None) {
        Ok(pm) => pm,
        Err(e) => {
            log::error!("[model_config] package manager init failed: {e}");
            set_message(&job, &format!("Package manager init failed: {e}"));
            set_state(&job, STATE_FAILED);
            return;
        }
    };

    set_message(&job, "Installing llm component (llama.cpp)");
    match pm.install("llm").await {
        Ok(_) => {
            if llama_server_binary().exists() {
                set_message(&job, "Runtime installed successfully");
                set_state(&job, STATE_DONE);
                log::info!("[model_config] llm runtime installed successfully");
            } else {
                set_message(&job, "Install completed but binary not found");
                set_state(&job, STATE_FAILED);
                log::error!("[model_config] llm install completed but binary not at expected path");
            }
        }
        Err(e) => {
            log::error!("[model_config] llm install failed: {e}");
            set_message(&job, &format!("Install failed: {e}"));
            set_state(&job, STATE_FAILED);
        }
    }
}

// ── GET /api/llm/runtime/status ──────────────────────────────────────────

pub(super) async fn handle_status(
    Extension(user): Extension<AuthenticatedUser>,
) -> ApiResult {
    require_admin(&user)?;
    let status = runtime_status().await;
    Ok(Json(json!(status)))
}

// ── GET /api/llm/runtime/jobs/:id ────────────────────────────────────────

pub(super) async fn handle_job_status(
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<String>,
) -> ApiResult {
    require_admin(&user)?;
    let guard = INSTALL_JOB.lock().map_err(|_| {
        api_error(StatusCode::INTERNAL_SERVER_ERROR, "lock_failed")
    })?;
    match guard.as_ref() {
        Some(job) if job.id == id => {
            let state = job.state.lock().map(|g| *g).unwrap_or(STATE_FAILED);
            Ok(Json(json!(JobSnapshot {
                job_id: job.id.clone(),
                state,
                message: job.message.lock().ok().map(|g| g.clone()).unwrap_or_default(),
                elapsed_ms: job.started.elapsed().as_millis() as u64,
                bytes_done: job.bytes_done.load(Ordering::Relaxed),
                bytes_total: job.bytes_total.load(Ordering::Relaxed),
            })))
        }
        _ => Err(api_error(StatusCode::NOT_FOUND, "job_not_found")),
    }
}

