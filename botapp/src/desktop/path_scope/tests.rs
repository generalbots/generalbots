//! Offline invariants for the IPC path containment rule (#2497).

use super::{resolve_in_scopes, scoped_path, ScopeError, OUTSIDE_SCOPE};
use std::path::{Path, PathBuf};

/// A synthetic absolute root that is never created, so containment is checked
/// lexically without depending on the host's directory layout.
fn test_root() -> PathBuf {
    let base = match std::fs::canonicalize(std::env::temp_dir()) {
        Ok(path) => path,
        Err(_) => std::env::temp_dir(),
    };
    base.join("gb-path-scope-tests")
        .join(format!("root-{}", std::process::id()))
}

#[test]
fn test_accepts_absolute_child_of_root() {
    let root = test_root();
    let candidate = root.join("docs").join("readme.txt");
    assert_eq!(scoped_path(&root, &candidate.to_string_lossy()), Ok(candidate));
}

#[test]
fn test_accepts_relative_child_of_root() {
    let root = test_root();
    let expected = root.join("docs").join("readme.txt");
    assert_eq!(scoped_path(&root, "docs/readme.txt"), Ok(expected));
}

#[test]
fn test_accepts_root_itself() {
    let root = test_root();
    let text = root.to_string_lossy().to_string();
    assert_eq!(scoped_path(&root, &text), Ok(root));
}

#[test]
fn test_collapses_dot_components() {
    let root = test_root();
    let expected = root.join("docs").join("readme.txt");
    assert_eq!(scoped_path(&root, "./docs/./readme.txt"), Ok(expected));
}

#[test]
fn test_rejects_parent_traversal() {
    let root = test_root();
    let absolute = format!("{}/../secret.txt", root.display());
    assert_eq!(scoped_path(&root, &absolute), Err(ScopeError::ParentTraversal));
    assert_eq!(scoped_path(&root, "../secret.txt"), Err(ScopeError::ParentTraversal));
    assert_eq!(
        scoped_path(&root, "docs/../../secret.txt"),
        Err(ScopeError::ParentTraversal)
    );
}

#[test]
fn test_rejects_absolute_outside_root() {
    let root = test_root();
    let sibling = root.with_file_name("outside-scope").join("secret.txt");
    assert_eq!(
        scoped_path(&root, &sibling.to_string_lossy()),
        Err(ScopeError::OutsideRoot)
    );
    let prefix_confusion = PathBuf::from(format!("{}-suffix/secret.txt", root.display()));
    assert_eq!(
        scoped_path(&root, &prefix_confusion.to_string_lossy()),
        Err(ScopeError::OutsideRoot)
    );
}

#[test]
fn test_rejects_nul_and_empty() {
    let root = test_root();
    assert_eq!(scoped_path(&root, ""), Err(ScopeError::EmptyPath));
    assert_eq!(scoped_path(&root, "ok\0.txt"), Err(ScopeError::NulByte));
    assert_eq!(scoped_path(&root, "\0"), Err(ScopeError::NulByte));
}

#[test]
fn test_rejects_invalid_roots() {
    assert_eq!(
        scoped_path(Path::new("relative/root"), "file.txt"),
        Err(ScopeError::InvalidRoot)
    );
    assert_eq!(
        scoped_path(Path::new(""), "file.txt"),
        Err(ScopeError::EmptyPath)
    );
    assert_eq!(
        scoped_path(Path::new("/tmp/../etc"), "file.txt"),
        Err(ScopeError::InvalidRoot)
    );
}

#[test]
#[cfg(unix)]
fn test_rejects_symlink_escape() {
    use std::os::unix::fs::symlink;

    let base = match std::fs::canonicalize(std::env::temp_dir()) {
        Ok(path) => path,
        Err(_) => return,
    };
    let case = base.join(format!("gb-scope-symlink-{}", std::process::id()));
    let root = case.join("root");
    let outside = case.join("outside");
    let _ = std::fs::remove_dir_all(&case);
    if std::fs::create_dir_all(&root).is_err() || std::fs::create_dir_all(&outside).is_err() {
        return;
    }
    if std::fs::write(outside.join("secret.txt"), b"secret").is_err() {
        return;
    }
    if symlink(&outside, root.join("link")).is_err() {
        return;
    }
    let candidate = root.join("link").join("secret.txt");
    assert_eq!(
        scoped_path(&root, &candidate.to_string_lossy()),
        Err(ScopeError::OutsideRoot)
    );
    let _ = std::fs::remove_dir_all(&case);
}

#[test]
fn test_resolve_in_scopes_accepts_in_root() {
    let root = test_root();
    let roots = vec![root.clone()];
    assert_eq!(resolve_in_scopes("notes.txt", &roots), Ok(root.join("notes.txt")));
}

#[test]
fn test_resolve_in_scopes_fails_closed_without_roots() {
    assert_eq!(
        resolve_in_scopes("notes.txt", &[]),
        Err(OUTSIDE_SCOPE.to_string())
    );
}

#[test]
fn test_resolve_in_scopes_returns_generic_error() {
    let root = test_root();
    let roots = vec![root.clone()];
    let outside = root.with_file_name("outside-scope").join("secret.txt");
    let error = resolve_in_scopes(&outside.to_string_lossy(), &roots).expect_err("must refuse");
    assert_eq!(error, OUTSIDE_SCOPE);
    assert!(!error.contains(&root.display().to_string()));
}
