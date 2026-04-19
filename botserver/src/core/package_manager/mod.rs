pub mod cache;
pub mod component;
pub mod container;
pub mod installer;
pub mod os;
pub mod setup;
pub mod alm_setup;
pub use cache::{CacheResult, DownloadCache};
pub use container::{ContainerOperations, ContainerSettings, NatRule};
pub use installer::PackageManager;
pub mod cli;
pub mod facade;
use serde::{Serialize, Deserialize};
use rand::Rng;

/// Generate a cryptographically strong random string for passwords, tokens, etc.
pub fn generate_random_string(length: usize) -> String {
    let charset = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rng();
    (0..length)
        .map(|_| {
            let idx = rng.random_range(0..charset.len());
            charset[idx] as char
        })
        .collect()
}
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
pub use alm_setup::setup_alm;





/// Initialize Directory (Zitadel) with default admin user and OAuth application
/// This should be called after Zitadel has started and is responding
#[cfg(feature = "directory")]
pub async fn setup_directory() -> anyhow::Result<crate::core::package_manager::setup::DirectoryConfig> {
    use std::path::PathBuf;
    use std::collections::HashMap;
    use crate::core::shared::utils::get_stack_path;

    let stack_path = get_stack_path();

    let base_url = "".to_string();
    let config_path = PathBuf::from(&stack_path).join("conf/system/directory_config.json");

    // Check if config already exists in Vault first
    if let Ok(secrets_manager) = crate::core::secrets::SecretsManager::get() {
        if secrets_manager.is_enabled() {
            if let Ok(secrets) = secrets_manager.get_secret(crate::core::secrets::SecretPaths::DIRECTORY).await {
                if let (Some(client_id), Some(client_secret)) = (secrets.get("client_id"), secrets.get("client_secret")) {
                    // Validate that credentials are real, not placeholders
                    let is_valid = !client_id.is_empty()
                        && !client_secret.is_empty()
                        && client_secret != "..."
                        && client_id.contains('@') // OAuth client IDs contain @
                        && client_secret.len() > 10; // Real secrets are longer than placeholders

                    if is_valid {
                        log::info!("Directory already configured with OAuth client in Vault");
                        // Reconstruct config from Vault
                        let config = crate::core::package_manager::setup::DirectoryConfig {
                            base_url: base_url.clone(),
                            issuer_url: secrets.get("issuer_url").cloned().unwrap_or_else(|| base_url.clone()),
                            issuer: secrets.get("issuer").cloned().unwrap_or_else(|| base_url.clone()),
                            client_id: client_id.clone(),
                            client_secret: client_secret.clone(),
                            redirect_uri: secrets.get("redirect_uri").cloned().unwrap_or_else(|| "/auth/callback".to_string()),
                            project_id: secrets.get("project_id").cloned().unwrap_or_default(),
                            api_url: secrets.get("api_url").cloned().unwrap_or_else(|| base_url.clone()),
                            service_account_key: secrets.get("service_account_key").cloned(),
                        };
                        return Ok(config);
                    }
                }
            }
        }
    }

    // Check if config already exists with valid OAuth client in file
    if config_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str::<crate::core::package_manager::setup::DirectoryConfig>(&content) {
                // Validate that credentials are real, not placeholders
                let is_valid = !config.client_id.is_empty()
                    && !config.client_secret.is_empty()
                    && config.client_secret != "..."
                    && config.client_id.contains('@')
                    && config.client_secret.len() > 10;

                if is_valid {
                    log::info!("Directory already configured with OAuth client");
                    return Ok(config);
                }
            }
        }
    }

    // Initialize directory with default credentials
    let mut directory_setup = crate::core::package_manager::setup::DirectorySetup::new(base_url.clone(), config_path.clone());
    let config = directory_setup.initialize().await
        .map_err(|e| anyhow::anyhow!("Failed to initialize directory: {}", e))?;

    // Store credentials in Vault
    if let Ok(secrets_manager) = crate::core::secrets::SecretsManager::get() {
        if secrets_manager.is_enabled() {
            let mut secrets = HashMap::new();
            secrets.insert("url".to_string(), config.base_url.clone());
            secrets.insert("issuer_url".to_string(), config.issuer_url.clone());
            secrets.insert("issuer".to_string(), config.issuer.clone());
            secrets.insert("client_id".to_string(), config.client_id.clone());
            secrets.insert("client_secret".to_string(), config.client_secret.clone());
            secrets.insert("redirect_uri".to_string(), config.redirect_uri.clone());
            secrets.insert("project_id".to_string(), config.project_id.clone());
            secrets.insert("api_url".to_string(), config.api_url.clone());
            if let Some(key) = &config.service_account_key {
                secrets.insert("service_account_key".to_string(), key.clone());
            }

            match secrets_manager.put_secret(crate::core::secrets::SecretPaths::DIRECTORY, secrets).await {
                Ok(_) => log::info!("Directory credentials stored in Vault"),
                Err(e) => log::warn!("Failed to store directory credentials in Vault: {}", e),
            }
        }
    }

    Ok(config)
}
