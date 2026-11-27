//! #1167 / #1172 / #1580 — Always-On Agent Mode backend.
//!
//! Per-user Incus VMs (`agent_sessions`), per-conversation folders
//! (`agent_conversation_folders`), org API keys (`org_api_keys`) and the
//! sandboxed code-execution service (`sandbox_runs`). All Incus invocations
//! go through `botlib::security::command_guard::SafeCommand`.
//!
//! #2062 — the workspace image is described by profiles (`vm::profile`):
//! `base`, `office` (default: document libraries + embedded headless
//! Chromium), `media` and `data` (opt-in). The prefabricated Incus image is
//! built with `scripts/build_vm_image.sh --profile office`, which bakes every
//! artifact into the image so a cold container needs no package download.

pub mod api;
pub mod antigravity;
pub mod claude;
pub mod cli_models;
mod cli_turn;
pub mod codex;
pub mod deliverables;
pub mod duplex;
pub mod keys;
pub mod models;
pub mod plan_limit;
pub mod plan_usage;
mod runtime_host;
pub mod runtime_state;
pub mod sandbox;
pub mod schema;
pub mod snapshots;
pub mod state;
pub mod stream;
pub mod transfer;
pub mod vm;
pub mod warm;

use std::sync::Arc;

/// Shared service handle wired into axum routers by the integrator.
pub struct AgentService {
    pool: state::DbPool,
    incus_bin: String,
    max_snapshots_per_session: i64,
    idle_timeout_secs: i64,
}

/// Environment variable that names the Incus executable used by the agent.
pub const AGENT_INCUS_BIN_ENV: &str = "AGENT_INCUS_BIN";

/// Why agent mode cannot run on this host, or `None` when it can.
///
/// `/api/product` exposes this as `agent_backend_unavailable` next to the
/// stable code `agent_runtime_state` (`runtime_state::AgentRuntimeState`).
/// The agent error path calls it before spawning Incus so the caller gets
/// this reason instead of the generic sanitized agent-workspace failure.
pub fn agent_backend_unavailable() -> Option<String> {
    let configured = match std::env::var(AGENT_INCUS_BIN_ENV) {
        Ok(value) => Some(value),
        Err(error) => {
            tracing::debug!("{AGENT_INCUS_BIN_ENV} is not set or is not unicode: {error}");
            None
        }
    };
    agent_backend_unavailable_for(std::env::consts::OS, configured.as_deref(), binary_on_path)
}

fn agent_backend_unavailable_for(
    platform: &str,
    configured: Option<&str>,
    exists: impl Fn(&str) -> bool,
) -> Option<String> {
    let configured = configured.map(str::trim).filter(|value| !value.is_empty());
    let binary = configured.unwrap_or("incus");
    match platform {
        "windows" => {
            // An explicit wrapper (or remote client) is trusted: it may not
            // need WSL at all. `botagent` executes a single binary name, so
            // the default `incus` cannot reach the CLI inside the WSL2 distro.
            if configured.is_some_and(|value| value != "incus") {
                return None;
            }
            if !exists("wsl") {
                return Some(
                    "agent workspace unavailable on Windows: WSL2 is not installed (wsl.exe not found); run `PROD-DEPENDENCIES.ps1 -Check` for the exact remediation"
                        .to_string(),
                );
            }
            Some(
                "agent workspace unavailable on Windows: botagent cannot execute Incus inside WSL2; set AGENT_INCUS_BIN to a WSL wrapper (for example a script running `wsl -d <distro> -u root -- incus`) or to a remote Incus client, then run `PROD-DEPENDENCIES.ps1 -Check` to verify WSL2 + Incus"
                    .to_string(),
            )
        }
        "macos" => {
            if exists(binary) {
                None
            } else {
                Some(
                    "agent workspace unavailable on macOS: `incus` was not found on PATH and there is no WSL2 backend; the desktop bundle is not built for macOS"
                        .to_string(),
                )
            }
        }
        _ => {
            if exists(binary) {
                None
            } else {
                Some(format!(
                    "agent workspace unavailable: `{binary}` was not found on PATH; install Incus (`DEPENDENCIES.sh`) or set AGENT_INCUS_BIN to a remote Incus client"
                ))
            }
        }
    }
}

/// Checks for an executable on `PATH` without spawning a process.
fn binary_on_path(binary: &str) -> bool {
    if binary.contains('/') || binary.contains('\\') {
        return std::path::Path::new(binary).is_file();
    }
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| executable_in(&dir, binary))
}

fn executable_in(dir: &std::path::Path, binary: &str) -> bool {
    if dir.join(binary).is_file() {
        return true;
    }
    #[cfg(windows)]
    {
        ["exe", "cmd", "bat", "com"]
            .iter()
            .any(|extension| dir.join(format!("{binary}.{extension}")).is_file())
    }
    #[cfg(not(windows))]
    {
        false
    }
}

impl AgentService {
    pub fn new(pool: state::DbPool) -> Self {
        let incus_bin = std::env::var(AGENT_INCUS_BIN_ENV).unwrap_or_else(|_| "incus".to_string());
        let max_snapshots_per_session = std::env::var("AGENT_MAX_SNAPSHOTS_PER_SESSION")
            .ok()
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(10);
        let idle_timeout_secs = std::env::var("AGENT_IDLE_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(3600);
        Self {
            pool,
            incus_bin,
            max_snapshots_per_session,
            idle_timeout_secs,
        }
    }

    pub fn pool(&self) -> &state::DbPool {
        &self.pool
    }

    pub fn incus_bin(&self) -> &str {
        &self.incus_bin
    }

    pub fn max_snapshots_per_session(&self) -> i64 {
        self.max_snapshots_per_session
    }

    pub fn idle_timeout_secs(&self) -> i64 {
        self.idle_timeout_secs
    }
}

pub fn configure_routes() -> axum::Router<Arc<AgentService>> {
    use axum::routing::{delete, get, post};
    axum::Router::new()
        // Legacy per-session toggle (kept for backward compat).
        .route("/api/agent/sessions/mode", post(api::set_agent_mode))
        .route("/api/agent/sessions/current", get(api::current_session))
        // Per-user VM (#1580).
        .route("/api/agent/vm/resources", get(api::user_vm_resources))
        .route("/api/agent/vm/status", get(api::user_vm_status))
        .route("/api/agent/vm/exec", post(api::vm_exec))
        .route("/api/agent/vm/file", get(api::vm_file))
        .route("/api/agent/vm/files", get(api::vm_files))
        .route(
            "/api/agent/conversations/switch",
            post(api::switch_conversation),
        )
        // Snapshots.
        .route(
            "/api/agent/sessions/:id/snapshots",
            get(snapshots::list_snapshots).post(snapshots::create_snapshot),
        )
        .route(
            "/api/agent/snapshots/:id/restore",
            post(snapshots::restore_snapshot),
        )
        .route(
            "/api/agent/snapshots/:id",
            delete(snapshots::delete_snapshot),
        )
        // Sandbox.
        .route("/api/v1/sandbox/exec", post(api::sandbox_exec))
        // Org keys.
        .route(
            "/api/cloud/orgkeys",
            get(keys::list_org_keys).post(keys::create_org_key),
        )
        .route("/api/cloud/orgkeys/:id", delete(keys::revoke_org_key))
}
