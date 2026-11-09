//! Local (non-LXC) Vault initialization.

use anyhow::{Context, Result};
use botlib::security::SafeCommand;
use log::{info, warn};

use super::kv_mount::ensure_kv_mount;
use super::seed_defaults::seed_vault_defaults;
use super::unseal::{recover_existing_vault, unseal_vault};

pub fn initialize_vault_local(base_path: &std::path::Path) -> Result<()> {
    use std::io::Write;

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
        .filter(|a| !a.is_empty())
        .ok_or_else(|| anyhow::anyhow!("VAULT_ADDR must be set in .env or environment"))?;
    let ca_cert = base_path.join("conf/system/certificates/ca/ca.crt");

    if vault_data_exists {
        let health_output = SafeCommand::new("curl")
            .and_then(|c| {
                c.args(&[
                    "-sf",
                    "--connect-timeout",
                    "2",
                    "-k",
                    &format!("{}/v1/sys/health", vault_addr),
                ])
            })
            .ok()
            .and_then(|c| c.execute().ok());

        let already_initialized = if let Some(ref output) = health_output {
            if output.status.success() {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(
                    &String::from_utf8_lossy(&output.stdout),
                ) {
                    json.get("initialized")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                } else {
                    false
                }
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.contains("\"initialized\":true")
                    || stderr.contains("\"initialized\":true")
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
    let output = SafeCommand::new(&vault_bin_str)
        .and_then(|c| {
            c.args(&[
                "operator",
                "init",
                "-tls-skip-verify",
                "-key-shares=5",
                "-key-threshold=3",
                "-format=json",
                &format!("-address={}", vault_addr),
            ])
        })
        .and_then(|c| c.execute())
        .ok()
        .ok_or_else(|| anyhow::anyhow!("Failed to execute vault init command"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("already initialized") {
            info!("Vault already initialized, recovering existing data");
            return recover_existing_vault(base_path);
        }
        return Err(anyhow::anyhow!("Failed to initialize Vault: {}", stderr));
    }

    let init_output = String::from_utf8_lossy(&output.stdout);
    let init_json_val: serde_json::Value =
        serde_json::from_str(&init_output).context("Failed to parse Vault init output")?;

    let unseal_keys = init_json_val["unseal_keys_b64"]
        .as_array()
        .context("No unseal keys in output")?;
    let root_token = init_json_val["root_token"]
        .as_str()
        .context("No root token in output")?;

    let init_json = base_path.join("conf/vault/init.json");
    std::fs::create_dir_all(init_json.parent().unwrap_or(std::path::Path::new("")))?;
    std::fs::write(&init_json, serde_json::to_string_pretty(&init_json_val)?)?;
    #[cfg(unix)]
    let _ = botlib::os::fs::get_permissions_manager().set_readonly_owner(&init_json);
    info!("Created {} (chmod 600)", init_json.display());

    let env_file = std::path::PathBuf::from(".env");
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
            warn!(".env already contains VAULT_ADDR, not overwriting");
        } else {
            let mut file = std::fs::OpenOptions::new().append(true).open(&env_file)?;
            file.write_all(env_content.as_bytes())?;
            info!("Appended Vault config to .env");
        }
    } else {
        std::fs::write(&env_file, env_content.trim_start())?;
        info!("Created .env with Vault config");
    }
    #[cfg(unix)]
    let _ = botlib::os::fs::get_permissions_manager().set_readonly_owner(&env_file);

    let unseal_keys_file = base_path.join("vault-unseal-keys");
    let keys_content: String = unseal_keys
        .iter()
        .enumerate()
        .map(|(i, key): (usize, &serde_json::Value)| {
            format!(
                "VAULT_UNSEAL_KEY_{}={}\n",
                i + 1,
                key.as_str().unwrap_or("")
            )
        })
        .collect();

    std::fs::write(&unseal_keys_file, keys_content)?;

    #[cfg(unix)]
    {
        let _ = botlib::os::fs::get_permissions_manager().set_readonly_owner(&unseal_keys_file);
    }

    info!("Created {} (chmod 600)", unseal_keys_file.display());

    unseal_vault(base_path, &vault_bin, &vault_addr)?;

    info!("Vault initialized and unsealed successfully");
    info!("Created .env with VAULT_ADDR, VAULT_TOKEN");
    info!("Created vault-unseal-keys (chmod 600)");

    // #2021 — one idempotent mount helper shared with the recovery path.
    ensure_kv_mount(&vault_addr, root_token, &ca_cert, &vault_bin)?;

    seed_vault_defaults(&vault_addr, root_token, &ca_cert, &vault_bin)?;

    // Write token to /tmp for developer tools
    if let Err(e) = std::fs::write("/tmp/vault-token-gb", root_token) {
        log::warn!("Failed to write vault token to /tmp: {e}");
    }

    Ok(())
}
