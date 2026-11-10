//! Synchronous Vault reads for boot-time settings (#1968, #2141).
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
use std::sync::mpsc;
use std::time::Duration;

/// How long a boot-time read may block before the caller falls back.
const READ_TIMEOUT: Duration = Duration::from_secs(5);

/// Read one Vault KV v2 path synchronously.
///
/// Returns `None` when Vault is not configured, the read fails or it does not
/// answer within [`READ_TIMEOUT`]. Callers then fall back to environment
/// variables and their own defaults — an empty map and `None` mean the same
/// thing: Vault did not supply a value.
pub fn secret_sync(path: &str) -> Option<BTreeMap<String, String>> {
    let manager = SecretsManager::get_clone().ok()?;
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build();
        let result = match rt {
            Ok(rt) => rt.block_on(async move {
                manager
                    .get_secret(path)
                    .await
                    .ok()
                    .map(|secrets| secrets.into_iter().collect::<BTreeMap<_, _>>())
            }),
            Err(error) => {
                log::warn!("secret {path}: could not start Vault runtime: {error}");
                None
            }
        };
        let _ = tx.send(result);
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
    CSRF_SECRET_KEYS.iter().find_map(|key| {
        secrets
            .get(*key)
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_key_is_not_a_secret() {
        let mut secrets = BTreeMap::new();
        secrets.insert("csrf_secret".to_string(), "   ".to_string());
        let found = CSRF_SECRET_KEYS.iter().find_map(|key| {
            secrets
                .get(*key)
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        });
        assert!(found.is_none());
    }

    #[test]
    fn the_first_documented_key_wins() {
        let mut secrets = BTreeMap::new();
        secrets.insert("csrf-secret".to_string(), "from-hyphen".to_string());
        secrets.insert("csrf_secret".to_string(), "from-underscore".to_string());
        let found = CSRF_SECRET_KEYS.iter().find_map(|key| {
            secrets
                .get(*key)
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        });
        assert_eq!(found.as_deref(), Some("from-underscore"));
    }
}
