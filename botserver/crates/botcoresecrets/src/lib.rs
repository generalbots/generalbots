pub mod env_defaults;
pub mod hooks;
pub mod manager;
pub mod mtls;
pub mod paths;
pub mod peer;
pub mod service_configs;
pub mod sync;
pub mod tenant;
pub mod vault_addr;

pub use env_defaults::init_secrets_manager;
pub use manager::SecretsManager;
pub use paths::SecretPaths;
pub use peer::peer_secret_sync;
pub use service_configs::ServiceConfigResult;
pub use sync::{csrf_secret_sync, put_csrf_secret_sync, secret_sync};
pub use tenant::{alm_config, app_runtime, internal_api_token, TenantSecrets};
pub use vault_addr::{local_vault_addr, pick_vault_addr, vault_addr_from_env};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;

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
