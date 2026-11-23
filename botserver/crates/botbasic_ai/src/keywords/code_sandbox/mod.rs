pub(crate) use botbasic_types::BasicRuntime;
pub(crate) use botbasic_types::UserSession;
pub(crate) use botsecurity::command_guard::SafeCommand;
pub(crate) use log::{trace, warn};
pub(crate) use rhai::{Dynamic, Engine};
pub(crate) use serde::{Deserialize, Serialize};
pub(crate) use std::collections::HashMap;

/// Backend used to isolate untrusted, model/tool-generated code.
///
/// `Process` executes code directly on the botserver host and must never be the
/// default. `Unknown` is the fail-closed sink for unrecognized config strings.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SandboxRuntime {
    LXC,

    Docker,

    Firecracker,

    Process,

    Unknown,
}

impl Default for SandboxRuntime {
    fn default() -> Self {
        // Untrusted code must never default to host execution. Firecracker is
        // the preferred isolated backend; if it is unavailable execution fails
        // closed instead of falling back to `Process`.
        Self::Firecracker
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CodeLanguage {
    Python,
    JavaScript,
    Bash,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub enabled: bool,

    pub runtime: SandboxRuntime,

    pub timeout_seconds: u64,

    pub memory_limit_mb: u64,

    pub cpu_limit_percent: u32,

    pub network_enabled: bool,

    pub work_dir: String,

    pub env_vars: HashMap<String, String>,

    pub allowed_paths: Vec<String>,

    pub python_packages: Vec<String>,
}

/// Explicit, unsafe operator override required to run untrusted code directly
/// on the botserver host. Any other value (including unset) denies `Process`.
pub(crate) const UNISOLATED_PROCESS_OVERRIDE_ENV: &str = "GB_SANDBOX_ALLOW_UNISOLATED_PROCESS";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RuntimeSelectionError {
    Disabled,
    UnknownRuntime,
    UnisolatedProcessRequiresOverride,
}

impl std::fmt::Display for RuntimeSelectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disabled => f.write_str(
                "code sandbox is disabled (per-bot opt-in: sandbox-enabled=true)",
            ),
            Self::UnknownRuntime => f.write_str("unknown sandbox runtime (fail-closed)"),
            Self::UnisolatedProcessRequiresOverride => f.write_str(
                "runtime 'process' runs untrusted code on the host and requires GB_SANDBOX_ALLOW_UNISOLATED_PROCESS=true",
            ),
        }
    }
}

/// The enforced policy handed to every runtime. Runtimes consume this instead
/// of the raw config so network/filesystem/resource limits cannot be skipped
/// when a backend is added or changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RuntimeSelection {
    pub runtime: SandboxRuntime,
    pub network_enabled: bool,
    pub allowed_paths: Vec<String>,
    pub memory_limit_mb: u64,
    pub cpu_limit_percent: u32,
}

/// Pure runtime-selection predicate: no I/O and no environment access. The
/// caller passes the override flag explicitly so the decision is unit-testable.
pub(crate) fn select_runtime(
    config: &SandboxConfig,
    allow_unisolated_process: bool,
) -> Result<RuntimeSelection, RuntimeSelectionError> {
    if !config.enabled {
        return Err(RuntimeSelectionError::Disabled);
    }

    let runtime = match config.runtime {
        SandboxRuntime::Unknown => return Err(RuntimeSelectionError::UnknownRuntime),
        SandboxRuntime::Process => {
            if !allow_unisolated_process {
                return Err(RuntimeSelectionError::UnisolatedProcessRequiresOverride);
            }
            SandboxRuntime::Process
        }
        isolated => isolated,
    };

    Ok(RuntimeSelection {
        runtime,
        network_enabled: config.network_enabled,
        allowed_paths: config.allowed_paths.clone(),
        memory_limit_mb: config.memory_limit_mb,
        cpu_limit_percent: config.cpu_limit_percent,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub stdout: String,

    pub stderr: String,

    pub exit_code: i32,

    pub execution_time_ms: u64,

    pub timed_out: bool,

    pub killed: bool,

    pub error: Option<String>,
}

pub struct CodeSandbox {
    config: SandboxConfig,
    session_id: Uuid,
}

use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

mod execute_file;
mod file_extension;
mod from_bot_config;
mod generate_python_lxc_config;
pub mod preview_store;
mod read_bot_config_from_vault;
mod run_python_keyword;
mod sandbox_runtime;
mod success;

pub use execute_file::*;
pub use generate_python_lxc_config::*;
pub use preview_store::{
    classify, shared, PreviewEntry, PreviewStatus, PreviewStore, DEFAULT_CAPACITY, DEFAULT_TTL_MS,
};
use read_bot_config_from_vault::*;
pub use run_python_keyword::*;

#[cfg(test)]
mod tests {
    use super::*;

    fn configured(runtime: SandboxRuntime, enabled: bool) -> SandboxConfig {
        let mut config = SandboxConfig::default();
        config.enabled = enabled;
        config.runtime = runtime;
        config.network_enabled = false;
        config.allowed_paths = vec!["/data".to_string(), "/tmp".to_string()];
        config
    }

    #[test]
    fn fresh_install_is_disabled_and_isolated() {
        let config = SandboxConfig::default();
        assert!(!config.enabled);
        assert_ne!(config.runtime, SandboxRuntime::Process);
        assert_eq!(
            select_runtime(&config, true),
            Err(RuntimeSelectionError::Disabled)
        );
    }

    #[test]
    fn unknown_runtime_fails_closed() {
        assert_eq!(SandboxRuntime::from("nonsense"), SandboxRuntime::Unknown);
        let config = configured(SandboxRuntime::Unknown, true);
        assert_eq!(
            select_runtime(&config, true),
            Err(RuntimeSelectionError::UnknownRuntime)
        );
    }

    #[test]
    fn host_process_requires_explicit_override() {
        let config = configured(SandboxRuntime::Process, true);
        assert_eq!(
            select_runtime(&config, false),
            Err(RuntimeSelectionError::UnisolatedProcessRequiresOverride)
        );
        let selected = select_runtime(&config, true).expect("override permits process");
        assert_eq!(selected.runtime, SandboxRuntime::Process);
    }

    #[test]
    fn isolated_runtime_enforces_network_and_allowed_paths() {
        let config = configured(SandboxRuntime::Firecracker, true);
        let selected = select_runtime(&config, false).expect("isolated runtime is allowed");
        assert_eq!(selected.runtime, SandboxRuntime::Firecracker);
        assert!(!selected.network_enabled);
        assert_eq!(selected.allowed_paths, vec!["/data", "/tmp"]);
        assert_eq!(selected.memory_limit_mb, config.memory_limit_mb);
        assert_eq!(selected.cpu_limit_percent, config.cpu_limit_percent);
    }

    #[test]
    fn network_flag_is_carried_into_enforced_policy() {
        let mut config = configured(SandboxRuntime::Docker, true);
        config.network_enabled = true;
        let selected = select_runtime(&config, false).expect("docker is isolated");
        assert!(selected.network_enabled);
    }
}
