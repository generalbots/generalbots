//! The one-use recovery secret and the file that hands it to the desktop shell.
//!
//! The file lives in the stack folder inside the person's own profile, so only
//! this Windows account (and the machine's administrators) can read it; on
//! Unix it is also restricted to mode 0600. A request with the right secret
//! consumes it and a new one is written at once.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Header the shell sends the secret in.
pub const SECRET_HEADER: &str = "x-gb-recovery-secret";

pub fn secret_path() -> PathBuf {
    PathBuf::from(botcore::shared::utils::get_stack_path()).join("conf/directory/local-recovery.json")
}

fn fresh_secret() -> String {
    let bytes: [u8; 32] = rand::random();
    hex::encode(bytes)
}

/// Constant-time comparison; an empty expected secret never matches.
pub fn secret_matches(expected: &str, presented: &str) -> bool {
    !expected.is_empty()
        && presented.len() == expected.len()
        && botsecurity_core::constant_time::eq_str(expected, presented)
}

fn write_secret_file(path: &Path, port: u16, secret: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let raw = serde_json::json!({ "port": port, "secret": secret }).to_string();
    std::fs::write(path, raw)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

pub struct SecretStore {
    path: PathBuf,
    port: u16,
    current: Mutex<String>,
}

impl SecretStore {
    /// Creates the store and writes its first secret.
    pub fn create(path: PathBuf, port: u16) -> std::io::Result<Self> {
        let secret = fresh_secret();
        write_secret_file(&path, port, &secret)?;
        Ok(Self {
            path,
            port,
            current: Mutex::new(secret),
        })
    }

    /// True when `presented` is the current secret, which is then replaced so
    /// it works only once. A wrong secret leaves the current one in place.
    pub fn consume(&self, presented: &str) -> bool {
        let Ok(mut current) = self.current.lock() else {
            return false;
        };
        if !secret_matches(&current, presented) {
            return false;
        }
        let next = fresh_secret();
        match write_secret_file(&self.path, self.port, &next) {
            Ok(()) => *current = next,
            Err(e) => {
                log::warn!("Local recovery: could not rotate the secret file: {e}");
                current.clear();
            }
        }
        true
    }

    pub fn remove_file(&self) {
        if let Err(e) = std::fs::remove_file(&self.path)
        {
            log::debug!("recovery secret removal failed: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_exact_secret_matches() {
        let secret = fresh_secret();
        assert_eq!(secret.len(), 64);
        assert!(secret_matches(&secret, &secret));
        assert!(!secret_matches(&secret, ""));
        assert!(!secret_matches(&secret, &secret[..63]));
        assert!(!secret_matches(&secret, &fresh_secret()));
        assert!(!secret_matches("", ""));
    }

    #[test]
    fn a_secret_works_once_and_is_rotated_on_disk() {
        let dir = std::env::temp_dir().join(format!("gb-recovery-test-{}", uuid::Uuid::new_v4()));
        let path = dir.join("local-recovery.json");
        let store = SecretStore::create(path.clone(), 4242).expect("store");
        let read = |p: &Path| -> String {
            let raw = std::fs::read_to_string(p).expect("file");
            let json: serde_json::Value = serde_json::from_str(&raw).expect("json");
            assert_eq!(json["port"], 4242);
            json["secret"].as_str().unwrap_or_default().to_string()
        };
        let first = read(&path);
        assert!(!store.consume("wrong"));
        assert_eq!(read(&path), first, "a wrong secret does not rotate");
        assert!(store.consume(&first));
        assert!(!store.consume(&first), "a used secret is refused");
        let second = read(&path);
        assert_ne!(first, second);
        assert!(store.consume(&second));
        store.remove_file();
        let _ = std::fs::remove_dir_all(dir);
    }
}
