//! Pure path-containment primitive for the desktop IPC (#2497).
//!
//! `fs_scope` decides *which* roots are configured; this module decides
//! whether one webview-supplied candidate stays inside one of them. It is
//! deliberately free of Tauri state so the containment rule can be tested
//! offline, including symlink escapes and paths that do not exist yet.
//!
//! The rule is fail-closed: both ends must be absolute and free of `..` and
//! NUL bytes, the candidate is joined to the root when relative, then the
//! deepest existing ancestor is canonicalised and the missing tail appended.
//! Only a result that is a component-wise child of the canonical root is
//! returned. Errors carry no host path, so a script that triggers one learns
//! nothing about the filesystem behind the webview.

use std::ffi::OsStr;
use std::fmt;
use std::path::{Component, Path, PathBuf};

/// Why a candidate was refused. Never carries a path or an OS error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeError {
    EmptyPath,
    NulByte,
    ParentTraversal,
    InvalidRoot,
    Unresolved,
    OutsideRoot,
}

impl fmt::Display for ScopeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let reason = match self {
            Self::EmptyPath => "empty path",
            Self::NulByte => "NUL byte in path",
            Self::ParentTraversal => "parent traversal in path",
            Self::InvalidRoot => "invalid scope root",
            Self::Unresolved => "path cannot be resolved",
            Self::OutsideRoot => "path outside scope",
        };
        f.write_str(reason)
    }
}

/// The only scope error text that crosses the IPC boundary.
pub const OUTSIDE_SCOPE: &str = "This location is outside the folders the desktop may access";

fn has_parent(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, Component::ParentDir))
}

fn has_nul(path: &Path) -> bool {
    path.as_os_str().to_string_lossy().contains('\0')
}

/// Collapse `.` and separators without touching the filesystem; `..` is never
/// allowed, not even when it would stay inside the root.
fn normalise_lexical(path: &Path) -> Result<PathBuf, ScopeError> {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => return Err(ScopeError::ParentTraversal),
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                result.push(component.as_os_str());
            }
        }
    }
    if result.as_os_str().is_empty() {
        Err(ScopeError::EmptyPath)
    } else {
        Ok(result)
    }
}

/// Canonicalise the deepest existing ancestor and append the missing tail, so
/// a symlink in a parent cannot be skipped by checking a path that does not
/// exist yet.
fn resolve_existing(path: &Path) -> Result<PathBuf, ScopeError> {
    if let Ok(canonical) = std::fs::canonicalize(path) {
        return Ok(canonical);
    }
    let mut ancestor = path;
    let mut tail: Vec<&OsStr> = Vec::new();
    loop {
        let parent = ancestor.parent().ok_or(ScopeError::Unresolved)?;
        if let Some(name) = ancestor.file_name() {
            tail.push(name);
        }
        ancestor = parent;
        if let Ok(base) = std::fs::canonicalize(ancestor) {
            let mut result = base;
            for name in tail.iter().rev() {
                result.push(name);
            }
            return Ok(result);
        }
    }
}

/// Roots from `fs_scope` are canonical already. The lexical fallback exists
/// for tests with synthetic roots and for a root removed between discovery
/// and use; in both cases the root must still be absolute and clean.
fn normalise_root(root: &Path) -> Result<PathBuf, ScopeError> {
    if root.as_os_str().is_empty() {
        return Err(ScopeError::EmptyPath);
    }
    if has_nul(root) {
        return Err(ScopeError::NulByte);
    }
    if !root.is_absolute() || has_parent(root) {
        return Err(ScopeError::InvalidRoot);
    }
    match std::fs::canonicalize(root) {
        Ok(canonical) => Ok(canonical),
        Err(_) => normalise_lexical(root),
    }
}

/// Resolve `candidate` inside exactly one `root`.
///
/// Absolute candidates must already point inside the root; relative ones are
/// joined to it. `.` components are collapsed, `..`, NUL and empty input are
/// refused, and symlinks are resolved through the deepest existing ancestor.
pub fn scoped_path(root: &Path, candidate: &str) -> Result<PathBuf, ScopeError> {
    if candidate.is_empty() {
        return Err(ScopeError::EmptyPath);
    }
    if candidate.contains('\0') {
        return Err(ScopeError::NulByte);
    }
    let base = normalise_root(root)?;
    let raw = Path::new(candidate);
    if has_parent(raw) {
        return Err(ScopeError::ParentTraversal);
    }
    let joined = if raw.is_absolute() {
        raw.to_path_buf()
    } else {
        base.join(raw)
    };
    let lexical = normalise_lexical(&joined)?;
    let resolved = resolve_existing(&lexical)?;
    if !resolved.starts_with(&base) {
        return Err(ScopeError::OutsideRoot);
    }
    Ok(resolved)
}

/// Try every configured root, keeping the detailed reason in the process log
/// and returning one fixed message to the webview. No roots means no access.
pub fn resolve_in_scopes(raw: &str, roots: &[PathBuf]) -> Result<PathBuf, String> {
    if roots.is_empty() {
        log::warn!("IPC scope: no roots configured");
        return Err(OUTSIDE_SCOPE.to_string());
    }
    for root in roots {
        match scoped_path(root, raw) {
            Ok(path) => {
                if let Err(reason) = crate::desktop::fs_scope::check_not_protected(&path) {
                    log::warn!("IPC scope: protected path refused: {reason}");
                    return Err(OUTSIDE_SCOPE.to_string());
                }
                return Ok(path);
            }
            Err(reason) => log::debug!("IPC scope: root refused candidate: {reason}"),
        }
    }
    log::warn!("IPC scope: candidate refused by all roots");
    Err(OUTSIDE_SCOPE.to_string())
}

#[cfg(test)]
#[path = "path_scope/tests.rs"]
mod tests;
