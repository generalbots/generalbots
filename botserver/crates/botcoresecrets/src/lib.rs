pub mod manager;
pub mod paths;
pub mod service_configs;
pub mod tenant;
pub mod env_defaults;

pub use manager::SecretsManager;
pub use paths::SecretPaths;
pub use service_configs::ServiceConfigResult;
pub use tenant::TenantSecrets;
pub use env_defaults::init_secrets_manager;

use serde::{Deserialize, Serialize};
use std::env;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapConfig {
    pub vault_addr: String,
    pub vault_token: String,
}

impl BootstrapConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            vault_addr: env::var("VAULT_ADDR")?,
            vault_token: env::var("VAULT_TOKEN")?,
        })
    }

    pub fn is_configured() -> bool {
        env::var("VAULT_ADDR").is_ok() && env::var("VAULT_TOKEN").is_ok()
    }
}
