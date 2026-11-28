//! The `conf/` folders every stack has, created by the provisioner.
//!
//! The server writes its own state (JWT secret, model choice, certificates)
//! under `conf/system`. That folder used to appear as a side effect of the
//! directory service or of staging the model choice; an edition with neither
//! must still get it before the first boot.

use std::path::Path;

/// Folders relative to the stack root that the server expects to exist.
pub(crate) const STANDARD_CONF_DIRS: [&str; 1] = ["conf/system"];

/// Creates each missing standard folder (owner-only on Unix). Returns one
/// message per folder that could not be created; existing folders are left
/// untouched.
pub(crate) fn ensure_standard_conf_dirs(stack: &Path) -> Vec<String> {
    STANDARD_CONF_DIRS
        .iter()
        .filter_map(|relative| {
            let dir = stack.join(relative);
            if dir.is_dir() {
                return None;
            }
            create_private_dir(&dir)
                .err()
                .map(|e| format!("{}: {e}", dir.display()))
        })
        .collect()
}

fn create_private_dir(dir: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_standard_folders_are_created() {
        let root = tempfile::tempdir().expect("tempdir");
        assert!(ensure_standard_conf_dirs(root.path()).is_empty());
        for relative in STANDARD_CONF_DIRS {
            assert!(root.path().join(relative).is_dir(), "{relative} missing");
        }
    }

    #[test]
    fn existing_folders_and_their_files_are_kept() {
        let root = tempfile::tempdir().expect("tempdir");
        let system = root.path().join("conf/system");
        std::fs::create_dir_all(&system).expect("seed dir");
        std::fs::write(system.join("directory_config.json"), "{}").expect("seed file");

        assert!(ensure_standard_conf_dirs(root.path()).is_empty());
        assert!(system.join("directory_config.json").is_file());
    }
}
