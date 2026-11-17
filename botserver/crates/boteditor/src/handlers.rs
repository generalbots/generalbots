use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    Form,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::fs;

use botcore::shared::state::AppState;
use botsecurity_core::error_sanitizer::log_and_sanitize;
use botsecurity_core::path_guard::{PathGuard, PathGuardConfig, PathGuardError};

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

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub async fn list_files(
    State(state): State<Arc<AppState>>,
) -> Result<Json<FileListResponse>, (StatusCode, String)> {
    let base = workspace_root(state.clone()).await;
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(&base).await {
        let mut stream = entries;
        while let Ok(Some(entry)) = stream.next_entry().await {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                files.push(name.to_string());
            }
        }
    }
    files.sort();
    Ok(Json(FileListResponse { files }))
}

async fn workspace_root(state: Arc<AppState>) -> String {
    if !state.bucket_name.is_empty() {
        let base = std::env::temp_dir().join("gb-editor").join(&state.bucket_name);
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

/// Guard for editor paths. Hidden files are valid editor targets (`.gbdialog`,
/// `.gbot`) and source files must stay editable, so only the traversal/escape
/// rules apply — not the extension deny list.
fn editor_path_guard() -> PathGuard {
    PathGuard::new(
        PathGuardConfig::new()
            .allow_hidden(true)
            .with_denied_extensions(Vec::new())
            .with_max_depth(64),
    )
}

/// Resolve one user-supplied editor path under `root` through `PathGuard`.
///
/// Single hardening entry point (#2377): this replaces the old
/// `trim_start_matches('/')` concatenation, which let `../` walk out of the
/// workspace. Absolute paths, NUL bytes, traversal, symlink escapes and
/// over-depth inputs are now rejected before any filesystem call.
fn resolve_under_root(root: &std::path::Path, path: &str) -> Result<std::path::PathBuf, PathGuardError> {
    editor_path_guard().validate_relative_under(root, std::path::Path::new(path))
}

/// Log a rejected path with this crate's logger and return a sanitized 400
/// body — no raw internal error ever reaches the client.
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

pub async fn read_file(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
) -> Result<Json<FileContentResponse>, (StatusCode, String)> {
    let full = match resolve_path(&state, &path).await {
        Ok(full) => full,
        Err(e) => return Err(reject_path("boteditor::read_file", &path, &e)),
    };
    match fs::read_to_string(&full).await {
        Ok(content) => Ok(Json(FileContentResponse { content })),
        Err(_) => Ok(Json(FileContentResponse {
            content: String::new(),
        })),
    }
}

pub async fn save_file(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
    Json(payload): Json<SaveFileRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let content = payload.content.unwrap_or_default();
    let full = match resolve_path(&state, &path).await {
        Ok(full) => full,
        Err(e) => return Err(reject_path("boteditor::save_file", &path, &e)),
    };
    if let Some(parent) = full.parent() {
        let _ = fs::create_dir_all(parent).await;
    }
    fs::write(&full, &content)
        .await
        .map_err(|e| {
            log::error!("Editor write error: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Write error".to_string())
        })?;

    let key = full.to_string_lossy().to_string();
    let registry = editor_registry();
    let mut map = registry.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(session) = map.get_mut(&key) {
        session.undo_stack.push(content.clone());
        session.redo_stack.clear();
    } else {
        map.insert(
            key,
            EditorSession {
                undo_stack: vec![content],
                redo_stack: Vec::new(),
            },
        );
    }

    Ok(Json(serde_json::json!({
        "status": "success",
        "path": path,
    })))
}

pub async fn handle_save_as() -> impl IntoResponse {
    Html(
        r##"<div class="save-dialog">
        <h3>Save As</h3>
        <p class="hint">Enter a file path relative to your bot workspace.</p>
        <form hx-post="/api/editor/save" hx-target="#save-dialog" hx-swap="innerHTML">
            <input type="text" name="path" placeholder="src/script.bas" />
            <textarea name="content" placeholder="Paste file content here..." rows="6"></textarea>
            <button class="btn btn-primary btn-small" type="submit">Save</button>
        </form>
        </div>"##,
    )
}

pub async fn handle_undo(
    State(_state): State<Arc<AppState>>,
    Query(query): Query<EditorQuery>,
    Form(payload): Form<FormContent>,
) -> impl IntoResponse {
    let key = query.path.unwrap_or_else(|| "untitled".to_string());
    let registry = editor_registry();
    let mut map = registry.lock().unwrap_or_else(|e| e.into_inner());
    let session = map.entry(key.clone()).or_insert_with(|| EditorSession {
        undo_stack: Vec::new(),
        redo_stack: Vec::new(),
    });
    let current = payload.content.unwrap_or_default();
    session.redo_stack.push(current.clone());
    if session.undo_stack.is_empty() {
        session.undo_stack.push(current.clone());
        return Html(html_escape(&current));
    }
    let restored = session.undo_stack.pop().unwrap_or(current);
    Html(html_escape(&restored))
}

pub async fn handle_redo(
    State(_state): State<Arc<AppState>>,
    Query(query): Query<EditorQuery>,
    Form(payload): Form<FormContent>,
) -> impl IntoResponse {
    let key = query.path.unwrap_or_else(|| "untitled".to_string());
    let registry = editor_registry();
    let mut map = registry.lock().unwrap_or_else(|e| e.into_inner());
    let session = map.entry(key.clone()).or_insert_with(|| EditorSession {
        undo_stack: Vec::new(),
        redo_stack: Vec::new(),
    });
    let current = payload.content.unwrap_or_default();
    session.undo_stack.push(current.clone());
    let restored = session.redo_stack.pop().unwrap_or(current);
    Html(html_escape(&restored))
}

pub async fn handle_format(
    Form(payload): Form<FormContent>,
) -> impl IntoResponse {
    let content = payload.content.unwrap_or_default();
    let trimmed = content.trim().to_string();

    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        match serde_json::from_str::<serde_json::Value>(&trimmed) {
            Ok(value) => match serde_json::to_string_pretty(&value) {
                Ok(pretty) => Html(html_escape(&pretty)),
                Err(_) => Html(html_escape(&content)),
            },
            Err(_) => Html(html_escape(&content)),
        }
    } else {
        Html(html_escape(&content))
    }
}

pub async fn handle_magic(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<MagicRequest>,
) -> Result<Json<MagicResponse>, (StatusCode, String)> {
    let code = payload.code.unwrap_or_default();
    if code.trim().is_empty() {
        return Ok(Json(MagicResponse {
            improved_code: None,
            explanation: None,
            suggestions: Some(vec![MagicSuggestion {
                title: "Empty file".to_string(),
                description: "Write some code to get AI-powered suggestions.".to_string(),
            }]),
        }));
    }

    if let Some(llm) = state.llm_provider.clone() {
        let prompt = format!(
            "You are a senior code reviewer. Analyze the following code and return JSON with two keys: \
             'improved_code' (the code with improvements applied, or null if already good) and \
             'explanation' (a short explanation of the changes). Only respond with valid JSON.\n\n\
             ```\n{}\n```",
            code
        );
        match llm.generate_simple(&prompt).await {
            Ok(answer) => {
                let cleaned = answer
                    .trim_start_matches("```json")
                    .trim_start_matches("```")
                    .trim_end_matches("```")
                    .trim();
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(cleaned) {
                    let improved = value
                        .get("improved_code")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let explanation = value
                        .get("explanation")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    return Ok(Json(MagicResponse {
                        improved_code: improved,
                        explanation,
                        suggestions: None,
                    }));
                }
                Ok(Json(MagicResponse {
                    improved_code: None,
                    explanation: Some(answer),
                    suggestions: None,
                }))
            }
            Err(e) => {
                log::error!("Editor magic LLM error: {e}");
                Err((
                    StatusCode::SERVICE_UNAVAILABLE,
                    "AI service unavailable".to_string(),
                ))
            }
        }
    } else {
        Ok(Json(MagicResponse {
            improved_code: None,
            explanation: None,
            suggestions: Some(vec![MagicSuggestion {
                title: "No AI provider configured".to_string(),
                description: "Configure an LLM provider to get code improvement suggestions.".to_string(),
            }]),
        }))
    }
}

pub async fn save_file_alt(
    State(state): State<Arc<AppState>>,
    Query(query): Query<EditorQuery>,
    Form(payload): Form<FormContent>,
) -> Result<Html<String>, (StatusCode, String)> {
    let path = query.path.unwrap_or_else(|| "untitled.txt".to_string());
    let content = payload.content.unwrap_or_default();
    let full = match resolve_path(&state, &path).await {
        Ok(full) => full,
        Err(e) => return Err(reject_path("boteditor::save_file_alt", &path, &e)),
    };
    if let Some(parent) = full.parent() {
        let _ = fs::create_dir_all(parent).await;
    }
    match fs::write(&full, &content).await {
        Ok(()) => {
            let key = full.to_string_lossy().to_string();
            let registry = editor_registry();
            let mut map = registry.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(session) = map.get_mut(&key) {
                session.undo_stack.push(content);
                session.redo_stack.clear();
            } else {
                map.insert(
                    key,
                    EditorSession {
                        undo_stack: vec![content],
                        redo_stack: Vec::new(),
                    },
                );
            }
            Ok(Html(
                "<span class=\"save-notification ok\">Saved</span>".to_string(),
            ))
        }
        Err(e) => {
            log::error!("Editor save error: {e}");
            Ok(Html(
                "<span class=\"save-notification error\">Save failed</span>".to_string(),
            ))
        }
    }
}

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
