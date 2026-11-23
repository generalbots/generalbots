//! #2789 — port of `botcore::package_manager::safe_staging` for this crate.
//! `botcorepkg` cannot depend on `botcore` (botcore already depends on
//! botcorepkg), so the primitive is duplicated here with the atomic-write
//! hardening requested by the issue.
//!
//! A predictable `/tmp/{name}` can be pre-created or symlinked by a local
//! user and race provisioning into an arbitrary root-owned systemd unit.
//! Staging instead happens in a fresh 0700 directory with an unpredictable
//! name; the file is opened with `create_new` + `0600`, written, fsynced and
//! only then `rename`d into place. The final bytes are read back and compared
//! to the generated template before the caller pushes them into the
//! privileged container.

use std::io::Write;
use std::path::{Path, PathBuf};

/// Name used for the not-yet-published file. Keeping this in a helper makes
/// the atomic-write path generation directly testable.
fn temp_staging_name(name: &str, token: &str) -> String {
    format!("{name}.tmp-{token}")
}

/// A file in a freshly created 0700 directory with a random name. Dropping the
/// value removes the whole directory, cleaning up every staging artifact —
/// including on an error path in the caller.
pub struct StagedFile {
    pub path: PathBuf,
    dir: PathBuf,
}

impl StagedFile {
    /// Stage `content` atomically and return the published path.
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

        let final_path = dir.join(name);
        let temp_path = dir.join(temp_staging_name(name, &uuid::Uuid::new_v4().to_string()));
        let mut opts = std::fs::OpenOptions::new();
        opts.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600);
        }
        let mut file = opts
            .open(&temp_path)
            .map_err(|e| anyhow::anyhow!("create staging file {name}: {e}"))?;
        file.write_all(content)
            .map_err(|e| anyhow::anyhow!("write staging file {name}: {e}"))?;
        file.sync_all()
            .map_err(|e| anyhow::anyhow!("fsync staging file {name}: {e}"))?;
        drop(file);

        // Same-filesystem rename is atomic: readers see either the old state
        // (no file) or the fully written, fsynced file — never a partial unit.
        std::fs::rename(&temp_path, &final_path)
            .map_err(|e| anyhow::anyhow!("publish staging file {name}: {e}"))?;

        // #2789 — compare the bytes that will be pushed to the generated
        // template, so a mismatch fails closed instead of enabling a unit the
        // caller did not produce.
        let published = std::fs::read(&final_path)
            .map_err(|e| anyhow::anyhow!("read back staging file {name}: {e}"))?;
        if published != content {
            return Err(anyhow::anyhow!(
                "staged file {name} differs from the generated template"
            ));
        }

        Ok(Self {
            path: final_path,
            dir,
        })
    }

    pub fn as_path(&self) -> &Path {
        &self.path
    }
}

impl Drop for StagedFile {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_dir_all(&self.dir) {
            log::debug!("staging cleanup failed for {}: {error}", self.dir.display());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{temp_staging_name, StagedFile};

    #[test]
    fn atomic_write_paths_are_unique_and_distinct_from_final() {
        let final_name = "unit.service";
        let a = temp_staging_name(final_name, "token-a");
        let b = temp_staging_name(final_name, "token-b");
        assert_ne!(a, b, "two staging attempts must use different temp paths");
        assert_ne!(
            a, final_name,
            "temp path must differ from the published path"
        );
        assert!(a.starts_with("unit.service.tmp-"));
    }

    #[test]
    fn staged_file_is_published_atomically_and_privately() {
        let a = StagedFile::create("unit.service", b"content").expect("stage A");
        let b = StagedFile::create("unit.service", b"content").expect("stage B");
        assert_ne!(a.path, b.path, "staging paths must be unpredictable");
        assert_eq!(
            std::fs::read(&a.path).expect("read"),
            b"content",
            "published bytes must equal the template"
        );
        assert_eq!(
            a.path.file_name().and_then(|n| n.to_str()),
            Some("unit.service"),
            "the temp name must be renamed away"
        );
        let dir = a.path.parent().expect("parent");
        let entries = std::fs::read_dir(dir).expect("read_dir").count();
        assert_eq!(entries, 1, "only the published file should remain");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(dir).expect("meta").permissions().mode() & 0o777;
            assert_eq!(mode, 0o700);
            let fmode = std::fs::metadata(&a.path)
                .expect("meta")
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(fmode, 0o600);
        }
    }

    #[cfg(unix)]
    #[test]
    fn precreated_symlink_cannot_hijack_staging() {
        // `create_new` is the primitive that makes a pre-created path or a
        // symlink at the target name fatal. The random 0700 directory makes
        // such a pre-creation unreachable in practice; this test proves the
        // underlying open still refuses it if attempted.
        let dir = std::env::temp_dir().join(format!("gb-stage-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&dir).expect("create test dir");
        let victim = dir.join("victim");
        std::fs::write(&victim, b"ORIGINAL").expect("victim");
        let link = dir.join("unit.service");
        std::os::unix::fs::symlink(&victim, &link).expect("symlink");
        let mut opts = std::fs::OpenOptions::new();
        opts.write(true).create_new(true);
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
        assert!(
            opts.open(&link).is_err(),
            "create_new must refuse an existing symlink"
        );
        assert_eq!(std::fs::read(&victim).expect("read"), b"ORIGINAL");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
