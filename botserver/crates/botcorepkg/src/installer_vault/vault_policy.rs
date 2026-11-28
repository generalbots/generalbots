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

/// The root token is discarded after init, so the sidecar token can never be
/// re-issued: it is periodic (no maximum lifetime) and renewed at each start.
/// A plain TTL token would stop working for good at the system maximum TTL.
pub const SIDECAR_TOKEN_PERIOD: &str = "768h";

/// Renew the scoped token with itself (`auth/token/renew-self`, granted by the
/// default policy). A failure is only logged: the token may still be valid.
pub fn renew_scoped_token(vault_addr: &str, token: &str, ca_cert: &Path, vault_bin: &Path) {
    let output = vault_command(vault_bin)
        .and_then(|command| Ok(command.env("VAULT_ADDR", vault_addr)?.env("VAULT_TOKEN", token)?))
        .and_then(|command| with_tls(command, ca_cert))
        .and_then(|command| Ok(command.args(&["token", "renew", "-format=json"])?.execute()?));
    match output {
        Ok(output) if output.status.success() => log::info!("Scoped Vault token renewed"),
        Ok(output) => warn!(
            "Scoped Vault token renewal failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ),
        Err(e) => warn!("Scoped Vault token renewal could not run: {e}"),
    }
}

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
    if let Err(e) = std::fs::remove_file(&policy_path) {
        log::warn!("Failed to remove temporary policy file {}: {e}", policy_path.display());
    }
    match result {
        Ok(output) if output.status.success() => {}
        Ok(output) => {
            return Err(anyhow::anyhow!(
                "Cannot install the {} Vault policy: {}",
                SIDECAR_POLICY,
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        Err(error) => return Err(anyhow::anyhow!("Cannot install Vault policy: {error}")),
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
            "-orphan",
            &format!("-policy={SIDECAR_POLICY}"),
            &format!("-period={SIDECAR_TOKEN_PERIOD}"),
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
