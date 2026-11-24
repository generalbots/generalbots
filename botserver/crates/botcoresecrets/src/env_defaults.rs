use crate::manager::SecretsManager;
use anyhow::Result;
use std::collections::HashMap;

/// True for fields that carry a secret and therefore must never have a
/// non-empty value in the built-in fallback defaults (#2764). Kept free of
/// I/O so the rule can be unit-tested without Vault or an environment.
fn is_secret_field(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    key.contains("password")
        || key.contains("secret")
        || key.contains("token")
        || key.contains("key")
}

/// Fail closed: blank any non-empty built-in secret default. A guessable
/// literal here would be shipped to every installation and could work in
/// production; an empty value forces callers to supply a real Vault/env secret.
fn fail_closed_secret_defaults(secrets: &mut HashMap<String, String>) {
    for (key, value) in secrets.iter_mut() {
        if is_secret_field(key) && !value.is_empty() {
            log::warn!(
                "Refusing non-empty built-in default for secret field '{key}'; using empty value"
            );
            value.clear();
        }
    }
}

pub fn init_secrets_manager() -> Result<SecretsManager> {
    SecretsManager::from_env()
}

pub fn get_from_env(path: &str) -> Result<HashMap<String, String>> {
    let mut secrets = HashMap::new();

    let normalized = if path.starts_with("gbo/system/") {
        path.strip_prefix("gbo/system/").unwrap_or(path)
    } else {
        path
    };

    match normalized {
        "tables" | "gbo/tables" | "system/tables" => {
            secrets.insert("host".into(), "localhost".into());
            secrets.insert("port".into(), "5432".into());
            secrets.insert("database".into(), "botserver".into());
            secrets.insert("username".into(), "gbuser".into());
            // #2764 — never ship a usable fallback password. With Vault
            // unavailable, an empty value makes the DB connection fail closed
            // instead of authenticating with a guessable, shared credential.
            secrets.insert("password".into(), String::new());
        }
        "directory" | "gbo/directory" | "system/directory" => {
            secrets.insert("url".into(), String::new());
            secrets.insert("host".into(), "localhost".into());
            // Zitadel runs on 8300 in the local/desktop stack (see
            // register_directory); keep the Vault-absent fallback in sync so a
            // fresh boot does not probe localhost:9000 and fail the admin
            // bootstrap before the OAuth client exists.
            secrets.insert("port".into(), "8300".into());
            secrets.insert("project_id".into(), String::new());
            secrets.insert("client_id".into(), String::new());
            secrets.insert("client_secret".into(), String::new());
        }
        "drive" | "gbo/drive" | "system/drive" => {
            secrets.insert("host".into(), String::new());
            secrets.insert("port".into(), String::new());
            secrets.insert("accesskey".into(), String::new());
            secrets.insert("secret".into(), String::new());
        }
        "cache" | "gbo/cache" | "system/cache" => {
            secrets.insert("host".into(), "localhost".into());
            secrets.insert("port".into(), "6379".into());
            secrets.insert("password".into(), String::new());
        }
        "email" | "gbo/email" | "system/email" => {
            secrets.insert("smtp_host".into(), String::new());
            secrets.insert("smtp_port".into(), "587".into());
            secrets.insert("smtp_user".into(), String::new());
            secrets.insert("smtp_password".into(), String::new());
            secrets.insert("smtp_from".into(), String::new());
        }
        "llm" | "gbo/llm" | "system/llm" => {
            secrets.insert("url".into(), String::new());
            secrets.insert("model".into(), "gpt-4".into());
            secrets.insert("openai_key".into(), String::new());
            secrets.insert("anthropic_key".into(), String::new());
            secrets.insert("ollama_url".into(), String::new());
        }
        "encryption" | "gbo/encryption" | "system/encryption" => {
            secrets.insert("master_key".into(), String::new());
        }
        "meet" | "gbo/meet" | "system/meet" => {
            secrets.insert("url".into(), String::new());
            secrets.insert("app_id".into(), String::new());
            secrets.insert("app_secret".into(), String::new());
        }
        "vectordb" | "gbo/vectordb" | "system/vectordb" => {
            secrets.insert("url".into(), String::new());
            secrets.insert("host".into(), "localhost".into());
            secrets.insert("port".into(), "6333".into());
            secrets.insert("grpc_port".into(), "6334".into());
            secrets.insert("api_key".into(), String::new());
        }
        "observability" | "gbo/observability" | "system/observability" => {
            secrets.insert("url".into(), String::new());
            secrets.insert("org".into(), "system".into());
            secrets.insert("bucket".into(), "metrics".into());
            secrets.insert("token".into(), String::new());
        }
        "alm" | "gbo/alm" | "system/alm" => {
            secrets.insert("url".into(), String::new());
            secrets.insert("token".into(), String::new());
            secrets.insert("default_org".into(), String::new());
        }
        "security" | "gbo/security" | "system/security" => {
            secrets.insert("require_auth".into(), "true".into());
            secrets.insert("anonymous_paths".into(), String::new());
        }
        "cloud" | "gbo/cloud" | "system/cloud" => {
            secrets.insert("region".into(), "us-east-1".into());
            secrets.insert("access_key".into(), String::new());
            secrets.insert("secret_key".into(), String::new());
        }
        "app" | "gbo/app" | "system/app" => {
            secrets.insert("url".into(), String::new());
            secrets.insert("environment".into(), "development".into());
        }
        "jwt" | "gbo/jwt" | "system/jwt" => {
            secrets.insert("secret".into(), String::new());
        }
        "models" | "gbo/models" | "system/models" => {
            secrets.insert("url".into(), String::new());
        }
        "peer" | "gbo/peer" | "system/peer" => {
            // #1968 — peer-node settings live in Vault `secret/gbo/peer`.
            // Without Vault every field stays empty so the `GB_PEER_*`
            // variables and the built-in defaults apply unchanged.
            for key in [
                "enabled",
                "node-id",
                "node-name",
                "advertise-addr",
                "data-dir",
                "heartbeat-ttl-secs",
                "token",
                "signing-key",
            ] {
                secrets.insert(key.into(), String::new());
            }
        }
        "vdi" | "gbo/vdi" | "system/vdi" => {
            // Default VDI connection (host/port/name) — never hardcoded in the
            // repository; provisioned at deploy time in Vault `secret/gbo/vdi`.
            secrets.insert("default-host".into(), String::new());
            secrets.insert("default-port".into(), String::new());
            secrets.insert("default-name".into(), String::new());
        }
        _ => {
            log::debug!("No default values for secret path: {}", path);
        }
    }

    fail_closed_secret_defaults(&mut secrets);
    Ok(secrets)
}

#[cfg(test)]
mod tests {
    use super::{fail_closed_secret_defaults, get_from_env, is_secret_field};
    use std::collections::HashMap;

    const SECRET_PATHS: &[&str] = &[
        "tables",
        "directory",
        "drive",
        "cache",
        "email",
        "llm",
        "encryption",
        "meet",
        "vectordb",
        "observability",
        "alm",
        "security",
        "cloud",
        "app",
        "jwt",
        "models",
        "peer",
        "vdi",
    ];

    #[test]
    fn missing_vault_tables_secret_returns_empty_password() {
        let secrets = get_from_env("tables").unwrap_or_else(|e| {
            log::warn!("get_from_env(tables) failed ({e}); using empty secret map");
            HashMap::new()
        });
        assert_eq!(secrets.get("password").map(String::as_str), Some(""));
        assert_eq!(secrets.get("username").map(String::as_str), Some("gbuser"));
    }

    #[test]
    fn is_secret_field_matches_only_secret_bearing_keys() {
        for key in [
            "password",
            "password_hash",
            "client_secret",
            "api_key",
            "accesskey",
            "token",
            "signing-key",
            "smtp_password",
            "master_key",
        ] {
            assert!(is_secret_field(key), "expected secret field: {key}");
        }
        for key in [
            "host",
            "port",
            "database",
            "username",
            "model",
            "url",
            "environment",
            "require_auth",
            "project_id",
            "default_org",
        ] {
            assert!(!is_secret_field(key), "expected non-secret field: {key}");
        }
    }

    #[test]
    fn every_builtin_secret_default_is_empty() {
        for path in SECRET_PATHS {
            let secrets = get_from_env(path).unwrap_or_else(|e| {
                log::warn!("get_from_env({path}) failed ({e}); using empty secret map");
                HashMap::new()
            });
            for (key, value) in &secrets {
                assert!(
                    !is_secret_field(key) || value.is_empty(),
                    "{path}: non-empty built-in default for secret field '{key}'"
                );
            }
        }
    }

    #[test]
    fn fail_closed_helper_blanks_injected_secret_defaults() {
        let mut secrets = HashMap::from([
            ("password".to_string(), "hunter2".to_string()),
            ("api_key".to_string(), "leaked".to_string()),
            ("host".to_string(), "localhost".to_string()),
        ]);
        fail_closed_secret_defaults(&mut secrets);
        assert_eq!(secrets.get("password").map(String::as_str), Some(""));
        assert_eq!(secrets.get("api_key").map(String::as_str), Some(""));
        assert_eq!(secrets.get("host").map(String::as_str), Some("localhost"));
    }
}
