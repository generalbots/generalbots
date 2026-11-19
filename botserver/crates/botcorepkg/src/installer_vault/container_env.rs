//! Vault bootstrap and `.env` generation for a container-hosted Vault.
//!
//! #2498 — the root token and the unseal keys are kept in memory / in the
//! private credential store, never in the user-owned stack. The token persisted
//! for the sidecar is a scoped `gb-sidecar` token, not the root token.

use crate::facade_container::{get_container_ip, safe_incus};
use crate::installer_vault::secret_store as secrets;
use anyhow::{Context, Result};
use botlib::security::SafeCommand;
use log::{info, warn};
use std::path::{Path, PathBuf};

/// Run a shell script inside a container. The script is passed as a single
/// trusted `-c` argument (URLs and Vault flags contain characters rejected by
/// plain argument validation). The script is built only from code-controlled
/// values plus transient secret material that never lands on disk.
fn incus_exec(container_name: &str, script: &str) -> Result<std::process::Output> {
    SafeCommand::new("incus")
        .context("Cannot run incus")?
        .args(&["exec", container_name, "--", "bash", "-c"])?
        .trusted_shell_script_arg(script)?
        .execute()
        .context("Cannot execute command in container")
}

fn host_ca_cert() -> Option<PathBuf> {
    let base = std::env::var("BOTSERVER_STACK_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let candidate = base.join("conf/system/certificates/ca/ca.crt");
    if candidate.is_file() {
        Some(candidate)
    } else {
        None
    }
}

/// Discover whether the container listener uses TLS and build both the address
/// the host uses and the in-container loopback address.
fn vault_listener(container_name: &str, ip: &str) -> (String, String, Option<PathBuf>) {
    let tls = incus_exec(
        container_name,
        "test -f /opt/gbo/conf/system/certificates/vault/server.crt && echo tls || echo plain",
    )
    .map(|output| {
        output.status.success() && String::from_utf8_lossy(&output.stdout).trim() == "tls"
    })
    .unwrap_or(false);

    if tls {
        (
            format!("https://{ip}:8200"),
            "https://127.0.0.1:8200".to_string(),
            host_ca_cert(),
        )
    } else {
        warn!(
            "Vault container {container_name} has no TLS certificate; using loopback-only plain HTTP"
        );
        (
            format!("http://{ip}:8200"),
            "http://127.0.0.1:8200".to_string(),
            None,
        )
    }
}

/// Run a Vault CLI command inside the container. The root token is passed in
/// the transient environment of this one command, never written to a file.
fn vault_in_container(
    container_name: &str,
    local_addr: &str,
    token: Option<&str>,
    args: &[&str],
) -> Result<std::process::Output> {
    let mut script = String::new();
    if let Some(token) = token {
        script.push_str(&format!(
            "VAULT_ADDR={local_addr} VAULT_TOKEN={token} "
        ));
    } else {
        script.push_str(&format!("VAULT_ADDR={local_addr} "));
    }
    script.push_str("/opt/gbo/bin/vault");
    for arg in args.iter().filter(|arg| !arg.is_empty()) {
        script.push(' ');
        script.push_str(arg);
    }
    incus_exec(container_name, &script)
}

fn scoped_token(
    container_name: &str,
    local_addr: &str,
    root_token: &str,
    ca_cert: Option<&Path>,
) -> Result<String> {
    let policy = std::env::temp_dir().join(format!(
        "gb-vault-policy-{}.hcl",
        std::process::id()
    ));
    std::fs::write(&policy, secrets::SIDECAR_POLICY_TEXT)?;
    // `vault policy write` needs a file inside the container; stream it with
    // `incus file push` through the helper below, then remove it immediately.
    push_transient_file(container_name, &policy, "/tmp/gb-sidecar-policy.hcl")?;
    let _ = std::fs::remove_file(&policy);

    let write = vault_in_container(
        container_name,
        local_addr,
        Some(root_token),
        &["policy", "write", secrets::SIDECAR_POLICY, "/tmp/gb-sidecar-policy.hcl"],
    )?;
    delete_in_container(container_name, "/tmp/gb-sidecar-policy.hcl");
    if !write.status.success() {
        return Err(anyhow::anyhow!(
            "Cannot install scoped Vault policy: {}",
            String::from_utf8_lossy(&write.stderr)
        ));
    }

    let tls_flag = if ca_cert.is_some() {
        "-ca-cert=/opt/gbo/conf/system/certificates/ca/ca.crt"
    } else {
        "-tls-skip-verify"
    };
    let output = vault_in_container(
        container_name,
        local_addr,
        Some(root_token),
        &[
            "token",
            "create",
            &format!("-policy={}", secrets::SIDECAR_POLICY),
            "-ttl=720h",
            "-renewable=true",
            "-format=json",
            tls_flag,
        ],
    )?;
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

fn push_transient_file(container_name: &str, source: &Path, destination: &str) -> Result<()> {
    let source_arg = source.to_string_lossy().to_string();
    let target = format!("{container_name}:{destination}");
    let output = safe_incus(&["file", "push", &source_arg, &target])
        .context("Cannot copy transient policy into container")?;
    if !output.status.success() {
        return Err(anyhow::anyhow!("Cannot copy transient policy into container"));
    }
    Ok(())
}

fn delete_in_container(container_name: &str, path: &str) {
    let script = format!("rm -f {path}");
    let _ = incus_exec(container_name, &script);
}

/// Initialize a container-hosted Vault, unseal it, and persist only a scoped
/// token plus the unseal keys in the private credential store.
pub fn initialize_vault(container_name: &str, ip: &str) -> Result<()> {
    info!("Initializing Vault...");
    std::thread::sleep(std::time::Duration::from_secs(5));

    let (vault_addr, local_addr, ca_cert) = vault_listener(container_name, ip);
    let tls_flag = if ca_cert.is_some() {
        "-ca-cert=/opt/gbo/conf/system/certificates/ca/ca.crt"
    } else {
        "-tls-skip-verify"
    };
    let output = vault_in_container(
        container_name,
        &local_addr,
        None,
        &[
            "operator",
            "init",
            "-key-shares=5",
            "-key-threshold=3",
            "-format=json",
            tls_flag,
        ],
    )?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("already initialized") {
            info!("Vault already initialized, skipping secret generation");
            return Ok(());
        }
        return Err(anyhow::anyhow!("Failed to initialize Vault: {stderr}"));
    }

    let init_json: serde_json::Value =
        serde_json::from_slice(&output.stdout).context("Failed to parse Vault init output")?;
    let unseal_keys: Vec<String> = init_json["unseal_keys_b64"]
        .as_array()
        .context("No unseal keys in output")?
        .iter()
        .filter_map(|key| key.as_str().map(String::from))
        .collect();
    let root_token = init_json["root_token"]
        .as_str()
        .context("No root token in output")?;

    let token = scoped_token(container_name, &local_addr, root_token, ca_cert.as_deref())?;
    secrets::write(secrets::VAULT_TOKEN_SECRET, &token)?;
    secrets::write(secrets::UNSEAL_KEYS_SECRET, &unseal_keys.join("\n"))?;

    for (index, key) in unseal_keys.iter().take(3).enumerate() {
        let output = vault_in_container(
            container_name,
            &local_addr,
            None,
            &["operator", "unseal", tls_flag, key],
        )?;
        if !output.status.success() {
            warn!("Unseal step {} may have failed", index + 1);
        }
    }

    // Migration: earlier releases left unseal keys in the user-owned stack.
    let legacy_keys = PathBuf::from("vault-unseal-keys");
    if legacy_keys.exists() {
        let _ = std::fs::remove_file(&legacy_keys);
    }
    secrets::write_env_file(&PathBuf::from(".env"), &vault_addr, ca_cert.as_deref())?;

    info!("Vault initialized; scoped token stored outside the stack");
    Ok(())
}

/// Regenerate `.env` from the private store for an already initialized Vault.
/// The returned text contains no token value.
pub fn generate_env_from_vault(container_name: &str) -> Result<String> {
    let container_ip = get_container_ip(container_name)?;
    let (vault_addr, local_addr, ca_cert) = vault_listener(container_name, &container_ip);
    info!("Generating .env from vault at {vault_addr}");

    let tls_flag = if ca_cert.is_some() {
        "-ca-cert=/opt/gbo/conf/system/certificates/ca/ca.crt"
    } else {
        "-tls-skip-verify"
    };
    let status = vault_in_container(
        container_name,
        &local_addr,
        None,
        &["status", "-format=json", tls_flag],
    )?;
    if !status.status.success() {
        return Err(anyhow::anyhow!(
            "Vault in container {container_name} is not reachable or still sealed"
        ));
    }

    // Never fall back to `/root/.vault-token` (the root token): the sidecar
    // token must have come from first-run provisioning.
    secrets::load_token_into_env()?;
    if secrets::read(secrets::VAULT_TOKEN_SECRET).is_err() {
        return Err(anyhow::anyhow!(
            "No scoped Vault token in the credential store; re-run first-run provisioning"
        ));
    }

    secrets::write_env_file(&PathBuf::from(".env"), &vault_addr, ca_cert.as_deref())?;
    info!("Generated .env with a reference to the scoped Vault token");
    secrets::env_lines(&vault_addr, ca_cert.as_deref())
}
