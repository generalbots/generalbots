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

/// Admin credentials structure
#[cfg(feature = "directory")]
struct AdminCredentials {
    email: String,
    password: String,
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

    // Check if config already exists with valid OAuth client
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

    // Try to get credentials from multiple sources
    let credentials = get_admin_credentials(&stack_path).await?;

    let mut directory_setup = crate::core::package_manager::setup::DirectorySetup::with_admin_credentials(
        base_url,
        config_path.clone(),
        credentials.email,
        credentials.password,
    );

    directory_setup.initialize().await
        .map_err(|e| anyhow::anyhow!("Failed to initialize directory: {}", e))
}

/// Get admin credentials from multiple sources
#[cfg(feature = "directory")]
async fn get_admin_credentials(stack_path: &str) -> anyhow::Result<AdminCredentials> {
    // Approach 1: Read from ~/.gb-setup-credentials (most reliable - from first bootstrap)
    if let Some(creds) = read_saved_credentials() {
        log::info!("Using credentials from ~/.gb-setup-credentials");
        return Ok(creds);
    }

    // Approach 2: Try to extract from Zitadel logs (fallback)
    let log_path = std::path::PathBuf::from(stack_path).join("logs/directory/zitadel.log");
    if let Some((email, password)) = extract_initial_admin_from_log(&log_path) {
        log::info!("Using credentials extracted from Zitadel log");
        return Ok(AdminCredentials { email, password });
    }

    // Approach 3: Error with helpful message
    log::error!("═══════════════════════════════════════════════════════════════");
    log::error!("❌ FAILED TO GET ZITADEL ADMIN CREDENTIALS");
    log::error!("═══════════════════════════════════════════════════════════════");
    log::error!("Could not find credentials in:");
    log::error!("  - ~/.gb-setup-credentials");
    log::error!("  - {}/logs/directory/zitadel.log", stack_path);
    log::error!("");
    log::error!("SOLUTION: Run a fresh bootstrap to create initial admin user:");
    log::error!("  1. Delete .env and botserver-stack/conf/system/.bootstrap_completed");
    log::error!("  2. Run: ./reset.sh");
    log::error!("  3. Admin credentials will be displayed and saved");
    log::error!("═══════════════════════════════════════════════════════════════");

    anyhow::bail!("No admin credentials found. Run fresh bootstrap to create them.")
}

/// Read credentials from ~/.gb-setup-credentials file
#[cfg(feature = "directory")]
fn read_saved_credentials() -> Option<AdminCredentials> {
    let home = std::env::var("HOME").ok()?;
    let creds_path = std::path::PathBuf::from(&home).join(".gb-setup-credentials");

    if !creds_path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(&creds_path).ok()?;

    // Parse credentials from file
    let mut username = None;
    let mut password = None;
    let mut email = None;

    for line in content.lines() {
        if line.contains("Username:") {
            username = line.split("Username:")
                .nth(1)
                .map(|s| s.trim().to_string());
        }
        if line.contains("Password:") {
            password = line.split("Password:")
                .nth(1)
                .map(|s| s.trim().to_string());
        }
        if line.contains("Email:") {
            email = line.split("Email:")
                .nth(1)
                .map(|s| s.trim().to_string());
        }
    }

    if let (Some(_username), Some(password), Some(email)) = (username, password, email) {
        Some(AdminCredentials { email, password })
    } else {
        None
    }
}
