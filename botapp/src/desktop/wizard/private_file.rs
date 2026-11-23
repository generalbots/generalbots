//! Owner-only credential writes for the desktop wizard (#2691).
//!
//! `std::fs::write` follows the process umask (0644 on a typical multi-user
//! host), so the admin bearer token in `desktop-session.json` used to be
//! readable by every local user. `OpenOptions::mode` alone is not enough: it
//! applies only when a *new* inode is created, so truncating an existing
//! world-readable file leaves its loose mode in place. Every credential write
//! therefore rotates the old file and creates a fresh 0600 inode with O_EXCL
//! inside a 0700 directory, and refuses to hand a token to the caller when a
//! private write cannot be guaranteed.

use std::io::{Error, ErrorKind, Result, Write};
use std::path::Path;

/// Owner read/write, nothing for group or other (#2691).
#[cfg(unix)]
const PRIVATE_MODE: u32 = 0o600;

/// Pure predicate over a Unix mode word. Kept filesystem-free so the security
/// property is directly testable, including the original 0644 regression.
#[cfg(unix)]
fn is_private_mode(mode: u32) -> bool {
    // Credential files must be exactly owner-read/write (0600); 0700 is a
    // directory mode and must never be accepted for a file.
    mode & 0o777 == 0o600
}

/// Creates, or tightens, the directory that holds a credential. Existing
/// ancestors are left alone; the leaf is made 0700 so other users cannot list
/// or open the files inside it.
#[cfg(unix)]
fn private_dir(path: &Path) -> Result<()> {
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
    let mut builder = std::fs::DirBuilder::new();
    builder.recursive(true).mode(0o700);
    builder.create(path)?;
    let metadata = std::fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "credential directory is not a real directory",
        ));
    }
    if metadata.permissions().mode() & 0o077 != 0 {
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o700);
        std::fs::set_permissions(path, permissions)?;
    }
    Ok(())
}

#[cfg(not(unix))]
fn private_dir(path: &Path) -> Result<()> {
    std::fs::create_dir_all(path)
}

/// Removes a previous credential so `create_new` can establish a fresh inode.
/// `symlink_metadata` does not follow links, so a planted symlink is removed
/// itself and never redirects the write to another file.
fn rotate_private_file(path: &Path) -> Result<()> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() => Err(Error::new(
            ErrorKind::IsADirectory,
            "credential path is a directory",
        )),
        Ok(_) => std::fs::remove_file(path),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}

/// Writes a credential with owner-only permissions from creation.
///
/// O_EXCL (`create_new`) is essential: `truncate` preserves the mode of an
/// already-world-readable file. Every failure path removes the partial file,
/// so a secret is never left behind with a loose mode.
pub(crate) fn write_private_file(path: &Path, contents: &[u8]) -> Result<()> {
    let result = write_private_file_inner(path, contents);
    if result.is_err() {
        let _ = std::fs::remove_file(path);
    }
    result
}

fn write_private_file_inner(path: &Path, contents: &[u8]) -> Result<()> {
    let parent = path.parent().ok_or_else(|| {
        Error::new(
            ErrorKind::InvalidInput,
            "credential path has no parent directory",
        )
    })?;
    private_dir(parent)?;
    rotate_private_file(path)?;

    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(PRIVATE_MODE);
    }
    let mut file = options.open(path)?;
    file.write_all(contents)?;
    file.sync_all()?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = file.metadata()?.permissions().mode() & 0o777;
        if !is_private_mode(mode) {
            return Err(Error::new(
                ErrorKind::PermissionDenied,
                format!("credential file has unsafe mode {mode:o}"),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(label: &str) -> std::path::PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir().join(format!("gb-{label}-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&path).expect("temp dir");
        path
    }

    #[cfg(unix)]
    #[test]
    fn only_owner_read_write_is_a_private_mode() {
        assert!(is_private_mode(0o600));
        for loose in [
            0o000, 0o200, 0o400, 0o700, 0o640, 0o644, 0o660, 0o666, 0o777,
        ] {
            assert!(!is_private_mode(loose), "accepted mode {loose:o}");
        }
    }

    #[cfg(unix)]
    #[test]
    fn a_world_readable_token_file_is_rotated_to_0600() {
        use std::os::unix::fs::PermissionsExt;
        let base = temp_dir("session");
        let file = base.join("desktop-session.json");
        std::fs::write(&file, b"old").expect("seed");
        let mut loose = std::fs::metadata(&file).expect("stat").permissions();
        loose.set_mode(0o644);
        std::fs::set_permissions(&file, loose).expect("chmod");

        write_private_file(&file, b"{\"access_token\":\"secret\"}").expect("write");
        let metadata = std::fs::metadata(&file).expect("stat");
        assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
        assert_eq!(
            std::fs::read(&file).expect("read"),
            b"{\"access_token\":\"secret\"}"
        );
        let dir_mode = std::fs::metadata(&base)
            .expect("stat dir")
            .permissions()
            .mode();
        assert_eq!(dir_mode & 0o077, 0, "credential directory must be private");
        std::fs::remove_dir_all(&base).ok();
    }

    #[cfg(unix)]
    #[test]
    fn a_planted_symlink_is_removed_not_followed() {
        use std::os::unix::fs::symlink;
        let base = temp_dir("symlink");
        let target = base.join("outside.json");
        std::fs::write(&target, b"keep").expect("seed");
        let link = base.join("desktop-session.json");
        symlink(&target, &link).expect("symlink");

        write_private_file(&link, b"token").expect("write");
        assert_eq!(std::fs::read(&target).expect("read"), b"keep");
        assert!(!std::fs::symlink_metadata(&link)
            .expect("stat")
            .file_type()
            .is_symlink());
        assert_eq!(std::fs::read(&link).expect("read"), b"token");
        std::fs::remove_dir_all(&base).ok();
    }

    #[test]
    fn a_directory_at_the_credential_path_is_refused() {
        let base = temp_dir("dirconflict");
        let file = base.join("desktop-session.json");
        std::fs::create_dir(&file).expect("seed dir");
        assert!(write_private_file(&file, b"token").is_err());
        assert!(file.is_dir(), "the directory must not be removed");
        std::fs::remove_dir_all(&base).ok();
    }
}
