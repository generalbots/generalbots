//! #2863 — symlink-escape guard for materialized git-owned bot trees.
//!
//! Git stores symlinks as first-class objects on Linux. `.gbdialog/` and
//! `.gbot/` are materialized from a git checkout into `work/`, so a repo can
//! otherwise ship `foo.bas -> /etc/passwd` (absolute escape) or
//! `foo.bas -> ../../../secret` (relative escape) and have the monitor copy
//! host files into the bot's script directory, then queue them for
//! compilation. A symlinked directory makes the recursive copy walk an
//! external tree.
//!
//! Every materialization loop calls [`classify_entry`] instead of
//! `is_dir()` / `is_file()` / `fs::copy`, and [`destination_is_safe`] before
//! writing, so no link is ever followed through the work-dir boundary.

use std::path::Path;

/// What a materialization loop may do with one directory entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EntryKind {
    /// A real directory — safe to recurse into.
    Directory,
    /// A real file, or a symlink resolving to a file inside `root` — safe to
    /// copy the bytes of.
    File,
    /// An absolute, escaping, dangling or directory symlink, or a special
    /// file. The caller must skip it: never read from or write through it.
    Refuse,
}

/// Classify `path` (an entry of the tree rooted at `root`) without ever
/// following a symlink implicitly. A symlink is accepted only when its
/// canonical target is a regular file that still lives inside `root`.
pub(crate) fn classify_entry(root: &Path, path: &Path) -> EntryKind {
    // A symlinked tree root would make every "in-tree" check below resolve
    // against the external tree instead of the checkout. Fail closed.
    match std::fs::symlink_metadata(root) {
        Ok(meta) if meta.file_type().is_symlink() => return EntryKind::Refuse,
        Err(_) => return EntryKind::Refuse,
        Ok(_) => {}
    }
    let meta = match std::fs::symlink_metadata(path) {
        Ok(meta) => meta,
        Err(_) => return EntryKind::Refuse,
    };
    if !meta.file_type().is_symlink() {
        if meta.is_dir() {
            return EntryKind::Directory;
        }
        if meta.is_file() {
            return EntryKind::File;
        }
        return EntryKind::Refuse;
    }
    let target = match std::fs::read_link(path) {
        Ok(target) => target,
        Err(_) => return EntryKind::Refuse,
    };
    // Absolute targets are refused even if they happen to point inside the
    // tree: a committed absolute path is not portable and is never trusted.
    if target.is_absolute() {
        return EntryKind::Refuse;
    }
    let parent = match path.parent() {
        Some(parent) => parent,
        None => return EntryKind::Refuse,
    };
    let resolved = match std::fs::canonicalize(parent.join(target)) {
        Ok(resolved) => resolved,
        Err(_) => return EntryKind::Refuse, // dangling link
    };
    let canonical_root = match std::fs::canonicalize(root) {
        Ok(canonical_root) => canonical_root,
        Err(_) => return EntryKind::Refuse,
    };
    if !resolved.starts_with(&canonical_root) {
        return EntryKind::Refuse;
    }
    // Directory symlinks are refused too: following one can cycle forever
    // (a link to an ancestor) and materialization only needs file content.
    match std::fs::metadata(&resolved) {
        Ok(meta) if meta.is_file() => EntryKind::File,
        _ => EntryKind::Refuse,
    }
}

/// True when `to` may be written without crossing the `root` boundary: no
/// symlink occupies `to` and its parent chain resolves inside `root`.
pub(crate) fn destination_is_safe(root: &Path, to: &Path) -> bool {
    match std::fs::symlink_metadata(to) {
        Ok(meta) if meta.file_type().is_symlink() => return false,
        Ok(_) => {}
        Err(_) => {} // not present yet — the parent check below still applies
    }
    let parent = match to.parent() {
        Some(parent) => parent,
        None => return false,
    };
    // The top-level destination directory is created by create_dir_all; its
    // own parent may legitimately live outside the work-dir root.
    if parent == root {
        return true;
    }
    let canonical_root = match std::fs::canonicalize(root) {
        Ok(canonical_root) => canonical_root,
        Err(_) => return false,
    };
    let canonical_parent = match std::fs::canonicalize(parent) {
        Ok(canonical_parent) => canonical_parent,
        Err(_) => return false,
    };
    canonical_parent.starts_with(&canonical_root)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("gbpath-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    #[cfg(unix)]
    fn absolute_symlink_is_refused() {
        let tmp = scratch("abs");
        let root = tmp.join("checkout");
        std::fs::create_dir_all(&root).unwrap();
        let outside = tmp.join("outside.bas");
        std::fs::write(&outside, "SECRET").unwrap();
        std::os::unix::fs::symlink(&outside, root.join("leak.bas")).unwrap();
        assert_eq!(classify_entry(&root, &root.join("leak.bas")), EntryKind::Refuse);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    #[cfg(unix)]
    fn relative_escape_symlink_is_refused() {
        let tmp = scratch("rel");
        let root = tmp.join("box/checkout");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(tmp.join("box/outside.bas"), "SECRET").unwrap();
        // From checkout/, `../outside.bas` lands in box/ — outside root.
        std::os::unix::fs::symlink("../outside.bas", root.join("leak.bas")).unwrap();
        assert_eq!(classify_entry(&root, &root.join("leak.bas")), EntryKind::Refuse);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    #[cfg(unix)]
    fn in_tree_file_symlink_is_allowed() {
        let tmp = scratch("safe");
        let root = tmp.join("checkout");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("start.bas"), "TALK \"hi\"").unwrap();
        std::os::unix::fs::symlink("start.bas", root.join("alias.bas")).unwrap();
        assert_eq!(classify_entry(&root, &root.join("alias.bas")), EntryKind::File);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    #[cfg(unix)]
    fn dangling_and_directory_symlinks_are_refused() {
        let tmp = scratch("dangling");
        let root = tmp.join("checkout");
        std::fs::create_dir_all(root.join("sub")).unwrap();
        std::os::unix::fs::symlink("missing.bas", root.join("dangling.bas")).unwrap();
        std::os::unix::fs::symlink("sub", root.join("dirlink")).unwrap();
        assert_eq!(
            classify_entry(&root, &root.join("dangling.bas")),
            EntryKind::Refuse
        );
        assert_eq!(classify_entry(&root, &root.join("dirlink")), EntryKind::Refuse);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    #[cfg(unix)]
    fn symlinked_tree_root_is_refused() {
        let tmp = scratch("rootlink");
        let real = tmp.join("real");
        std::fs::create_dir_all(&real).unwrap();
        std::fs::write(real.join("start.bas"), "TALK").unwrap();
        let link = tmp.join("link");
        std::os::unix::fs::symlink(&real, &link).unwrap();
        assert_eq!(
            classify_entry(&link, &link.join("start.bas")),
            EntryKind::Refuse
        );
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    #[cfg(unix)]
    fn planted_destination_symlink_is_refused() {
        let tmp = scratch("dest");
        let work = tmp.join("work");
        std::fs::create_dir_all(&work).unwrap();
        let outside = tmp.join("outside.bas");
        std::fs::write(&outside, "ORIGINAL").unwrap();
        std::os::unix::fs::symlink(&outside, work.join("leak.bas")).unwrap();
        assert!(!destination_is_safe(&work, &work.join("leak.bas")));
        assert!(destination_is_safe(&work, &work.join("fresh.bas")));
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
