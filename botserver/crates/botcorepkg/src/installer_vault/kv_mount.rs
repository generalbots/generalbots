//! Idempotent mount of the KV v2 secrets engine at `secret/`.
//!
//! #2021 — the mount used to be enabled only on the fresh-init path
//! (`initialize_vault_local`). Every later boot goes through
//! `recover_existing_vault`, which unsealed and seeded **without** ensuring the
//! mount existed: when the mount was missing (fresh-install ordering, a reset
//! that kept Vault data, or an enable step that failed once) every
//! `secret/gbo/*` read/write returned `no handler for route
//! "secret/data/gbo/..."` and seeding retried on each boot forever. Both paths
//! call this helper before seeding.

use anyhow::Result;
use botlib::security::SafeCommand;
use log::{info, warn};
use std::path::Path;

/// Ensure the KV v2 engine is mounted at `secret/`.
///
/// Tolerates "path is already in use" so it is safe to call on every boot.
/// A failure is logged rather than fatal: the caller still proceeds to the
/// seed step, which reports the real error with the rest of the bootstrap
/// state (and the next boot retries).
pub fn ensure_kv_mount(
    vault_addr: &str,
    token: &str,
    ca_cert: &Path,
    vault_bin: &Path,
) -> Result<()> {
    let vault_bin_str = vault_bin.to_string_lossy().to_string();
    let ca_cert_str = ca_cert.to_string_lossy().to_string();

    info!("Ensuring the KV2 secrets engine is mounted at 'secret/'...");
    let result = SafeCommand::new(&vault_bin_str)
        .and_then(|c| c.env("VAULT_ADDR", vault_addr))
        .and_then(|c| c.env("VAULT_TOKEN", token))
        .and_then(|c| c.env("VAULT_CACERT", &ca_cert_str))
        .and_then(|c| c.args(&["secrets", "enable", "-path=secret", "kv-v2"]))
        .and_then(|c| c.execute());

    match result {
        Ok(output) if output.status.success() => {
            info!("KV2 secrets engine mounted at 'secret/'");
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("path is already in use") {
                info!("KV2 secrets engine already mounted at 'secret/'");
            } else {
                warn!("Failed to mount the KV2 secrets engine: {}", stderr.trim());
            }
        }
        Err(e) => {
            warn!("Failed to execute the KV2 mount command: {}", e);
        }
    }

    Ok(())
}
