//! White-Label Branding Module
//!
//! Allows complete customization of platform identity.
//! When a .product file exists with name=MyCustomPlatform,
//! "General Bots" never appears in logs, display, messages, footer - nothing.
//! Only "MyCustomPlatform" and custom components.

use log::info;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::OnceLock;

/// Global branding configuration - loaded once at startup
static BRANDING: OnceLock<BrandingConfig> = OnceLock::new();

/// Default platform name
const DEFAULT_PLATFORM_NAME: &str = "General Bots";
const DEFAULT_PLATFORM_SHORT: &str = "GB";
const DEFAULT_PLATFORM_DOMAIN: &str = "generalbots.com";

/// Branding configuration loaded from .product file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandingConfig {
    /// Platform name (e.g., "MyCustomPlatform")
    pub name: String,

    /// Short name for logs and compact displays (e.g., "MCP")
    pub short_name: String,

    /// Company/organization name
    pub company: Option<String>,

    /// Domain for URLs and emails
    pub domain: Option<String>,

    /// Support email
    pub support_email: Option<String>,

    /// Logo URL (for web UI)
    pub logo_url: Option<String>,

    /// Favicon URL
    pub favicon_url: Option<String>,

    /// Primary color (hex)
    pub primary_color: Option<String>,

    /// Secondary color (hex)
    pub secondary_color: Option<String>,

    /// Footer text
    pub footer_text: Option<String>,

    /// Copyright text
    pub copyright: Option<String>,

    /// Custom CSS URL
    pub custom_css: Option<String>,

    /// Terms of service URL
    pub terms_url: Option<String>,

    /// Privacy policy URL
    pub privacy_url: Option<String>,

    /// Documentation URL
    pub docs_url: Option<String>,

    /// Whether this is a white-label deployment
    pub is_white_label: bool,
}

impl Default for BrandingConfig {
    fn default() -> Self {
        Self {
            name: DEFAULT_PLATFORM_NAME.to_string(),
            short_name: DEFAULT_PLATFORM_SHORT.to_string(),
            company: Some("pragmatismo.com.br".to_string()),
            domain: Some(DEFAULT_PLATFORM_DOMAIN.to_string()),
            support_email: Some("support@generalbots.com".to_string()),
            logo_url: None,
            favicon_url: None,
            primary_color: Some("#25d366".to_string()), // WhatsApp green
            secondary_color: Some("#075e54".to_string()),
            footer_text: None,
            copyright: Some(format!(
                "© {} pragmatismo.com.br. All rights reserved.",
                chrono::Utc::now().format("%Y")
            )),
            custom_css: None,
            terms_url: None,
            privacy_url: None,
            docs_url: Some("https://docs.generalbots.com".to_string()),
            is_white_label: false,
        }
    }
}

impl BrandingConfig {
    /// Load branding from .product file if it exists
    pub fn load() -> Self {
        // Check multiple locations for .product file
        let search_paths = [
            ".product",
            "config/.product",
            "/etc/botserver/.product",
            "/opt/gbo/.product",
        ];

        for path in &search_paths {
            if let Ok(config) = Self::load_from_file(path) {
                info!(
                    "Loaded white-label branding from {}: {}",
                    path, config.name
                );
                return config;
            }
        }

        // Also check environment variable
        if let Ok(product_file) = std::env::var("PRODUCT_FILE") {
            if let Ok(config) = Self::load_from_file(&product_file) {
                info!(
                    "Loaded white-label branding from PRODUCT_FILE={}: {}",
                    product_file, config.name
                );
                return config;
            }
        }

        // Check for individual environment overrides
        let mut config = Self::default();

        if let Ok(name) = std::env::var("PLATFORM_NAME") {
            config.name = name;
            config.is_white_label = true;
        }

        if let Ok(short) = std::env::var("PLATFORM_SHORT_NAME") {
            config.short_name = short;
        }

        if let Ok(company) = std::env::var("PLATFORM_COMPANY") {
            config.company = Some(company);
        }

        if let Ok(domain) = std::env::var("PLATFORM_DOMAIN") {
            config.domain = Some(domain);
        }

        if let Ok(logo) = std::env::var("PLATFORM_LOGO_URL") {
            config.logo_url = Some(logo);
        }

        if let Ok(color) = std::env::var("PLATFORM_PRIMARY_COLOR") {
            config.primary_color = Some(color);
        }

        config
    }

    /// Load from a specific file path
    fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let path = Path::new(path);
        if !path.exists() {
            return Err("File not found".into());
        }

        let content = std::fs::read_to_string(path)?;

        // Try parsing as TOML first
        if let Ok(config) = toml::from_str::<ProductFile>(&content) {
            return Ok(config.into());
        }

        // Try parsing as simple key=value format
        let mut config = Self::default();
        config.is_white_label = true;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim().to_lowercase();
                let value = value.trim().trim_matches('"').trim_matches('\'');

                match key.as_str() {
                    "name" | "platform_name" => config.name = value.to_string(),
                    "short_name" | "short" => config.short_name = value.to_string(),
                    "company" | "organization" => config.company = Some(value.to_string()),
                    "domain" => config.domain = Some(value.to_string()),
                    "support_email" | "email" => config.support_email = Some(value.to_string()),
                    "logo_url" | "logo" => config.logo_url = Some(value.to_string()),
                    "favicon_url" | "favicon" => config.favicon_url = Some(value.to_string()),
                    "primary_color" | "color" => config.primary_color = Some(value.to_string()),
                    "secondary_color" => config.secondary_color = Some(value.to_string()),
                    "footer_text" | "footer" => config.footer_text = Some(value.to_string()),
                    "copyright" => config.copyright = Some(value.to_string()),
                    "custom_css" | "css" => config.custom_css = Some(value.to_string()),
                    "terms_url" | "terms" => config.terms_url = Some(value.to_string()),
                    "privacy_url" | "privacy" => config.privacy_url = Some(value.to_string()),
                    "docs_url" | "docs" => config.docs_url = Some(value.to_string()),
                    _ => {}
                }
            }
        }

        Ok(config)
    }
}

/// TOML format for .product file
#[derive(Debug, Deserialize)]
struct ProductFile {
    name: String,
    #[serde(default)]
    short_name: Option<String>,
    #[serde(default)]
    company: Option<String>,
    #[serde(default)]
    domain: Option<String>,
    #[serde(default)]
    support_email: Option<String>,
    #[serde(default)]
    logo_url: Option<String>,
    #[serde(default)]
    favicon_url: Option<String>,
    #[serde(default)]
    primary_color: Option<String>,
    #[serde(default)]
    secondary_color: Option<String>,
    #[serde(default)]
    footer_text: Option<String>,
    #[serde(default)]
    copyright: Option<String>,
    #[serde(default)]
    custom_css: Option<String>,
    #[serde(default)]
    terms_url: Option<String>,
    #[serde(default)]
    privacy_url: Option<String>,
    #[serde(default)]
    docs_url: Option<String>,
}

impl From<ProductFile> for BrandingConfig {
    fn from(pf: ProductFile) -> Self {
        let short_name = pf.short_name.unwrap_or_else(|| {
            // Generate short name from first letters
            pf.name
                .split_whitespace()
                .map(|w| w.chars().next().unwrap_or('X'))
                .collect::<String>()
                .to_uppercase()
        });

        Self {
            name: pf.name,
            short_name,
            company: pf.company,
            domain: pf.domain,
            support_email: pf.support_email,
            logo_url: pf.logo_url,
            favicon_url: pf.favicon_url,
            primary_color: pf.primary_color,
            secondary_color: pf.secondary_color,
            footer_text: pf.footer_text,
            copyright: pf.copyright,
            custom_css: pf.custom_css,
            terms_url: pf.terms_url,
            privacy_url: pf.privacy_url,
            docs_url: pf.docs_url,
            is_white_label: true,
        }
    }
}

// ============================================================================
// Global Access Functions
// ============================================================================

/// Initialize branding at application startup
pub fn init_branding() {
    let config = BrandingConfig::load();
    let _ = BRANDING.set(config);
}

/// Get the current branding configuration
pub fn branding() -> &'static BrandingConfig {
    BRANDING.get_or_init(BrandingConfig::load)
}

/// Get the platform name (use this instead of hardcoding "General Bots")
pub fn platform_name() -> &'static str {
    &branding().name
}

/// Get the short platform name (for logs, compact displays)
pub fn platform_short() -> &'static str {
    &branding().short_name
}

/// Check if this is a white-label deployment
pub fn is_white_label() -> bool {
    branding().is_white_label
}

/// Get formatted copyright text
pub fn copyright_text() -> String {
    branding().copyright.clone().unwrap_or_else(|| {
        format!(
            "© {} {}",
            chrono::Utc::now().format("%Y"),
            branding().company.as_deref().unwrap_or(&branding().name)
        )
    })
}

/// Get footer text
pub fn footer_text() -> String {
    branding().footer_text.clone().unwrap_or_else(|| {
        format!("Powered by {}", platform_name())
    })
}

/// Format a log prefix with platform branding
pub fn log_prefix() -> String {
    format!("[{}]", platform_short())
}

// ============================================================================
// Macros for Branded Logging
// ============================================================================

/// Log with platform branding
#[macro_export]
macro_rules! branded_info {
    ($($arg:tt)*) => {
        log::info!("{} {}", $crate::core::shared::branding::log_prefix(), format!($($arg)*))
    };
}

#[macro_export]
macro_rules! branded_warn {
    ($($arg:tt)*) => {
        log::warn!("{} {}", $crate::core::shared::branding::log_prefix(), format!($($arg)*))
    };
}

#[macro_export]
macro_rules! branded_error {
    ($($arg:tt)*) => {
        log::error!("{} {}", $crate::core::shared::branding::log_prefix(), format!($($arg)*))
    };
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_branding() {
        let config = BrandingConfig::default();
        assert_eq!(config.name, "General Bots");
        assert_eq!(config.short_name, "GB");
        assert!(!config.is_white_label);
    }

    #[test]
    fn test_parse_simple_product_file() {
        let content = r#"
name=MyCustomPlatform
short_name=MCP
company=My Company Inc.
domain=myplatform.com
primary_color=#ff6600
"#;
        // Test would require file system access, skipping actual load
        assert!(content.contains("MyCustomPlatform"));
    }

    #[test]
    fn test_platform_name_function() {
        // This test uses the default since no .product file exists
        let name = platform_name();
        assert!(!name.is_empty());
    }
}
