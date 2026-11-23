//! #2616 — private, unpredictable staging for files later pushed into a
//! privileged container. A predictable `/tmp/{name}` can be pre-created or
//! symlinked by a local user and race provisioning into an arbitrary
//! root-owned systemd unit or binary.

use std::io::Write;
use std::path::{Path, PathBuf};

/// A file in a freshly created 0700 directory with a random name, opened with
/// `create_new` + `0600` so it never follows a pre-existing symlink. Removing
/// the whole directory on drop cleans up every staging artifact.
pub struct StagedFile {
    pub path: PathBuf,
    dir: PathBuf,
}

impl StagedFile {
    pub fn create(name: &str, content: &[u8]) -> anyhow::Result<Self> {
        let dir = std::env::temp_dir().join(format!("gb-stage-{}-{}", name, uuid::Uuid::new_v4()));
        std::fs::create_dir(&dir)
            .map_err(|e| anyhow::anyhow!("create staging dir for {name}: {e}"))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))
                .map_err(|e| anyhow::anyhow!("chmod staging dir for {name}: {e}"))?;
        }
        let path = dir.join(name);
        let mut opts = std::fs::OpenOptions::new();
        opts.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600);
        }
        let mut file = opts
            .open(&path)
            .map_err(|e| anyhow::anyhow!("create staging file {name}: {e}"))?;
        file.write_all(content)
            .map_err(|e| anyhow::anyhow!("write staging file {name}: {e}"))?;
        Ok(Self { path, dir })
    }

    pub fn as_path(&self) -> &Path {
        &self.path
    }
}

impl Drop for StagedFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

#[cfg(test)]
mod tests {
    use super::StagedFile;

    #[cfg(unix)]
    #[test]
    fn precreated_symlink_cannot_hijack_staging() {
        let dir = std::env::temp_dir().join(format!("gb-stage-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&dir).expect("create test dir");
        let victim = dir.join("victim");
        std::fs::write(&victim, b"ORIGINAL").expect("victim");
        let link = dir.join("link");
        std::os::unix::fs::symlink(&victim, &link).expect("symlink");
        let mut opts = std::fs::OpenOptions::new();
        opts.write(true).create_new(true);
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
        assert!(opts.open(&link).is_err(), "create_new must refuse an existing symlink");
        assert_eq!(std::fs::read(&victim).expect("read"), b"ORIGINAL");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn staged_file_is_private_and_unique() {
        let a = StagedFile::create("unit.service", b"content").expect("stage A");
        let b = StagedFile::create("unit.service", b"content").expect("stage B");
        assert_ne!(a.path, b.path, "staging paths must be unpredictable");
        assert_eq!(std::fs::read(&a.path).expect("read"), b"content");
        let dir = a.path.parent().expect("parent");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(dir).expect("meta").permissions().mode() & 0o777;
            assert_eq!(mode, 0o700);
            let fmode = std::fs::metadata(&a.path).expect("meta").permissions().mode() & 0o777;
            assert_eq!(fmode, 0o600);
        }
    }
}
