use super::*;

pub async fn create_session(
    State(sessions): State<SessionMap>,
    caller: SessionCaller,
    Json(payload): Json<CreateSessionRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    create_session_inner(sessions, payload, &caller).await
}

/// Shared by `POST /api/browser/session` and the VM tab endpoints (#1843).
///
/// The owner is taken from the verified caller, never from client input, and a
/// fresh capability is minted by [`BrowserSession::new_with_meta`]. Both are
/// bound before the session becomes visible in the process-global map (#2879).
pub(super) async fn create_session_inner(
    sessions: SessionMap,
    payload: CreateSessionRequest,
    caller: &SessionCaller,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    // #2878 — reject an unsafe initial URL before allocating a browser tab.
    if let Some(url) = payload.url.as_deref().filter(|u| !u.trim().is_empty()) {
        if let Err(reason) = crate::url_policy::validate_navigation_url(url) {
            return Err((axum::http::StatusCode::BAD_REQUEST, reason));
        }
    }
    let headless = payload.headless.unwrap_or(true);
    let meta = BrowserTabMeta {
        owner: Some(caller.owner.clone()),
        vm: payload.vm.clone(),
        conversation_id: payload.conversation_id.clone(),
        vm_controlled: payload.vm_controlled.unwrap_or(false),
        title: payload.title.clone(),
    };
    match BrowserSession::new_with_meta(headless, meta).await {
        Ok(session) => {
            let id = session.id.clone();
            let capability = session.capability.clone();
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
                "capability": capability,
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

/// Authoritative session lookup for every session-scoped route (#2879).
///
/// A session is returned only when the caller is the recorded owner *and*
/// presents the unguessable capability minted at creation. A foreign owner or
/// a wrong/missing capability yields `403 Forbidden`; a genuinely unknown id
/// yields `404 Not Found`. This is the single choke point through which all
/// navigate/click/fill/execute/extract/screenshot/agent/close requests pass.
pub(super) async fn get_session(
    sessions: &SessionMap,
    id: &str,
    caller: &SessionCaller,
) -> Result<BrowserSession, (axum::http::StatusCode, String)> {
    let map = sessions.lock().await;
    let session = map.get(id).ok_or_else(|| {
        (
            axum::http::StatusCode::NOT_FOUND,
            format!("Session {id} not found"),
        )
    })?;
    if !session_access_allowed(
        session.owner.as_ref(),
        &session.capability,
        &caller.owner,
        caller.capability.as_deref(),
    ) {
        log::warn!("Browser session access denied for {id}: owner/capability mismatch");
        return Err((
            axum::http::StatusCode::FORBIDDEN,
            "Session access denied".to_string(),
        ));
    }
    Ok(session.clone())
}
