use anyhow::Result;
use botcoresecrets::SecretsManager;
use std::collections::HashMap;
use std::time::Duration;

pub fn get_database_url_sync() -> Result<String> {
    let manager = SecretsManager::get_clone()?;
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build();
        let result = match rt {
            Ok(rt) => rt.block_on(manager.get_database_url()),
            Err(e) => Err(anyhow::anyhow!("Failed to create runtime: {}", e)),
        };
        if let Err(error) = tx.send(result) {
            log::warn!("Vault secret lookup result dropped: {error}");
        }
    });
    rx.recv_timeout(std::time::Duration::from_secs(10))
        .map_err(|e| anyhow::anyhow!("Database credential lookup did not finish: {e}"))?
}

/// Read one field of a Vault secret synchronously (used by the installers,
/// which are sync command pipelines). Empty/missing values are returned as
/// `Err` so callers can fall back to an explicitly generated value.
pub fn get_secret_value_sync(path: &str, key: &str) -> Result<String> {
    let manager = SecretsManager::get_clone()?;
    let path = path.to_string();
    let key = key.to_string();
    let err_path = path.clone();
    let err_key = key.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let result = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
            Ok(rt) => rt.block_on(manager.get_value(&path, &key)),
            Err(error) => Err(anyhow::anyhow!("Failed to create runtime: {error}")),
        };
        if let Err(error) = tx.send(result) {
            log::warn!("Vault secret lookup result dropped: {error}");
        }
    });
    let value = rx
        .recv_timeout(std::time::Duration::from_secs(10))
        .map_err(|error| anyhow::anyhow!("Vault secret lookup did not finish: {error}"))??;
    if value.is_empty() {
        Err(anyhow::anyhow!("Vault field {err_path}::{err_key} is empty"))
    } else {
        Ok(value)
    }
}

pub fn parse_database_url(url: &str) -> (String, String, String, u32, String) {
    if let Some(stripped) = url.strip_prefix("postgres://") {
        let parts: Vec<&str> = stripped.split('@').collect();
        if parts.len() == 2 {
            let user_pass: Vec<&str> = parts[0].split(':').collect();
            let host_db: Vec<&str> = parts[1].split('/').collect();
            if user_pass.len() >= 2 && host_db.len() >= 2 {
                let username = user_pass[0].to_string();
                let password = user_pass[1].to_string();
                let host_port: Vec<&str> = host_db[0].split(':').collect();
                let server = host_port[0].to_string();
                let port = host_port
                    .get(1)
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(5432);
                let database = host_db[1].to_string();
                return (username, password, server, port, database);
            }
        }
    }
    (
        "".to_string(),
        "".to_string(),
        "".to_string(),
        5432,
        "".to_string(),
    )
}

/// Placeholder used by the directory installer for the `zitadel` PostgreSQL
/// role password. Every rendering path must substitute it before a shell or
/// SQL command is run; no literal password is allowed in the registry.
pub const ZITADEL_PASSWORD_PLACEHOLDER: &str = "{{ZITADEL_PASSWORD}}";

const DIRECTORY_SECRET_PATH: &str = "secret/gbo/directory";
const ZITADEL_PASSWORD_KEY: &str = "password";
const ZITADEL_PASSWORD_ENV: &str = "ZITADEL_DB_PASSWORD";

/// Pure strength gate for the Zitadel DB role password (#2763).
///
/// `generate_random_string(32)` yields 32 alphanumeric characters (~190 bits
/// of entropy). Restricting the alphabet to ASCII alphanumerics also keeps the
/// value safe to interpolate into both shell commands and SQL string literals
/// (no quotes, backslashes or `$`). The deny-list records that the legacy
/// fixed value is never acceptable, including for operator overrides.
pub fn is_strong_zitadel_password(value: &str) -> bool {
    let bytes = value.as_bytes();
    value.len() >= 16
        && bytes.iter().all(|byte| byte.is_ascii_alphanumeric())
        && !bytes.iter().all(|byte| *byte == bytes[0])
        && !matches!(
            value.to_ascii_lowercase().as_str(),
            "zitadel" | "changeme" | "password" | "admin" | "secret"
        )
}

/// Strict Vault read, for callers that must fail closed (#2763).
fn get_secret_strict_sync(path: &str) -> Result<HashMap<String, String>> {
    let manager = SecretsManager::get_clone()?;
    let secret_path = path.to_string();
    let worker_path = secret_path.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build();
        let result = match runtime {
            Ok(runtime) => runtime.block_on(manager.get_secret_strict(&worker_path)),
            Err(error) => Err(anyhow::anyhow!("Failed to create runtime: {error}")),
        };
        if tx.send(result).is_err() {
            log::warn!("Vault read worker for '{worker_path}' outlived its caller");
        }
    });
    rx.recv_timeout(Duration::from_secs(10))
        .map_err(|error| anyhow::anyhow!("Vault read for '{secret_path}' did not finish: {error}"))?
}

/// Strict Vault write, for callers that must fail closed (#2763).
fn put_secret_sync(path: &str, data: HashMap<String, String>) -> Result<()> {
    let manager = SecretsManager::get_clone()?;
    let secret_path = path.to_string();
    let worker_path = secret_path.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build();
        let result = match runtime {
            Ok(runtime) => runtime.block_on(manager.put_secret(&worker_path, data)),
            Err(error) => Err(anyhow::anyhow!("Failed to create runtime: {error}")),
        };
        if tx.send(result).is_err() {
            log::warn!("Vault write worker for '{worker_path}' outlived its caller");
        }
    });
    rx.recv_timeout(Duration::from_secs(10))
        .map_err(|error| anyhow::anyhow!("Vault write for '{secret_path}' did not finish: {error}"))?
}

/// Resolve the password for the `zitadel` PostgreSQL role.
///
/// Precedence: an explicit operator override, then the value seeded into
/// `secret/gbo/directory`. When the seeded value is missing or weak a fresh
/// random password is generated and persisted back to Vault, so two fresh
/// installs never share a credential and an upgrade repairs an old secret.
/// Any Vault failure is propagated — never fall back to a fixed literal or an
/// empty value.
pub fn zitadel_db_password() -> Result<String> {
    if let Ok(override_value) = std::env::var(ZITADEL_PASSWORD_ENV) {
        if !override_value.is_empty() {
            if is_strong_zitadel_password(&override_value) {
                return Ok(override_value);
            }
            return Err(anyhow::anyhow!(
                "{ZITADEL_PASSWORD_ENV} must be at least 16 ASCII alphanumeric characters and must not be a known weak value"
            ));
        }
    }

    let mut secret = get_secret_strict_sync(DIRECTORY_SECRET_PATH)?;
    if let Some(existing) = secret.get(ZITADEL_PASSWORD_KEY) {
        if is_strong_zitadel_password(existing) {
            return Ok(existing.clone());
        }
    }

    let generated = crate::generate_random_string(32);
    secret.insert(ZITADEL_PASSWORD_KEY.to_string(), generated.clone());
    put_secret_sync(DIRECTORY_SECRET_PATH, secret)?;
    Ok(generated)
}

#[cfg(test)]
mod zitadel_password_tests {
    use super::*;

    #[test]
    fn two_installs_generate_distinct_strong_passwords() {
        let first = crate::generate_random_string(32);
        let second = crate::generate_random_string(32);
        assert_ne!(first, second, "two installs must not share a DB password");
        assert!(is_strong_zitadel_password(&first));
        assert!(is_strong_zitadel_password(&second));
        assert_ne!(first, "zitadel");
        assert_ne!(second, "zitadel");
    }

    #[test]
    fn weak_or_unsafe_passwords_are_rejected() {
        let weak = [
            String::new(),
            "zitadel".to_string(),
            "short".to_string(),
            "123456789012345".to_string(),
            "has spaces here".to_string(),
            "quote'and$meta".to_string(),
            "a".repeat(20),
        ];
        for candidate in &weak {
            assert!(
                !is_strong_zitadel_password(candidate),
                "weak value accepted: {candidate}"
            );
        }
    }
}
