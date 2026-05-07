use crate::generate_random_string;
use crate::installer::safe_sh_command;
use crate::installer_vault::vault_seeds_exist;
use anyhow::{Context, Result};
use botlib::security::SafeCommand;
use log::{info, warn};
use std::path::Path;

pub fn seed_vault_defaults(
    base_path: &std::path::Path,
    vault_addr: &str,
    root_token: &str,
    ca_cert: &std::path::Path,
    vault_bin: &std::path::Path,
) -> Result<()> {
    info!("Seeding default credentials into Vault...");

    let drive_user = generate_random_string(16);
    let drive_pass = generate_random_string(32);
    let cache_pass = generate_random_string(32);
    let db_pass = generate_random_string(32);
    let master_key = generate_random_string(64);
    let meet_app_id = generate_random_string(24);
    let meet_app_secret = generate_random_string(48);
    let alm_token = generate_random_string(40);

    info!(
        "Generated strong random credentials for: drive, cache, tables, encryption, meet, alm"
    );

    let defaults: Vec<(&str, Vec<(String, String)>)> = vec![
        (
            "secret/gbo/drive",
            vec![
                ("accesskey".to_string(), drive_user),
                ("secret".to_string(), drive_pass),
                ("host".to_string(), "localhost".to_string()),
                ("port".to_string(), "9000".to_string()),
                ("url".to_string(), "".to_string()),
            ],
        ),
        (
            "secret/gbo/cache",
            vec![
                ("password".to_string(), cache_pass),
                ("host".to_string(), "localhost".to_string()),
                ("port".to_string(), "6379".to_string()),
                ("url".to_string(), "redis://localhost:6379".to_string()),
            ],
        ),
        (
            "secret/gbo/tables",
            vec![
                ("password".to_string(), db_pass),
                ("host".to_string(), "localhost".to_string()),
                ("port".to_string(), "5432".to_string()),
                ("database".to_string(), "botserver".to_string()),
                ("username".to_string(), "gbuser".to_string()),
                ("url".to_string(), "postgres://localhost:5432".to_string()),
            ],
        ),
        (
            "secret/gbo/directory",
            vec![
                ("url".to_string(), "".to_string()),
                ("host".to_string(), "localhost".to_string()),
                ("port".to_string(), "9000".to_string()),
                ("project_id".to_string(), "none".to_string()),
                ("client_id".to_string(), "none".to_string()),
                ("client_secret".to_string(), "none".to_string()),
            ],
        ),
        (
            "secret/gbo/email",
            vec![
                ("smtp_host".to_string(), "none".to_string()),
                ("smtp_port".to_string(), "587".to_string()),
                ("smtp_user".to_string(), "none".to_string()),
                ("smtp_password".to_string(), "none".to_string()),
                ("smtp_from".to_string(), "none".to_string()),
            ],
        ),
        (
            "secret/gbo/llm",
            vec![
                ("url".to_string(), "".to_string()),
                ("host".to_string(), "localhost".to_string()),
                ("port".to_string(), "8081".to_string()),
                ("model".to_string(), "gpt-4".to_string()),
                ("openai_key".to_string(), "none".to_string()),
                ("anthropic_key".to_string(), "none".to_string()),
                ("ollama_url".to_string(), "".to_string()),
                ("embedding_url".to_string(), "http://localhost:8082/v1/embeddings".to_string()),
                ("embedding_model".to_string(), "bge-small-en-v1.5-f32.gguf".to_string()),
                ("embedding_port".to_string(), "8082".to_string()),
                ("embedding_dimensions".to_string(), "384".to_string()),
            ],
        ),
        (
            "secret/gbo/encryption",
            vec![("master_key".to_string(), master_key)],
        ),
        (
            "secret/gbo/meet",
            vec![
                ("url".to_string(), "".to_string()),
                ("host".to_string(), "localhost".to_string()),
                ("port".to_string(), "7880".to_string()),
                ("app_id".to_string(), meet_app_id),
                ("app_secret".to_string(), meet_app_secret),
            ],
        ),
        (
            "secret/gbo/vectordb",
            vec![
                ("url".to_string(), "http://localhost:6333".to_string()),
                ("host".to_string(), "localhost".to_string()),
                ("port".to_string(), "6333".to_string()),
                ("grpc_port".to_string(), "6334".to_string()),
                ("api_key".to_string(), "none".to_string()),
            ],
        ),
        (
            "secret/gbo/alm",
            vec![
                ("url".to_string(), "".to_string()),
                ("host".to_string(), "localhost".to_string()),
                ("port".to_string(), "9000".to_string()),
                ("token".to_string(), alm_token),
                ("default_org".to_string(), "none".to_string()),
            ],
        ),
    ];

    for (path, kv_pairs) in &defaults {
        let mut args = vec![
            "kv".to_string(),
            "put".to_string(),
            "-tls-skip-verify".to_string(),
            format!("-address={}", vault_addr),
            path.to_string(),
        ];
        for (k, v) in kv_pairs.iter() {
            args.push(format!("{}={}", k, v));
        }

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
                    info!("Seeded Vault defaults at {}", path);
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    warn!("Failed to seed {} in Vault: {}", path, stderr);
                }
            }
            Err(e) => {
                warn!("Failed to execute vault put for {}: {}", path, e);
            }
        }
    }

    info!("Vault defaults seeded successfully");
    Ok(())
}

pub fn recover_existing_vault(base_path: &std::path::Path) -> Result<()> {
    use std::io::Write;

    info!("Recovering existing Vault installation...");

    let vault_addr =
        std::env::var("VAULT_ADDR").unwrap_or_else(|_| "https://localhost:8200".to_string());
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
        if vault_seeds_exist(base_path, &vault_addr, token, &ca_cert, &vault_bin)? {
            info!("Vault credentials already exist, skipping seed on recovery");
        } else {
            let _ = seed_vault_defaults(base_path, &vault_addr, token, &ca_cert, &vault_bin);
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
    let vault_addr =
        std::env::var("VAULT_ADDR").unwrap_or_else(|_| "https://localhost:8200".to_string());

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
