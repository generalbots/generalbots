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

/// A Vault CLI invocation whose token travels in the child environment, never
/// in the `sh -c` command string that is visible in `ps` (#2754).
struct VaultKvInvocation {
    command: String,
    env: Vec<(String, String)>,
}

/// Build the token-free shell command and the secret child environment.
///
/// Kept separate from the process call so a test can assert the invariant that
/// no credential literal ever appears in `command`.
fn build_vault_kv_invocation(
    vault_bin: &str,
    vault_addr: &str,
    vault_cacert: &str,
    vault_token: &str,
) -> VaultKvInvocation {
    VaultKvInvocation {
        command: format!("{vault_bin} kv get -field=password secret/gbo/tables 2>/dev/null"),
        env: vec![
            ("VAULT_ADDR".to_string(), vault_addr.to_string()),
            ("VAULT_CACERT".to_string(), vault_cacert.to_string()),
            ("VAULT_TOKEN".to_string(), vault_token.to_string()),
        ],
    }
}

/// Get database password from Vault
pub fn get_db_password_from_vault() -> Option<String> {
    use crate::bootstrap::bootstrap_utils::safe_sh_command_with_env;

    let vault_addr = match env::var("VAULT_ADDR") {
        Ok(addr) => addr,
        Err(_) => String::new(),
    };
    // #2754 — keep the root token out of argv: bind it from the environment
    // once here and pass it only through the child's environment.
    let vault_token = match env::var("VAULT_TOKEN") {
        Ok(token) => token,
        Err(_) => return None,
    };
    if vault_addr.is_empty() {
        return None;
    }
    let vault_cacert = env::var("VAULT_CACERT")
        .unwrap_or_else(|_| format!("{}/conf/system/certificates/ca/ca.crt", get_stack_path()));
    let vault_bin = format!("{}/bin/vault/vault", get_stack_path());

    let invocation =
        build_vault_kv_invocation(&vault_bin, &vault_addr, &vault_cacert, &vault_token);
    let child_env: Vec<(&str, &str)> = invocation
        .env
        .iter()
        .map(|(key, value)| (key.as_str(), value.as_str()))
        .collect();
    let output = safe_sh_command_with_env(&invocation.command, &child_env);
    if output.is_empty() {
        None
    } else {
        Some(output.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::build_vault_kv_invocation;

    #[test]
    fn vault_token_is_never_in_the_shell_command() {
        const SECRET: &str = "hvs.SUPER_SECRET_TEST_TOKEN_9f3c1b";
        let invocation = build_vault_kv_invocation(
            "/opt/gbo/bin/vault/vault",
            "https://vault.internal:8200",
            "/opt/gbo/conf/system/certificates/ca/ca.crt",
            SECRET,
        );

        assert!(
            !invocation.command.contains(SECRET),
            "vault token leaked into the sh -c command string"
        );
        assert!(
            !invocation.command.contains("VAULT_TOKEN"),
            "no token assignment may appear in the sh -c command string"
        );
        assert!(invocation
            .command
            .contains("kv get -field=password secret/gbo/tables"));
        assert!(
            invocation
                .env
                .iter()
                .any(|(key, value)| key == "VAULT_TOKEN" && value == SECRET),
            "the token must be passed through the child environment"
        );
        assert!(invocation.env.iter().any(|(key, _)| key == "VAULT_ADDR"));
        assert!(invocation.env.iter().any(|(key, _)| key == "VAULT_CACERT"));
    }

    #[test]
    fn command_argv_is_independent_of_the_token_value() {
        let first = build_vault_kv_invocation("vault", "addr", "cacert", "token-A");
        let second = build_vault_kv_invocation("vault", "addr", "cacert", "token-B");

        assert_eq!(
            first.command, second.command,
            "changing the token must never change the sh -c argv"
        );
        assert_ne!(first.env, second.env, "the token must reach the child env");
    }
}
