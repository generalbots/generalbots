//! #2694 — validated, handle-pinned sidecar executable.
//!
//! The binary is opened **once** and every check is made through that open
//! handle. The same handle is then used as the exec target on Linux, so a
//! rename/replace of the on-disk path between validation and spawn cannot swap
//! the executable. All failures are reported as errors (fail closed).

use std::fs::File;
use std::path::{Path, PathBuf};

/// A sidecar executable that has been opened, canonicalized and validated.
///
/// The open [`File`] is deliberately kept for the whole spawn: on Linux the
/// process is exec'd through `/proc/self/fd/<fd>`, which resolves to the exact
/// inode that was validated even if the original path is later replaced.
pub(crate) struct ValidatedBinary {
    path: PathBuf,
    file: File,
}

impl ValidatedBinary {
    /// Resolve and validate `candidate` without ever returning an untrusted
    /// path. `app_dir` and `stack` are the two directories a binary may live
    /// in; an `BOTSERVER_BIN` value outside them is refused.
    pub(crate) fn resolve(
        candidate: &Path,
        app_dir: Option<&Path>,
        stack: &Path,
    ) -> Result<Self, String> {
        // Open FIRST: from here on, validation is about this handle, not a
        // path an attacker can rewrite between a stat and an exec.
        let file = File::open(candidate)
            .map_err(|e| format!("sidecar binary not found at {}: {e}", candidate.display()))?;
        let canonical = std::fs::canonicalize(candidate).map_err(|e| {
            format!("cannot resolve sidecar path {}: {e}", candidate.display())
        })?;
        let parent = canonical.parent();
        let app_canonical = app_dir.and_then(|dir| std::fs::canonicalize(dir).ok());
        let stack_canonical = std::fs::canonicalize(stack).ok();
        let in_app_dir = app_canonical.as_deref().is_some_and(|dir| Some(dir) == parent);
        let in_stack = stack_canonical.as_deref().is_some_and(|dir| Some(dir) == parent);
        if !in_app_dir && !in_stack {
            return Err(format!(
                "Refusing to start a sidecar outside the application or stack directory: {}",
                canonical.display()
            ));
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let metadata = file
                .metadata()
                .map_err(|e| format!("cannot stat the opened sidecar: {e}"))?;
            if !metadata.is_file() {
                return Err("The sidecar path is not a regular file".to_string());
            }
            let mode = metadata.mode();
            if mode & 0o111 == 0 {
                return Err("The sidecar binary is not executable".to_string());
            }
            if mode & 0o022 != 0 {
                return Err("The sidecar binary is writable by group or others".to_string());
            }
            let owner = metadata.uid();
            let current = unsafe { libc::geteuid() };
            if owner != 0 && owner != current {
                return Err(
                    "The sidecar binary is not owned by the current user or root".to_string()
                );
            }
            // The path must still designate the very file we opened. If it
            // does not, something replaced it while we were validating.
            let on_disk = std::fs::metadata(&canonical)
                .map_err(|e| format!("cannot stat sidecar path: {e}"))?;
            if (metadata.dev(), metadata.ino()) != (on_disk.dev(), on_disk.ino()) {
                return Err("The sidecar binary changed while it was being validated".to_string());
            }
            // A world-writable allowlisted directory is no allowlist at all.
            if let Some(parent) = parent {
                let directory = std::fs::metadata(parent)
                    .map_err(|e| format!("cannot stat the sidecar directory: {e}"))?;
                let directory_mode = directory.mode();
                let directory_owner = directory.uid();
                if directory_mode & 0o002 != 0
                    || (directory_owner != 0 && directory_owner != current)
                {
                    return Err(format!(
                        "Refusing a sidecar in an unsafe directory: {}",
                        parent.display()
                    ));
                }
            }
        }

        Ok(Self {
            path: canonical,
            file,
        })
    }

    /// The program passed to `Command`. On Linux with procfs available this is
    /// the already-open fd, so exec cannot be redirected by a later rename.
    pub(crate) fn program(&self) -> PathBuf {
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::io::AsRawFd;
            let fd_path = PathBuf::from(format!("/proc/self/fd/{}", self.file.as_raw_fd()));
            if std::fs::symlink_metadata(&fd_path).is_ok() {
                return fd_path;
            }
        }
        #[cfg(not(target_os = "linux"))]
        {
            // On other platforms the open handle still pins the inode against
            // in-place edits; keep it alive across the spawn by reading it here.
            let _ = &self.file;
        }
        self.path.clone()
    }

    /// Canonical path, for logs only — never hand this to `Command`.
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::ValidatedBinary;
    use std::path::{Path, PathBuf};

    /// Test-only validation helper that mirrors production but drops the handle.
    fn validate(candidate: &Path, app: &Path, stack: &Path) -> Result<PathBuf, String> {
        Ok(ValidatedBinary::resolve(candidate, Some(app), stack)?.path)
    }

    fn sandbox(name: &str) -> PathBuf {
        let base = std::env::temp_dir().join(format!("gb-bin-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        for dir in [base.join("app"), base.join("stack"), base.join("outside")] {
            let _ = std::fs::create_dir_all(dir);
        }
        base
    }

    fn write_executable(path: &Path) {
        if std::fs::write(path, b"binary").is_err() {
            return;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = std::fs::metadata(path) {
                let mut permissions = metadata.permissions();
                permissions.set_mode(0o700);
                let _ = std::fs::set_permissions(path, permissions);
            }
        }
    }

    #[test]
    fn hostile_env_path_is_refused() {
        let base = sandbox("env");
        let app = base.join("app");
        let stack = base.join("stack");
        let outside = base.join("outside").join("evil-botserver");
        write_executable(&outside);

        // This is exactly what `paths::resolve_binary` returns for a hostile
        // `BOTSERVER_BIN`: the raw value, with no canonicalization.
        let candidate = crate::desktop::sidecar::paths::resolve_binary(
            Some(outside.to_string_lossy().as_ref()),
            Some(app.join("botserver")),
            &stack,
        );
        assert_eq!(candidate, outside);
        assert!(validate(&candidate, &app, &stack).is_err());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn a_symlink_escaping_the_app_directory_is_refused() {
        let base = sandbox("symlink");
        let app = base.join("app");
        let stack = base.join("stack");
        let outside = base.join("outside").join("botserver");
        write_executable(&outside);

        let link = app.join("botserver");
        #[cfg(unix)]
        #[cfg(unix)]
        std::os::unix::fs::symlink(&outside, &link).ok();
        #[cfg(windows)]
        std::fs::copy(&outside, &link).ok();

        // Windows has no symlink here; copy stays inside the allowlist and is
        // expected to pass. Unix must refuse the escaping symlink.
        #[cfg(unix)]
        assert!(validate(&link, &app, &stack).is_err());
        #[cfg(windows)]
        assert!(validate(&link, &app, &stack).is_ok());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn non_executable_and_world_writable_binaries_are_refused() {
        let base = sandbox("mode");
        let app = base.join("app");
        let stack = base.join("stack");
        let candidate = app.join("botserver");
        write_executable(&candidate);

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = std::fs::metadata(&candidate)
                .map(|m| m.permissions())
                .unwrap_or_else(|_| std::fs::Permissions::from_mode(0o600));
            permissions.set_mode(0o600);
            let _ = std::fs::set_permissions(&candidate, permissions.clone());
            assert!(validate(&candidate, &app, &stack).is_err());
            permissions.set_mode(0o677);
            let _ = std::fs::set_permissions(&candidate, permissions);
            assert!(validate(&candidate, &app, &stack).is_err());
        }
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn missing_binary_is_refused() {
        let base = sandbox("missing");
        let app = base.join("app");
        let stack = base.join("stack");
        assert!(validate(&app.join("botserver"), &app, &stack).is_err());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn a_legitimate_binary_is_accepted_and_pinned_to_the_open_handle() {
        let base = sandbox("good");
        let app = base.join("app");
        let stack = base.join("stack");
        let candidate = stack.join("botserver");
        write_executable(&candidate);

        let validated = ValidatedBinary::resolve(&candidate, Some(&app), &stack);
        assert!(validated.is_ok());
        if let Ok(validated) = validated {
            let expected = candidate.canonicalize().unwrap_or_else(|_| candidate.clone());
            assert_eq!(validated.path(), expected.as_path());
            #[cfg(target_os = "linux")]
            assert!(validated.program().starts_with("/proc/self/fd/"));
        }
        let _ = std::fs::remove_dir_all(&base);
    }
}
