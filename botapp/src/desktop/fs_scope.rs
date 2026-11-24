//! Filesystem scopes for the desktop shell IPC (#2497).
//!
//! The Tauri commands used to accept **any** absolute path from the webview:
//! `delete_path('/var/lib/gbdesk')` wiped the whole appliance, and a single
//! injected script (chat message, marketplace app, document preview) could
//! read or destroy the database, the Drive objects and the secrets. Tauri's
//! capability file is not the boundary — it trusts every `http://localhost:*`
//! origin, which is exactly what the suite and the HTMX apps are served from.
//!
//! Every command that touches the filesystem now goes through this module:
//! a path is accepted only when it resolves (after canonicalisation, so `..`
//! and symlinks cannot escape) inside one of the configured scopes, and never
//! when any component is one of the secret store files.
//!
//! Scopes are seeded from the places a desktop app legitimately browses —
//! the user's home, Downloads, the application data directory, the bot stack
//! and the Drive root — and can be extended by the operator with
//! `GB_FS_SCOPES` (path separator separated) or `GB_FS_ALLOW_ANY=1` for a
//! development machine where the picker reaches arbitrary folders.

use std::path::{Component, Path, PathBuf};

/// File names that are never readable, writable or deletable through the IPC,
/// even when they sit inside a legitimate scope (#2498). The Vault token and
/// the unseal keys are the keys to every credential in the deployment; the
/// admin credential file is a login factor.
const PROTECTED_NAMES: &[&str] = &[
    ".env",
    "desktop-session.json",
    "vault-unseal-keys",
    "admin-credentials.json",
    "init.json",
    "vault-token",
];

/// Whether the operator opted out of scoping (development machines).
fn allow_any() -> bool {
    std::env::var("GB_FS_ALLOW_ANY")
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// Canonical form of a path that may not exist yet: the deepest existing
/// ancestor is canonicalised and the remaining components are appended, so a
/// symlink in the middle of the path cannot be used to escape a scope.
fn canonical_for_check(path: &Path) -> Result<PathBuf, String> {
    if let Ok(full) = std::fs::canonicalize(path) {
        return Ok(full);
    }
    let mut ancestor = path;
    let mut tail: Vec<&std::ffi::OsStr> = Vec::new();
    loop {
        match ancestor.parent() {
            Some(parent) => {
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
            None => return Err("Path has no usable parent directory".to_string()),
        }
    }
}

/// The roots the IPC may touch.
pub fn scopes() -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = Vec::new();
    let mut push = |candidate: PathBuf| {
        if let Ok(canonical) = std::fs::canonicalize(&candidate) {
            if !roots.contains(&canonical) {
                roots.push(canonical);
            }
        }
    };

    if let Ok(list) = std::env::var("GB_FS_SCOPES") {
        // Absolute paths, so the separator is the platform's *path list*
        // separator (':' on Unix, ';' on Windows) — never the path component
        // separator, which would split every root into fragments.
        let list_separator = if cfg!(windows) { ';' } else { ':' };
        for entry in list.split(list_separator) {
            let trimmed = entry.trim();
            if !trimmed.is_empty() {
                push(PathBuf::from(trimmed));
            }
        }
    }
    if let Some(home) = dirs::home_dir() {
        push(home.clone());
        push(home.join("Downloads"));
        push(home.join("GeneralBots"));
    }
    if let Some(data) = dirs::data_dir() {
        push(data.join("generalbots"));
    }
    if let Ok(stack) = std::env::var("BOTSERVER_STACK_PATH") {
        push(PathBuf::from(stack));
    }
    // The kiosk appliance keeps all mutable state here.
    push(PathBuf::from("/var/lib/gbdesk"));
    roots
}

/// Rejects a path whose name — or any parent name — belongs to the secret
/// store. Runs before the scope check so a scope that happens to contain the
/// stack still cannot be used to read or replace its secrets.
pub fn check_not_protected(path: &Path) -> Result<(), String> {
    if allow_any() {
        return Ok(());
    }
    for component in path.components() {
        if let Component::Normal(name) = component {
            let text = name.to_string_lossy();
            if PROTECTED_NAMES
                .iter()
                .any(|protected| text.eq_ignore_ascii_case(protected))
            {
                return Err("This file belongs to the local secret store".to_string());
            }
        }
    }
    Ok(())
}

/// Resolves `raw` to an absolute path inside a configured scope.
///
/// # Errors
/// Returns a user-facing message when the path is relative, contains `..`,
/// names a protected file, or resolves outside every scope.
pub fn resolve(raw: &str) -> Result<PathBuf, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("A path is required".to_string());
    }
    let path = PathBuf::from(trimmed);
    if !path.is_absolute() {
        return Err("Only absolute paths are accepted".to_string());
    }
    if path
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err("Paths may not contain '..'".to_string());
    }
    check_not_protected(&path)?;
    if allow_any() {
        return Ok(path);
    }

    let canonical = canonical_for_check(&path)?;
    let roots = scopes();
    if roots.is_empty() {
        return Err("No filesystem scope is configured".to_string());
    }
    if roots.iter().any(|root| canonical.starts_with(root)) {
        return Ok(canonical);
    }
    Err("This location is outside the folders the desktop may access".to_string())
}

/// A single path segment (folder or file name), never a path.
///
/// # Errors
/// Returns a message when the name is empty, contains a separator, is `.` or
/// `..`, or is a protected file name.
pub fn validate_name(name: &str) -> Result<String, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("A name is required".to_string());
    }
    if trimmed.contains('/') || trimmed.contains('\\') {
        return Err("A name cannot contain a path separator".to_string());
    }
    if trimmed == "." || trimmed == ".." {
        return Err("A name cannot be '.' or '..'".to_string());
    }
    if trimmed.chars().any(|c| c.is_control()) {
        return Err("A name cannot contain control characters".to_string());
    }
    check_not_protected(Path::new(trimmed))?;
    Ok(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::{check_not_protected, resolve, validate_name};
    use std::path::Path;

    #[test]
    fn rejects_relative_paths() {
        assert!(resolve("drive/file.txt").is_err());
        assert!(resolve("").is_err());
    }

    #[test]
    fn rejects_parent_traversal() {
        assert!(resolve("/tmp/../etc/passwd").is_err());
    }

    #[test]
    fn rejects_secret_store_names() {
        assert!(check_not_protected(Path::new("/var/lib/gbdesk/.env")).is_err());
        assert!(check_not_protected(Path::new("/var/lib/gbdesk/vault-unseal-keys")).is_err());
        assert!(check_not_protected(Path::new("/var/lib/gbdesk/admin-credentials.json")).is_err());
        assert!(check_not_protected(Path::new("/var/lib/gbdesk/desktop-session.json")).is_err());
        assert!(check_not_protected(Path::new("/var/lib/gbdesk/notes.txt")).is_ok());
    }

    #[test]
    fn ipc_refuses_the_admin_token_even_inside_a_configured_scope() {
        // /var/lib/gbdesk is a built-in scope, but the protected-name check runs
        // first so generic read/write/delete IPC can never reach the JWT.
        assert!(resolve("/var/lib/gbdesk/desktop-session.json").is_err());
        // The positive control for non-secret names is covered by
        // `rejects_secret_store_names`; here the built-in appliance scope
        // directory may not exist on the test host, so `resolve` can fail for
        // that reason alone and would give a false negative.
    }

    #[test]
    fn names_reject_separators() {
        assert!(validate_name("../etc").is_err());
        assert!(validate_name("a/b").is_err());
        assert!(validate_name("..").is_err());
        assert!(validate_name("reports").is_ok());
    }

    #[test]
    fn outside_scope_is_refused() {
        // /etc is not home, Downloads, app data, the stack or /var/lib/gbdesk.
        assert!(resolve("/etc/shadow").is_err());
    }
}
