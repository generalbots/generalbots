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
    "domain/security",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_as_str_and_display() {
        assert_eq!(PermissionMode::Manual.as_str(), "manual");
        assert_eq!(PermissionMode::Auto.as_str(), "auto");
        assert_eq!(PermissionMode::Bypass.as_str(), "bypass");
        assert_eq!(format!("{}", PermissionMode::Auto), "auto");
    }

    #[test]
    fn mode_serde_round_trip() {
        let json = serde_json::to_string(&PermissionMode::Auto).unwrap();
        assert_eq!(json, "\"auto\"");
        let back: PermissionMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, PermissionMode::Auto);
    }

    #[test]
    fn manual_requires_schema_flag_or_destructive() {
        let engine = PermissionEngine::new();
        assert!(engine.requires_approval(true, "web/search", PermissionMode::Manual));
        assert!(engine.requires_approval(false, "file/delete", PermissionMode::Manual));
        assert!(!engine.requires_approval(false, "web/search", PermissionMode::Manual));
    }

    #[test]
    fn auto_requires_destructive_only() {
        let engine = PermissionEngine::new();
        assert!(!engine.requires_approval(true, "web/search", PermissionMode::Auto));
        assert!(engine.requires_approval(false, "git/push", PermissionMode::Auto));
    }

    #[test]
    fn bypass_never_requires_approval() {
        let engine = PermissionEngine::new();
        assert!(!engine.requires_approval(true, "git/push", PermissionMode::Bypass));
    }

    #[test]
    fn destructive_prefix_matching() {
        let engine = PermissionEngine::new();
        assert!(engine.is_destructive("file/delete"));
        assert!(engine.is_destructive("git/commit"));
        assert!(engine.is_destructive("ops/restart"));
        assert!(!engine.is_destructive("file/list"));
        assert!(!engine.is_destructive("web/search"));
    }

    #[tokio::test]
    async fn mode_defaults_to_manual_and_can_change() {
        let engine = PermissionEngine::new();
        assert_eq!(engine.mode().await, PermissionMode::Manual);
        engine.set_mode(PermissionMode::Bypass).await;
        assert_eq!(engine.mode().await, PermissionMode::Bypass);
        assert_eq!(PermissionEngine::default().mode().await, PermissionMode::Manual);
    }
}
