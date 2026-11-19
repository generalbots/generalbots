//! Local (non-LXC) Vault initialization.
//!
//! #2498 — only a scoped token is persisted. The root token and unseal keys
//! live in memory during init and in the 0700 credential store afterwards,
//! never in the user-owned stack.

use anyhow::{Context, Result};
use botlib::security::SafeCommand;
use log::info;

use super::kv_mount::ensure_kv_mount;
use super::secret_store as secrets;
use super::seed_defaults::seed_vault_defaults;
use super::unseal::{recover_existing_vault, unseal_vault};

pub fn initialize_vault_local(base_path: &std::path::Path) -> Result<()> {
    info!("Initializing Vault locally (non-LXC mode)...");

    let vault_bin = base_path.join("bin/vault/vault");
    let vault_data = base_path.join("data/vault");
    let vault_data_exists = vault_data.exists();

    if !vault_data_exists {
        info!("Vault data directory not found, will initialize fresh");
    } else {
        info!("Vault data directory found, checking health...");
    }

    info!("Waiting for Vault to start...");
    std::thread::sleep(std::time::Duration::from_secs(3));

    let vault_addr = std::env::var("VAULT_ADDR")
        .ok()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow::anyhow!("VAULT_ADDR must be set in .env or environment"))?;
    let ca_cert = base_path.join("conf/system/certificates/ca/ca.crt");

    if vault_data_exists {
        let health_output = SafeCommand::new("curl")
            .and_then(|c| c.args(&["-sf", "--connect-timeout", "2", "-k", &format!("{vault_addr}/v1/sys/health")]))
            .ok()
            .and_then(|c| c.execute().ok());

        let already_initialized = if let Some(ref output) = health_output {
            if output.status.success() {
                serde_json::from_slice::<serde_json::Value>(&output.stdout)
                    .ok()
                    .and_then(|json| json.get("initialized").and_then(|value| value.as_bool()))
                    .unwrap_or(false)
            } else {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                stdout.contains("\"initialized\":true") || stderr.contains("\"initialized\":true")
            }
        } else {
            false
        };

        if already_initialized {
            info!("Vault already initialized (detected via health/data), skipping init");
            return recover_existing_vault(base_path);
        }
    }

    info!("Running vault operator init...");
    let vault_bin_str = vault_bin.to_string_lossy().to_string();
    let mut init_command = SafeCommand::new(&vault_bin_str)
        .context("Cannot run the vault binary")?
        .env("VAULT_ADDR", &vault_addr)?
        .args(&[
            "operator",
            "init",
            "-key-shares=5",
            "-key-threshold=3",
            "-format=json",
        ])?;
    if ca_cert.is_file() {
        init_command = init_command.env("VAULT_CACERT", &ca_cert.to_string_lossy())?;
    } else {
        init_command = init_command.arg("-tls-skip-verify")?;
    }
    let output = init_command.execute().context("Cannot run vault init")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("already initialized") {
            info!("Vault already initialized, recovering existing data");
            return recover_existing_vault(base_path);
        }
        return Err(anyhow::anyhow!("Failed to initialize Vault: {stderr}"));
    }

    let init_json_val: serde_json::Value =
        serde_json::from_slice(&output.stdout).context("Failed to parse Vault init output")?;
    let unseal_keys: Vec<String> = init_json_val["unseal_keys_b64"]
        .as_array()
        .context("No unseal keys in output")?
        .iter()
        .filter_map(|key| key.as_str().map(String::from))
        .collect();
    let root_token = init_json_val["root_token"]
        .as_str()
        .context("No root token in output")?
        .to_string();

    // Persist a sanitized copy: it is useful for operators but can never be
    // used to recover the root token or the unseal keys.
    let init_json_path = base_path.join("conf/vault/init.json");
    if let Some(parent) = init_json_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let safe_json = serde_json::to_string_pretty(&secrets::sanitized_init_json(&init_json_val))?;
    std::fs::write(&init_json_path, safe_json)?;
    #[cfg(unix)]
    let _ = botlib::os::fs::get_permissions_manager().set_readonly_owner(&init_json_path);

    let scoped = secrets::create_scoped_token(&vault_addr, &root_token, &ca_cert, &vault_bin)?;
    secrets::write(secrets::VAULT_TOKEN_SECRET, &scoped)?;
    secrets::write(secrets::UNSEAL_KEYS_SECRET, &unseal_keys.join("\n"))?;
    secrets::load_token_into_env()?;

    secrets::write_env_file(&std::path::PathBuf::from(".env"), &vault_addr, Some(&ca_cert))?;

    unseal_vault(base_path, &vault_bin, &vault_addr)?;

    ensure_kv_mount(&vault_addr, &root_token, &ca_cert, &vault_bin)?;
    seed_vault_defaults(&vault_addr, &root_token, &ca_cert, &vault_bin)?;

    // Emit the scoped token for developer tooling; the root token is discarded.
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let token_file = std::env::temp_dir().join("vault-token-gb");
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .mode(0o600)
            .open(&token_file)
        {
            use std::io::Write;
            let _ = file.write_all(scoped.as_bytes());
        }
        info!("Developer token file {} contains the scoped sidecar token", token_file.display());
    }

    info!("Vault initialized and unsealed; scoped token stored outside the stack");
    Ok(())
}
