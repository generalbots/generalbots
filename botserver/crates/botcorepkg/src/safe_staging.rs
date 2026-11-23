//! #2789 — private, unpredictable staging for files pushed into privileged
//! containers. Mirrors `botcore::package_manager::safe_staging` without taking
//! a dependency on `botcore` (see #1501).

use std::io::Write;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

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
            .map_err(|error| anyhow::anyhow!("create staging dir for {name}: {error}"))?;
        #[cfg(unix)]
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))
            .map_err(|error| anyhow::anyhow!("chmod staging dir for {name}: {error}"))?;

        let path = dir.join(name);
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        options.mode(0o600);
        let mut file = options
            .open(&path)
            .map_err(|error| anyhow::anyhow!("create staging file {name}: {error}"))?;
        file.write_all(content)
            .map_err(|error| anyhow::anyhow!("write staging file {name}: {error}"))?;
        file.flush()
            .map_err(|error| anyhow::anyhow!("flush staging file {name}: {error}"))?;
        drop(file);
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
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
        assert!(options.open(&link).is_err(), "create_new must refuse a symlink");
        assert_eq!(std::fs::read(&victim).expect("read"), b"ORIGINAL");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn staged_file_is_private_and_unique() {
        let a = StagedFile::create("unit.service", b"content").expect("stage A");
        let b = StagedFile::create("unit.service", b"content").expect("stage B");
        assert_ne!(a.as_path(), b.as_path());
        assert_eq!(std::fs::read(a.as_path()).expect("read A"), b"content");
    }
}
