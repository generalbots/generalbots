pub(crate) use botbasic_types::BasicRuntime;
pub(crate) use botbasic_types::UserSession;
pub(crate) use botsecurity::command_guard::SafeCommand;
pub(crate) use log::{trace, warn};
pub(crate) use rhai::{Dynamic, Engine};
pub(crate) use serde::{Deserialize, Serialize};
pub(crate) use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum SandboxRuntime {
    LXC,

    Docker,

    Firecracker,

    #[default]
    Process,
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
use tokio::time::timeout;
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
