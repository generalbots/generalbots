//! Vault credential lookup from the `secret/gbo/*` KV paths.

use botlib::security::SafeCommand;
use log::{info, trace};
use std::collections::HashMap;

#[cfg(unix)]
pub fn fetch_vault_credentials() -> HashMap<String, String> {
    fetch_vault_credentials_impl()
}

#[cfg(not(unix))]
pub fn fetch_vault_credentials() -> HashMap<String, String> {
    HashMap::new()
}

pub fn fetch_vault_credentials_impl() -> HashMap<String, String> {
    let mut credentials = HashMap::new();

    if let Ok(stack) = std::env::var("BOTSERVER_STACK_PATH") {
        if let Err(e) = dotenvy::from_path(std::path::PathBuf::from(stack).join(".env")) {
            log::warn!("Failed to load stack .env file: {e}");
        }
    } else {
        if let Err(e) = dotenvy::dotenv() {
            log::warn!("Failed to load .env file: {e}");
        }
    }

    let base_path = std::env::var("BOTSERVER_STACK_PATH")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."))
                .join("botserver-stack")
        });

    // #2498 — the token is referenced through VAULT_TOKEN_FILE, never stored
    // in the user-owned `.env`. Resolve it from the private credential store.
    if let Err(e) = super::secret_store::load_token_into_env() {
        log::warn!("Failed to load Vault token into environment: {e}");
    }
    let vault_addr = std::env::var("VAULT_ADDR").unwrap_or_else(|e| {
        log::warn!("VAULT_ADDR unavailable: {e}");
        String::new()
    });
    let vault_token = std::env::var("VAULT_TOKEN").unwrap_or_else(|e| {
        log::warn!("VAULT_TOKEN unavailable: {e}");
        String::new()
    });

    if vault_token.is_empty() || vault_addr.is_empty() {
        info!("VAULT_ADDR or VAULT_TOKEN not set yet, using bootstrap defaults");
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

    #[cfg(target_os = "windows")]
    let vault_bin = base_path.join("bin/vault/vault.exe");
    #[cfg(not(target_os = "windows"))]
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
