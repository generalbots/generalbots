//! Product Configuration Module
//!
//! This module handles white-label settings loaded from the `.product` file.
//! It provides a global configuration that can be used throughout the application
//! to customize branding, enabled apps, and default theme.

use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::sync::RwLock;
use tracing::{info, warn};

/// Global product configuration instance
pub static PRODUCT_CONFIG: Lazy<RwLock<ProductConfig>> =
    Lazy::new(|| RwLock::new(ProductConfig::load().unwrap_or_default()));

/// Product configuration structure
#[derive(Debug, Clone)]
pub struct ProductConfig {
    /// Product name (replaces "General Bots" throughout the application)
    pub name: String,

    /// Set of active apps
    pub apps: HashSet<String>,

    /// Applications that are only surfaced while Preview mode is on (#1348).
    /// A preview app is deliberately absent from `apps`, so it stays out of the
    /// launcher and the sidebar until the user turns the switch on.
    pub preview_apps: HashSet<String>,

    /// Default theme
    pub theme: String,

    /// Logo URL (optional)
    pub logo: Option<String>,

    /// Favicon URL (optional)
    pub favicon: Option<String>,

    /// Primary color override (optional)
    pub primary_color: Option<String>,

    /// Support email (optional)
    pub support_email: Option<String>,

    /// Documentation URL (optional)
    pub docs_url: Option<String>,

    /// Copyright text (optional)
    pub copyright: Option<String>,

    /// Search mechanism enabled (optional)
    /// Controls whether the omnibox/search toolbar is displayed in the suite
    pub search_enabled: Option<bool>,

    /// Menu launcher enabled (optional)
    /// Controls whether the apps menu launcher is displayed in the suite
    pub menu_launcher_enabled: Option<bool>,

    /// Sidebar visibility (optional)
    /// Controls whether the left sidebar is shown in the suite.
    /// When false, the sidebar is hidden and the main content takes full width.
    pub sidebar: Option<bool>,
}

impl Default for ProductConfig {
    fn default() -> Self {
        let mut apps = HashSet::new();
        // All apps enabled by default (docs/slides excluded — OOXML SDK gate)
        for app in &[
            "chat",
            "mail",
            "calendar",
            "drive",
            "tasks",
            "paper",
            "sheet",
            "meet",
            "research",
            "sources",
            "analytics",
            "admin",
            "monitoring",
            "settings",
            "attendant",
            "tools",
            "video",
            "player",
            "canvas",
            "social",
            "people",
            "crm",
            "tickets",
            "billing",
            "products",
            "designer",
            "workspace",
            "project",
            "goals",
            "editor",
            "learn",
            "vibe",
            "campaigns",
            "lists",
            "templates",
            "terminal",
            "browser",
            "database",
            "plan",
            "compliance",
            "tax",
            "vision",
            "fraud",
            "erp",
            "integrations",
            "itsm",
            "hr",
            "banking",
            "sales",
            "pos",
            "retail",
            "handoff",
            "kyc",
            "biometry",
            "timeclock",
            "m365",
            "office365",
            "minutes",
        ] {
            apps.insert(app.to_string());
        }

        Self {
            name: "General Bots".to_string(),
            apps,
            preview_apps: HashSet::new(),
            theme: "sentient".to_string(),
            logo: None,
            favicon: None,
            primary_color: None,
            support_email: None,
            docs_url: None,
            copyright: None,
            search_enabled: Some(false),
            menu_launcher_enabled: Some(false),
            sidebar: None,
        }
    }
}

impl ProductConfig {
    /// Load configuration from .product file
    pub fn load() -> Result<Self, ProductConfigError> {
        let paths = [".product", "./botserver/.product", "../.product"];

        let mut content = None;
        for path in &paths {
            if Path::new(path).exists() {
                content = Some(fs::read_to_string(path).map_err(ProductConfigError::IoError)?);
                info!("Loaded product configuration from: {}", path);
                break;
            }
        }

        let content = match content {
            Some(c) => c,
            None => {
                warn!("No .product file found, using default configuration");
                return Ok(Self::default());
            }
        };

        Self::parse(&content)
    }

    /// Parse configuration from string content
    pub fn parse(content: &str) -> Result<Self, ProductConfigError> {
        let mut config = Self::default();
        let mut apps_specified = false;

        for line in content.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse key=value pairs
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim().to_lowercase();
                let value = value.trim();

                match key.as_str() {
                    "name" => {
                        if !value.is_empty() {
                            config.name = value.to_string();
                        }
                    }
                    "apps" => {
                        apps_specified = true;
                        config.apps.clear();
                        for app in value.split(',') {
                            let app = app.trim().to_lowercase();
                            if !app.is_empty() {
                                config.apps.insert(app);
                            }
                        }
                    }
                    "preview_apps" => {
                        config.preview_apps.clear();
                        for app in value.split(',') {
                            let app = app.trim().to_lowercase();
                            if !app.is_empty() {
                                config.preview_apps.insert(app);
                            }
                        }
                    }
                    "theme" => {
                        if !value.is_empty() {
                            config.theme = value.to_string();
                        }
                    }
                    "logo" => {
                        if !value.is_empty() {
                            config.logo = Some(value.to_string());
                        }
                    }
                    "favicon" => {
                        if !value.is_empty() {
                            config.favicon = Some(value.to_string());
                        }
                    }
                    "primary_color" => {
                        if !value.is_empty() {
                            config.primary_color = Some(value.to_string());
                        }
                    }
                    "support_email" => {
                        if !value.is_empty() {
                            config.support_email = Some(value.to_string());
                        }
                    }
                    "docs_url" => {
                        if !value.is_empty() {
                            config.docs_url = Some(value.to_string());
                        }
                    }
                    "copyright" => {
                        if !value.is_empty() {
                            config.copyright = Some(value.to_string());
                        }
                    }
                    "search_enabled" => {
                        let enabled = value.eq_ignore_ascii_case("true")
                            || value == "1"
                            || value.eq_ignore_ascii_case("yes");
                        config.search_enabled = Some(enabled);
                    }
                    "menu_launcher_enabled" => {
                        let enabled = value.eq_ignore_ascii_case("true")
                            || value == "1"
                            || value.eq_ignore_ascii_case("yes");
                        config.menu_launcher_enabled = Some(enabled);
                    }
                    "sidebar" => {
                        let enabled = value.eq_ignore_ascii_case("true")
                            || value == "1"
                            || value.eq_ignore_ascii_case("yes");
                        config.sidebar = Some(enabled);
                    }
                    _ => {
                        warn!("Unknown product configuration key: {}", key);
                    }
                }
            }
        }

        if !apps_specified {
            info!("No apps specified in .product, all apps enabled by default");
        }

        info!(
            "Product config loaded: name='{}', apps={:?}, theme='{}'",
            config.name, config.apps, config.theme
        );

        Ok(config)
    }

    /// Check if an app is enabled
    pub fn is_app_enabled(&self, app: &str) -> bool {
        self.apps.contains(&app.to_lowercase())
    }

    /// Check whether an app is a preview application, i.e. one that is only
    /// surfaced while Preview mode is on (#1348). Preview applications are not
    /// part of `apps`, so `is_app_enabled` still answers `false` for them: the
    /// switch decides what is visible, never whether the app is installed.
    pub fn is_app_preview(&self, app: &str) -> bool {
        self.preview_apps.contains(&app.to_lowercase())
    }

    /// Preview applications, sorted so the catalog and the manifest are stable.
    pub fn get_preview_apps(&self) -> Vec<String> {
        let mut preview: Vec<String> = self.preview_apps.iter().cloned().collect();
        preview.sort();
        preview
    }

    /// Get the product name
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Get the default theme
    pub fn get_theme(&self) -> &str {
        &self.theme
    }

    /// Replace "General Bots" with the product name in a string
    pub fn replace_branding(&self, text: &str) -> String {
        text.replace("General Bots", &self.name)
            .replace("general bots", &self.name.to_lowercase())
            .replace("GENERAL BOTS", &self.name.to_uppercase())
    }

    /// Get copyright text with year substitution
    pub fn get_copyright(&self) -> String {
        let year = chrono::Utc::now().format("%Y").to_string();
        let template = self
            .copyright
            .as_deref()
            .unwrap_or("© {year} {name}. All rights reserved.");

        template
            .replace("{year}", &year)
            .replace("{name}", &self.name)
    }

    /// Get all enabled apps as a vector
    pub fn get_enabled_apps(&self) -> Vec<String> {
        self.apps.iter().cloned().collect()
    }

    /// Reload configuration from file
    pub fn reload() -> Result<(), ProductConfigError> {
        let new_config = Self::load()?;
        let mut config = PRODUCT_CONFIG
            .write()
            .map_err(|_| ProductConfigError::LockError)?;
        *config = new_config;
        info!("Product configuration reloaded");
        Ok(())
    }
}

/// Error type for product configuration
#[derive(Debug)]
pub enum ProductConfigError {
    IoError(std::io::Error),
    ParseError(String),
    LockError,
}

impl std::fmt::Display for ProductConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "IO error reading .product file: {}", e),
            Self::ParseError(msg) => write!(f, "Parse error in .product file: {}", msg),
            Self::LockError => write!(f, "Failed to acquire lock on product configuration"),
        }
    }
}

impl std::error::Error for ProductConfigError {}

/// Helper function to get product name
pub fn get_product_name() -> String {
    PRODUCT_CONFIG
        .read()
        .map(|c| c.name.clone())
        .unwrap_or_else(|_| "General Bots".to_string())
}


/// Helper function to check whether an app is a preview application (#1348).
/// When the product configuration cannot be read, the answer is `false`: an
/// unreadable configuration must not promote unreleased apps into the launcher.
pub fn is_app_preview(app: &str) -> bool {
    PRODUCT_CONFIG
        .read()
        .map(|c| c.is_app_preview(app))
        .unwrap_or(false)
}

/// Helper function to get default theme
pub fn get_default_theme() -> String {
    PRODUCT_CONFIG
        .read()
        .map(|c| c.theme.clone())
        .unwrap_or_else(|_| "sentient".to_string())
}

/// Helper function to replace branding in text
pub fn replace_branding(text: &str) -> String {
    PRODUCT_CONFIG
        .read()
        .map(|c| c.replace_branding(text))
        .unwrap_or_else(|_| text.to_string())
}

/// Helper function to get product config for serialization
pub fn get_product_config_json() -> serde_json::Value {
    // Get compiled features from our new module
    let compiled = crate::features::COMPILED_FEATURES;

    // Get current config
    let config = PRODUCT_CONFIG.read().ok();

    // Effective apps come straight from the .product file. There is no runtime
    // guard: the `apps` list controls what the launcher and sidebar show, and
    // nothing more. Removing an app from it does not disable its routes.
    let effective_apps: Vec<String> = config
        .as_ref()
        .map(|c| c.get_enabled_apps())
        .unwrap_or_default()
        .into_iter()
        .collect();

    match config {
        Some(c) => serde_json::json!({
            "name": c.name,
            "apps": effective_apps,
            "preview_apps": c.get_preview_apps(),
            "compiled_features": compiled,
            "version": env!("CARGO_PKG_VERSION"),
            "theme": c.theme,
            "logo": c.logo,
            "favicon": c.favicon,
            "primary_color": c.primary_color,
            "docs_url": c.docs_url,
            "copyright": c.get_copyright(),
            "search_enabled": c.search_enabled.unwrap_or(false),
            "menu_launcher_enabled": c.menu_launcher_enabled.unwrap_or(false),
            "sidebar": c.sidebar,
        }),
        None => serde_json::json!({
            "name": "General Bots",
            "apps": compiled, // If no config, show all compiled
            "preview_apps": Vec::<String>::new(),
            "compiled_features": compiled,
            "version": env!("CARGO_PKG_VERSION"),
            "theme": "sentient",
            "search_enabled": false,
            "menu_launcher_enabled": false,
            "sidebar": null,
        }),
    }
}

/// Get workspace manifest with detailed feature information
pub fn get_workspace_manifest() -> serde_json::Value {
    let manifest = crate::manifest::WorkspaceManifest::new();
    serde_json::to_value(manifest).unwrap_or_else(|_| serde_json::json!({}))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ProductConfig::default();
        assert_eq!(config.name, "General Bots");
        assert_eq!(config.theme, "sentient");
        assert!(config.is_app_enabled("chat"));
        assert!(config.is_app_enabled("drive"));
    }

    #[test]
    fn test_parse_config() {
        let content = r#"
# Test config
name=My Custom Bot
apps=chat,drive,tasks
theme=dark
        "#;

        let config = ProductConfig::parse(content).unwrap();
        assert_eq!(config.name, "My Custom Bot");
        assert_eq!(config.theme, "dark");
        assert!(config.is_app_enabled("chat"));
        assert!(config.is_app_enabled("drive"));
        assert!(config.is_app_enabled("tasks"));
        assert!(!config.is_app_enabled("mail"));
        assert!(!config.is_app_enabled("calendar"));
    }

    #[test]
    fn test_replace_branding() {
        let config = ProductConfig {
            name: "Acme Bot".to_string(),
            ..Default::default()
        };

        assert_eq!(
            config.replace_branding("Welcome to General Bots"),
            "Welcome to Acme Bot"
        );
    }

    #[test]
    fn test_preview_apps_are_parsed_and_stay_disabled() {
        let content = "apps=chat,drive\npreview_apps=designer, marketPlace ,fraud";
        let config = ProductConfig::parse(content).unwrap();

        assert!(config.is_app_preview("designer"));
        assert!(config.is_app_preview("DESIGNER"));
        assert!(config.is_app_preview("marketplace"));
        assert!(!config.is_app_preview("chat"));

        // A preview application is not an active application: the Preview
        // switch decides what is visible, not whether the app is installed.
        assert!(!config.is_app_enabled("designer"));
        assert_eq!(
            config.get_preview_apps(),
            vec!["designer", "fraud", "marketplace"]
        );
    }

    #[test]
    fn test_missing_preview_key_leaves_the_set_empty() {
        let config = ProductConfig::parse("apps=chat").unwrap();
        assert!(config.get_preview_apps().is_empty());
        assert!(!config.is_app_preview("chat"));
    }

    #[test]
    fn test_case_insensitive_apps() {
        let content = "apps=Chat,DRIVE,Tasks";
        let config = ProductConfig::parse(content).unwrap();

        assert!(config.is_app_enabled("chat"));
        assert!(config.is_app_enabled("CHAT"));
        assert!(config.is_app_enabled("Chat"));
        assert!(config.is_app_enabled("drive"));
        assert!(config.is_app_enabled("tasks"));
    }
}
