//! Vault unseal and recovery of an existing installation.

use crate::installer::safe_sh_command;
use anyhow::{Context, Result};
use log::{info, warn};

use super::seed_defaults::seed_vault_defaults;
use super::seeds_exist::vault_seeds_exist;

pub fn recover_existing_vault(base_path: &std::path::Path) -> Result<()> {
    use std::io::Write;

    info!("Recovering existing Vault installation...");

    let vault_addr = std::env::var("VAULT_ADDR")
        .ok()
        .filter(|a| !a.is_empty())
        .ok_or_else(|| anyhow::anyhow!("VAULT_ADDR must be set in .env or environment"))?;
    let ca_cert = base_path.join("conf/system/certificates/ca/ca.crt");
    let vault_bin = base_path.join("bin/vault/vault");

    let unseal_keys_file = base_path.join("vault-unseal-keys");
    let unseal_keys = if unseal_keys_file.exists() {
        info!("Found existing vault-unseal-keys file");
        let content = std::fs::read_to_string(&unseal_keys_file)?;
        content
            .lines()
            .filter_map(|line| {
                line.strip_prefix("VAULT_UNSEAL_KEY_")
                    .and_then(|rest| rest.split_once('='))
                    .map(|(_, key)| key.to_string())
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let init_json = base_path.join("conf/vault/init.json");
    let root_token = if init_json.exists() {
        let content = std::fs::read_to_string(&init_json)?;
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            json.get("root_token")
                .and_then(|v| v.as_str())
                .map(String::from)
        } else {
            None
        }
    } else {
        None
    };

    if !unseal_keys.is_empty() {
        info!("Unsealing Vault with existing keys...");
        for (i, key) in unseal_keys.iter().take(3).enumerate() {
            let unseal_cmd = format!(
                "{} operator unseal -tls-skip-verify -address={} {}",
                vault_bin.display(),
                vault_addr,
                key
            );
            let unseal_output = safe_sh_command(&unseal_cmd);
            if let Some(ref output) = unseal_output {
                if !output.status.success() {
                    warn!("Unseal step {} may have failed", i + 1);
                }
            }
        }
    }

    if let Some(ref token) = root_token {
        let env_file = std::path::PathBuf::from(".env");
        let env_content = format!(
            r#"
# Vault Configuration (auto-generated)
VAULT_ADDR={}
VAULT_TOKEN={}
VAULT_CACERT={}
"#,
            vault_addr,
            token,
            ca_cert.display()
        );

        if env_file.exists() {
            let existing = std::fs::read_to_string(&env_file)?;
            if !existing.contains("VAULT_ADDR=") {
                let mut file = std::fs::OpenOptions::new().append(true).open(&env_file)?;
                file.write_all(env_content.as_bytes())?;
                info!("Appended Vault config to .env");
            }
        } else {
            std::fs::write(&env_file, env_content.trim_start())?;
            info!("Created .env with Vault config");
        }
    } else {
        warn!("No root token found - Vault may need manual recovery");
    }

    if let Some(ref token) = root_token {
            if vault_seeds_exist(&vault_addr, token, &ca_cert, &vault_bin)? {
            info!("Vault credentials already exist, skipping seed on recovery");
        } else {
            let _ = seed_vault_defaults(&vault_addr, token, &ca_cert, &vault_bin);
        }
    }

    info!("Vault recovery complete");
    Ok(())
}

pub fn unseal_vault(
    base_path: &std::path::Path,
    vault_bin: &std::path::Path,
    vault_addr: &str,
) -> Result<()> {
    info!("Unsealing Vault...");
    let unseal_keys_file = base_path.join("vault-unseal-keys");
    if unseal_keys_file.exists() {
        let content = std::fs::read_to_string(&unseal_keys_file)?;
        let keys: Vec<String> = content
            .lines()
            .filter_map(|line| {
                line.strip_prefix("VAULT_UNSEAL_KEY_")
                    .and_then(|rest| rest.split_once('='))
                    .map(|(_, key)| key.to_string())
            })
            .collect();

        for (i, key) in keys.iter().take(3).enumerate() {
            let unseal_cmd = format!(
                "{} operator unseal -tls-skip-verify -address={} {}",
                vault_bin.display(),
                vault_addr,
                key
            );
            let unseal_output = safe_sh_command(&unseal_cmd);
            if let Some(ref output) = unseal_output {
                if !output.status.success() {
                    warn!("Unseal step {} may have failed", i + 1);
                }
            }
        }
    }
    Ok(())
}

pub fn ensure_env_file_exists(base_path: &std::path::Path) -> Result<()> {
    let init_json = base_path.join("conf/vault/init.json");
    let env_file = std::path::PathBuf::from(".env");

    if !init_json.exists() {
        return Ok(());
    }

    let init_content = std::fs::read_to_string(&init_json)?;
    let init_json_val: serde_json::Value = serde_json::from_str(&init_content)?;

    let root_token = init_json_val["root_token"]
        .as_str()
        .context("No root_token in init.json")?;

    let ca_cert = base_path.join("conf/system/certificates/ca/ca.crt");
    let vault_addr = std::env::var("VAULT_ADDR").unwrap_or_default();

    let env_content = format!(
        r#"
# Vault Configuration (auto-generated)
VAULT_ADDR={}
VAULT_TOKEN={}
VAULT_CACERT={}
"#,
        vault_addr,
        root_token,
        ca_cert.display()
    );

    if env_file.exists() {
        let existing = std::fs::read_to_string(&env_file)?;
        if existing.contains("VAULT_ADDR=") {
            return Ok(());
        }
        let mut file = std::fs::OpenOptions::new().append(true).open(&env_file)?;
        use std::io::Write;
        file.write_all(env_content.as_bytes())?;
    } else {
        std::fs::write(&env_file, env_content.trim_start())?;
    }

    info!("Created .env with Vault credentials");
    Ok(())
}
