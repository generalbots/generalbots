//! Secure staging of privileged configuration before it is pushed into
//! containers (issue #2790).
//!
//! Hardening code used to write Caddy/CoreDNS/fail2ban config to fixed,
//! world-predictable paths such as `/tmp/gb-caddy-host` and `/tmp/gb-corefile`
//! before pushing them into privileged containers with `incus file push`. A
//! local user could pre-create those paths (or plant a symlink) and have root
//! push attacker-chosen config into the container.
//!
//! This module creates a fresh random `0700` directory, stages every payload
//! inside it with `create_new(true)` and `0600` permissions, verifies the bytes
//! actually written, and removes the whole directory on drop.

use anyhow::{anyhow, Context, Result};
use botsecurity_core::command_guard::SafeCommand;
use std::fs::{self, DirBuilder, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};

/// Single-use staging directory. Dropping it removes the directory and every
/// staged payload, so privileged config never lingers in `/tmp`.
pub struct SecureStaging {
    dir: PathBuf,
}

impl SecureStaging {
    /// Create a fresh `0700` staging directory under the system temp dir.
    pub fn new() -> Result<Self> {
        Self::create_in(&std::env::temp_dir())
    }

    fn create_in(base: &Path) -> Result<Self> {
        for _ in 0..8 {
            let name = format!("gb-secure-stage-{}", uuid::Uuid::new_v4());
            match Self::create_with_name(base, &name) {
                Ok(staging) => return Ok(staging),
                Err(e) if e.downcast_ref::<std::io::Error>().is_some_and(|io| {
                    io.kind() == std::io::ErrorKind::AlreadyExists
                }) => continue,
                Err(e) => return Err(e),
            }
        }
        Err(anyhow!("could not allocate a unique secure staging directory"))
    }

    fn create_with_name(base: &Path, name: &str) -> Result<Self> {
        let dir = base.join(name);
        let mut builder = DirBuilder::new();
        #[cfg(unix)]
        builder.mode(0o700);
        builder
            .create(&dir)
            .with_context(|| format!("creating secure staging directory {}", dir.display()))?;
        Ok(Self { dir })
    }

    /// Stage `content` in a new `0600` file with a random name.
    pub fn stage(&self, content: &[u8]) -> Result<PathBuf> {
        self.stage_named(&format!("payload-{}", uuid::Uuid::new_v4()), content)
    }

    /// Stage `content` in a new `0600` file named `name` inside this private
    /// directory. `create_new(true)` refuses to follow an existing file or
    /// symlink; the bytes are read back before the path is returned.
    pub(crate) fn stage_named(&self, name: &str, content: &[u8]) -> Result<PathBuf> {
        let path = self.dir.join(name);
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        options.mode(0o600);
        let mut file = options
            .open(&path)
            .with_context(|| format!("creating staged file {}", path.display()))?;
        file.write_all(content)
            .with_context(|| format!("writing staged file {}", path.display()))?;
        file.flush()
            .with_context(|| format!("flushing staged file {}", path.display()))?;
        drop(file);

        let mut written = Vec::with_capacity(content.len());
        fs::File::open(&path)
            .and_then(|mut f| f.read_to_end(&mut written))
            .with_context(|| format!("reading back staged file {}", path.display()))?;
        if written.as_slice() != content {
            return Err(anyhow!(
                "staged file {} failed byte verification",
                path.display()
            ));
        }
        Ok(path)
    }

    /// Absolute path of this private staging directory.
    pub(crate) fn path(&self) -> &Path {
        &self.dir
    }
}

impl Drop for SecureStaging {
    fn drop(&mut self) {
        if let Err(e) = fs::remove_dir_all(&self.dir) {
            tracing::warn!(
                error = %e,
                path = %self.dir.display(),
                "failed to clean up secure staging directory"
            );
        }
    }
}

/// Confirm the bytes that arrived in the container match what was staged.
pub fn verify_pushed(container: &str, destination: &str, expected: &[u8]) -> Result<()> {
    let output = SafeCommand::new("incus")?
        .arg("exec")?
        .arg(container)?
        .arg("--")?
        .arg("cat")?
        .arg(destination)?
        .execute()
        .context("incus exec cat verification failed")?;
    if !output.status.success() {
        return Err(anyhow!(
            "verification read of {destination} in {container} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    if output.stdout.as_slice() != expected {
        return Err(anyhow!(
            "bytes pushed to {destination} in {container} do not match staged content"
        ));
    }
    Ok(())
}

/// A random, collision-resistant name fragment used for in-container staging.
pub fn random_suffix() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_base(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!("gb-secure-stage-test-{tag}-{}", uuid::Uuid::new_v4()))
    }

    #[test]
    fn staging_directories_are_unique_and_random() {
        let first = SecureStaging::new().unwrap();
        let second = SecureStaging::new().unwrap();
        assert_ne!(first.path(), second.path());
        let name = first.path().file_name().unwrap().to_string_lossy();
        assert!(name.starts_with("gb-secure-stage-"), "unexpected name: {name}");
        assert_ne!(name, second.path().file_name().unwrap().to_string_lossy());
    }

    #[test]
    fn directory_precreation_is_refused() {
        let base = test_base("dir");
        fs::create_dir_all(&base).unwrap();
        let first = SecureStaging::create_with_name(&base, "taken").unwrap();
        assert!(SecureStaging::create_with_name(&base, "taken").is_err());
        assert!(first.path().is_dir());
        drop(first);
        fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn staged_file_names_are_unique() {
        let staging = SecureStaging::new().unwrap();
        let one = staging.stage(b"one").unwrap();
        let two = staging.stage(b"two").unwrap();
        assert_ne!(one, two);
        assert_eq!(fs::read(&one).unwrap(), b"one".to_vec());
        assert_eq!(fs::read(&two).unwrap(), b"two".to_vec());
    }

    #[test]
    fn precreated_payload_is_refused() {
        let base = test_base("precreate");
        fs::create_dir_all(&base).unwrap();
        let staging = SecureStaging::create_in(&base).unwrap();
        let taken = staging.path().join("taken");
        fs::write(&taken, b"attacker").unwrap();
        let error = staging.stage_named("taken", b"privileged").unwrap_err();
        assert!(error.to_string().contains("creating staged file"), "{error}");
        assert_eq!(fs::read(&taken).unwrap(), b"attacker".to_vec());
        drop(staging);
        fs::remove_dir_all(&base).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn directory_and_file_are_private() {
        use std::os::unix::fs::PermissionsExt;
        let staging = SecureStaging::new().unwrap();
        let dir_mode = fs::metadata(staging.path()).unwrap().permissions().mode() & 0o777;
        assert_eq!(dir_mode, 0o700);
        let path = staging.stage(b"privileged config").unwrap();
        let file_mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(file_mode, 0o600);
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_payload_is_not_followed() {
        let base = test_base("symlink");
        fs::create_dir_all(&base).unwrap();
        let staging = SecureStaging::create_in(&base).unwrap();
        let outside = base.join("outside");
        fs::write(&outside, b"attacker").unwrap();
        std::os::unix::fs::symlink(&outside, staging.path().join("linked")).unwrap();
        assert!(staging.stage_named("linked", b"privileged").is_err());
        assert_eq!(fs::read(&outside).unwrap(), b"attacker".to_vec());
        drop(staging);
        fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn random_suffixes_are_unique() {
        assert_ne!(random_suffix(), random_suffix());
    }
}
