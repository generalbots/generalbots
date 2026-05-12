use crate::installer::safe_sh_command;
use crate::installer_vault2;
use anyhow::{Context, Result};
use botlib::security::SafeCommand;
use log::{info, trace, warn};
use std::collections::HashMap;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

pub fn fetch_vault_credentials() -> HashMap<String, String> {
    let mut credentials = HashMap::new();

    dotenvy::dotenv().ok();

    let base_path = std::env::var("BOTSERVER_STACK_PATH")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."))
                .join("botserver-stack")
        });

    let vault_addr =
        std::env::var("VAULT_ADDR").unwrap_or_else(|_| "https://localhost:8200".to_string());
    let vault_token = std::env::var("VAULT_TOKEN").unwrap_or_default();

    if vault_token.is_empty() {
        warn!("VAULT_TOKEN not set, cannot fetch credentials from Vault");
        return credentials;
    }

    let client_cert = base_path.join("conf/system/certificates/botserver/client.crt");
    let client_key = base_path.join("conf/system/certificates/botserver/client.key");
    let vault_check = SafeCommand::new("curl")
        .and_then(|c| {
            c.args(&[
                "-sfk",
                "--cert",
                &client_cert.to_string_lossy(),
                "--key",
                &client_key.to_string_lossy(),
                &format!("{}/v1/sys/health", vault_addr),
            ])
        })
        .and_then(|c| c.execute())
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !vault_check {
        trace!(
            "Vault not reachable at {}, skipping credential fetch",
            vault_addr
        );
        return credentials;
    }

    let vault_bin = base_path.join("bin/vault/vault");
    let ca_cert_path = std::env::var("VAULT_CACERT").unwrap_or_else(|_| {
        base_path
            .join("conf/system/certificates/ca/ca.crt")
            .to_string_lossy()
            .to_string()
    });

    let services = [
        ("drive", "secret/gbo/drive"),
        ("cache", "secret/gbo/cache"),
        ("tables", "secret/gbo/tables"),
        ("vectordb", "secret/gbo/vectordb"),
        ("directory", "secret/gbo/directory"),
        ("llm", "secret/gbo/llm"),
        ("meet", "secret/gbo/meet"),
        ("alm", "secret/gbo/alm"),
        ("encryption", "secret/gbo/encryption"),
    ];

    for (service_name, vault_path) in &services {
        let result = SafeCommand::new(vault_bin.to_str().unwrap_or("vault"))
            .and_then(|c| {
                c.env("VAULT_ADDR", &vault_addr)
                    .and_then(|c| c.env("VAULT_TOKEN", &vault_token))
                    .and_then(|c| c.env("VAULT_CACERT", &ca_cert_path))
            })
            .and_then(|c| {
                c.args(&["kv", "get", "-format=json", "-tls-skip-verify", vault_path])
            })
            .and_then(|c| c.execute());

        if let Ok(output) = result {
            if output.status.success() {
                let json_str = String::from_utf8_lossy(&output.stdout);
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&json_str) {
                    if let Some(data) = json.get("data").and_then(|d| d.get("data")) {
                        if let Some(obj) = data.as_object() {
                            let prefix = service_name.to_uppercase();
                            for (key, value) in obj {
                                if let Some(v) = value.as_str() {
                                    let env_key = format!("{}_{}", prefix, key.to_uppercase());
                                    credentials.insert(env_key, v.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    trace!("Fetched {} credentials from Vault", credentials.len());
    credentials
}

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

    let vault_addr =
        std::env::var("VAULT_ADDR").unwrap_or_else(|_| "https://localhost:8200".to_string());
    let ca_cert = base_path.join("conf/system/certificates/ca/ca.crt");

    if vault_data_exists {
        let health_cmd = format!(
            "curl -f -s --connect-timeout 2 -k {}/v1/sys/health",
            vault_addr
        );
        let health_output = safe_sh_command(&health_cmd);

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
            return installer_vault2::recover_existing_vault(base_path);
        }
    }

    let init_cmd = format!(
        "{} operator init -tls-skip-verify -key-shares=5 -key-threshold=3 -format=json -address={}",
        vault_bin.display(),
        vault_addr
    );

    info!("Running vault operator init...");
    let output = safe_sh_command(&init_cmd)
        .ok_or_else(|| anyhow::anyhow!("Failed to execute vault init command"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("already initialized") {
            warn!("Vault already initialized, recovering existing data");
            return installer_vault2::recover_existing_vault(base_path);
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
    info!("Created {}", init_json.display());

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
        std::fs::set_permissions(&unseal_keys_file, std::fs::Permissions::from_mode(0o600))?;
    }
    #[cfg(not(unix))]
    {
        let _ = &unseal_keys_file;
    }
    info!("Created {} (chmod 600)", unseal_keys_file.display());

    installer_vault2::unseal_vault(base_path, &vault_bin, &vault_addr)?;

    info!("Vault initialized and unsealed successfully");
    info!("Created .env with VAULT_ADDR, VAULT_TOKEN");
    info!("Created vault-unseal-keys (chmod 600)");

    info!("Enabling KV2 secrets engine at 'secret/'...");
    let enable_kv2_cmd = format!(
        "VAULT_ADDR={} VAULT_TOKEN={} VAULT_CACERT={} {} secrets enable -path=secret kv-v2",
        vault_addr,
        root_token,
        ca_cert.display(),
        vault_bin.display()
    );
    match safe_sh_command(&enable_kv2_cmd) {
        Some(output) => {
            if output.status.success() {
                info!("KV2 secrets engine enabled at 'secret/'");
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if stderr.contains("path is already in use") {
                    info!("KV2 secrets engine already enabled");
                } else {
                    warn!("Failed to enable KV2 secrets engine: {}", stderr);
                }
            }
        }
        None => {
            warn!("Failed to execute KV2 enable command");
        }
    }

    installer_vault2::seed_vault_defaults(&vault_addr, root_token, &ca_cert, &vault_bin)?;

    Ok(())
}

pub fn vault_seeds_exist(
    vault_addr: &str,
    root_token: &str,
    ca_cert: &std::path::Path,
    vault_bin: &std::path::Path,
) -> Result<bool> {
    let args = vec![
        "kv".to_string(),
        "get".to_string(),
        "-tls-skip-verify".to_string(),
        format!("-address={}", vault_addr),
        "-field=accesskey".to_string(),
        "secret/gbo/drive".to_string(),
    ];

    let result = SafeCommand::new(vault_bin.to_str().unwrap_or("vault"))
        .and_then(|c| {
            let mut cmd = c;
            for arg in &args {
                cmd = cmd.trusted_arg(arg)?;
            }
            Ok(cmd)
        })
        .and_then(|c| {
            c.env("VAULT_ADDR", vault_addr)
                .and_then(|c| c.env("VAULT_TOKEN", root_token))
                .and_then(|c| c.env("VAULT_CACERT", ca_cert.to_str().unwrap_or("")))
        })
        .and_then(|c| c.execute());

    match result {
        Ok(output) => {
            if output.status.success() {
                let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
                Ok(!value.is_empty())
            } else {
                Ok(false)
            }
        }
        Err(_) => Ok(false),
    }
}
