//! Where the first-run secret store lives when the appliance credential
//! directory is absent, and how paths are written into the stack `.env`.

use std::path::{Path, PathBuf};

/// Windows desktop: the stack belongs to the signed-in user alone, so the
/// store lives inside it and `secret_store::write` seals each file with DPAPI
/// for that user. A per-process temp directory is not found again by the next
/// start (another process id) nor after a temp clean-up, and the unseal keys
/// would be lost with it.
#[cfg(windows)]
pub(super) fn fallback_directory() -> PathBuf {
    stack_root().join("conf").join("vault").join("credentials")
}

/// Development installs without the appliance account: a private
/// per-process temp directory (#2498).
#[cfg(not(windows))]
pub(super) fn fallback_directory() -> PathBuf {
    std::env::temp_dir().join(format!("gb-vault-secrets-{}", std::process::id()))
}

#[cfg(windows)]
fn stack_root() -> PathBuf {
    ["BOTSERVER_STACK_PATH", "GBO_STACK_PATH"]
        .iter()
        .find_map(|key| std::env::var(key).ok().filter(|value| !value.trim().is_empty()))
        .map(PathBuf::from)
        .unwrap_or_else(|| match std::env::current_dir() {
            Ok(dir) => dir.join("botserver-stack"),
            Err(e) => {
                log::warn!("Current directory unavailable ({e}); secret store is relative");
                PathBuf::from("botserver-stack")
            }
        })
}

/// A `.env` value that dotenv reads back unchanged: double-quoted, with `\`,
/// `"` and `$` escaped. On Windows the path is written with forward slashes
/// and without the verbatim `\\?\` prefix; Windows accepts that spelling and
/// the stack-relocation rewrite matches it.
pub fn env_value(value: &str) -> String {
    let plain = if cfg!(windows) {
        value.strip_prefix(r"\\?\").unwrap_or(value).replace('\\', "/")
    } else {
        value.to_string()
    };
    let mut out = String::with_capacity(plain.len() + 2);
    out.push('"');
    for c in plain.chars() {
        if matches!(c, '\\' | '"' | '$') {
            out.push('\\');
        }
        out.push(c);
    }
    out.push('"');
    out
}

pub fn env_path(path: &Path) -> String {
    env_value(&path.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(line: &str) -> Result<(String, String), dotenvy::Error> {
        let mut entries = dotenvy::from_read_iter(line.as_bytes());
        match entries.next() {
            Some(entry) => entry,
            None => Err(dotenvy::Error::LineParse(line.to_string(), 0)),
        }
    }

    #[test]
    fn windows_path_with_backslash_u_space_and_quote_reads_back() -> Result<(), dotenvy::Error> {
        let raw = r"C:\Users\Jo O'Neil\App Data\$HOME\gb stack\conf\vault\credentials\vault-token";
        let (key, value) = parse(&format!("VAULT_TOKEN_FILE={}\n", env_value(raw)))?;
        assert_eq!(key, "VAULT_TOKEN_FILE");
        let expected = if cfg!(windows) { raw.replace('\\', "/") } else { raw.to_string() };
        assert_eq!(value, expected);
        Ok(())
    }

    #[test]
    fn unquoted_windows_path_is_what_failed() {
        assert!(parse("VAULT_TOKEN_FILE=C:\\Users\\x\\vault-token\n").is_err());
    }

    #[test]
    fn verbatim_prefix_is_dropped_on_windows() -> Result<(), dotenvy::Error> {
        let (_, value) = parse(&format!("P={}\n", env_value(r"\\?\C:\Users\a b\ca.crt")))?;
        if cfg!(windows) {
            assert_eq!(value, "C:/Users/a b/ca.crt");
        } else {
            assert_eq!(value, r"\\?\C:\Users\a b\ca.crt");
        }
        Ok(())
    }

    #[test]
    fn token_readers_recognise_sealed_files() {
        assert_eq!(
            crate::protected_file::SEALED_MAGIC,
            botcoresecrets::token_source::SEALED_MAGIC
        );
    }

    #[test]
    fn unix_path_and_double_quote_survive() -> Result<(), dotenvy::Error> {
        let (_, value) = parse(&format!("P={}\n", env_value("/opt/gbo/a \"b\"/ca.crt")))?;
        assert_eq!(value, "/opt/gbo/a \"b\"/ca.crt");
        Ok(())
    }
}
