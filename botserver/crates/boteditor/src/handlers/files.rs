use super::*;

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
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            Err((StatusCode::NOT_FOUND, "File not found".to_string()))
        }
        Err(e) => {
            log::warn!("boteditor::read_file: failed to read {path:?}: {e}");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                log_and_sanitize(&e, "boteditor::read_file", None).message,
            ))
        }
    }
}

pub async fn save_file(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
    Json(payload): Json<SaveFileRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let content = payload.content.unwrap_or_else(String::new);
    let full = match resolve_path(&state, &path).await {
        Ok(full) => full,
        Err(e) => return Err(reject_path("boteditor::save_file", &path, &e)),
    };
    if let Some(parent) = full.parent() {
        let _ = fs::create_dir_all(parent).await;
    }
    fs::write(&full, &content).await.map_err(|e| {
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

pub async fn save_file_alt(
    State(state): State<Arc<AppState>>,
    Query(query): Query<EditorQuery>,
    Form(payload): Form<FormContent>,
) -> Result<Html<String>, (StatusCode, String)> {
    let path = query.path.unwrap_or_else(|| "untitled.txt".to_string());
    let content = payload.content.unwrap_or_else(String::new);
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
