//! #2686 — private staging for exported backup tarballs.
//!
//! `incus export` creates the tarball itself, so the process umask decides its
//! mode (0644 on most hosts). A VM backup holds the whole project disk and must
//! never be world-readable. We pre-create the target as a 0600 file: the
//! exporter opens an existing target with O_TRUNC and keeps its mode. A
//! post-export chmod is applied as defence in depth in case a new inode is
//! created instead.

use std::path::Path;

/// Owner read/write only — the mode every backup tarball must end up with.
pub(crate) const BACKUP_FILE_MODE: u32 = 0o600;

/// Pre-create `path` as a private (0600) empty file before an external tool
/// writes into it. Mirrors the private-staging pattern already used for the
/// pg_dump temp file in `ops_tools.rs`.
pub(crate) fn create_private_file(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(BACKUP_FILE_MODE)
            .open(path)
            .map(|_| ())
            .map_err(|e| format!("create private backup file {}: {e}", path.display()))
    }
    #[cfg(not(unix))]
    {
        // No POSIX mode bits off Unix; still create/truncate the staging file
        // so the export target exists before the external tool runs.
        std::fs::File::create(path)
            .map(|_| ())
            .map_err(|e| format!("create backup staging file {}: {e}", path.display()))
    }
}

/// Re-assert 0600 after the exporter ran, in case it recreated the file with
/// umask defaults (e.g. by writing to a new inode).
pub(crate) fn enforce_private_file(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(BACKUP_FILE_MODE))
            .map_err(|e| format!("chmod backup tarball {}: {e}", path.display()))
    }
    #[cfg(not(unix))]
    {
        if !path.exists() {
            return Err(format!("backup tarball missing: {}", path.display()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backup_file_mode_is_owner_only() {
        assert_eq!(BACKUP_FILE_MODE, 0o600);
        assert_eq!(BACKUP_FILE_MODE & 0o077, 0, "no group/other bits");
    }

    #[cfg(unix)]
    #[test]
    fn create_private_file_stages_mode_0600() {
        use std::os::unix::fs::PermissionsExt;
        let path = std::env::temp_dir().join(format!(
            "gb-backup-priv-{}-{}.tar.gz",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        create_private_file(&path).unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
        std::fs::remove_file(&path).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn enforce_private_file_restores_mode() {
        use std::os::unix::fs::PermissionsExt;
        let path = std::env::temp_dir().join(format!(
            "gb-backup-enforce-{}-{}.tar.gz",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        std::fs::File::create(&path).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        enforce_private_file(&path).unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
        std::fs::remove_file(&path).unwrap();
    }
}
