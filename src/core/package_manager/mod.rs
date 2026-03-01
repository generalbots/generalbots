pub mod cache;
pub mod component;
pub mod installer;
pub mod os;
pub mod setup;
pub use cache::{CacheResult, DownloadCache};
pub use installer::PackageManager;
pub mod cli;
pub mod facade;
use serde::{Serialize, Deserialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstallMode {
    Local,
    Container,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OsType {
    Linux,
    MacOS,
    Windows,
}
#[derive(Debug)]
pub struct ComponentInfo {
    pub name: &'static str,
    pub termination_command: &'static str,
}
pub fn get_all_components() -> Vec<ComponentInfo> {
    vec![
        ComponentInfo {
            name: "tables",
            termination_command: "postgres",
        },
        ComponentInfo {
            name: "cache",
            termination_command: "redis-server",
        },
        ComponentInfo {
            name: "drive",
            termination_command: "minio",
        },
        ComponentInfo {
            name: "llm",
            termination_command: "llama-server",
        },
    ]
}

/// Parse Zitadel log file to extract initial admin credentials
#[cfg(feature = "directory")]
fn extract_initial_admin_from_log(log_path: &std::path::Path) -> Option<(String, String)> {
    use std::fs;

    let log_content = fs::read_to_string(log_path).ok()?;

    // Try different log formats from Zitadel
    // Format 1: "initial admin user created. email: admin@<domain> password: <password>"
    for line in log_content.lines() {
        let line_lower = line.to_lowercase();
        if line_lower.contains("initial admin") || line_lower.contains("admin credentials") {
            // Try to extract email and password
            let email = if let Some(email_start) = line.find("email:") {
                let rest = &line[email_start + 6..];
                rest.trim()
                    .split_whitespace()
                    .next()
                    .map(|s| s.trim_end_matches(',').to_string())
            } else if let Some(email_start) = line.find("Email:") {
                let rest = &line[email_start + 6..];
                rest.trim()
                    .split_whitespace()
                    .next()
                    .map(|s| s.trim_end_matches(',').to_string())
            } else {
                None
            };

            let password = if let Some(pwd_start) = line.find("password:") {
                let rest = &line[pwd_start + 9..];
                rest.trim()
                    .split_whitespace()
                    .next()
                    .map(|s| s.trim_end_matches(',').to_string())
            } else if let Some(pwd_start) = line.find("Password:") {
                let rest = &line[pwd_start + 9..];
                rest.trim()
                    .split_whitespace()
                    .next()
                    .map(|s| s.trim_end_matches(',').to_string())
            } else {
                None
            };

            if let (Some(email), Some(password)) = (email, password) {
                if !email.is_empty() && !password.is_empty() {
                    log::info!("Extracted initial admin credentials from log: {}", email);
                    return Some((email, password));
                }
            }
        }
    }

    // Try multiline format
    // Admin credentials:
    //   Email: admin@localhost
    //   Password: xxxxx
    let lines: Vec<&str> = log_content.lines().collect();
    for i in 0..lines.len().saturating_sub(2) {
        if lines[i].to_lowercase().contains("admin credentials") {
            let mut email = None;
            let mut password = None;

            for j in (i + 1)..std::cmp::min(i + 5, lines.len()) {
                let line = lines[j];
                if line.contains("Email:") {
                    email = line.split("Email:")
                        .nth(1)
                        .map(|s| s.trim().to_string());
                }
                if line.contains("Password:") {
                    password = line.split("Password:")
                        .nth(1)
                        .map(|s| s.trim().to_string());
                }
            }

            if let (Some(e), Some(p)) = (email, password) {
                if !e.is_empty() && !p.is_empty() {
                    log::info!("Extracted initial admin credentials from multiline log: {}", e);
                    return Some((e, p));
                }
            }
        }
    }

    None
}

/// Initialize Directory (Zitadel) with default admin user and OAuth application
/// This should be called after Zitadel has started and is responding
#[cfg(feature = "directory")]
pub async fn setup_directory() -> anyhow::Result<crate::core::package_manager::setup::DirectoryConfig> {
    use std::path::PathBuf;

    let stack_path = std::env::var("BOTSERVER_STACK_PATH")
        .unwrap_or_else(|_| "./botserver-stack".to_string());

    let base_url = "http://localhost:8300".to_string();
    let config_path = PathBuf::from(&stack_path).join("conf/system/directory_config.json");

    // Check if config already exists
    if config_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str::<crate::core::package_manager::setup::DirectoryConfig>(&content) {
                if !config.client_id.is_empty() && !config.client_secret.is_empty() {
                    log::info!("Directory already configured with OAuth client");
                    return Ok(config);
                }
            }
        }
    }

    // Try to get initial admin credentials from Zitadel log
    let log_path = PathBuf::from(&stack_path).join("logs/zitadel.log");
    let admin_credentials = extract_initial_admin_from_log(&log_path);

    let mut directory_setup = if let Some((email, password)) = admin_credentials {
        log::info!("Using initial admin credentials from log for OAuth client creation");
        crate::core::package_manager::setup::DirectorySetup::with_admin_credentials(
            base_url,
            config_path.clone(),
            email,
            password,
        )
    } else {
        log::warn!(
            "Could not extract initial admin credentials from Zitadel log at {}. \
             OAuth client creation may fail. Check if Zitadel has started properly.",
            log_path.display()
        );
        crate::core::package_manager::setup::DirectorySetup::new(
            base_url,
            config_path.clone()
        )
    };

    directory_setup.initialize().await
        .map_err(|e| anyhow::anyhow!("Failed to initialize directory: {}", e))
}
