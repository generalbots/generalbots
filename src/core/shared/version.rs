//! Version Tracking Module
//!
//! Tracks versions of all components and checks for updates.
//! Displays in Monitor tab of Suite and UITree (Console).

use chrono::{DateTime, Utc};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

/// Global version registry
static VERSION_REGISTRY: RwLock<Option<VersionRegistry>> = RwLock::new(None);

/// Current botserver version from Cargo.toml
pub const BOTSERVER_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const BOTSERVER_NAME: &str = env!("CARGO_PKG_NAME");

/// Component version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentVersion {
    /// Component name (e.g., "drive", "llm", "email")
    pub name: String,

    /// Current installed version
    pub version: String,

    /// Latest available version (if known)
    pub latest_version: Option<String>,

    /// Whether an update is available
    pub update_available: bool,

    /// Component status
    pub status: ComponentStatus,

    /// Last check time
    pub last_checked: Option<DateTime<Utc>>,

    /// Source/origin of the component
    pub source: ComponentSource,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Component status
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
            ComponentStatus::Running => write!(f, "✅ Running"),
            ComponentStatus::Stopped => write!(f, "⏹️ Stopped"),
            ComponentStatus::Error => write!(f, "❌ Error"),
            ComponentStatus::Updating => write!(f, "🔄 Updating"),
            ComponentStatus::NotInstalled => write!(f, "⚪ Not Installed"),
            ComponentStatus::Unknown => write!(f, "❓ Unknown"),
        }
    }
}

/// Component source type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ComponentSource {
    /// Built into botserver
    Builtin,
    /// Docker container
    Docker,
    /// LXC container
    Lxc,
    /// System package (apt, yum, etc.)
    System,
    /// Downloaded binary
    Binary,
    /// External service
    External,
}

impl std::fmt::Display for ComponentSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComponentSource::Builtin => write!(f, "Built-in"),
            ComponentSource::Docker => write!(f, "Docker"),
            ComponentSource::Lxc => write!(f, "LXC"),
            ComponentSource::System => write!(f, "System"),
            ComponentSource::Binary => write!(f, "Binary"),
            ComponentSource::External => write!(f, "External"),
        }
    }
}

/// Version registry holding all component versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionRegistry {
    /// Botserver core version
    pub core_version: String,

    /// All registered components
    pub components: HashMap<String, ComponentVersion>,

    /// Last global update check
    pub last_update_check: Option<DateTime<Utc>>,

    /// Update check URL
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
    /// Create a new version registry
    pub fn new() -> Self {
        let mut registry = Self::default();
        registry.register_builtin_components();
        registry
    }

    /// Register built-in components
    fn register_builtin_components(&mut self) {
        // Core botserver
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

        // BASIC interpreter
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

        // LLM module
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

    /// Register a component
    pub fn register_component(&mut self, component: ComponentVersion) {
        debug!(
            "Registered component: {} v{}",
            component.name, component.version
        );
        self.components.insert(component.name.clone(), component);
    }

    /// Update component status
    pub fn update_status(&mut self, name: &str, status: ComponentStatus) {
        if let Some(component) = self.components.get_mut(name) {
            component.status = status;
        }
    }

    /// Update component version
    pub fn update_version(&mut self, name: &str, version: &str) {
        if let Some(component) = self.components.get_mut(name) {
            component.version = version.to_string();
            component.last_checked = Some(Utc::now());
        }
    }

    /// Check for updates for all components
    pub async fn check_updates(
        &mut self,
    ) -> Result<Vec<UpdateInfo>, Box<dyn std::error::Error + Send + Sync>> {
        info!("Checking for component updates...");
        self.last_update_check = Some(Utc::now());

        let mut updates = Vec::new();

        // Check GitHub releases for botserver
        if let Ok(latest) = Self::check_github_release("GeneralBots", "botserver").await {
            if let Some(component) = self.components.get_mut("botserver") {
                component.latest_version = Some(latest.clone());
                component.update_available = Self::is_newer_version(&component.version, &latest);
                component.last_checked = Some(Utc::now());

                if component.update_available {
                    updates.push(UpdateInfo {
                        component: "botserver".to_string(),
                        current_version: component.version.clone(),
                        new_version: latest,
                        release_notes: None,
                    });
                }
            }
        }

        // Check botmodels
        if let Ok(latest) = Self::check_github_release("GeneralBots", "botmodels").await {
            if let Some(component) = self.components.get_mut("botmodels") {
                component.latest_version = Some(latest.clone());
                component.update_available = Self::is_newer_version(&component.version, &latest);
                component.last_checked = Some(Utc::now());

                if component.update_available {
                    updates.push(UpdateInfo {
                        component: "botmodels".to_string(),
                        current_version: component.version.clone(),
                        new_version: latest,
                        release_notes: None,
                    });
                }
            }
        }

        if updates.is_empty() {
            info!("All components are up to date");
        } else {
            info!("{} update(s) available", updates.len());
        }

        Ok(updates)
    }

    /// Check GitHub releases for latest version
    async fn check_github_release(
        owner: &str,
        repo: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            owner, repo
        );

        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("User-Agent", format!("botserver/{}", BOTSERVER_VERSION))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("GitHub API error: {}", response.status()).into());
        }

        let release: GitHubRelease = response.json().await?;
        Ok(release.tag_name.trim_start_matches('v').to_string())
    }

    /// Compare versions (semver-like)
    fn is_newer_version(current: &str, latest: &str) -> bool {
        let parse_version = |v: &str| -> Vec<u32> {
            v.trim_start_matches('v')
                .split('.')
                .filter_map(|s| s.parse().ok())
                .collect()
        };

        let current_parts = parse_version(current);
        let latest_parts = parse_version(latest);

        for i in 0..3 {
            let c = current_parts.get(i).copied().unwrap_or(0);
            let l = latest_parts.get(i).copied().unwrap_or(0);
            if l > c {
                return true;
            } else if c > l {
                return false;
            }
        }
        false
    }

    /// Get all components with updates available
    pub fn get_available_updates(&self) -> Vec<&ComponentVersion> {
        self.components
            .values()
            .filter(|c| c.update_available)
            .collect()
    }

    /// Get component by name
    pub fn get_component(&self, name: &str) -> Option<&ComponentVersion> {
        self.components.get(name)
    }

    /// Get all components
    pub fn get_all_components(&self) -> Vec<&ComponentVersion> {
        self.components.values().collect()
    }

    /// Generate version summary for display
    pub fn summary(&self) -> String {
        let mut lines = vec![
            format!("═══════════════════════════════════════════════"),
            format!("  Component Versions"),
            format!("═══════════════════════════════════════════════"),
        ];

        let mut components: Vec<_> = self.components.values().collect();
        components.sort_by(|a, b| a.name.cmp(&b.name));

        for component in components {
            let update_indicator = if component.update_available {
                format!(" ⬆️ {}", component.latest_version.as_deref().unwrap_or("?"))
            } else {
                String::new()
            };

            lines.push(format!(
                "  {:15} v{:10} {}{}",
                component.name, component.version, component.status, update_indicator
            ));
        }

        if let Some(last_check) = self.last_update_check {
            lines.push(format!("───────────────────────────────────────────────"));
            lines.push(format!(
                "  Last checked: {}",
                last_check.format("%Y-%m-%d %H:%M UTC")
            ));
        }

        lines.push(format!("═══════════════════════════════════════════════"));
        lines.join("\n")
    }

    /// Generate JSON version info for API
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "core_version": self.core_version,
            "components": self.components,
            "last_update_check": self.last_update_check,
            "updates_available": self.get_available_updates().len()
        })
    }
}

/// GitHub release response
#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    #[allow(dead_code)]
    name: Option<String>,
    #[allow(dead_code)]
    body: Option<String>,
}

/// Update information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub component: String,
    pub current_version: String,
    pub new_version: String,
    pub release_notes: Option<String>,
}

// ============================================================================
// Global Functions
// ============================================================================

/// Initialize the global version registry
pub fn init_version_registry() {
    let mut guard = VERSION_REGISTRY.write().unwrap();
    if guard.is_none() {
        *guard = Some(VersionRegistry::new());
        info!(
            "Version registry initialized - botserver v{}",
            BOTSERVER_VERSION
        );
    }
}

/// Get the global version registry
pub fn version_registry() -> std::sync::RwLockReadGuard<'static, Option<VersionRegistry>> {
    VERSION_REGISTRY.read().unwrap()
}

/// Get mutable access to the global version registry
pub fn version_registry_mut() -> std::sync::RwLockWriteGuard<'static, Option<VersionRegistry>> {
    VERSION_REGISTRY.write().unwrap()
}

/// Register a new component in the global registry
pub fn register_component(component: ComponentVersion) {
    if let Some(ref mut registry) = *version_registry_mut() {
        registry.register_component(component);
    } else {
        warn!(
            "Version registry not initialized when registering component: {}",
            component.name
        );
    }
}

/// Update component status in the global registry
pub fn update_component_status(name: &str, status: ComponentStatus) {
    if let Some(ref mut registry) = *version_registry_mut() {
        registry.update_status(name, status);
    }
}

/// Get version of a specific component
pub fn get_component_version(name: &str) -> Option<String> {
    version_registry()
        .as_ref()
        .and_then(|r| r.get_component(name))
        .map(|c| c.version.clone())
}

/// Get botserver version
pub fn get_botserver_version() -> &'static str {
    BOTSERVER_VERSION
}

/// Generate version string for display
pub fn version_string() -> String {
    format!("{} v{}", BOTSERVER_NAME, BOTSERVER_VERSION)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert!(VersionRegistry::is_newer_version("1.0.0", "1.0.1"));
        assert!(VersionRegistry::is_newer_version("1.0.0", "1.1.0"));
        assert!(VersionRegistry::is_newer_version("1.0.0", "2.0.0"));
        assert!(!VersionRegistry::is_newer_version("1.0.1", "1.0.0"));
        assert!(!VersionRegistry::is_newer_version("1.0.0", "1.0.0"));
        assert!(VersionRegistry::is_newer_version("v1.0.0", "v1.0.1"));
    }

    #[test]
    fn test_registry_creation() {
        let registry = VersionRegistry::new();
        assert!(!registry.components.is_empty());
        assert!(registry.get_component("botserver").is_some());
    }

    #[test]
    fn test_component_registration() {
        let mut registry = VersionRegistry::new();
        registry.register_component(ComponentVersion {
            name: "test-component".to_string(),
            version: "1.0.0".to_string(),
            latest_version: None,
            update_available: false,
            status: ComponentStatus::Running,
            last_checked: None,
            source: ComponentSource::External,
            metadata: HashMap::new(),
        });

        assert!(registry.get_component("test-component").is_some());
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", ComponentStatus::Running), "✅ Running");
        assert_eq!(format!("{}", ComponentStatus::Error), "❌ Error");
    }

    #[test]
    fn test_version_string() {
        let vs = version_string();
        assert!(vs.contains(BOTSERVER_VERSION));
    }
}
