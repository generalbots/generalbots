use super::*;

pub async fn create_session(
    State(sessions): State<SessionMap>,
    Json(payload): Json<CreateSessionRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    create_session_inner(sessions, payload).await
}

/// Shared by `POST /api/browser/session` and the VM tab endpoints (#1843).
pub(super) async fn create_session_inner(
    sessions: SessionMap,
    payload: CreateSessionRequest,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let headless = payload.headless.unwrap_or(true);
    let meta = BrowserTabMeta {
        vm: payload.vm.clone(),
        conversation_id: payload.conversation_id.clone(),
        vm_controlled: payload.vm_controlled.unwrap_or(false),
        title: payload.title.clone(),
    };
    match BrowserSession::new_with_meta(headless, meta).await {
        Ok(session) => {
            let id = session.id.clone();
            if let Some(url) = payload.url.as_deref().filter(|u| !u.trim().is_empty()) {
                if let Err(e) = session.navigate(url).await {
                    // The tab is still useful: surface the failure in the tab
                    // list instead of dropping the freshly created session.
                    log::warn!("Initial navigate for browser tab {id} failed: {e}");
                }
            }
            let tab = session.tab().await;
            sessions.lock().await.insert(id.clone(), session);
            log::info!("Browser session created: {id}");
            Ok(Json(serde_json::json!({
                "id": id,
                "status": "created",
                "headless": headless,
                "tab": tab
            })))
        }
        Err(e) => {
            log::error!("Failed to create browser session: {e}");
            Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to create browser: {e}"),
            ))
        }
    }
}

pub(super) async fn get_session(
    sessions: &SessionMap,
    id: &str,
) -> Result<BrowserSession, (axum::http::StatusCode, String)> {
    let map = sessions.lock().await;
    map.get(id).cloned().ok_or_else(|| {
        (
            axum::http::StatusCode::NOT_FOUND,
            format!("Session {id} not found"),
        )
    })
}
