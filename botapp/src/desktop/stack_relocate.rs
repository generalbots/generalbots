//! Moves a stack folder and repoints the few files that hold its absolute
//! path.
//!
//! Most services get their folders as start arguments, but some files are
//! written once at install with the stack's absolute path: the vault's
//! `config.hcl` (its raft `path`), the vector database's `config.yaml`
//! (`storage_path`), the stack's `.env` (`VAULT_CACERT`) and the media
//! models' Python environment (`pyvenv.cfg`). A vault pointed at the old path
//! would start on empty storage, so the move is all or nothing: every new
//! content is prepared before the folder is renamed, and any failure after the
//! rename restores the files and renames the folder back.

use std::io;
use std::path::{Path, PathBuf};

/// Files under the stack that carry its absolute path, relative to the stack.
pub const PATH_FILES: &[&str] = &[
    "conf/vault/config.hcl",
    "conf/vector_db/config.yaml",
    ".env",
    "bin/botmodels/.venv/pyvenv.cfg",
    // Not read at start on Windows (pg_ctl runs with `-D pgdata`; Zitadel's
    // steps run once), repointed so no stale path is left behind.
    "conf/postgresql.conf",
    "conf/directory/zitadel-init-steps.yaml",
];

/// The two spellings of a Windows path the installers write: as is, and with
/// forward slashes (the vault's HCL).
fn spellings(path: &Path) -> [String; 2] {
    let native = path.display().to_string();
    let forward = native.replace('\\', "/");
    [native, forward]
}

/// `content` with every spelling of `old` replaced by the same spelling of
/// `new`; `None` when it does not mention `old`.
pub fn repoint(content: &str, old: &Path, new: &Path) -> Option<String> {
    let [old_native, old_forward] = spellings(old);
    let [new_native, new_forward] = spellings(new);
    if !content.contains(&old_native) && !content.contains(&old_forward) {
        return None;
    }
    let first = content.replace(&old_native, &new_native);
    Some(if old_forward == old_native {
        first
    } else {
        first.replace(&old_forward, &new_forward)
    })
}

/// Renames `old` to `new` and repoints the path files. On any failure nothing
/// has changed on disk when it returns.
pub fn relocate(old: &Path, new: &Path) -> io::Result<usize> {
    let mut planned: Vec<(PathBuf, String, String)> = Vec::new();
    for relative in PATH_FILES {
        let current = old.join(relative);
        if !current.is_file() {
            continue;
        }
        let original = std::fs::read_to_string(&current)?;
        if let Some(updated) = repoint(&original, old, new) {
            planned.push((new.join(relative), original, updated));
        }
    }
    std::fs::rename(old, new)?;
    let mut written: Vec<&(PathBuf, String, String)> = Vec::new();
    for entry in &planned {
        if let Err(e) = write_atomically(&entry.0, &entry.2) {
            for (path, original, _) in written {
                let _ = write_atomically(path, original);
            }
            return match std::fs::rename(new, old) {
                Ok(()) => Err(e),
                Err(back) => Err(io::Error::other(format!(
                    "{e}; and the folder could not be moved back from {}: {back}",
                    new.display()
                ))),
            };
        }
        written.push(entry);
    }
    Ok(planned.len())
}

fn write_atomically(path: &Path, content: &str) -> io::Result<()> {
    let mut temporary = path.as_os_str().to_owned();
    temporary.push(".relocating");
    let temporary = PathBuf::from(temporary);
    std::fs::write(&temporary, content)?;
    std::fs::rename(&temporary, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_spellings_are_repointed() {
        let old = PathBuf::from(r"C:\Users\p\AppData\Local\generalbots\stack");
        let new = PathBuf::from(r"C:\Users\p\AppData\Local\GeneralBotsLite\stack");
        if cfg!(windows) {
            let hcl = "storage \"raft\" {\n  path = \"C:/Users/p/AppData/Local/generalbots/stack/data/vault\"\n}\n";
            assert_eq!(
                repoint(hcl, &old, &new).as_deref(),
                Some("storage \"raft\" {\n  path = \"C:/Users/p/AppData/Local/GeneralBotsLite/stack/data/vault\"\n}\n")
            );
        }
        let env = "VAULT_CACERT=C:\\Users\\p\\AppData\\Local\\generalbots\\stack\\conf\\ca.crt\n";
        assert_eq!(
            repoint(env, &old, &new).as_deref(),
            Some("VAULT_CACERT=C:\\Users\\p\\AppData\\Local\\GeneralBotsLite\\stack\\conf\\ca.crt\n")
        );
        assert_eq!(repoint("VAULT_ADDR=http://127.0.0.1:8200\n", &old, &new), None);
    }

    fn temp_root(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("gb-relocate-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        root
    }

    #[test]
    fn the_folder_moves_and_its_path_files_follow() {
        let root = temp_root("move");
        let old = root.join("generalbots").join("stack");
        let new = root.join("GeneralBotsLite").join("stack");
        std::fs::create_dir_all(old.join("conf/vault")).expect("conf");
        std::fs::create_dir_all(old.join("data/vault/raft")).expect("raft");
        std::fs::create_dir_all(new.parent().expect("parent")).expect("new parent");
        let hcl = format!("path = \"{}\"\n", old.join("data/vault").display());
        std::fs::write(old.join("conf/vault/config.hcl"), &hcl).expect("hcl");
        std::fs::write(old.join(".env"), "VAULT_ADDR=http://127.0.0.1:8200\n").expect("env");

        assert_eq!(relocate(&old, &new).expect("relocate"), 1);
        assert!(!old.exists());
        assert!(new.join("data/vault/raft").is_dir());
        let moved = std::fs::read_to_string(new.join("conf/vault/config.hcl")).expect("read");
        assert_eq!(moved, format!("path = \"{}\"\n", new.join("data/vault").display()));
        let env = std::fs::read_to_string(new.join(".env")).expect("env");
        assert_eq!(env, "VAULT_ADDR=http://127.0.0.1:8200\n");
        assert!(!new.join("conf/vault/config.hcl.relocating").exists());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_failed_rename_changes_nothing() {
        let root = temp_root("fail");
        let old = root.join("generalbots").join("stack");
        std::fs::create_dir_all(old.join("conf/vault")).expect("conf");
        let hcl = format!("path = \"{}\"\n", old.join("data").display());
        std::fs::write(old.join("conf/vault/config.hcl"), &hcl).expect("hcl");
        // The new parent folder does not exist, so the rename fails.
        let new = root.join("missing").join("deeper").join("stack");
        assert!(relocate(&old, &new).is_err());
        assert_eq!(std::fs::read_to_string(old.join("conf/vault/config.hcl")).expect("read"), hcl);
        let _ = std::fs::remove_dir_all(&root);
    }
}
