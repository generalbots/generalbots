//! Vault-related functions for bootstrap
//!
//! Extracted from mod.rs

use anyhow::Result;
use log::info;
use std::env;
use std::fs;
use std::path::PathBuf;

/// Check if stack has been installed
pub fn has_installed_stack() -> bool {
    let stack_path = env::var("BOTSERVER_STACK_PATH")
        .unwrap_or_else(|_| "./botserver-stack".to_string());
    let stack_dir = PathBuf::from(&stack_path);
    if !stack_dir.exists() {
        return false;
    }

    let indicators = [
        stack_dir.join("bin/vault/vault"),
        stack_dir.join("data/vault"),
        stack_dir.join("conf/vault/config.hcl"),
    ];

    indicators.iter().any(|path| path.exists())
}

/// Reset Vault configuration (only if stack is not installed)
pub fn reset_vault_only() -> Result<()> {
    if has_installed_stack() {
        log::error!("REFUSING to reset Vault credentials - botserver-stack is installed!");
        log::error!("If you need to re-initialize, manually delete botserver-stack directory first");
            return Err(anyhow::anyhow!(
                "Cannot reset Vault - existing installation detected. Manual intervention required."
            ));
    }

    let stack_path = env::var("BOTSERVER_STACK_PATH")
        .unwrap_or_else(|_| "./botserver-stack".to_string());
    let vault_init = PathBuf::from(&stack_path).join("conf/vault/init.json");
    let env_file = PathBuf::from("./.env");

    if vault_init.exists() {
        info!("Removing vault init.json for re-initialization...");
        fs::remove_file(&vault_init)?;
    }

    if env_file.exists() {
        info!("Removing .env file for re-initialization...");
        fs::remove_file(&env_file)?;
    }

    Ok(())
}

/// Get database password from Vault
pub fn get_db_password_from_vault() -> Option<String> {
    use crate::core::bootstrap::bootstrap_utils::safe_sh_command;

    let vault_addr = env::var("VAULT_ADDR").unwrap_or_else(|_| "https://localhost:8200".to_string());
    let vault_token = env::var("VAULT_TOKEN").ok()?;
    let vault_cacert = env::var("VAULT_CACERT").unwrap_or_else(|_| "./botserver-stack/conf/system/certificates/ca/ca.crt".to_string());
    let vault_bin = format!("{}/bin/vault/vault", env::var("BOTSERVER_STACK_PATH").unwrap_or_else(|_| "./botserver-stack".to_string()));

    let cmd = format!(
        "VAULT_ADDR={} VAULT_TOKEN={} VAULT_CACERT={} {} kv get -field=password secret/gbo/tables 2>/dev/null",
        vault_addr, vault_token, vault_cacert, vault_bin
    );

    let output = safe_sh_command(&cmd);
    if output.is_empty() {
        None
    } else {
        Some(output.trim().to_string())
    }
}
