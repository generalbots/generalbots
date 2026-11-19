//! Container Vault lifecycle facade.
//!
//! #2498 — the plaintext writes that used to live here (root token in `.env`,
//! `/root/.vault-token` fallback, unseal keys on disk) are gone. Both entry
//! points delegate to `botcorepkg::installer_vault::container_env`, which keeps
//! the root token transient and persists only a scoped sidecar token in the
//! private credential store.

use super::*;

impl PackageManager {
    pub(super) fn initialize_vault(container_name: &str, ip: &str) -> Result<()> {
        botcorepkg::installer_vault::container_env::initialize_vault(container_name, ip)
    }

    pub fn generate_env_from_vault(container_name: &str) -> Result<String> {
        botcorepkg::installer_vault::container_env::generate_env_from_vault(container_name)
    }
}
