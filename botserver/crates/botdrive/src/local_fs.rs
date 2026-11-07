//! Local-filesystem Drive backend (#1801).
//!
//! Desktop installs run botserver as a sidecar with the same OS user as the
//! Tauri shell, so the server can browse the real home folder. This module is
//! the only place that touches that filesystem; the public HTTP surface stays
//! `/api/files`, with `backend=local&root=home`.
//!
//! Safety contract:
//! - enabled only for the single-user desktop profile (`no_security`), never
//!   for a multi-tenant deployment;
//! - roots are an allowlist under the user's home;
//! - every requested path is canonicalized and must stay inside its root;
//! - dotfiles and a small sensitive-name list are never listed;
//! - symlinks that resolve outside the root are skipped.
//!
//! Read-only for now: listing and downloads. Writes, renames, moves, deletes
//! and cross-backend copies are documented follow-up work on #1801.

use std::path::{Path, PathBuf};

use crate::drive_types::FileListItem;

/// Bucket prefix the Drive UI uses to address the local backend.
const LOCAL_BUCKET_PREFIX: &str = "local:";
/// Default root when the caller does not send one.
pub const DEFAULT_ROOT: &str = "home";

/// Allowlisted root aliases, resolved under the user's home directory.
const ROOT_ALIASES: &[(&str, &str)] = &[
    ("home", ""),
    ("documents", "Documents"),
    ("music", "Music"),
    ("pictures", "Pictures"),
    ("downloads", "Downloads"),
    ("desktop", "Desktop"),
];

/// Names never exposed even when they are not dot-prefixed. These hold
/// credentials or browser profiles on the platforms where they exist.
const DENIED_NAMES: &[&str] = &["appdata", "library", "snap"];

/// True when the local backend may be used at all.
///
/// Multi-tenant deployments must never read the server's filesystem through
/// Drive, so this is the desktop profile only.
pub fn enabled() -> bool {
    cfg!(feature = "desktop-local")
        || botcore::product::PRODUCT_CONFIG
            .read()
            .map(|product| product.no_security)
            .unwrap_or(false)
}

fn ensure_enabled() -> Result<(), String> {
    if enabled() {
        Ok(())
    } else {
        Err("Local files are only available in the desktop edition".to_string())
    }
}

/// True when a request addresses the local backend, either explicitly with
/// `backend=local` or implicitly through a `local:<root>` bucket.
pub fn is_local_backend(backend: Option<&str>, bucket: Option<&str>) -> bool {
    let explicit = backend
        .map(|value| value.eq_ignore_ascii_case("local") || value.eq_ignore_ascii_case("fs"))
        .unwrap_or(false);
    explicit || bucket.and_then(parse_local_bucket).is_some()
}

/// Extracts the root alias from a `local:<root>` bucket name.
pub fn parse_local_bucket(bucket: &str) -> Option<&str> {
    bucket.strip_prefix(LOCAL_BUCKET_PREFIX)
}

/// Bucket name handed back to the Drive UI for local rows.
pub fn local_bucket(root: &str) -> String {
    format!("{LOCAL_BUCKET_PREFIX}{root}")
}

fn home_dir() -> Result<PathBuf, String> {
    // The override exists so packaging/tests can point the backend at a
    // sandbox without touching the real home folder.
    if let Ok(value) = std::env::var("GB_LOCAL_FILES_ROOT") {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Ok(PathBuf::from(trimmed));
        }
    }
    dirs::home_dir().ok_or_else(|| "Could not determine the user's home directory".to_string())
}

/// Resolves an allowlisted root alias to a canonical directory.
fn resolve_root(name: Option<&str>) -> Result<(String, PathBuf), String> {
    let requested = name.unwrap_or(DEFAULT_ROOT).trim().to_ascii_lowercase();
    let suffix = ROOT_ALIASES
        .iter()
        .find(|(alias, _)| *alias == requested.as_str())
        .map(|(_, suffix)| *suffix)
        .ok_or_else(|| format!("Unsupported local root '{requested}'"))?;

    let home = home_dir()?
        .canonicalize()
        .map_err(|e| format!("Home directory is not accessible: {e}"))?;
    let candidate = if suffix.is_empty() {
        home.clone()
    } else {
        home.join(suffix)
    };
    if !candidate.is_dir() {
        return Err(format!("Local root '{requested}' is not available"));
    }
    let canonical = candidate
        .canonicalize()
        .map_err(|e| format!("Local root '{requested}' is not accessible: {e}"))?;
    if !canonical.starts_with(&home) {
        return Err("Local root escapes the home directory".to_string());
    }
    Ok((requested, canonical))
}

/// Rejects absolute paths, traversal, dotfiles and sensitive names.
fn sanitize_relative(path: &str) -> Result<PathBuf, String> {
    if path.contains('\0') {
        return Err("Invalid path".to_string());
    }
    if path.starts_with('/') || path.starts_with('\\') || Path::new(path).is_absolute() {
        return Err("Absolute paths are not allowed".to_string());
    }
    let mut relative = PathBuf::new();
    for segment in path.split(|c| c == '/' || c == '\\') {
        if segment.is_empty() || segment == "." {
            continue;
        }
        if segment == ".." {
            return Err("Path traversal is not allowed".to_string());
        }
        let lower = segment.to_ascii_lowercase();
        if lower.starts_with('.') {
            return Err("Hidden files are not accessible".to_string());
        }
        if DENIED_NAMES.contains(&lower.as_str()) {
            return Err(format!("'{segment}' is not accessible"));
        }
        relative.push(segment);
    }
    Ok(relative)
}

/// Canonicalizes a request path and proves it stays inside `root`.
fn resolve_target(root: &Path, path: &str) -> Result<(PathBuf, PathBuf), String> {
    let relative = sanitize_relative(path)?;
    let target = root.join(&relative);
    let canonical = target
        .canonicalize()
        .map_err(|e| format!("Path not found: {e}"))?;
    if !canonical.starts_with(root) {
        return Err("Path escapes the local root".to_string());
    }
    Ok((relative, canonical))
}

fn is_visible(name: &str) -> bool {
    if name.is_empty() || name.starts_with('.') {
        return false;
    }
    !DENIED_NAMES.contains(&name.to_ascii_lowercase().as_str())
}

fn modified_rfc3339(meta: &std::fs::Metadata) -> Option<String> {
    let modified = meta.modified().ok()?;
    Some(chrono::DateTime::<chrono::Utc>::from(modified).to_rfc3339())
}

/// Lists one local folder using the same file model as the MinIO listing.
pub async fn list_local_files(
    root_name: Option<&str>,
    path: Option<&str>,
) -> Result<Vec<FileListItem>, String> {
    ensure_enabled()?;
    let (root_key, root) = resolve_root(root_name)?;
    let (base, target) = resolve_target(&root, path.unwrap_or(""))?;
    if !target.is_dir() {
        return Err("Not a directory".to_string());
    }

    let bucket = local_bucket(&root_key);
    let mut entries = tokio::fs::read_dir(&target)
        .await
        .map_err(|e| format!("Failed to read directory: {e}"))?;
    let mut items = Vec::new();

    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| format!("Failed to read directory: {e}"))?
    {
        let name = entry.file_name().to_string_lossy().to_string();
        if !is_visible(&name) {
            continue;
        }
        let entry_path = entry.path();
        let meta = match tokio::fs::metadata(&entry_path).await {
            Ok(meta) => meta,
            Err(_) => continue,
        };
        // A symlink that resolves outside the approved root is not part of
        // the Home folder, even though it lives inside it.
        let canonical = match entry_path.canonicalize() {
            Ok(path) => path,
            Err(_) => continue,
        };
        if !canonical.starts_with(&root) {
            continue;
        }

        let is_dir = meta.is_dir();
        let relative = base.join(&name);
        items.push(FileListItem {
            name,
            path: relative.to_string_lossy().replace('\\', "/"),
            is_dir,
            size: if is_dir { 0 } else { meta.len() },
            modified: modified_rfc3339(&meta),
            is_kb: false,
            is_public: false,
            bucket: Some(bucket.clone()),
        });
    }

    items.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    Ok(items)
}

/// Reads one local file for download/preview handlers.
pub async fn read_local_file(root_name: Option<&str>, path: &str) -> Result<Vec<u8>, String> {
    ensure_enabled()?;
    let (_, root) = resolve_root(root_name)?;
    let (_, target) = resolve_target(&root, path)?;
    let meta = tokio::fs::metadata(&target)
        .await
        .map_err(|e| format!("File not found: {e}"))?;
    if !meta.is_file() {
        return Err("Not a file".to_string());
    }
    tokio::fs::read(&target)
        .await
        .map_err(|e| format!("Failed to read file: {e}"))
}

/// Root alias from an explicit `root` parameter or the `local:<root>` bucket.
pub fn requested_root<'a>(root: Option<&'a str>, bucket: Option<&'a str>) -> Option<&'a str> {
    root.or_else(|| bucket.and_then(parse_local_bucket))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_rejects_traversal_and_hidden_paths() {
        assert!(sanitize_relative("../etc/passwd").is_err());
        assert!(sanitize_relative("docs/../../secret").is_err());
        assert!(sanitize_relative(".ssh/id_rsa").is_err());
        assert!(sanitize_relative("Library/Keychains").is_err());
        assert!(sanitize_relative("/etc/passwd").is_err());
    }

    #[test]
    fn sanitize_keeps_plain_relative_paths() {
        assert_eq!(
            sanitize_relative("Documents/report.pdf").ok(),
            Some(PathBuf::from("Documents/report.pdf"))
        );
        assert_eq!(
            sanitize_relative("./Music/").ok(),
            Some(PathBuf::from("Music"))
        );
    }

    #[test]
    fn local_bucket_round_trips() {
        let bucket = local_bucket("home");
        assert_eq!(bucket, "local:home");
        assert_eq!(parse_local_bucket(&bucket), Some("home"));
        assert!(is_local_backend(Some("local"), None));
        assert!(is_local_backend(None, Some(&bucket)));
        assert!(!is_local_backend(Some("minio"), Some("default.gbai")));
    }
}
