//! Shared container-browser tab strip (#1843).
//!
//! One VM owns one tab set. Conversations open/attach/close tabs through these
//! routes instead of spawning an isolated Chromium process each. The legacy
//! `/api/browser/session` routes remain as the headless fallback.

use super::*;

const DEFAULT_VM: &str = "default";

fn normalize_vm(vm: &str) -> String {
    let trimmed = vm.trim();
    if trimmed.is_empty() {
        DEFAULT_VM.to_string()
    } else {
        trimmed.to_string()
    }
}

async fn sorted_tabs(sessions: &SessionMap, vm: Option<&str>) -> Vec<BrowserTab> {
    let map = sessions.lock().await;
    let mut tabs = Vec::new();
    for session in map.values() {
        if vm.is_some_and(|wanted| session.vm != wanted) {
            continue;
        }
        tabs.push(session.tab().await);
    }
    tabs.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    tabs
}

/// GET /api/browser/tabs — every tab in this botserver process. Kept for the
/// documented browser API; the VM-scoped route below is what the app uses.
pub async fn list_tabs(State(sessions): State<SessionMap>) -> Json<serde_json::Value> {
    let tabs = sorted_tabs(&sessions, None).await;
    Json(serde_json::json!({ "tabs": tabs, "total": tabs.len() }))
}

/// POST /api/browser/tabs — documented alias for opening a tab.
pub async fn open_tab(
    State(sessions): State<SessionMap>,
    Json(payload): Json<CreateSessionRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    create_session_inner(sessions, payload).await
}

/// GET /api/browser/vm/:vm/tabs
pub async fn list_vm_tabs(
    State(sessions): State<SessionMap>,
    Path(vm): Path<String>,
) -> Json<serde_json::Value> {
    let vm = normalize_vm(&vm);
    let tabs = sorted_tabs(&sessions, Some(&vm)).await;
    Json(serde_json::json!({ "vm": vm, "tabs": tabs, "total": tabs.len() }))
}

/// POST /api/browser/vm/:vm/tabs — open a tab in the shared container tab set.
pub async fn open_vm_tab(
    State(sessions): State<SessionMap>,
    Path(vm): Path<String>,
    Json(mut payload): Json<CreateSessionRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    payload.vm = Some(normalize_vm(&vm));
    create_session_inner(sessions, payload).await
}

/// POST /api/browser/vm/:vm/tabs/:id/attach — explicit share/attach. A
/// conversation must opt in before another conversation's tab is reused.
pub async fn attach_vm_tab(
    State(sessions): State<SessionMap>,
    Path((_vm, id)): Path<(String, String)>,
    Json(payload): Json<AttachTabRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let session = get_session(&sessions, &id).await?;
    session
        .attach_conversation(payload.conversation_id.clone())
        .await;
    let tab = session.tab().await;
    Ok(Json(serde_json::json!({
        "status": "attached",
        "conversation_id": payload.conversation_id,
        "tab": tab
    })))
}

/// DELETE /api/browser/tabs/:id — documented alias.
pub async fn close_tab(
    State(sessions): State<SessionMap>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    close_session_by_id(sessions, id).await
}

/// DELETE /api/browser/vm/:vm/tabs/:id
pub async fn close_vm_tab(
    State(sessions): State<SessionMap>,
    Path((_vm, id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    close_session_by_id(sessions, id).await
}

async fn close_session_by_id(
    sessions: SessionMap,
    id: String,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let session = get_session(&sessions, &id).await?;
    session.close().await.map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Close failed: {e}"),
        )
    })?;
    sessions.lock().await.remove(&id);
    log::info!("Browser VM tab closed: {id}");
    Ok(Json(serde_json::json!({ "status": "closed", "id": id })))
}

#[cfg(test)]
mod tests {
    use super::normalize_vm;

    #[test]
    fn empty_vm_is_the_default_tab_set() {
        assert_eq!(normalize_vm(""), "default");
        assert_eq!(normalize_vm("   "), "default");
        assert_eq!(normalize_vm(" ag-user "), "ag-user");
    }
}
