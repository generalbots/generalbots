//! Chat runtime lifecycle: query, stop, start, wait (#2561).
//!
//! Single source of truth: the model the runtime is running **must** be the
//! stored local choice.  This module is called on activation (switch to local)
//! and at app start to enforce that invariant, replacing orphan processes when
//! necessary.
//!
//! Only the **chat** runtime on port 8081 is managed here; the embedding
//! runtime on 8082 is only started again after an engine update stopped it.

use serde::Serialize;
use std::time::Duration;

use super::runtime_stop;

/// Serialises calls to [`ensure_inner`] so that concurrent callers (boot start,
/// recovery path, Settings switch) cannot launch two engines on the same port.
static ENSURE_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// Maximum time to wait for the runtime to become healthy after a start.
/// A cold disk-cache load of a large model (5+ GB) can take over two minutes,
/// so this must be generous enough to survive that without killing a live load.
const HEALTH_TIMEOUT: Duration = Duration::from_secs(300);

/// Poll interval while waiting for health.
const HEALTH_POLL: Duration = Duration::from_secs(2);

/// Chat-completions port bound by the managed llama-server.
pub const CHAT_PORT: u16 = 8081;

/// Result of probing the running chat runtime.
#[derive(Debug, Clone, Serialize)]
pub struct RuntimeProbe {
    pub reachable: bool,
    pub loaded_model: String,
}

/// What the caller wants to run.
#[derive(Debug, Clone)]
pub struct DesiredRuntime {
    pub model_file: String,
    pub model_path: String,
    pub server_dir: String,
    /// Empty means "omit the flag" (use build default).
    pub gpu_layers: String,
    pub ctx_size: String,
}

/// Outcome of an ensure call.
#[derive(Debug)]
pub enum EnsureOutcome {
    AlreadyCorrect,
    Started,
    Failed(String),
}

impl EnsureOutcome {
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::AlreadyCorrect | Self::Started)
    }
    pub fn error_message(&self) -> Option<&str> {
        match self {
            Self::Failed(msg) => Some(msg),
            _ => None,
        }
    }
}

// ── probe ───────────────────────────────────────────────────────────────

/// Whether something accepts connections on the chat port. A stopped engine
/// is the common case on the settings page, and on Windows a refused
/// connection to localhost only comes back after about two seconds of
/// retries: give up early instead, a listening port answers at once.
pub async fn port_open() -> bool {
    tokio::time::timeout(
        Duration::from_millis(350),
        tokio::net::TcpStream::connect(format!("127.0.0.1:{CHAT_PORT}")),
    )
    .await
    .map(|result| result.is_ok())
    .unwrap_or(false)
}

/// Query what model the chat runtime has loaded. Short timeout (~1 s).
pub async fn probe() -> RuntimeProbe {
    if !port_open().await {
        return RuntimeProbe { reachable: false, loaded_model: String::new() };
    }
    probe_with_timeout(Duration::from_secs(1)).await
}

async fn probe_with_timeout(timeout: Duration) -> RuntimeProbe {
    let client = reqwest::Client::builder()
        .connect_timeout(timeout)
        .timeout(timeout)
        .build()
        .unwrap_or_default();
    let url = format!("http://127.0.0.1:{CHAT_PORT}/v1/models");
    let resp = match client.get(&url).send().await {
        Ok(r) if r.status().is_success() => r,
        _ => return RuntimeProbe { reachable: false, loaded_model: String::new() },
    };
    let body: serde_json::Value = match resp.json().await {
        Ok(v) => v,
        Err(_) => return RuntimeProbe { reachable: true, loaded_model: String::new() },
    };
    let model_id = body["data"]
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|entry| entry["id"].as_str())
        .unwrap_or("")
        .to_string();
    RuntimeProbe { reachable: true, loaded_model: model_id }
}

pub fn model_matches(loaded: &str, desired_file: &str) -> bool {
    if loaded.is_empty() || desired_file.is_empty() { return false; }
    let loaded_bare = loaded.rsplit(['/', '\\']).next().unwrap_or(loaded);
    let desired_bare = desired_file.rsplit(['/', '\\']).next().unwrap_or(desired_file);
    loaded_bare == desired_bare
}

pub fn needs_restart(probe: &RuntimeProbe, desired_file: &str) -> bool {
    if !probe.reachable { return true; }
    !model_matches(&probe.loaded_model, desired_file)
}

/// True when the stored model looks like a local GGUF file.
pub fn is_local_model_name(model: &str) -> bool {
    let trimmed = model.trim();
    !trimmed.is_empty() && trimmed.ends_with(".gguf")
}

// ── start ───────────────────────────────────────────────────────────────

/// Spawn llama-server for chat. Returns the child PID.
pub fn start_chat_runtime(desired: &DesiredRuntime) -> Result<u32, String> {
    if !std::path::Path::new(&desired.model_path).exists() {
        return Err(format!("Model file not found at {}", desired.model_path));
    }
    log::info!(
        "[runtime_ctl] starting chat runtime: model={}, gpu_layers={}, ctx_size={}",
        desired.model_file,
        if desired.gpu_layers.is_empty() { "(build default)" } else { &desired.gpu_layers },
        desired.ctx_size,
    );
    super::runtime_spawn::spawn_server(&desired.server_dir, &build_args(desired), "llm.log")
}

/// Command-line args matching the package manager's line plus `--ctx-size`.
pub fn build_args(desired: &DesiredRuntime) -> Vec<String> {
    let mut args = vec![
        "--port".into(), CHAT_PORT.to_string(),
        "--host".into(), "127.0.0.1".into(),
        "-m".into(), desired.model_path.clone(),
        "--ubatch-size".into(), "512".into(),
        "--ctx-size".into(), desired.ctx_size.clone(),
    ];
    if !desired.gpu_layers.is_empty() {
        args.push("--n-gpu-layers".into());
        args.push(desired.gpu_layers.clone());
    }
    args
}

// ── wait ────────────────────────────────────────────────────────────────

/// Wait until healthy AND reporting the desired model. Detects early exit.
pub async fn wait_healthy(desired_file: &str, child_pid: u32) -> Result<(), String> {
    let deadline = tokio::time::Instant::now() + HEALTH_TIMEOUT;
    loop {
        if tokio::time::Instant::now() >= deadline {
            return Err(format!(
                "Chat runtime did not become healthy within {} s",
                HEALTH_TIMEOUT.as_secs()
            ));
        }
        if !runtime_stop::is_pid_alive(child_pid) {
            return Err(
                "Chat runtime exited during startup (possible VRAM exhaustion or bad model)"
                    .to_string(),
            );
        }
        tokio::time::sleep(HEALTH_POLL).await;
        let p = probe_with_timeout(Duration::from_secs(3)).await;
        if p.reachable && model_matches(&p.loaded_model, desired_file) {
            log::info!("[runtime_ctl] chat runtime healthy, model={}", p.loaded_model);
            return Ok(());
        }
    }
}

// ── ensure ──────────────────────────────────────────────────────────────

/// Make sure the chat runtime is running the desired model. On failure the
/// port is left empty: at app start the stored choice is the only model that
/// may answer, so nothing else is started in its place.
pub async fn ensure(desired: &DesiredRuntime) -> EnsureOutcome {
    ensure_inner(desired, false).await
}

/// [`ensure`] for a switch asked by the user: when the new model cannot be
/// started, the model that was running before is started again, so the
/// choice that is still stored keeps answering.
pub async fn ensure_or_restore(desired: &DesiredRuntime) -> EnsureOutcome {
    ensure_inner(desired, true).await
}

async fn ensure_inner(desired: &DesiredRuntime, restore_previous: bool) -> EnsureOutcome {
    let _serial = ENSURE_LOCK.lock().await;
    let current = wait_for_initial_probe().await;

    if current.reachable && model_matches(&current.loaded_model, &desired.model_file) {
        record_failure(None);
        return EnsureOutcome::AlreadyCorrect;
    }
    let log_mark = llm_log_len();

    if current.reachable {
        log::info!(
            "[runtime_ctl] stopping chat runtime (loaded={}, desired={})",
            current.loaded_model, desired.model_file,
        );
        match runtime_stop::stop_chat_runtime(CHAT_PORT) {
            Ok(pid) => {
                log::info!("[runtime_ctl] killed PID {pid}, waiting for port release");
                wait_port_free().await;
            }
            Err(reason) => {
                return EnsureOutcome::Failed(format!("Cannot stop chat runtime: {reason}"));
            }
        }
    }

    let started = match start_chat_runtime(desired) {
        Ok(child_pid) => match wait_healthy(&desired.model_file, child_pid).await {
            Ok(()) => Ok(()),
            Err(reason) => {
                log::warn!("[runtime_ctl] health wait failed, cleaning up PID {child_pid}");
                let _ = runtime_stop::kill_pid(child_pid);
                Err(reason)
            }
        },
        Err(reason) => Err(reason),
    };
    let reason = match started {
        Ok(()) => {
            record_failure(None);
            return EnsureOutcome::Started;
        }
        Err(reason) => reason,
    };

    let code = classify_failure(&llm_log_since(log_mark));
    record_failure(Some((desired.model_file.clone(), code)));
    if restore_previous && current.reachable {
        wait_port_free().await;
        restore(&current.loaded_model, desired).await;
    }
    EnsureOutcome::Failed(format!("{code}: {reason}"))
}

// ── failure cause ───────────────────────────────────────────────────────

/// The engine does not know this model family (it is older than the model).
pub const CODE_ENGINE_TOO_OLD: &str = "engine_too_old";
/// The model does not fit the memory that is free.
pub const CODE_NOT_ENOUGH_MEMORY: &str = "not_enough_memory";
/// Any other start failure.
pub const CODE_START_FAILED: &str = "runtime_start_failed";

/// Why the engine stopped, from what it wrote to its log during the start.
pub fn classify_failure(log_text: &str) -> &'static str {
    let lower = log_text.to_ascii_lowercase();
    if lower.contains("unknown model architecture") || lower.contains("unknown pre-tokenizer") {
        CODE_ENGINE_TOO_OLD
    } else if lower.contains("out of memory")
        || lower.contains("failed to allocate")
        || lower.contains("unable to allocate")
    {
        CODE_NOT_ENOUGH_MEMORY
    } else {
        CODE_START_FAILED
    }
}

/// The code at the start of a [`EnsureOutcome::Failed`] message.
pub fn failure_code(reason: &str) -> &'static str {
    [CODE_ENGINE_TOO_OLD, CODE_NOT_ENOUGH_MEMORY]
        .into_iter()
        .find(|code| reason.starts_with(code))
        .unwrap_or(CODE_START_FAILED)
}

static LAST_FAILURE: std::sync::Mutex<Option<(String, &'static str)>> = std::sync::Mutex::new(None);

fn record_failure(failure: Option<(String, &'static str)>) {
    if let Ok(mut slot) = LAST_FAILURE.lock() {
        *slot = failure;
    }
}

/// Forget the last start failure (the engine was replaced since).
pub fn clear_last_failure() {
    record_failure(None);
}

/// The model that last failed to start and why; `None` after a good start.
pub fn last_failure() -> Option<(String, &'static str)> {
    LAST_FAILURE.lock().ok().and_then(|slot| slot.clone())
}

fn llm_log_path() -> String {
    format!("{}/logs/llm/llm.log", botcore::shared::utils::get_stack_path())
}

fn llm_log_len() -> u64 {
    std::fs::metadata(llm_log_path()).map(|meta| meta.len()).unwrap_or(0)
}

/// What the engine wrote to its log after `mark` (at most the last 64 KB).
fn llm_log_since(mark: u64) -> String {
    use std::io::{Read, Seek, SeekFrom};
    let Ok(mut file) = std::fs::File::open(llm_log_path()) else {
        return String::new();
    };
    let len = file.metadata().map(|meta| meta.len()).unwrap_or(0);
    let from = mark.min(len).max(len.saturating_sub(65_536));
    if file.seek(SeekFrom::Start(from)).is_err() {
        return String::new();
    }
    let mut bytes = Vec::new();
    let _ = file.read_to_end(&mut bytes);
    String::from_utf8_lossy(&bytes).into_owned()
}

/// Starts the model that was loaded before a failed switch.
async fn restore(previous_model: &str, failed: &DesiredRuntime) {
    let bare = previous_model.rsplit(['/', '\\']).next().unwrap_or(previous_model);
    let Some(dir) = std::path::Path::new(&failed.model_path).parent() else {
        return;
    };
    let path = dir.join(bare);
    if bare.is_empty() || !path.exists() {
        log::warn!("[runtime_ctl] previous model {bare} is not on disk; nothing restored");
        return;
    }
    let previous = DesiredRuntime {
        model_file: bare.to_string(),
        model_path: path.to_string_lossy().into_owned(),
        server_dir: failed.server_dir.clone(),
        gpu_layers: failed.gpu_layers.clone(),
        ctx_size: failed.ctx_size.clone(),
    };
    match start_chat_runtime(&previous) {
        Ok(pid) => match wait_healthy(bare, pid).await {
            Ok(()) => log::info!("[runtime_ctl] previous model {bare} restored"),
            Err(reason) => {
                log::warn!("[runtime_ctl] previous model {bare} did not come back: {reason}");
                let _ = runtime_stop::kill_pid(pid);
            }
        },
        Err(reason) => log::warn!("[runtime_ctl] cannot restore {bare}: {reason}"),
    }
}

/// Wait for the port to have something reachable, or time out (60 s).
async fn wait_for_initial_probe() -> RuntimeProbe {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
    loop {
        let p = probe_with_timeout(Duration::from_secs(3)).await;
        if p.reachable { return p; }
        let bound = tokio::time::timeout(
            Duration::from_millis(500),
            tokio::net::TcpStream::connect(format!("127.0.0.1:{CHAT_PORT}")),
        ).await.map(|r| r.is_ok()).unwrap_or(false);
        if !bound || tokio::time::Instant::now() >= deadline { return p; }
        tokio::time::sleep(HEALTH_POLL).await;
    }
}

/// Wait until the chat port is free (bounded, 10 s).
async fn wait_port_free() {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    while tokio::time::Instant::now() < deadline {
        let bound = tokio::time::timeout(
            Duration::from_millis(300),
            tokio::net::TcpStream::connect(format!("127.0.0.1:{CHAT_PORT}")),
        ).await.map(|r| r.is_ok()).unwrap_or(false);
        if !bound { return; }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    log::warn!("[runtime_ctl] port {CHAT_PORT} still bound after 10 s");
}

// ── helpers ─────────────────────────────────────────────────────────────

pub fn desired_from_config(
    model_file: &str, gpu_layers: &str, ctx_size: &str,
) -> Option<DesiredRuntime> {
    if model_file.trim().is_empty() { return None; }
    let stack = botcore::shared::utils::get_stack_path();
    let bare = model_file.rsplit(['/', '\\']).next().unwrap_or(model_file);
    let model_path = format!("{stack}/data/llm/{bare}");
    if !std::path::Path::new(&model_path).exists() { return None; }
    let server_dir = format!("{stack}/bin/llm");
    let gpu = match gpu_layers.trim() {
        "" | "auto" => String::new(),
        other => other.to_string(),
    };
    let ctx = match ctx_size.trim() {
        "" | "0" => {
            let model_bytes = std::fs::metadata(&model_path).map(|m| m.len()).unwrap_or(0);
            let hardware = super::hardware::probe();
            let vram_gb = hardware.cuda_usable.then_some(hardware.vram_gb);
            ctx_size_for(model_bytes, vram_gb).to_string()
        }
        other => other.to_string(),
    };
    Some(DesiredRuntime {
        model_file: bare.to_string(), model_path, server_dir,
        gpu_layers: gpu, ctx_size: ctx,
    })
}

/// Context the chat engine starts with when none is configured: 8192 tokens,
/// more when the graphics card keeps room beside the model. The context takes
/// video memory too, and a model that spills out of it answers far slower.
pub fn ctx_size_for(model_bytes: u64, vram_gb: Option<u16>) -> u32 {
    let (Some(vram_gb), true) = (vram_gb, model_bytes > 0) else { return 8192 };
    let spare_mib = (u64::from(vram_gb) * 1024).saturating_sub(model_bytes / (1024 * 1024));
    match spare_mib {
        6144.. => 32768,
        2048.. => 16384,
        _ => 8192,
    }
}

#[cfg(test)]
#[path = "runtime_ctl_tests.rs"]
mod runtime_ctl_tests;
