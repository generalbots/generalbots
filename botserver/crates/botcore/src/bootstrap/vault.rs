//! Vault-related functions for bootstrap
//!
//! Extracted from mod.rs

use crate::shared::utils::get_stack_path;
use anyhow::Result;
use log::info;
use std::env;
use std::fs;
use std::path::PathBuf;

/// Check if stack has been installed
pub fn has_installed_stack() -> bool {
    let stack_path = get_stack_path();
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
        log::error!(
            "If you need to re-initialize, manually delete botserver-stack directory first"
        );
        return Err(anyhow::anyhow!(
            "Cannot reset Vault - existing installation detected. Manual intervention required."
        ));
    }

    let stack_path = get_stack_path();
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

/// Pure command builder for the Vault password lookup (#2754). The token is
/// deliberately absent: it is inherited from the parent process environment.
fn vault_password_command(vault_addr: &str, vault_cacert: &str, vault_bin: &str) -> String {
    format!(
        "VAULT_ADDR={vault_addr} VAULT_CACERT={vault_cacert} {vault_bin} kv get -field=password secret/gbo/tables 2>/dev/null"
    )
}

#[cfg(test)]
mod vault_argv_tests {
    use super::vault_password_command;

    #[test]
    fn token_is_never_part_of_the_command() {
        let command = vault_password_command("https://vault.example:8200", "/tmp/ca.crt", "/opt/vault");
        assert!(!command.contains("VAULT_TOKEN"));
        assert!(command.contains("kv get -field=password secret/gbo/tables"));
    }
}

/// Get database password from Vault
pub fn get_db_password_from_vault() -> Option<String> {
    use crate::bootstrap::bootstrap_utils::safe_sh_command;

    let vault_addr = env::var("VAULT_ADDR").unwrap_or_default();
    if env::var("VAULT_TOKEN").ok().is_none() || vault_addr.is_empty() {
        return None;
    }
    let vault_cacert = env::var("VAULT_CACERT")
        .unwrap_or_else(|_| format!("{}/conf/system/certificates/ca/ca.crt", get_stack_path()));
    let vault_bin = format!("{}/bin/vault/vault", get_stack_path());

    // #2754 — the token is already in botserver's environment and is inherited
    // by the child. Never place it in `sh -c` text: that becomes a visible argv
    // element for every local user running `ps`.
    let cmd = vault_password_command(&vault_addr, &vault_cacert, &vault_bin);

    let output = safe_sh_command(&cmd);
    if output.is_empty() {
        None
    } else {
        Some(output.trim().to_string())
    }
}
