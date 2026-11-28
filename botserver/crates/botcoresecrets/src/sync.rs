//! Synchronous Vault reads and writes for boot-time settings (#1968, #2141).
//!
//! The secrets manager is async, but several settings must be resolved before
//! the first request (peer node config, CSRF signing secret). Both go through
//! [`secret_sync`] so the blocking bridge exists exactly once and every caller
//! gets the same 5-second budget.
//!
//! Values are never logged here; callers report only the *origin* of a value.

use crate::manager::SecretsManager;
use crate::paths::SecretPaths;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::mpsc;
use std::time::Duration;

/// How long a boot-time read may block before the caller falls back.
const READ_TIMEOUT: Duration = Duration::from_secs(5);

use crate::token_source::vault_env_configured;

/// The secrets manager for one boot-time Vault call.
///
/// The [`SecretsManager::get`] singleton is fixed the first time anything asks
/// for it. On a desktop boot that is the peer-network read, before the
/// bootstrap has loaded the server's Vault token into the environment, so the
/// singleton stays disabled for the whole process and every read or write
/// through it falls back to the environment. A manager built from the current
/// environment sees the token; the singleton is the fallback when the
/// environment names no Vault.
fn boot_manager() -> Option<SecretsManager> {
    if vault_env_configured() {
        match SecretsManager::from_env() {
            Ok(manager) => return Some(manager),
            Err(error) => log::debug!("boot-time Vault client from the environment: {error}"),
        }
    }
    SecretsManager::get_clone().ok()
}

/// Read one Vault KV v2 path synchronously.
///
/// Returns `None` when Vault is not configured, the read fails or it does not
/// answer within [`READ_TIMEOUT`]. Callers then fall back to environment
/// variables and their own defaults — an empty map and `None` mean the same
/// thing: Vault did not supply a value.
pub fn secret_sync(path: &str) -> Option<BTreeMap<String, String>> {
    let manager = boot_manager()?;
    let read_path = path.to_string();
    let log_path = path.to_string();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build();
        let result = match rt {
            Ok(rt) => rt.block_on(async move {
                manager
                    .get_secret(&read_path)
                    .await
                    .ok()
                    .map(|secrets| secrets.into_iter().collect::<BTreeMap<_, _>>())
            }),
            Err(error) => {
                log::warn!("secret {log_path}: could not start Vault runtime: {error}");
                None
            }
        };
        if let Err(e) = tx.send(result) {
            log::warn!("secret {log_path}: could not deliver Vault read result to caller: {e}");
        }
    });
    rx.recv_timeout(READ_TIMEOUT).ok().flatten()
}

/// Keys accepted for the CSRF signing secret in `secret/gbo/system/security`.
///
/// `csrf_secret` matches the `internal_token` convention on the same path; the
/// hyphenated and uppercase spellings are accepted so an operator who copied
/// the environment variable name into Vault is not silently ignored.
const CSRF_SECRET_KEYS: [&str; 3] = ["csrf_secret", "csrf-secret", "CSRF_SECRET"];

/// CSRF signing secret from Vault `secret/gbo/system/security` (#2141).
///
/// `None` means Vault supplied nothing; the caller must then decide between the
/// `CSRF_SECRET` environment variable, an ephemeral per-process secret or
/// refusing to start. This function never invents a secret.
pub fn csrf_secret_sync() -> Option<String> {
    let secrets = secret_sync(SecretPaths::SECURITY)?;
    select_csrf_secret(&secrets)
}

/// Pick the first non-blank documented key from a Vault secret map.
fn select_csrf_secret(secrets: &BTreeMap<String, String>) -> Option<String> {
    CSRF_SECRET_KEYS.iter().find_map(|key| {
        secrets
            .get(*key)
            .map(String::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    })
}

/// How often, and how far apart, a failed CSRF secret write is retried.
const PERSIST_ATTEMPTS: u32 = 6;
const PERSIST_DELAY: Duration = Duration::from_secs(10);

/// The map to write back: every key already at the path, with `csrf_secret`
/// set to `value`. KV v2 replaces the whole secret, so nothing may be dropped.
fn with_csrf_secret(mut existing: HashMap<String, String>, value: String) -> HashMap<String, String> {
    existing.insert("csrf_secret".to_string(), value);
    existing
}

/// Whether a strict Vault read failed only because nothing is stored there yet.
fn is_not_found(error: &anyhow::Error) -> bool {
    matches!(
        error.downcast_ref::<vaultrs::error::ClientError>(),
        Some(vaultrs::error::ClientError::APIError { code: 404, .. })
    )
}

/// Store the CSRF signing secret in Vault at [`SecretPaths::SECURITY`] so
/// later boots reuse the same key.
///
/// The write merges into the map already stored there. That map is read
/// strictly from Vault: the lenient read falls back to environment defaults,
/// which must never be written into Vault, and a failed read aborts the write
/// rather than replacing the other keys on the path. The error names the
/// reason only, never the value. A failure is not fatal to the caller.
pub fn put_csrf_secret_sync(value: &str) -> Result<(), String> {
    let manager = boot_manager().ok_or_else(|| "the secrets manager is unavailable".to_string())?;
    if !manager.is_enabled() {
        return Err("no Vault address or token in this process yet".to_string());
    }
    let path = SecretPaths::SECURITY.to_string();
    let secret_value = value.to_string();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build();
        let result = match rt {
            Ok(rt) => rt.block_on(async {
                let existing = match manager.get_secret_strict(&path).await {
                    Ok(data) => data,
                    Err(error) if is_not_found(&error) => HashMap::new(),
                    Err(error) => return Err(format!("reading {path} failed: {error}")),
                };
                manager
                    .put_secret(&path, with_csrf_secret(existing, secret_value))
                    .await
                    .map_err(|error| format!("writing {path} failed: {error}"))
            }),
            Err(error) => Err(format!("could not start the Vault runtime: {error}")),
        };
        if tx.send(result).is_err() {
            log::debug!("CSRF secret persist: the caller stopped waiting for Vault");
        }
    });
    rx.recv_timeout(READ_TIMEOUT)
        .map_err(|_| "Vault did not answer in time".to_string())?
}

/// Runs `attempt` up to `attempts` times, sleeping `delay` between failures,
/// and returns the first success or the last failure.
fn retry_blocking(
    attempts: u32,
    delay: Duration,
    mut attempt: impl FnMut() -> Result<(), String>,
) -> Result<(), String> {
    let mut last = Err("no attempt was made".to_string());
    for round in 0..attempts {
        if round > 0 {
            std::thread::sleep(delay);
        }
        last = attempt();
        if last.is_ok() {
            break;
        }
    }
    last
}

/// Keeps trying to store a generated CSRF secret after the boot-time write
/// failed, so a Vault that becomes reachable a little later still receives it
/// and the next boot signs with the same key. Runs on its own thread; the
/// outcome is logged without the value.
pub fn persist_csrf_secret_in_background(value: String) {
    let spawned = std::thread::Builder::new()
        .name("csrf-secret-persist".to_string())
        .spawn(move || {
            std::thread::sleep(PERSIST_DELAY);
            match retry_blocking(PERSIST_ATTEMPTS, PERSIST_DELAY, || put_csrf_secret_sync(&value)) {
                Ok(()) => log::info!("CSRF signing secret persisted to Vault"),
                Err(reason) => log::warn!(
                    "Could not persist the generated CSRF signing secret to Vault ({reason}); \
                     tokens will not survive a restart"
                ),
            }
        });
    if let Err(error) = spawned {
        log::warn!("CSRF secret persist: could not start the retry thread: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_missing_or_blank_key_is_not_a_secret() {
        let mut secrets = BTreeMap::new();
        secrets.insert("csrf_secret".to_string(), "   ".to_string());
        assert!(select_csrf_secret(&secrets).is_none());
        secrets.insert("csrf_secret".to_string(), "  value  ".to_string());
        assert_eq!(select_csrf_secret(&secrets).as_deref(), Some("value"));
    }

    #[test]
    fn test_first_documented_key_wins() {
        let mut secrets = BTreeMap::new();
        secrets.insert("csrf-secret".to_string(), "from-hyphen".to_string());
        secrets.insert("csrf_secret".to_string(), "from-underscore".to_string());
        assert_eq!(select_csrf_secret(&secrets).as_deref(), Some("from-underscore"));
    }

    #[test]
    fn test_the_csrf_write_keeps_every_other_key_on_the_path() {
        let mut existing = HashMap::new();
        existing.insert("internal_token".to_string(), "keep-me".to_string());
        existing.insert("csrf_secret".to_string(), "old".to_string());
        let merged = with_csrf_secret(existing, "new".to_string());
        assert_eq!(merged.get("internal_token").map(String::as_str), Some("keep-me"));
        assert_eq!(merged.get("csrf_secret").map(String::as_str), Some("new"));
        assert_eq!(merged.len(), 2);
    }

    #[test]
    fn test_only_a_vault_404_counts_as_nothing_stored_yet() {
        let missing = anyhow::Error::from(vaultrs::error::ClientError::APIError {
            code: 404,
            errors: Vec::new(),
        });
        assert!(is_not_found(&missing));
        let denied = anyhow::Error::from(vaultrs::error::ClientError::APIError {
            code: 403,
            errors: vec!["permission denied".to_string()],
        });
        assert!(!is_not_found(&denied));
        assert!(!is_not_found(&anyhow::anyhow!("Vault is not enabled")));
    }

    #[test]
    fn test_retry_stops_at_the_first_success() {
        let mut calls = 0;
        let result = retry_blocking(5, Duration::ZERO, || {
            calls += 1;
            if calls < 3 {
                Err(format!("attempt {calls}"))
            } else {
                Ok(())
            }
        });
        assert_eq!(result, Ok(()));
        assert_eq!(calls, 3);
    }

    #[test]
    fn test_retry_reports_the_last_failure() {
        let mut calls = 0;
        let result = retry_blocking(4, Duration::ZERO, || {
            calls += 1;
            Err(format!("attempt {calls}"))
        });
        assert_eq!(result, Err("attempt 4".to_string()));
        assert_eq!(calls, 4);
        assert!(retry_blocking(0, Duration::ZERO, || Ok(())).is_err());
    }

    #[test]
    fn test_uppercase_alias_is_accepted_when_primary_is_blank() {
        let mut secrets = BTreeMap::new();
        secrets.insert("csrf_secret".to_string(), String::new());
        secrets.insert("CSRF_SECRET".to_string(), "from-env-alias".to_string());
        assert_eq!(select_csrf_secret(&secrets).as_deref(), Some("from-env-alias"));
    }
}
