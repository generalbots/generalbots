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
pub static PRODUCT_CONFIG: Lazy<RwLock<ProductConfig>> = Lazy::new(|| {
    RwLock::new(ProductConfig::load().unwrap_or_default())
});

/// Product configuration structure
#[derive(Debug, Clone)]
pub struct ProductConfig {
    /// Product name (replaces "General Bots" throughout the application)
    pub name: String,

    /// Set of active apps
    pub apps: HashSet<String>,

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
}

impl Default for ProductConfig {
    fn default() -> Self {
        let mut apps = HashSet::new();
        // All apps enabled by default
        for app in &[
            "chat", "mail", "calendar", "drive", "tasks", "docs", "paper",
            "sheet", "slides", "meet", "research", "sources", "analytics",
            "admin", "monitoring", "settings",
        ] {
            apps.insert(app.to_string());
        }

        Self {
            name: "General Bots".to_string(),
            apps,
            theme: "sentient".to_string(),
            logo: None,
            favicon: None,
            primary_color: None,
            support_email: None,
            docs_url: Some("https://docs.pragmatismo.com.br".to_string()),
            copyright: None,
        }
    }
}

impl ProductConfig {
    /// Load configuration from .product file
    pub fn load() -> Result<Self, ProductConfigError> {
        let paths = [
            ".product",
            "./botserver/.product",
            "../.product",
        ];

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
        let template = self.copyright.as_deref()
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
        let mut config = PRODUCT_CONFIG.write()
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

/// Helper function to check if an app is enabled
pub fn is_app_enabled(app: &str) -> bool {
    PRODUCT_CONFIG
        .read()
        .map(|c| c.is_app_enabled(app))
        .unwrap_or(true)
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
    let compiled = crate::core::features::COMPILED_FEATURES;
    
    // Get current config
    let config = PRODUCT_CONFIG.read().ok();

    // Determine effective apps (intersection of enabled + compiled)
    let effective_apps: Vec<String> = config
        .as_ref()
        .map(|c| c.get_enabled_apps())
        .unwrap_or_default()
        .into_iter()
        .filter(|app| compiled.contains(&app.as_str()) || app == "settings" || app == "auth") // Always allow settings/auth
        .collect();

    match config {
        Some(c) => serde_json::json!({
            "name": c.name,
            "apps": effective_apps,
            "compiled_features": compiled,
            "version": env!("CARGO_PKG_VERSION"),
            "theme": c.theme,
            "logo": c.logo,
            "favicon": c.favicon,
            "primary_color": c.primary_color,
            "docs_url": c.docs_url,
            "copyright": c.get_copyright(),
        }),
        None => serde_json::json!({
            "name": "General Bots",
            "apps": compiled, // If no config, show all compiled
            "compiled_features": compiled,
            "version": env!("CARGO_PKG_VERSION"),
            "theme": "sentient",
        })
    }
}

/// Get workspace manifest with detailed feature information
pub fn get_workspace_manifest() -> serde_json::Value {
    let manifest = crate::core::manifest::WorkspaceManifest::new();
    serde_json::to_value(manifest).unwrap_or_else(|_| serde_json::json!({}))
}


/// Middleware to check if an app is enabled before allowing API access
pub async fn app_gate_middleware(
    req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    let path = req.uri().path();

    // Map API paths to app names
    let app_name = match path {
        p if p.starts_with("/api/calendar") => Some("calendar"),
        p if p.starts_with("/api/mail") || p.starts_with("/api/email") => Some("mail"),
        p if p.starts_with("/api/drive") || p.starts_with("/api/files") => Some("drive"),
        p if p.starts_with("/api/tasks") => Some("tasks"),
        p if p.starts_with("/api/docs") => Some("docs"),
        p if p.starts_with("/api/paper") => Some("paper"),
        p if p.starts_with("/api/sheet") => Some("sheet"),
        p if p.starts_with("/api/slides") => Some("slides"),
        p if p.starts_with("/api/meet") => Some("meet"),
        p if p.starts_with("/api/research") => Some("research"),
        p if p.starts_with("/api/sources") => Some("sources"),
        p if p.starts_with("/api/analytics") => Some("analytics"),
        p if p.starts_with("/api/admin") => Some("admin"),
        p if p.starts_with("/api/monitoring") => Some("monitoring"),
        p if p.starts_with("/api/settings") => Some("settings"),
        p if p.starts_with("/api/ui/calendar") => Some("calendar"),
        p if p.starts_with("/api/ui/mail") => Some("mail"),
        p if p.starts_with("/api/ui/drive") => Some("drive"),
        p if p.starts_with("/api/ui/tasks") => Some("tasks"),
        p if p.starts_with("/api/ui/docs") => Some("docs"),
        p if p.starts_with("/api/ui/paper") => Some("paper"),
        p if p.starts_with("/api/ui/sheet") => Some("sheet"),
        p if p.starts_with("/api/ui/slides") => Some("slides"),
        p if p.starts_with("/api/ui/meet") => Some("meet"),
        p if p.starts_with("/api/ui/research") => Some("research"),
        p if p.starts_with("/api/ui/sources") => Some("sources"),
        p if p.starts_with("/api/ui/analytics") => Some("analytics"),
        p if p.starts_with("/api/ui/admin") => Some("admin"),
        p if p.starts_with("/api/ui/monitoring") => Some("monitoring"),
        p if p.starts_with("/api/ui/settings") => Some("settings"),
        _ => None, // Allow all other paths
    };

    // Check if the app is enabled
    if let Some(app) = app_name {
        // First check: is it even compiled?
        // Note: settings, auth, admin are core features usually, but we check anyway if they are in features list
        // Some core apps like settings might not be in feature flags explicitly or always enabled.
        // For simplicity, if it's not in compiled features but is a known core route, we might allow it,
        // but here we enforce strict feature containment.
        // Exception: 'settings' and 'auth' are often core. 
        if app != "settings" && app != "auth" && !crate::core::features::is_feature_compiled(app) {
             let error_response = serde_json::json!({
                "error": "not_implemented",
                "message": format!("The '{}' feature is not compiled in this build", app),
                "code": 501
            });

            return (
                StatusCode::NOT_IMPLEMENTED,
                axum::Json(error_response)
            ).into_response();
        }

        if !is_app_enabled(app) {
            let error_response = serde_json::json!({
                "error": "app_disabled",
                "message": format!("The '{}' app is not enabled for this installation", app),
                "code": 403
            });

            return (
                StatusCode::FORBIDDEN,
                axum::Json(error_response)
            ).into_response();
        }
    }

    next.run(req).await
}

/// Get list of disabled apps for logging/debugging
pub fn get_disabled_apps() -> Vec<String> {
    let all_apps = vec![
        "chat", "mail", "calendar", "drive", "tasks", "docs", "paper",
        "sheet", "slides", "meet", "research", "sources", "analytics",
        "admin", "monitoring", "settings",
    ];

    all_apps
        .into_iter()
        .filter(|app| !is_app_enabled(app))
        .map(|s| s.to_string())
        .collect()
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
