use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionMode {
    Manual,
    Auto,
    Bypass,
}

impl PermissionMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::Auto => "auto",
            Self::Bypass => "bypass",
        }
    }
}

impl std::fmt::Display for PermissionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

const DESTRUCTIVE_TOOL_PREFIXES: &[&str] = &[
    "file/delete",
    "file/write",
    "shell/run",
    "git/commit",
    "git/init",
    "git/push",
    "git/pr",
    "git/checkout",
    "publish/project",
    "deploy_app",
    "domain/bind",
    "domain/unbind",
    "domain/tls",
    "ops/restart",
    "ops/rollback",
    "backup/restore",
    "backup/snapshot",
    "test/run",
    "canvas/update",
    "canvas/delete",
    "issue/update",
    "issue/close",
    "skill/delete",
    "browser/close",
];

pub struct PermissionEngine {
    mode: RwLock<PermissionMode>,
}

impl PermissionEngine {
    pub fn new() -> Self {
        Self {
            mode: RwLock::new(PermissionMode::Manual),
        }
    }

    pub async fn mode(&self) -> PermissionMode {
        *self.mode.read().await
    }

    pub async fn set_mode(&self, mode: PermissionMode) {
        *self.mode.write().await = mode;
    }

    pub fn is_destructive(&self, tool_name: &str) -> bool {
        DESTRUCTIVE_TOOL_PREFIXES
            .iter()
            .any(|prefix| tool_name.starts_with(prefix))
    }

    pub fn requires_approval(&self, schema_requires: bool, tool_name: &str, mode: PermissionMode) -> bool {
        match mode {
            PermissionMode::Manual => schema_requires || self.is_destructive(tool_name),
            PermissionMode::Auto => self.is_destructive(tool_name),
            PermissionMode::Bypass => false,
        }
    }
}

impl Default for PermissionEngine {
    fn default() -> Self {
        Self::new()
    }
}

pub type PermissionEngineRef = Arc<PermissionEngine>;
