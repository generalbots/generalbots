//! Local inference runtime: install, update, status (#2561).
//!
//! The `llm` component (llama.cpp) is installed by the package manager the same
//! way the wizard does it.  The endpoints here let a Settings panel install it
//! without re-running the wizard, and replace an installed engine with the
//! build this version pins (see `engine_update`).

use axum::{extract::{Extension, Path, State}, http::StatusCode, response::Json};
use botcore::config::ConfigManager;
use botcore::shared::state::AppState;
use botsecurity::auth::AuthenticatedUser;
use serde::Serialize;
use serde_json::json;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Instant;

use super::{api_error, engine_update, require_admin, local_hardware, ApiResult};

// ── runtime status ───────────────────────────────────────────────────────

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
    /// The llama.cpp build installed, when it was recorded at install time.
    pub build: Option<String>,
    /// The build this version of the product installs.
    pub available_build: Option<String>,
    /// An engine is installed and it is not `available_build`.
    pub update_available: bool,
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
    let installed = engine_update::server_binary().is_some();

    let reachable = crate::on_device_ai::runtime_ctl::port_open().await;

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
            download_size_bytes: engine_update::download_bytes(),
        },
        build: engine_update::installed_build(),
        available_build: engine_update::available_build(),
        update_available: engine_update::update_available(),
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
    /// What the job is doing (`engine_update::STAGE_*`).
    stage: Mutex<&'static str>,
    /// Why it failed (`engine_update::CODE_*`), empty otherwise.
    code: Mutex<&'static str>,
    /// The job replaces an installed engine.
    update: bool,
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
    pub stage: &'static str,
    pub code: &'static str,
    pub update: bool,
    pub elapsed_ms: u64,
    pub bytes_done: u64,
    pub bytes_total: u64,
}

impl InstallJob {
    fn state(&self) -> &'static str {
        self.state.lock().map(|g| *g).unwrap_or(STATE_FAILED)
    }

    fn snapshot(&self) -> JobSnapshot {
        JobSnapshot {
            job_id: self.id.clone(),
            state: self.state(),
            message: self.message.lock().ok().map(|g| g.clone()).unwrap_or_default(),
            stage: self.stage.lock().map(|g| *g).unwrap_or(""),
            code: self.code.lock().map(|g| *g).unwrap_or(""),
            update: self.update,
            elapsed_ms: self.started.elapsed().as_millis() as u64,
            bytes_done: self.bytes_done.load(Ordering::Relaxed),
            bytes_total: self.bytes_total.load(Ordering::Relaxed),
        }
    }

    fn set_stage(&self, stage: &'static str) {
        if let Ok(mut guard) = self.stage.lock() {
            *guard = stage;
        }
    }

    fn finish(&self, state: &'static str, code: &'static str, message: &str) {
        if let Ok(mut guard) = self.message.lock() {
            *guard = message.to_string();
        }
        if let Ok(mut guard) = self.code.lock() {
            *guard = code;
        }
        if let Ok(mut guard) = self.state.lock() {
            *guard = state;
        }
    }
}

fn current_install_job() -> (bool, Option<JobSnapshot>) {
    let guard = match INSTALL_JOB.lock() {
        Ok(g) => g,
        Err(_) => return (false, None),
    };
    match guard.as_ref() {
        Some(job) => (job.state() == STATE_RUNNING, Some(job.snapshot())),
        None => (false, None),
    }
}

// ── POST /api/llm/runtime/install ────────────────────────────────────────

/// Installs the engine, or replaces an installed one that is not the pinned
/// build. An engine that is already the pinned build is refused.
pub(super) async fn handle_install(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
) -> ApiResult {
    require_admin(&user)?;

    let update = engine_update::server_binary().is_some();
    if update && !engine_update::update_available() {
        return Err(api_error(StatusCode::CONFLICT, "already_installed"));
    }
    // Refuse if a job is already running.
    {
        let guard = INSTALL_JOB.lock().map_err(|_| {
            api_error(StatusCode::INTERNAL_SERVER_ERROR, "lock_failed")
        })?;
        if let Some(job) = guard.as_ref() {
            if job.state() == STATE_RUNNING {
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
        stage: Mutex::new(engine_update::STAGE_DOWNLOADING),
        code: Mutex::new(""),
        update,
        bytes_done: AtomicU64::new(0),
        bytes_total: AtomicU64::new(0),
    });

    {
        let mut guard = INSTALL_JOB.lock().map_err(|_| {
            api_error(StatusCode::INTERNAL_SERVER_ERROR, "lock_failed")
        })?;
        *guard = Some(Arc::clone(&job));
    }

    // What a restart of the running model needs, read before the job starts.
    let config = ConfigManager::new(state.conn.clone());
    let nil = uuid::Uuid::nil();
    let gpu_layers = config.get_config(&nil, "llm-server-gpu-layers", Some("")).unwrap_or_default();
    let ctx_size = config.get_config(&nil, "llm-server-ctx-size", Some("")).unwrap_or_default();

    tokio::spawn(async move {
        run_install(job, gpu_layers, ctx_size).await;
    });

    Ok(Json(json!({ "job_id": id, "update": update })))
}

async fn run_install(job: Arc<InstallJob>, gpu_layers: String, ctx_size: String) {
    if job.update {
        let reporter = Arc::clone(&job);
        let stage = move |stage: &'static str| reporter.set_stage(stage);
        match engine_update::update(&stage, &gpu_layers, &ctx_size).await {
            Ok(()) => job.finish(STATE_DONE, "", "Engine updated"),
            Err(code) => job.finish(STATE_FAILED, code, "Engine update failed; the previous engine is back"),
        }
        return;
    }
    match engine_update::install().await {
        Ok(()) => {
            log::info!("[model_config] llm runtime installed successfully");
            job.finish(STATE_DONE, "", "Runtime installed successfully");
        }
        Err(e) => {
            log::error!("[model_config] llm install failed: {e}");
            job.finish(STATE_FAILED, engine_update::CODE_DOWNLOAD_FAILED, &format!("Install failed: {e}"));
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
        Some(job) if job.id == id => Ok(Json(json!(job.snapshot()))),
        _ => Err(api_error(StatusCode::NOT_FOUND, "job_not_found")),
    }
}

