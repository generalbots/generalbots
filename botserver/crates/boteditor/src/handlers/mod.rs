use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    Form,
};
use botcore::shared::state::AppState;
use botsecurity_core::error_sanitizer::log_and_sanitize;
use botsecurity_core::path_guard::{PathGuard, PathGuardConfig, PathGuardError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::fs;

mod edits;
mod files;
mod magic;

#[derive(Serialize)]
pub struct FileListResponse {
    pub files: Vec<String>,
}

#[derive(Serialize)]
pub struct FileContentResponse {
    pub content: String,
}

#[derive(Deserialize)]
pub struct SaveFileRequest {
    pub content: Option<String>,
    pub name: Option<String>,
}

#[derive(Deserialize)]
pub struct FormContent {
    pub content: Option<String>,
}

#[derive(Deserialize)]
pub struct MagicRequest {
    pub code: Option<String>,
}

#[derive(Serialize)]
pub struct MagicResponse {
    pub improved_code: Option<String>,
    pub explanation: Option<String>,
    pub suggestions: Option<Vec<MagicSuggestion>>,
}

#[derive(Serialize)]
pub struct MagicSuggestion {
    pub title: String,
    pub description: String,
}

#[derive(Deserialize)]
pub struct EditorQuery {
    pub path: Option<String>,
}

pub struct EditorSession {
    pub undo_stack: Vec<String>,
    pub redo_stack: Vec<String>,
}

pub type EditorRegistry = Arc<Mutex<HashMap<String, EditorSession>>>;

pub fn editor_registry() -> EditorRegistry {
    Arc::new(Mutex::new(HashMap::new()))
}

pub(super) fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

async fn workspace_root(state: Arc<AppState>) -> String {
    if !state.bucket_name.is_empty() {
        let base = std::env::temp_dir()
            .join("gb-editor")
            .join(&state.bucket_name);
        return base.to_string_lossy().to_string();
    }
    std::env::var("EDITOR_WORKSPACE").unwrap_or_else(|_| {
        std::env::temp_dir()
            .join("gb-editor")
            .join("default")
            .to_string_lossy()
            .to_string()
    })
}

fn editor_path_guard() -> PathGuard {
    PathGuard::new(
        PathGuardConfig::new()
            .allow_hidden(true)
            .with_denied_extensions(Vec::new())
            .with_max_depth(64),
    )
}

fn resolve_under_root(
    root: &std::path::Path,
    path: &str,
) -> Result<std::path::PathBuf, PathGuardError> {
    editor_path_guard().validate_relative_under(root, std::path::Path::new(path))
}

fn reject_path(context: &str, path: &str, error: &PathGuardError) -> (StatusCode, String) {
    log::warn!("{context}: rejected path {path:?}: {error}");
    (
        StatusCode::BAD_REQUEST,
        log_and_sanitize(error, context, None).message,
    )
}

async fn resolve_path(
    state: &Arc<AppState>,
    path: &str,
) -> Result<std::path::PathBuf, PathGuardError> {
    let root = workspace_root(state.clone()).await;
    let _ = fs::create_dir_all(&root).await;
    resolve_under_root(std::path::Path::new(&root), path)
}

pub use edits::{handle_format, handle_redo, handle_undo};
pub use files::{handle_save_as, list_files, read_file, save_file, save_file_alt};
pub use magic::handle_magic;

#[cfg(test)]
mod path_guard_tests {
    use super::resolve_under_root;
    use std::path::Path;

    const ROOT: &str = "/tmp/gb-editor/test-root";

    #[test]
    fn hostile_editor_paths_are_rejected() {
        let root = Path::new(ROOT);
        for hostile in [
            "../secret.bas",
            "a/../../secret",
            "/etc/passwd",
            "/tmp/gb-editor/other/file.bas",
            "file\0.bas",
            "a\\b.bas",
            "file;rm.bas",
            ".",
        ] {
            assert!(
                resolve_under_root(root, hostile).is_err(),
                "{hostile:?} must be rejected"
            );
        }
    }

    #[test]
    fn over_depth_editor_paths_are_rejected() {
        let root = Path::new(ROOT);
        assert!(resolve_under_root(root, &vec!["a"; 70].join("/")).is_err());
    }

    #[test]
    fn valid_editor_paths_resolve_under_the_workspace() {
        let root = Path::new(ROOT);
        assert_eq!(
            resolve_under_root(root, "src/script.bas").ok(),
            Some(root.join("src/script.bas"))
        );
        assert_eq!(
            resolve_under_root(root, "./.gbdialog/start.bas").ok(),
            Some(root.join(".gbdialog/start.bas"))
        );
    }

    #[cfg(unix)]
    #[test]
    fn symlink_escape_is_rejected() {
        let root = std::env::temp_dir().join(format!("boteditor-pg-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create temp root");
        std::os::unix::fs::symlink("/etc", root.join("escape")).expect("create symlink");
        assert!(resolve_under_root(&root, "escape/passwd").is_err());
        let _ = std::fs::remove_dir_all(&root);
    }
}
