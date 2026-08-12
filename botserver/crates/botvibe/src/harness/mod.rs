//! #747 — Real tool harness for the Vibe agent.
//!
//! Replaces the `botvibe` stubs with real, sandboxed tools operating on a
//! per-project workspace:
//! - `file_*`: read/write/list/delete project files (workspace-scoped)
//! - `run_command`: execute a project command through the command guard
//! - `git_*`: status/log/diff/commit against the project workspace checkout
//! - `logs_*`: read/tail the project runtime logs
//! - `project_test`: run the project test suite
//!
//! All filesystem access is rooted at `VIBE_WORKSPACE_ROOT` (default
//! `/opt/gbo/data/vibe-workspaces`) and confined per project; commands go
//! through `cmd::run` which enforces the same discipline as
//! `botcore`'s SafeCommand (allowlist + forbidden shell metacharacters).

pub mod cmd;
pub mod file_tools;
pub mod git_tools;
pub mod log_tools;
pub mod run_tools;
pub mod test_tools;

use std::path::{Path, PathBuf};

/// Root that hosts every project workspace subdirectory.
pub fn workspace_root() -> PathBuf {
    std::env::var("VIBE_WORKSPACE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/opt/gbo/data/vibe-workspaces"))
}

/// Validate a project id: alphanumerics, dash, underscore, dot — no path
/// separators, no traversal.
pub fn sanitize_project_id(project: &str) -> Result<String, String> {
    if project.is_empty() || project.len() > 128 || project == "." || project == ".." {
        return Err("invalid project id: empty or too long".into());
    }
    if project.chars().any(|c| !(c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')) {
        return Err(format!("invalid project id: {project}"));
    }
    Ok(project.to_string())
}

/// Resolve a path inside the project workspace, refusing `..` escapes and
/// absolute paths.
pub fn resolve_workspace_path(project: &str, rel: &str) -> Result<PathBuf, String> {
    let project = sanitize_project_id(project)?;
    if rel.contains('\0') {
        return Err("path contains NUL byte".into());
    }
    for seg in rel.split(['/', '\\']) {
        match seg {
            ".." => return Err(format!("path escapes workspace: {rel}")),
            _ => {}
        }
    }
    if Path::new(rel).is_absolute() {
        return Err(format!("path is absolute: {rel}"));
    }
    let root = workspace_root().join(&project);
    Ok(root.join(rel))
}

/// Ensure the workspace subdir for a project exists.
pub fn ensure_workspace(project: &str) -> Result<PathBuf, String> {
    let root = workspace_root().join(sanitize_project_id(project)?);
    std::fs::create_dir_all(&root).map_err(|e| format!("create workspace: {e}"))?;
    Ok(root)
}

pub fn read_rel_file(project: &str, rel: &str, max_bytes: u64) -> Result<Vec<u8>, String> {
    let path = resolve_workspace_path(project, rel)?;
    let meta = std::fs::metadata(&path).map_err(|e| format!("stat {rel}: {e}"))?;
    if meta.is_dir() {
        return Err(format!("{rel} is a directory"));
    }
    if (meta.len() as u64) > max_bytes {
        return Err(format!("{rel} exceeds {max_bytes} bytes"));
    }
    std::fs::read(&path).map_err(|e| format!("read {rel}: {e}"))
}

pub fn write_rel_file(project: &str, rel: &str, bytes: &[u8]) -> Result<(), String> {
    let path = resolve_workspace_path(project, rel)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("mkdir {parent:?}: {e}"))?;
    }
    if bytes.len() > 4 * 1024 * 1024 {
        return Err("file exceeds 4 MiB write limit".into());
    }
    std::fs::write(&path, bytes).map_err(|e| format!("write {rel}: {e}"))
}

/// List entries under a workspace dir, recursively with depth limit.
pub fn list_rel(project: &str, rel: &str, depth: u8) -> Result<Vec<String>, String> {
    let dir = resolve_workspace_path(project, rel)?;
    if !dir.exists() {
        return Ok(Vec::new());
    }
    if !dir.is_dir() {
        return Err(format!("{rel} is not a directory"));
    }
    let mut out = Vec::new();
    walk(&dir, &dir, depth, &mut out)?;
    Ok(out)
}

fn walk(dir: &Path, root: &Path, depth: u8, out: &mut Vec<String>) -> Result<(), String> {
    let entries = std::fs::read_dir(dir).map_err(|e| format!("read_dir {dir:?}: {e}"))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("read_dir entry: {e}"))?;
        let path = entry.path();
        let rel = path.strip_prefix(root).map_err(|e| format!("strip prefix: {e}"))?;
        let rel = rel.to_string_lossy().to_string();
        let is_dir = path.is_dir();
        out.push(if is_dir { format!("{rel}/") } else { rel });
        if is_dir && depth < 6 {
            walk(&path, root, depth + 1, out)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_project_rejects_traversal() {
        assert!(sanitize_project_id("my-web").is_ok());
        assert!(sanitize_project_id("a/b").is_err());
        assert!(sanitize_project_id("..").is_err());
        assert!(sanitize_project_id("").is_err());
    }

    #[test]
    fn resolve_rejects_parent_escape() {
        assert!(resolve_workspace_path("proj", "../etc/passwd").is_err());
        assert!(resolve_workspace_path("proj", "a/../../x").is_err());
        assert!(resolve_workspace_path("proj", "a/./b").is_err());
        assert!(resolve_workspace_path("proj", "src/main.rs").is_ok());
    }

    #[test]
    fn write_read_list_roundtrip() {
        let tmp = std::env::temp_dir().join(format!("vibe-harness-test-{}", uuid::Uuid::new_v4()));
        std::env::set_var("VIBE_WORKSPACE_ROOT", &tmp);
        let project = "proj-a";

        ensure_workspace(project).expect("ensure workspace");
        write_rel_file(project, "src/main.rs", b"fn main() {}").expect("write");
        write_rel_file(project, "README.md", b"# demo").expect("write");

        let bytes = read_rel_file(project, "src/main.rs", 1024).expect("read");
        assert_eq!(bytes, b"fn main() {}");

        let entries = list_rel(project, "", 0).expect("list");
        assert!(entries.iter().any(|e| e == "src/"));
        assert!(entries.iter().any(|e| e == "README.md"));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn read_rejects_oversize() {
        let tmp = std::env::temp_dir().join(format!("vibe-harness-test-{}", uuid::Uuid::new_v4()));
        std::env::set_var("VIBE_WORKSPACE_ROOT", &tmp);
        let project = "proj-b";
        ensure_workspace(project).expect("ensure workspace");
        write_rel_file(project, "big.bin", &vec![0u8; 4096]).expect("write");
        assert!(read_rel_file(project, "big.bin", 1024).is_err());
        let _ = std::fs::remove_dir_all(&tmp);
    }
}