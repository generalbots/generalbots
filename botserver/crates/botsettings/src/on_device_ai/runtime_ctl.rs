//! Chat runtime lifecycle: query, stop, start, wait (#2561).
//!
//! Single source of truth: the model the runtime is running **must** be the
//! stored local choice.  This module is called on activation (switch to local)
//! and at app start to enforce that invariant, replacing orphan processes when
//! necessary.
//!
//! Only the **chat** runtime on port 8081 is managed here; the embedding
//! runtime on 8082 is never touched.

use botsecurity::command_guard::SafeCommand;
use serde::Serialize;
use std::time::Duration;

use super::runtime_stop;

/// Maximum time to wait for the runtime to become healthy after a start.
const HEALTH_TIMEOUT: Duration = Duration::from_secs(120);

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

/// Query what model the chat runtime has loaded. Short timeout (~1 s).
pub async fn probe() -> RuntimeProbe {
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
    let binary = if cfg!(windows) {
        format!("{}\\llama-server.exe", desired.server_dir)
    } else {
        format!("{}/llama-server", desired.server_dir)
    };
    if !std::path::Path::new(&binary).exists() {
        return Err(format!("llama-server binary not found at {binary}"));
    }
    if !std::path::Path::new(&desired.model_path).exists() {
        return Err(format!("Model file not found at {}", desired.model_path));
    }
    let args = build_args(desired);
    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let mut command = SafeCommand::new(&binary).map_err(|e| e.to_string())?;
    command = command.args(&arg_refs).map_err(|e| e.to_string())?;
    command = command
        .working_dir(std::path::Path::new(&desired.server_dir))
        .map_err(|e| e.to_string())?;
    if !cfg!(windows) {
        command = command
            .env("LD_LIBRARY_PATH", &desired.server_dir)
            .map_err(|e| e.to_string())?;
    }
    let stack = botcore::shared::utils::get_stack_path();
    let log_dir = format!("{stack}/logs/llm");
    let _ = std::fs::create_dir_all(&log_dir);
    let log_path = format!("{log_dir}/llm.log");
    match std::fs::OpenOptions::new().create(true).append(true).open(&log_path) {
        Ok(log_file) => {
            if let Ok(clone) = log_file.try_clone() {
                command = command.stdout(std::process::Stdio::from(clone));
            } else {
                command = command.stdout(std::process::Stdio::null());
            }
            command = command.stderr(std::process::Stdio::from(log_file));
        }
        Err(_) => {
            command = command.stdout(std::process::Stdio::null());
            command = command.stderr(std::process::Stdio::null());
        }
    }
    log::info!(
        "[runtime_ctl] starting chat runtime: model={}, gpu_layers={}, ctx_size={}",
        desired.model_file,
        if desired.gpu_layers.is_empty() { "(build default)" } else { &desired.gpu_layers },
        desired.ctx_size,
    );
    let child = command.spawn().map_err(|e| format!("spawn failed: {e}"))?;
    Ok(child.id())
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

/// Make sure the chat runtime is running the desired model.
pub async fn ensure(desired: &DesiredRuntime) -> EnsureOutcome {
    let current = wait_for_initial_probe().await;

    if current.reachable && model_matches(&current.loaded_model, &desired.model_file) {
        return EnsureOutcome::AlreadyCorrect;
    }

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

    let child_pid = match start_chat_runtime(desired) {
        Ok(pid) => pid,
        Err(reason) => return EnsureOutcome::Failed(reason),
    };

    match wait_healthy(&desired.model_file, child_pid).await {
        Ok(()) => EnsureOutcome::Started,
        Err(reason) => {
            log::warn!("[runtime_ctl] health wait failed, cleaning up PID {child_pid}");
            let _ = runtime_stop::kill_pid(child_pid);
            EnsureOutcome::Failed(reason)
        }
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
        "" | "0" => "8192".to_string(),
        other => other.to_string(),
    };
    Some(DesiredRuntime {
        model_file: bare.to_string(), model_path, server_dir,
        gpu_layers: gpu, ctx_size: ctx,
    })
}

#[cfg(test)]
#[path = "runtime_ctl_tests.rs"]
mod runtime_ctl_tests;
