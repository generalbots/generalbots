use chrono::{DateTime, Utc};
use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

static VERSION_REGISTRY: RwLock<Option<VersionRegistry>> = RwLock::new(None);

pub const BOTSERVER_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const BOTSERVER_NAME: &str = env!("CARGO_PKG_NAME");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentVersion {
    pub name: String,
    pub version: String,
    pub latest_version: Option<String>,
    pub update_available: bool,
    pub status: ComponentStatus,
    pub last_checked: Option<DateTime<Utc>>,
    pub source: ComponentSource,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ComponentStatus {
    Running,
    Stopped,
    Error,
    Updating,
    NotInstalled,
    Unknown,
}

impl std::fmt::Display for ComponentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Running => write!(f, "[OK] Running"),
            Self::Stopped => write!(f, "[STOP] Stopped"),
            Self::Error => write!(f, "[ERR] Error"),
            Self::Updating => write!(f, "[UPD] Updating"),
            Self::NotInstalled => write!(f, "[--] Not Installed"),
            Self::Unknown => write!(f, "[?] Unknown"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ComponentSource {
    Builtin,
    Docker,
    Lxc,
    System,
    Binary,
    External,
}

impl std::fmt::Display for ComponentSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Builtin => write!(f, "Built-in"),
            Self::Docker => write!(f, "Docker"),
            Self::Lxc => write!(f, "LXC"),
            Self::System => write!(f, "System"),
            Self::Binary => write!(f, "Binary"),
            Self::External => write!(f, "External"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionRegistry {
    pub core_version: String,
    pub components: HashMap<String, ComponentVersion>,
    pub last_update_check: Option<DateTime<Utc>>,
    pub update_url: Option<String>,
}

impl Default for VersionRegistry {
    fn default() -> Self {
        Self {
            core_version: BOTSERVER_VERSION.to_string(),
            components: HashMap::new(),
            last_update_check: None,
            update_url: Some("https://api.generalbots.com/updates".to_string()),
        }
    }
}

impl VersionRegistry {
    #[must_use]
    pub fn new() -> Self {
        let mut registry = Self::default();
        registry.register_builtin_components();
        registry
    }

    fn register_builtin_components(&mut self) {
        self.register_component(ComponentVersion {
            name: "botserver".to_string(),
            version: BOTSERVER_VERSION.to_string(),
            latest_version: None,
            update_available: false,
            status: ComponentStatus::Running,
            last_checked: Some(Utc::now()),
            source: ComponentSource::Builtin,
            metadata: HashMap::from([
                ("description".to_string(), "Core bot server".to_string()),
                (
                    "repo".to_string(),
                    "https://github.com/GeneralBots/botserver".to_string(),
                ),
            ]),
        });

        self.register_component(ComponentVersion {
            name: "basic".to_string(),
            version: BOTSERVER_VERSION.to_string(),
            latest_version: None,
            update_available: false,
            status: ComponentStatus::Running,
            last_checked: Some(Utc::now()),
            source: ComponentSource::Builtin,
            metadata: HashMap::from([(
                "description".to_string(),
                "BASIC script interpreter".to_string(),
            )]),
        });

        self.register_component(ComponentVersion {
            name: "llm".to_string(),
            version: BOTSERVER_VERSION.to_string(),
            latest_version: None,
            update_available: false,
            status: ComponentStatus::Running,
            last_checked: Some(Utc::now()),
            source: ComponentSource::Builtin,
            metadata: HashMap::from([(
                "description".to_string(),
                "LLM integration (Claude, GPT, etc.)".to_string(),
            )]),
        });
    }

    pub fn register_component(&mut self, component: ComponentVersion) {
        debug!(
            "Registered component: {} v{}",
            component.name, component.version
        );
        self.components.insert(component.name.clone(), component);
    }

    pub fn update_status(&mut self, name: &str, status: ComponentStatus) {
        if let Some(component) = self.components.get_mut(name) {
            component.status = status;
        }
    }

    pub fn update_version(&mut self, name: &str, version: String) {
        if let Some(component) = self.components.get_mut(name) {
            component.version = version;
            component.last_checked = Some(Utc::now());
        }
    }

    #[must_use]
    pub fn get_component(&self, name: &str) -> Option<&ComponentVersion> {
        self.components.get(name)
    }

    #[must_use]
    pub const fn get_all_components(&self) -> &HashMap<String, ComponentVersion> {
        &self.components
    }

    #[must_use]
    pub fn get_available_updates(&self) -> Vec<&ComponentVersion> {
        self.components
            .values()
            .filter(|c| c.update_available)
            .collect()
    }

    #[must_use]
    pub fn summary(&self) -> String {
        let running = self
            .components
            .values()
            .filter(|c| c.status == ComponentStatus::Running)
            .count();
        let total = self.components.len();
        let updates = self.get_available_updates().len();

        format!(
            "{BOTSERVER_NAME} v{} | {running}/{total} components running | {updates} updates available",
            self.core_version
        )
    }

    /// Serialize the registry to a JSON string.
    ///
    /// # Errors
    /// Returns an error if serialization fails.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

pub fn init_version_registry() {
    let registry = VersionRegistry::new();
    if let Ok(mut guard) = VERSION_REGISTRY.write() {
        *guard = Some(registry);
    }
}

#[must_use]
pub fn version_registry() -> Option<VersionRegistry> {
    VERSION_REGISTRY.read().ok()?.clone()
}

pub fn version_registry_mut(
) -> Option<std::sync::RwLockWriteGuard<'static, Option<VersionRegistry>>> {
    VERSION_REGISTRY.write().ok()
}

pub fn register_component(component: ComponentVersion) {
    if let Ok(mut guard) = VERSION_REGISTRY.write() {
        if let Some(ref mut registry) = *guard {
            registry.register_component(component);
        }
    }
}

pub fn update_component_status(name: &str, status: ComponentStatus) {
    if let Ok(mut guard) = VERSION_REGISTRY.write() {
        if let Some(ref mut registry) = *guard {
            registry.update_status(name, status);
        }
    }
}

#[must_use]
pub fn get_component_version(name: &str) -> Option<ComponentVersion> {
    VERSION_REGISTRY
        .read()
        .ok()?
        .as_ref()?
        .get_component(name)
        .cloned()
}

#[must_use]
pub const fn get_botserver_version() -> &'static str {
    BOTSERVER_VERSION
}

#[must_use]
pub fn version_string() -> String {
    format!("{BOTSERVER_NAME} v{BOTSERVER_VERSION}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = VersionRegistry::new();
        assert!(!registry.core_version.is_empty());
        assert!(registry.components.contains_key("botserver"));
    }

    #[test]
    fn test_component_registration() {
        let mut registry = VersionRegistry::new();
        registry.register_component(ComponentVersion {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            latest_version: None,
            update_available: false,
            status: ComponentStatus::Running,
            last_checked: None,
            source: ComponentSource::Builtin,
            metadata: HashMap::new(),
        });
        assert!(registry.get_component("test").is_some());
    }

    #[test]
    fn test_status_display() {
        assert_eq!(ComponentStatus::Running.to_string(), "[OK] Running");
        assert_eq!(ComponentStatus::Error.to_string(), "[ERR] Error");
    }

    #[test]
    fn test_version_string() {
        let vs = version_string();
        assert!(!vs.is_empty());
        assert!(vs.contains('v'));
    }

    #[test]
    fn test_source_display() {
        assert_eq!(ComponentSource::Builtin.to_string(), "Built-in");
        assert_eq!(ComponentSource::Docker.to_string(), "Docker");
    }

    #[test]
    fn test_update_status() {
        let mut registry = VersionRegistry::new();
        registry.update_status("botserver", ComponentStatus::Stopped);
        let component = registry.get_component("botserver");
        assert!(
            component.is_some(),
            "botserver component should exist in registry"
        );
        assert_eq!(component.map(|c| c.status), Some(ComponentStatus::Stopped));
    }

    #[test]
    fn test_summary() {
        let registry = VersionRegistry::new();
        let summary = registry.summary();
        assert!(summary.contains("components running"));
    }
}
