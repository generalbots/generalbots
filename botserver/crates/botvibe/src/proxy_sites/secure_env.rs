//! #2654 — private staging for the EnvironmentFile that carries a project DB
//! URL to `incus file push`. The old code wrote `DATABASE_URL={url}` to
//! `/tmp/gb-vibe-{unit_name}.env` with plain `fs::write` (mode from umask,
//! typically 0644) under a name derived from the public slug: any local user
//! could read a project's DB password during the publish window, and a failed
//! publish left the file behind.
//!
//! Here the secret is staged in a 0700 dir with an unpredictable name and a
//! 0600 `O_EXCL` file, removed on every exit path by an RAII guard. The proxy
//! side is pinned to 0600 owned by the service user and verified with `stat`.

use std::path::{Path, PathBuf};

/// Owner-only modes: no group/other access bits. Used to verify the private
/// staging dir/file and by the unit tests as the mode predicate.
#[cfg(unix)]
fn is_private_mode(mode: u32) -> bool {
    mode & 0o077 == 0
}

/// Unpredictable per-process path for the private env staging directory. The
/// name is never derived from the public project slug, so a local user cannot
/// pre-create or watch for it in the world-writable temp directory.
fn secure_env_tmp_dir() -> PathBuf {
    std::env::temp_dir().join(format!(
        "gb-vibe-env-{}-{}",
        std::process::id(),
        uuid::Uuid::new_v4()
    ))
}

/// Create `dir` as a 0700 directory (`create` fails if it already exists).
/// On Unix the mode is set explicitly so a lax umask cannot widen it; the
/// unpredictable name closes the TOCTOU window.
#[cfg(unix)]
fn create_private_dir(dir: &Path) -> Result<(), String> {
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
    std::fs::DirBuilder::new()
        .mode(0o700)
        .create(dir)
        .map_err(|e| format!("create private env dir: {e}"))?;
    let mode = std::fs::metadata(dir)
        .map_err(|e| format!("stat private env dir: {e}"))?
        .permissions()
        .mode();
    if !is_private_mode(mode) {
        return Err("private env dir has unsafe permissions".to_string());
    }
    Ok(())
}

#[cfg(not(unix))]
fn create_private_dir(dir: &Path) -> Result<(), String> {
    std::fs::create_dir(dir).map_err(|e| format!("create private env dir: {e}"))
}

/// Create the env file `O_EXCL` with mode 0600 and write the secret.
/// `create_new` maps to `O_CREAT | O_EXCL` so an existing (attacker-planted)
/// file is never reused. The URL lives only in `content` — never in argv,
/// logs or errors.
#[cfg(unix)]
fn write_secret_file(file: &Path, content: &str) -> Result<(), String> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let mut handle = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(file)
        .map_err(|e| format!("create env file: {e}"))?;
    handle
        .write_all(content.as_bytes())
        .map_err(|e| format!("write env file: {e}"))
}

#[cfg(not(unix))]
fn write_secret_file(file: &Path, content: &str) -> Result<(), String> {
    use std::io::Write;
    let mut handle = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(file)
        .map_err(|e| format!("create env file: {e}"))?;
    handle
        .write_all(content.as_bytes())
        .map_err(|e| format!("write env file: {e}"))
}

fn remove_dir_quietly(dir: &Path) {
    if let Err(e) = std::fs::remove_dir_all(dir) {
        log::debug!("env temp cleanup failed: {e}");
    }
}

/// RAII owner of the short-lived EnvironmentFile staging dir. `Drop` removes
/// the whole private directory on every exit path (early `?`, push failure,
/// panic), so a DB URL never survives a publish attempt.
pub(crate) struct SecureEnvFile {
    dir: PathBuf,
    file: PathBuf,
}

impl SecureEnvFile {
    pub(crate) fn create(content: &str) -> Result<Self, String> {
        let dir = secure_env_tmp_dir();
        create_private_dir(&dir)?;
        let file = dir.join("database.env");
        if let Err(e) = write_secret_file(&file, content) {
            remove_dir_quietly(&dir);
            return Err(e);
        }
        Ok(Self { dir, file })
    }

    pub(crate) fn path(&self) -> &Path {
        &self.file
    }
}

impl Drop for SecureEnvFile {
    fn drop(&mut self) {
        remove_dir_quietly(&self.dir);
    }
}

/// Pin the pushed EnvironmentFile to 0600 owned by the service user. The
/// python unit declares no `User=`, so systemd runs it as root; `chown` plus
/// `chmod` make the proxy-side file match the private staging file, and a
/// `stat` check verifies it. No URL is involved in any of these argv lists.
pub(crate) fn harden_proxy_env_file(env_file: &str) -> Result<(), String> {
    crate::proxy_sites::must_run(
        "chown env file",
        &[
            "chown".to_string(),
            "root:root".to_string(),
            env_file.to_string(),
        ],
        15,
    )?;
    crate::proxy_sites::must_run(
        "chmod env file",
        &["chmod".to_string(), "600".to_string(), env_file.to_string()],
        15,
    )?;
    let out = crate::proxy_sites::proxy_exec(
        &[
            "stat".to_string(),
            "-c".to_string(),
            "%a %U".to_string(),
            env_file.to_string(),
        ],
        15,
    )?;
    if out.exit_code != Some(0) {
        return Err("env file stat failed".to_string());
    }
    let mode_and_owner = out.stdout.trim();
    if !mode_and_owner.starts_with("600 ") || !mode_and_owner.ends_with(" root") {
        return Err(format!("unexpected env file permissions: {mode_and_owner}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secure_env_tmp_dir_is_unique_per_call() {
        let a = secure_env_tmp_dir();
        let b = secure_env_tmp_dir();
        assert_ne!(a, b, "two calls must yield distinct unpredictable paths");
        let name = a.file_name().and_then(|n| n.to_str()).unwrap_or("");
        assert!(
            name.starts_with("gb-vibe-env-"),
            "unexpected staging name: {name}"
        );
        assert!(name.contains(&std::process::id().to_string()));
    }

    #[cfg(unix)]
    #[test]
    fn is_private_mode_rejects_group_and_other_access() {
        assert!(is_private_mode(0o700));
        assert!(is_private_mode(0o600));
        assert!(!is_private_mode(0o644));
        assert!(!is_private_mode(0o755));
        assert!(!is_private_mode(0o777));
    }

    #[cfg(unix)]
    #[test]
    fn secure_env_file_is_0600_in_0700_dir_and_removed_on_drop() {
        use std::os::unix::fs::PermissionsExt;
        let file_path;
        let dir_path;
        {
            let guard = SecureEnvFile::create("DATABASE_URL=postgres://u:p@h/db\n")
                .expect("create secure env file");
            file_path = guard.path().to_path_buf();
            dir_path = guard.dir.clone();
            let fmode = std::fs::metadata(&file_path)
                .expect("stat env file")
                .permissions()
                .mode();
            assert_eq!(fmode & 0o777, 0o600, "env file must be mode 0600");
            assert!(is_private_mode(fmode));
            let dmode = std::fs::metadata(&dir_path)
                .expect("stat env dir")
                .permissions()
                .mode();
            assert_eq!(dmode & 0o777, 0o700, "env dir must be mode 0700");
            assert!(is_private_mode(dmode));
        }
        assert!(!file_path.exists(), "env temp file must be removed on drop");
        assert!(!dir_path.exists(), "env temp dir must be removed on drop");
    }
}
