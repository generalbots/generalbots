//! The Vault token a process runs with: `VAULT_TOKEN`, or else the file named
//! by `VAULT_TOKEN_FILE` (#2498 writes only that reference into `.env`).
//! The token value is never logged.

use std::path::Path;

/// Leading bytes of a file sealed for the current Windows user by
/// `botcorepkg::protected_file`. Such a file is not a token by itself: the
/// secret store opens it at boot and puts the token into `VAULT_TOKEN`.
pub const SEALED_MAGIC: &[u8] = b"GBSEALED1\n";

/// The token from the environment, or `None` when neither source has one.
pub fn vault_token() -> Option<String> {
    let direct = std::env::var("VAULT_TOKEN")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    if direct.is_some() {
        return direct;
    }
    let file = std::env::var("VAULT_TOKEN_FILE").ok()?;
    let file = file.trim();
    if file.is_empty() {
        return None;
    }
    token_from_file(Path::new(file))
}

/// The trimmed token stored in `path`; `None` for a missing, empty, non-text
/// or sealed file.
pub fn token_from_file(path: &Path) -> Option<String> {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) => {
            log::debug!("VAULT_TOKEN_FILE cannot be read: {e}");
            return None;
        }
    };
    if bytes.starts_with(SEALED_MAGIC) {
        log::debug!("VAULT_TOKEN_FILE is sealed; the secret store loads it into VAULT_TOKEN");
        return None;
    }
    String::from_utf8(bytes)
        .ok()
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
}

/// Whether the environment names a Vault address and a token for it.
pub fn vault_env_configured() -> bool {
    std::env::var("VAULT_ADDR").is_ok_and(|value| !value.trim().is_empty()) && vault_token().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("gb-token-source-{}-{name}", std::process::id()))
    }

    #[test]
    fn file_token_is_trimmed() -> std::io::Result<()> {
        let path = scratch("plain");
        std::fs::write(&path, "  hvs.scoped\r\n")?;
        assert_eq!(token_from_file(&path).as_deref(), Some("hvs.scoped"));
        std::fs::remove_file(&path)
    }

    #[test]
    fn sealed_empty_and_missing_files_give_no_token() -> std::io::Result<()> {
        let sealed = scratch("sealed");
        let mut bytes = SEALED_MAGIC.to_vec();
        bytes.extend_from_slice(&[0, 159, 146, 150]);
        std::fs::write(&sealed, bytes)?;
        assert_eq!(token_from_file(&sealed), None);
        let empty = scratch("empty");
        std::fs::write(&empty, " \n")?;
        assert_eq!(token_from_file(&empty), None);
        assert_eq!(token_from_file(&scratch("missing")), None);
        std::fs::remove_file(&sealed)?;
        std::fs::remove_file(&empty)
    }
}
