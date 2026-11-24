use super::*;

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
    let current = payload.content.unwrap_or_else(String::new);
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
    let current = payload.content.unwrap_or_else(String::new);
    session.undo_stack.push(current.clone());
    let restored = session.redo_stack.pop().unwrap_or(current);
    Html(html_escape(&restored))
}

pub async fn handle_format(Form(payload): Form<FormContent>) -> impl IntoResponse {
    let content = payload.content.unwrap_or_else(String::new);
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
