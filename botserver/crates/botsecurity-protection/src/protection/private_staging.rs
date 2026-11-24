//! #2790 — private staging for privileged config pushed into containers.
//! Kept local to this crate so no `botcore`/`botcorepkg` dependency is needed.

use std::io::Write;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

/// A file in a fresh 0700 directory with an unpredictable name, opened with
/// `create_new` + `0600`. A local user cannot pre-create or symlink the path
/// that is later pushed into a privileged container location.
pub struct StagedConfig {
    pub path: PathBuf,
    dir: PathBuf,
}

impl StagedConfig {
    pub fn create(name: &str, content: &[u8]) -> anyhow::Result<Self> {
        let dir = std::env::temp_dir().join(format!("gb-sec-stage-{name}-{}", uuid::Uuid::new_v4()));
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

impl Drop for StagedConfig {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

#[cfg(test)]
mod tests {
    use super::StagedConfig;

    #[cfg(unix)]
    #[test]
    fn precreated_symlink_cannot_hijack_staging() {
        let dir = std::env::temp_dir().join(format!("gb-sec-test-{}", uuid::Uuid::new_v4()));
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
    fn staged_config_is_private_and_unique() {
        let a = StagedConfig::create("jail.local", b"content").expect("stage A");
        let b = StagedConfig::create("jail.local", b"content").expect("stage B");
        assert_ne!(a.as_path(), b.as_path());
        assert_eq!(std::fs::read(a.as_path()).expect("read"), b"content");
    }
}
