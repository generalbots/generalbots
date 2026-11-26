//! Whether the agent workspace (Incus) can run on this host, as one code the
//! desktop UI reads from `/api/product` (`agent_runtime_state`).
//!
//! Windows rule (Incus lives in a WSL2 distro, reached through the
//! `incus.exe` wrapper named by `AGENT_INCUS_BIN`):
//!
//! - `AGENT_INCUS_REMOTE=1`: `AGENT_INCUS_BIN` is a client for a remote Incus;
//!   no WSL check, the client only has to exist.
//! - Product setup (the desktop shell sets `AGENT_INCUS_BIN`,
//!   `GBO_WSL_DISTRO` and `GBO_AGENT_RUNTIME_MARKER`), or nothing configured:
//!   WSL installed, the distro registered, the wrapper present and the runtime
//!   marker file named by `GBO_AGENT_RUNTIME_MARKER` present (the shell sets
//!   `<stack>/agent-runtime/ready.json`, written when the runtime install
//!   ends). With nothing configured there is no wrapper, so it is never ready.
//! - Manual setup (`AGENT_INCUS_BIN` set to a wrapper by the user and no
//!   `GBO_AGENT_RUNTIME_MARKER`): the same checks without the marker, since
//!   nothing of the product installed that runtime. The distro is
//!   `GBO_WSL_DISTRO`, so a manual wrapper aimed at another distro must name
//!   it there.
//!
//! Every check reads files or the registry; nothing starts a process and the
//! WSL VM is never booted.

use std::path::PathBuf;

pub use crate::runtime_host::DEFAULT_WSL_DISTRO;
use crate::runtime_host::{valid_distro_name, RuntimeHost, SystemHost};

pub const AGENT_INCUS_REMOTE_ENV: &str = "AGENT_INCUS_REMOTE";
pub const GBO_WSL_DISTRO_ENV: &str = "GBO_WSL_DISTRO";
pub const AGENT_RUNTIME_MARKER_ENV: &str = "GBO_AGENT_RUNTIME_MARKER";

/// How long the per-turn availability answer is reused.
const CACHE_TTL: std::time::Duration = std::time::Duration::from_secs(30);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentRuntimeState {
    /// This platform has no supported Incus backend.
    Unsupported,
    /// Windows without WSL.
    WslMissing,
    /// WSL present, the agent distro is not registered for this user.
    DistroMissing,
    /// The distro exists but the wrapper or the runtime marker is missing.
    RuntimeNotInstalled,
    Ready,
}

impl AgentRuntimeState {
    /// Stable code for clients: `unsupported`, `wsl_missing`,
    /// `distro_missing`, `runtime_not_installed` or `ready`.
    pub fn code(self) -> &'static str {
        match self {
            Self::Unsupported => "unsupported",
            Self::WslMissing => "wsl_missing",
            Self::DistroMissing => "distro_missing",
            Self::RuntimeNotInstalled => "runtime_not_installed",
            Self::Ready => "ready",
        }
    }
}

/// The state plus the operator-facing reason (`None` when ready).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentRuntime {
    pub state: AgentRuntimeState,
    pub reason: Option<String>,
}

impl AgentRuntime {
    fn ready() -> Self {
        Self {
            state: AgentRuntimeState::Ready,
            reason: None,
        }
    }

    fn not(state: AgentRuntimeState, reason: impl Into<String>) -> Self {
        Self {
            state,
            reason: Some(reason.into()),
        }
    }
}

/// Fresh evaluation for this host.
pub fn agent_runtime() -> AgentRuntime {
    evaluate(&SystemHost)
}

/// Evaluation reused for 30 s, for callers on the chat-turn path.
pub fn agent_runtime_state_cached() -> AgentRuntimeState {
    use std::sync::Mutex;
    use std::time::Instant;
    static CACHE: Mutex<Option<(Instant, AgentRuntimeState)>> = Mutex::new(None);
    let Ok(mut cached) = CACHE.lock() else {
        return agent_runtime().state;
    };
    if let Some((at, state)) = *cached {
        if at.elapsed() < CACHE_TTL {
            return state;
        }
    }
    let state = agent_runtime().state;
    *cached = Some((Instant::now(), state));
    state
}

fn configured_env(host: &impl RuntimeHost, name: &str) -> Option<String> {
    host.env(name)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub(crate) fn evaluate(host: &impl RuntimeHost) -> AgentRuntime {
    let configured = configured_env(host, crate::AGENT_INCUS_BIN_ENV);
    let binary = configured.clone().unwrap_or_else(|| "incus".to_string());
    match host.platform() {
        "windows" => evaluate_windows(host, configured.as_deref()),
        "macos" if host.program_exists(&binary) => AgentRuntime::ready(),
        "macos" => AgentRuntime::not(
            AgentRuntimeState::Unsupported,
            "agent workspace unavailable on macOS: `incus` was not found on PATH and there is no WSL2 backend; the desktop bundle is not built for macOS",
        ),
        _ if host.program_exists(&binary) => AgentRuntime::ready(),
        _ => AgentRuntime::not(
            AgentRuntimeState::RuntimeNotInstalled,
            format!("agent workspace unavailable: `{binary}` was not found on PATH; install Incus (`DEPENDENCIES.sh`) or set AGENT_INCUS_BIN to a remote Incus client"),
        ),
    }
}

fn evaluate_windows(host: &impl RuntimeHost, configured: Option<&str>) -> AgentRuntime {
    // A bare `incus` cannot reach the CLI inside the distro: botagent runs a
    // single program, so on Windows only a wrapper or a remote client counts.
    let wrapper = configured.filter(|value| !value.eq_ignore_ascii_case("incus"));
    if configured_env(host, AGENT_INCUS_REMOTE_ENV).as_deref() == Some("1") {
        return match wrapper {
            Some(client) if host.program_exists(client) => AgentRuntime::ready(),
            _ => AgentRuntime::not(
                AgentRuntimeState::RuntimeNotInstalled,
                "agent workspace unavailable: AGENT_INCUS_REMOTE=1 needs AGENT_INCUS_BIN to name an existing remote Incus client",
            ),
        };
    }
    let marker = configured_env(host, AGENT_RUNTIME_MARKER_ENV).map(PathBuf::from);
    let manual = wrapper.is_some() && marker.is_none();
    if !host.wsl_installed() {
        return AgentRuntime::not(
            AgentRuntimeState::WslMissing,
            "agent workspace unavailable on Windows: WSL2 is not installed",
        );
    }
    let distro = configured_env(host, GBO_WSL_DISTRO_ENV).unwrap_or_else(|| DEFAULT_WSL_DISTRO.to_string());
    if !valid_distro_name(&distro) {
        return AgentRuntime::not(
            AgentRuntimeState::DistroMissing,
            "agent workspace unavailable on Windows: GBO_WSL_DISTRO is not a valid distro name (letters, digits, `.`, `_` and `-` only)",
        );
    }
    if !host.distro_registered(&distro) {
        return AgentRuntime::not(
            AgentRuntimeState::DistroMissing,
            format!("agent workspace unavailable on Windows: the WSL distro `{distro}` is not installed for this user"),
        );
    }
    if !wrapper.is_some_and(|path| host.program_exists(path)) {
        return AgentRuntime::not(
            AgentRuntimeState::RuntimeNotInstalled,
            "agent workspace unavailable on Windows: the incus.exe wrapper is missing (AGENT_INCUS_BIN)",
        );
    }
    if manual {
        return AgentRuntime::ready();
    }
    match marker {
        Some(path) if host.file_exists(&path) => AgentRuntime::ready(),
        _ => AgentRuntime::not(
            AgentRuntimeState::RuntimeNotInstalled,
            "agent workspace unavailable on Windows: the agent runtime is not installed yet (no agent-runtime/ready.json)",
        ),
    }
}

#[cfg(test)]
#[path = "runtime_state_tests.rs"]
mod tests;
