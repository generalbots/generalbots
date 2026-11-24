//! #2659 — containment for BASIC `WITH FILE` paths.
//!
//! A bot script must never turn an arbitrary host path into interpreter
//! source. Both constraints below are mandatory (fail closed): canonical
//! containment in this bot's work tree, and membership in the sandbox's
//! configured `allowed_paths`.

use super::*;
use botsecurity::path_guard::{PathGuard, PathGuardConfig, PathGuardError};
use std::path::{Path, PathBuf};

/// Typed refusal reason. Only the `Display` text reaches the caller via
/// `ExecutionResult::error`; a hostile host path is never echoed back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SandboxPathError {
    EmptyPath,
    InvalidComponent,
    OutsideBotWorkTree,
    OutsideAllowedPaths,
    SymlinkNotAllowed,
    NotFound,
}

impl std::fmt::Display for SandboxPathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::EmptyPath => "empty file path",
            Self::InvalidComponent => "invalid path component",
            Self::OutsideBotWorkTree => "path is outside the bot work tree",
            Self::OutsideAllowedPaths => "path is not in the sandbox allowed paths",
            Self::SymlinkNotAllowed => "symlinked path is not allowed",
            Self::NotFound => "file does not exist",
        };
        f.write_str(message)
    }
}

impl From<PathGuardError> for SandboxPathError {
    fn from(error: PathGuardError) -> Self {
        match error {
            PathGuardError::EmptyPath => Self::EmptyPath,
            PathGuardError::SymlinkNotAllowed => Self::SymlinkNotAllowed,
            PathGuardError::AbsolutePath
            | PathGuardError::OutsideAllowedRoot
            | PathGuardError::PathTraversal
            | PathGuardError::InvalidComponent
            | PathGuardError::HiddenFileNotAllowed
            | PathGuardError::InvalidExtension
            | PathGuardError::PathTooLong
            | PathGuardError::NullByte => Self::InvalidComponent,
        }
    }
}

/// Guard used for `WITH FILE`: script directories are sometimes hidden
/// (`.gbdialog`), sources are never host executables, and symlinks are always
/// refused so a link cannot point outside the work tree.
fn file_path_guard() -> PathGuard {
    PathGuard::new(
        PathGuardConfig::new()
            .allow_hidden(true)
            .allow_symlinks(false)
            .with_denied_extensions(Vec::new()),
    )
}

/// Canonicalize an absolute input and require it to stay inside `root`.
/// A symlinked final component is refused outright; an escaping symlinked
/// ancestor is caught by the canonical containment check.
fn canonicalize_within(path: &Path, root: &Path) -> Result<PathBuf, SandboxPathError> {
    if path.is_symlink() {
        return Err(SandboxPathError::SymlinkNotAllowed);
    }
    let canonical = path
        .canonicalize()
        .map_err(|_| SandboxPathError::NotFound)?;
    let canonical_root = root
        .canonicalize()
        .map_err(|_| SandboxPathError::OutsideBotWorkTree)?;
    if !canonical.starts_with(&canonical_root) {
        return Err(SandboxPathError::OutsideBotWorkTree);
    }
    Ok(canonical)
}

/// Resolve a BASIC-supplied `WITH FILE` path without ever leaving this bot's
/// work tree. Canonical containment in one of `bot_work_roots` also rejects
/// `..`, absolute escapes and symlinks; the allowlist check is a second,
/// independent gate.
pub(crate) fn validate_sandbox_file_path(
    file_path: &str,
    bot_work_roots: &[PathBuf],
    allowed_paths: &[String],
) -> Result<PathBuf, SandboxPathError> {
    if file_path.is_empty() {
        return Err(SandboxPathError::EmptyPath);
    }
    if file_path.contains('\0') {
        return Err(SandboxPathError::InvalidComponent);
    }
    let candidate = Path::new(file_path);

    let guard = file_path_guard();
    let mut resolved: Option<PathBuf> = None;
    let mut first_error: Option<SandboxPathError> = None;
    for root in bot_work_roots {
        let attempt = if candidate.is_absolute() {
            canonicalize_within(candidate, root)
        } else {
            guard
                .validate_relative_under(root, candidate)
                .map_err(SandboxPathError::from)
        };
        match attempt {
            Ok(path) => {
                resolved = Some(path);
                break;
            }
            Err(error) => {
                if first_error.is_none() {
                    first_error = Some(error);
                }
            }
        }
    }
    let resolved =
        resolved.ok_or_else(|| first_error.unwrap_or(SandboxPathError::OutsideBotWorkTree))?;
    // `validate_relative_under` returns the joined (possibly symlinked-parent)
    // path; canonicalize again so both gates compare canonical byte paths.
    let resolved = resolved
        .canonicalize()
        .map_err(|_| SandboxPathError::NotFound)?;

    let in_allowed_paths = allowed_paths.iter().any(|root| {
        Path::new(root)
            .canonicalize()
            .is_ok_and(|canonical| resolved.starts_with(canonical))
    });
    if !in_allowed_paths {
        warn!("Refusing sandbox file read: path is outside configured allowed paths");
        return Err(SandboxPathError::OutsideAllowedPaths);
    }
    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("gb2659-{tag}-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create fixture dir");
        dir
    }

    #[test]
    fn with_file_accepts_a_file_inside_work_tree_and_allowed_paths() {
        let root = unique_temp_dir("ok");
        let file = root.join("script.py");
        std::fs::write(&file, "print('ok')").expect("write fixture");
        let allowed = vec![root.to_string_lossy().to_string()];
        let resolved =
            validate_sandbox_file_path("script.py", &[root.clone()], &allowed).expect("allowed");
        assert_eq!(resolved, file.canonicalize().expect("canonical fixture"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn with_file_rejects_absolute_secret_paths() {
        let root = unique_temp_dir("abs");
        let allowed = vec![root.to_string_lossy().to_string()];
        assert_eq!(
            validate_sandbox_file_path("/etc/shadow", &[root.clone()], &allowed).unwrap_err(),
            SandboxPathError::OutsideBotWorkTree
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn with_file_rejects_parent_traversal() {
        let root = unique_temp_dir("trav");
        let nested = root.join("nested");
        std::fs::create_dir_all(&nested).expect("create nested dir");
        let allowed = vec![root.to_string_lossy().to_string()];
        assert_eq!(
            validate_sandbox_file_path("../secret.py", &[nested], &allowed).unwrap_err(),
            SandboxPathError::InvalidComponent
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[cfg(unix)]
    #[test]
    fn with_file_rejects_symlink_escape() {
        let root = unique_temp_dir("link");
        let outside = unique_temp_dir("outside");
        let secret = outside.join("secret.py");
        std::fs::write(&secret, "SECRET").expect("write secret");
        let link = root.join("escape");
        std::os::unix::fs::symlink(&outside, &link).expect("create symlink");
        let allowed = vec![
            root.to_string_lossy().to_string(),
            outside.to_string_lossy().to_string(),
        ];
        assert_eq!(
            validate_sandbox_file_path("escape/secret.py", &[root.clone()], &allowed).unwrap_err(),
            SandboxPathError::SymlinkNotAllowed
        );
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_dir_all(&outside);
    }

    #[test]
    fn with_file_rejects_path_outside_work_tree_even_when_allowed() {
        let root = unique_temp_dir("wt");
        let other = unique_temp_dir("other");
        let file = other.join("secret.py");
        std::fs::write(&file, "SECRET").expect("write secret");
        let allowed = vec![other.to_string_lossy().to_string()];
        assert_eq!(
            validate_sandbox_file_path(file.to_string_lossy().as_ref(), &[root.clone()], &allowed)
                .unwrap_err(),
            SandboxPathError::OutsideBotWorkTree
        );
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_dir_all(&other);
    }

    #[test]
    fn with_file_rejects_path_inside_work_tree_but_not_allowed() {
        let root = unique_temp_dir("al");
        let file = root.join("script.py");
        std::fs::write(&file, "print(1)").expect("write fixture");
        let unrelated = unique_temp_dir("unrelated");
        let allowed = vec![unrelated.to_string_lossy().to_string()];
        assert_eq!(
            validate_sandbox_file_path("script.py", &[root.clone()], &allowed).unwrap_err(),
            SandboxPathError::OutsideAllowedPaths
        );
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_dir_all(&unrelated);
    }
}
