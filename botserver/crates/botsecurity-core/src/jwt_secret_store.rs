//! On-disk store of the SaaS JWT signing secret (`conf/system/directory_config.json`).
//!
//! Kept apart from the resolver so the file handling can be tested against a
//! temporary directory without touching the process environment.

use std::path::Path;

/// The `key` value of the JSON object at `path`, or `None` when the file is
/// missing, unreadable, not JSON or has no string under `key`.
pub(crate) fn read_secret_at(path: &Path, key: &str) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    json.get(key)
        .and_then(|value| value.as_str())
        .map(str::to_string)
}

/// Writes `secret` under `key` into the JSON object at `path`, keeping the
/// other keys, atomically: a temporary file in the same directory is renamed
/// over the target, so a crash never leaves a half-written config behind.
///
/// A missing parent directory is created (owner-only on Unix): on an edition
/// without the directory service nothing else creates `conf/system`, and a
/// missing folder must not stop the server from starting. The file itself is
/// owner-only on Unix; on Windows it inherits the stack folder's ACL.
pub(crate) fn write_secret_at(path: &Path, key: &str, secret: &str) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("{} has no parent directory", path.display()))?;
    if !parent.is_dir() {
        create_private_dir(parent)?;
    }
    let mut json = match std::fs::read_to_string(path) {
        Ok(content) => serde_json::from_str::<serde_json::Value>(&content)
            .map_err(|e| format!("parse {}: {e}", path.display()))?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => serde_json::json!({}),
        Err(error) => return Err(format!("read {}: {error}", path.display())),
    };
    if !json.is_object() {
        json = serde_json::json!({});
    }
    json[key] = serde_json::Value::String(secret.to_string());
    let pretty = serde_json::to_string_pretty(&json).map_err(|e| format!("serialize: {e}"))?;
    let temp = path.with_extension("json.tmp");
    write_private_file(&temp, format!("{pretty}\n").as_bytes())
        .map_err(|e| format!("write {}: {e}", temp.display()))?;
    std::fs::rename(&temp, path).map_err(|e| {
        if let Err(cleanup) = std::fs::remove_file(&temp) {
            log::debug!("could not remove {}: {cleanup}", temp.display());
        }
        format!("rename {}: {e}", path.display())
    })
}

fn create_private_dir(dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("create {}: {e}", dir.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))
            .map_err(|e| format!("restrict {}: {e}", dir.display()))?;
    }
    Ok(())
}

fn write_private_file(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    let mut options = std::fs::OpenOptions::new();
    options.create(true).write(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: &str = "saas_jwt_secret";

    #[test]
    fn a_missing_folder_is_created_and_the_secret_written() {
        let root = tempfile::tempdir().expect("tempdir");
        let path = root.path().join("conf/system/directory_config.json");
        assert!(!path.parent().expect("parent").exists());

        write_secret_at(&path, KEY, "first-secret").expect("persist into a missing folder");

        assert_eq!(read_secret_at(&path, KEY).as_deref(), Some("first-secret"));
        assert!(!path.with_extension("json.tmp").exists());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = |p: &Path| std::fs::metadata(p).expect("meta").permissions().mode() & 0o777;
            assert_eq!(mode(path.parent().expect("parent")), 0o700);
            assert_eq!(mode(&path), 0o600);
        }
    }

    #[test]
    fn other_keys_of_an_existing_config_are_kept() {
        let root = tempfile::tempdir().expect("tempdir");
        let path = root.path().join("directory_config.json");
        std::fs::write(&path, r#"{"base_url":"http://localhost:9000"}"#).expect("seed");

        write_secret_at(&path, KEY, "new-secret").expect("persist");

        let json: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).expect("read")).expect("json");
        assert_eq!(json["base_url"], "http://localhost:9000");
        assert_eq!(json[KEY], "new-secret");
    }

    #[test]
    fn a_missing_or_foreign_file_reads_as_no_secret() {
        let root = tempfile::tempdir().expect("tempdir");
        let path = root.path().join("directory_config.json");
        assert_eq!(read_secret_at(&path, KEY), None);
        std::fs::write(&path, r#"{"other":"x"}"#).expect("seed");
        assert_eq!(read_secret_at(&path, KEY), None);
    }
}
