
use log::info;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::OnceLock;

static BRANDING: OnceLock<BrandingConfig> = OnceLock::new();

const DEFAULT_PLATFORM_NAME: &str = "General Bots";
const DEFAULT_PLATFORM_SHORT: &str = "GB";
const DEFAULT_PLATFORM_DOMAIN: &str = "generalbots.com";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandingConfig {
    pub name: String,
    pub short_name: String,
    pub company: Option<String>,
    pub domain: Option<String>,
    pub support_email: Option<String>,
    pub logo_url: Option<String>,
    pub favicon_url: Option<String>,
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub footer_text: Option<String>,
    pub copyright: Option<String>,
    pub custom_css: Option<String>,
    pub terms_url: Option<String>,
    pub privacy_url: Option<String>,
    pub docs_url: Option<String>,
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
            primary_color: Some("#25d366".to_string()),
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
    #[must_use]
    pub fn load() -> Self {
        let search_paths = [
            ".product",
            "config/.product",
            "/etc/botserver/.product",
            "/opt/gbo/.product",
        ];

        for path in &search_paths {
            if let Ok(config) = Self::load_from_file(path) {
                info!("Loaded white-label branding from {path}: {}", config.name);
                return config;
            }
        }

        if let Ok(product_file) = std::env::var("PRODUCT_FILE") {
            if let Ok(config) = Self::load_from_file(&product_file) {
                info!(
                    "Loaded white-label branding from PRODUCT_FILE={product_file}: {}",
                    config.name
                );
                return config;
            }
        }

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

    fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let path = Path::new(path);
        if !path.exists() {
            return Err("File not found".into());
        }

        let content = std::fs::read_to_string(path)?;

        if let Ok(config) = toml::from_str::<ProductFile>(&content) {
            return Ok(config.into());
        }

        let mut config = Self {
            is_white_label: true,
            ..Self::default()
        };

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


pub fn init_branding() {
    let config = BrandingConfig::load();
    let _ = BRANDING.set(config);
}

#[must_use]
pub fn branding() -> &'static BrandingConfig {
    BRANDING.get_or_init(BrandingConfig::load)
}

#[must_use]
pub fn platform_name() -> &'static str {
    &branding().name
}

#[must_use]
pub fn platform_short() -> &'static str {
    &branding().short_name
}

#[must_use]
pub fn is_white_label() -> bool {
    branding().is_white_label
}

#[must_use]
pub fn copyright_text() -> String {
    branding().copyright.clone().unwrap_or_else(|| {
        format!(
            "© {} {}",
            chrono::Utc::now().format("%Y"),
            branding().company.as_deref().unwrap_or(&branding().name)
        )
    })
}

#[must_use]
pub fn footer_text() -> String {
    branding()
        .footer_text
        .clone()
        .unwrap_or_else(|| format!("Powered by {}", platform_name()))
}

#[must_use]
pub fn log_prefix() -> String {
    format!("[{}]", platform_short())
}

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
    fn test_platform_name_function() {
        let name = platform_name();
        assert!(!name.is_empty());
    }
}
