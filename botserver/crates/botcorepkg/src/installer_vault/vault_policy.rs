//! Scoped Vault policy and token provisioning (#2498).
//!
//! The root token exists only as a transient environment variable for the two
//! CLI calls below. The token handed to the sidecar can read `secret/gbo/*`
//! and health/mount metadata, but cannot manage policies, tokens, auth or raw
//! storage — those stay root-only and in memory during initialize.

use super::secret_store;
use anyhow::{Context, Result};
use botlib::security::SafeCommand;
use log::warn;
use std::path::Path;

pub const SIDECAR_POLICY: &str = "gb-sidecar";

pub const SIDECAR_POLICY_TEXT: &str = r#"
# #2498 — scoped token for the botserver sidecar. No auth/, sys/raw, token
# creation or policy management: those remain root-only and in memory at init.
path "secret/data/gbo/*" {
  capabilities = ["create", "read", "update", "delete", "list"]
}
path "secret/metadata/gbo/*" {
  capabilities = ["create", "read", "update", "delete", "list"]
}
path "sys/health" {
  capabilities = ["read"]
}
path "sys/mounts" {
  capabilities = ["read"]
}
path "sys/capabilities-self" {
  capabilities = ["read"]
}
"#;

fn vault_command(vault_bin: &Path) -> Result<SafeCommand> {
    let vault_bin_str = vault_bin.to_string_lossy().to_string();
    SafeCommand::new(&vault_bin_str).context("Cannot run vault")
}

fn with_tls(mut command: SafeCommand, ca_cert: &Path) -> Result<SafeCommand> {
    if ca_cert.is_file() {
        command = command.env("VAULT_CACERT", &ca_cert.to_string_lossy())?;
    } else {
        warn!("Vault CA certificate not found; TLS verification skipped");
        command = command.trusted_arg("-tls-skip-verify")?;
    }
    Ok(command)
}

/// Create the scoped policy and token. The root token is passed only as a
/// short-lived environment variable to these two CLI invocations.
pub fn create_scoped_token(
    vault_addr: &str,
    root_token: &str,
    ca_cert: &Path,
    vault_bin: &Path,
) -> Result<String> {
    let policy_path = std::env::temp_dir().join(format!(
        "gb-vault-policy-{}-{SIDECAR_POLICY}.hcl",
        std::process::id()
    ));
    secret_store::write_private(&policy_path, SIDECAR_POLICY_TEXT)?;
    let policy_arg = policy_path.to_string_lossy().to_string();

    let command = with_tls(
        vault_command(vault_bin)?
            .env("VAULT_ADDR", vault_addr)?
            .env("VAULT_TOKEN", root_token)?,
        ca_cert,
    )?;
    let result = command
        .args(&["policy", "write", SIDECAR_POLICY, &policy_arg])?
        .execute();
    let _ = std::fs::remove_file(&policy_path);
    match result {
        Ok(output) if output.status.success() => {}
        Ok(output) => {
            return Err(anyhow::anyhow!(
                "Cannot install the {} Vault policy: {}",
                SIDECAR_POLICY,
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        Err(error) => return Err(error.context("Cannot install Vault policy")),
    }

    let command = with_tls(
        vault_command(vault_bin)?
            .env("VAULT_ADDR", vault_addr)?
            .env("VAULT_TOKEN", root_token)?,
        ca_cert,
    )?;
    let output = command
        .args(&[
            "token",
            "create",
            &format!("-policy={SIDECAR_POLICY}"),
            "-ttl=720h",
            "-renewable=true",
            "-format=json",
        ])?
        .execute()?;
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Cannot create scoped Vault token: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).context("Cannot parse scoped token response")?;
    let token = json["auth"]["client_token"]
        .as_str()
        .context("No client_token in scoped token response")?;
    Ok(token.to_string())
}
