//! Vault unseal and recovery of an existing installation.
//!
//! #2498 — unseal keys and the scoped sidecar token are read from the private
//! credential store (or migrated from a legacy stack file once). The persisted
//! `init.json` no longer contains the root token.

use anyhow::Result;
use botlib::security::SafeCommand;
use log::{info, warn};
use std::path::{Path, PathBuf};

use super::kv_mount::ensure_kv_mount;
use super::secret_store as secrets;
use super::seed_defaults::seed_vault_defaults;
use super::seeds_exist::vault_seeds_exist;

/// Read unseal keys, migrating a legacy plaintext file into the private store
/// and deleting the stack copy. The legacy file is accepted once so existing
/// installs do not lose their Vault on upgrade.
fn unseal_keys(base_path: &Path) -> Result<Vec<String>> {
    if let Ok(value) = secrets::read(secrets::UNSEAL_KEYS_SECRET) {
        let keys: Vec<String> = value.lines().filter(|line| !line.trim().is_empty()).map(String::from).collect();
        if !keys.is_empty() {
            return Ok(keys);
        }
    }
    let legacy = base_path.join("vault-unseal-keys");
    if !legacy.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(&legacy)?;
    let keys: Vec<String> = content
        .lines()
        .filter_map(|line| {
            line.strip_prefix("VAULT_UNSEAL_KEY_")
                .and_then(|rest| rest.split_once('='))
                .map(|(_, key)| key.trim().to_string())
        })
        .filter(|key| !key.is_empty())
        .collect();
    if !keys.is_empty() {
        secrets::write(secrets::UNSEAL_KEYS_SECRET, &keys.join("\n"))?;
    }
    let _ = std::fs::remove_file(&legacy);
    info!("Migrated unseal keys out of the user-owned stack");
    Ok(keys)
}

fn ca_cert(base_path: &Path) -> PathBuf {
    base_path.join("conf/system/certificates/ca/ca.crt")
}

fn unseal_command(vault_bin: &Path, vault_addr: &str, ca_cert: &Path, key: &str) -> Result<()> {
    let vault_bin_str = vault_bin.to_string_lossy().to_string();
    let mut command = SafeCommand::new(&vault_bin_str)
        .context("Cannot run the vault binary")?
        .env("VAULT_ADDR", vault_addr)?
        .args(&["operator", "unseal"])?;
    if ca_cert.is_file() {
        command = command.env("VAULT_CACERT", &ca_cert.to_string_lossy())?;
    } else {
        command = command.arg("-tls-skip-verify")?;
    }
    command = command.trusted_arg(key)?;
    match command.execute() {
        Ok(output) if output.status.success() => Ok(()),
        Ok(output) => {
            warn!("Unseal step failed: {}", String::from_utf8_lossy(&output.stderr).trim());
            Ok(())
        }
        Err(error) => {
            warn!("Unseal command could not run: {error}");
            Ok(())
        }
    }
}

pub fn recover_existing_vault(base_path: &std::path::Path) -> Result<()> {
    info!("Recovering existing Vault installation...");

    let vault_addr = std::env::var("VAULT_ADDR")
        .ok()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow::anyhow!("VAULT_ADDR must be set in .env or environment"))?;
    let ca_cert = ca_cert(base_path);
    let vault_bin = base_path.join("bin/vault/vault");

    let keys = unseal_keys(base_path)?;
    if !keys.is_empty() {
        info!("Unsealing Vault with keys from the private credential store...");
        for key in keys.iter().take(3) {
            unseal_command(&vault_bin, &vault_addr, &ca_cert, key)?;
        }
    }

    // Only a scoped token is persisted. Never read `root_token` from init.json.
    secrets::load_token_into_env()?;
    match secrets::read(secrets::VAULT_TOKEN_SECRET) {
        Ok(token) => {
            ensure_kv_mount(&vault_addr, &token, &ca_cert, &vault_bin)?;
            if vault_seeds_exist(&vault_addr, &token, &ca_cert, &vault_bin)? {
                info!("Vault credentials already exist, skipping seed on recovery");
            } else if let Err(error) = seed_vault_defaults(&vault_addr, &token, &ca_cert, &vault_bin)
            {
                log::error!("Vault seeding during recovery failed: {error}");
            }
        }
        Err(_) => warn!("No scoped Vault token found — re-run first-run provisioning"),
    }

    secrets::write_env_file(&PathBuf::from(".env"), &vault_addr, Some(&ca_cert))?;
    info!("Vault recovery complete");
    Ok(())
}

pub fn unseal_vault(
    base_path: &std::path::Path,
    vault_bin: &std::path::Path,
    vault_addr: &str,
) -> Result<()> {
    info!("Unsealing Vault...");
    let keys = unseal_keys(base_path)?;
    for key in keys.iter().take(3) {
        unseal_command(vault_bin, vault_addr, &ca_cert(base_path), key)?;
    }
    Ok(())
}

pub fn ensure_env_file_exists(base_path: &std::path::Path) -> Result<()> {
    let vault_addr = std::env::var("VAULT_ADDR").unwrap_or_default();
    if vault_addr.is_empty() {
        return Ok(());
    }
    secrets::write_env_file(&PathBuf::from(".env"), &vault_addr, Some(&ca_cert(base_path)))
}
