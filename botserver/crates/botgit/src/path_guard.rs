//! Path-injection guard for the Source Control endpoints (#2862).
//!
//! Every filesystem path these handlers use is either (a) the server-defined
//! workspace root — derived from the running bot/tenant's stack configuration
//! and never from the request — or (b) a client-supplied *relative* project
//! id / repo path that is validated here before it reaches `git -C` or a
//! pathspec.
//!
//! The previous resolver accepted arbitrary absolute paths (`?repo=/etc`,
//! `?repo=/opt/gbo/...`) and even `/tmp`-prefixed strings verbatim. That let
//! any caller run git commands in any checkout the server process could read.
//! This module removes that escape hatch: absolute paths, `..`, separators in
//! a project id, NUL/control characters and symlink escapes are all rejected,
//! and a candidate must resolve *inside* the configured workspace root.

use std::path::{Path, PathBuf};

/// Server-controlled fallback checkout. It is a constant, not request data,
/// so it is never subject to the client validation below.
pub const DEFAULT_REPO: &str = "/tmp/git-repo";

const MAX_NAME_LEN: usize = 128;
const MAX_PATH_LEN: usize = 4096;

/// Root that hosts every vibe project workspace (mirrors
/// `botvibe::harness::workspace_root()` so the Source Control dialog reads
/// the SAME repo the Vibe agent edits). `VIBE_WORKSPACE_ROOT` wins when set;
/// otherwise the platform stack data directory is used. This is derived from
/// the bot/tenant runtime configuration, never from an HTTP parameter.
pub fn vibe_workspace_root() -> PathBuf {
    std::env::var("VIBE_WORKSPACE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(botlib::security::get_stack_path())
                .join("data")
                .join("vibe-workspaces")
        })
}

/// True for an absolute Windows path (`C:\…`, `\\server\share`). The slash
/// forms are covered by the explicit prefix checks in the validators; this
/// helper exists so the same rule holds on every platform the server runs on.
fn is_windows_absolute(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic()
}

/// Validate a client-supplied project id used as the `?repo=` argument.
///
/// A project id is a single path component: no separators, no traversal, no
/// NUL/control characters, bounded length. Absolute paths are rejected
/// outright — the caller may only select a project inside the server-defined
/// workspace root, never an arbitrary checkout.
pub fn validate_repo_name(repo: &str) -> Result<&str, String> {
    if repo.is_empty() {
        return Err("empty repo name".into());
    }
    if repo.contains('\0') {
        return Err("NUL byte in repo name".into());
    }
    if repo.len() > MAX_NAME_LEN {
        return Err("repo name too long".into());
    }
    if repo == "." || repo == ".." {
        return Err("invalid repo name".into());
    }
    if repo.starts_with('/') || repo.starts_with('\\') || is_windows_absolute(repo) {
        return Err("absolute repo path rejected".into());
    }
    if repo.contains('/') || repo.contains('\\') {
        return Err("separator in repo name".into());
    }
    // Content is otherwise unrestricted (project names may contain spaces or
    // unicode); separators, traversal and control characters are the dangerous
    // parts and were rejected above.
    if repo.chars().any(char::is_control) {
        return Err("control character in repo name".into());
    }
    Ok(repo)
}

/// Validate a client-supplied path inside a git work tree (diff pathspec,
/// tree directory, commit pathspec). `allow_empty` permits the repo-root
/// pathspec used by the tree view; every other caller must supply a name.
///
/// Rejects NUL/control characters, absolute paths (POSIX and Windows) and any
/// `..` segment — backslash is treated as a separator too, so a Windows-style
/// traversal cannot slip through on a POSIX host. Character *content* is
/// otherwise unrestricted so legitimate file names (spaces, unicode, globs)
/// keep working.
pub fn validate_git_path(raw: &str, allow_empty: bool) -> Result<&str, String> {
    let path = raw.trim();
    if path.is_empty() {
        if allow_empty {
            return Ok(path);
        }
        return Err("empty path".into());
    }
    if path.contains('\0') {
        return Err("NUL byte in path".into());
    }
    if path.len() > MAX_PATH_LEN {
        return Err("path too long".into());
    }
    if path.starts_with('/') || path.starts_with('\\') || is_windows_absolute(path) {
        return Err("absolute path rejected".into());
    }
    if path.chars().any(char::is_control) {
        return Err("control character in path".into());
    }
    for segment in path.split(['/', '\\']) {
        if segment == ".." {
            return Err("path traversal rejected".into());
        }
    }
    // A trailing slash is meaningless in a git pathspec and would break the
    // tree view's `strip_prefix("dir/")` reconciliation, so normalize it away.
    Ok(path.trim_end_matches('/'))
}

/// Validate a client-supplied git ref (branch) name before it is passed to
/// `git checkout`. Ref names are not filesystem paths, but they are still
/// request data: reject NUL/control characters and a leading dash so the value
/// can never be reinterpreted as a git option.
pub fn validate_ref_name(raw: &str) -> Result<&str, String> {
    let name = raw.trim();
    if name.is_empty() || name.starts_with('-') {
        return Err("invalid ref name".into());
    }
    if name.contains('\0') || name.chars().any(char::is_control) {
        return Err("control character in ref name".into());
    }
    if name.len() > MAX_NAME_LEN {
        return Err("ref name too long".into());
    }
    Ok(name)
}

/// Canonicalize `path`, falling back to its deepest existing ancestor for a
/// target that does not exist yet (new/untracked files). The non-existing tail
/// is appended afterwards so the result is an absolute path that can be
/// compared against the canonical root. Resolving the existing prefix through
/// the filesystem is what defeats symlink-based escapes.
fn canonicalize_or_ancestor(path: &Path) -> Result<PathBuf, String> {
    let mut cursor = path;
    loop {
        if let Ok(canonical) = cursor.canonicalize() {
            let tail = path
                .strip_prefix(cursor)
                .map_err(|e| format!("strip_prefix: {e}"))?;
            return Ok(canonical.join(tail));
        }
        cursor = cursor
            .parent()
            .ok_or_else(|| format!("cannot resolve {}", path.display()))?;
    }
}

/// Containment barrier: canonicalize `target` and prove it stays under `base`.
/// Symlinks and `..` are resolved by the filesystem itself, so the check cannot
/// be bypassed by crafted names.
pub fn ensure_contained(target: &Path, base: &Path) -> Result<PathBuf, String> {
    let canonical_base = canonicalize_or_ancestor(base)?;
    let resolved = canonicalize_or_ancestor(target)?;
    if !resolved.starts_with(&canonical_base) {
        return Err(format!(
            "path {} escapes root {}",
            resolved.display(),
            canonical_base.display()
        ));
    }
    Ok(resolved)
}

/// Validate a repo-relative path and prove it stays inside the resolved repo.
/// Used for diff pathspecs and commit pathspecs; a not-yet-created file is fine
/// because the containment check follows its existing ancestor chain.
pub fn ensure_git_path_contained(repo: &str, relative: &str) -> Result<PathBuf, String> {
    let root = Path::new(repo);
    let target = root.join(relative);
    ensure_contained(&target, root)
}

/// Resolve the `?repo=` argument to a single workspace checkout.
///
/// Client input is reduced to a validated project id, joined to the
/// server-controlled workspace root, canonicalized and containment-checked.
/// Any failure is fail-closed: the request is replayed against the fixed
/// server-owned `DEFAULT_REPO` instead of the supplied path.
pub fn resolve_repo(query_repo: Option<&str>) -> String {
    let Some(raw) = query_repo.map(str::trim).filter(|r| !r.is_empty()) else {
        return DEFAULT_REPO.to_string();
    };
    let name = match validate_repo_name(raw) {
        Ok(name) => name,
        Err(error) => {
            log::warn!("rejected repo path {raw:?}: {error}");
            return DEFAULT_REPO.to_string();
        }
    };
    let root = vibe_workspace_root();
    let candidate = match ensure_contained(&root.join(name), &root) {
        Ok(path) => path,
        Err(error) => {
            log::warn!("rejected repo {name:?}: {error}");
            return DEFAULT_REPO.to_string();
        }
    };
    if !candidate.join(".git").exists() {
        return DEFAULT_REPO.to_string();
    }
    candidate.to_string_lossy().into_owned()
}

#[cfg(test)]
#[path = "resolve_repo_tests.rs"]
mod resolve_repo_tests;
