use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Json;
use botcore::shared::state::AppState;
use diesel::prelude::*;
use serde::Deserialize;
use uuid::Uuid;

use super::{HandlerError, bearer_user, db_error};

#[derive(Deserialize)]
struct RenameSessionBody {
    title: String,
}

/// Renames one conversation owned by the authenticated caller.
pub async fn handle_rename_chat_history_session(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<Uuid>,
    request: axum::extract::Request,
) -> Result<Json<serde_json::Value>, HandlerError> {
    let user_uuid = bearer_user(&request)?;
    // Read body
    let bytes = axum::body::to_bytes(request.into_body(), 1024 * 1024)
        .await
        .map_err(|_| (StatusCode::BAD_REQUEST, "bad body".to_string()))?;
    let body: RenameSessionBody = serde_json::from_slice(&bytes)
        .map_err(|_| (StatusCode::BAD_REQUEST, "bad json".to_string()))?;
    let renamed = {
        let mut sm = state.session_manager.lock().await;
        sm.rename_session(session_id, user_uuid, &body.title)
            .map_err(|e| db_error("rename session", e.to_string()))?
    };
    if !renamed {
        return Err((StatusCode::NOT_FOUND, "Conversation not found".to_string()));
    }
    Ok(Json(serde_json::json!({
        "renamed": true,
        "session_id": session_id,
        "title": body.title.trim(),
    })))
}

/// Deletes one conversation from history and removes its agent workspace
/// folder. Ownership is checked against the authenticated caller; a session
/// owned by another user is reported as 404 (no existence leak).
pub async fn handle_delete_chat_history_session(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<Uuid>,
    request: axum::extract::Request,
) -> Result<Json<serde_json::Value>, HandlerError> {
    let user_uuid = bearer_user(&request)?;
    let deleted = {
        let mut sm = state.session_manager.lock().await;
        sm.delete_session(session_id, user_uuid)
            .map_err(|e| db_error("delete session", e.to_string()))?
    };
    if !deleted {
        return Err((StatusCode::NOT_FOUND, "Conversation not found".to_string()));
    }
    #[cfg(feature = "agent-vm")]
    crate::core::bot::agent_vm_hook::delete_conversation_folder(user_uuid, session_id).await;
    Ok(Json(serde_json::json!({
        "deleted": true,
        "session_id": session_id,
    })))
}

/// Clears all messages from one conversation while keeping the session row.
/// Equivalent to truncating from the first visible message.
pub async fn handle_clear_chat_history_session(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<Uuid>,
    request: axum::extract::Request,
) -> Result<Json<serde_json::Value>, HandlerError> {
    let user_uuid = bearer_user(&request)?;

    let owner = {
        let mut sm = state.session_manager.lock().await;
        sm.get_session_by_id(session_id)
            .map_err(|e| db_error("load session", e))?
    };
    let Some(sess) = owner else {
        return Err((StatusCode::NOT_FOUND, "Conversation not found".to_string()));
    };
    if sess.user_id.to_string() != user_uuid.to_string() {
        return Err((StatusCode::NOT_FOUND, "Conversation not found".to_string()));
    }

    let deleted = {
        let mut conn = state
            .conn
            .get()
            .map_err(|e| db_error("pool", e.to_string()))?;
        diesel::delete(
            botschema::message_history::table
                .filter(botschema::message_history::session_id.eq(session_id)),
        )
        .execute(&mut conn)
        .map_err(|e| db_error("clear messages", e.to_string()))?
    };

    Ok(Json(serde_json::json!({
        "cleared": true,
        "session_id": session_id,
        "deleted_count": deleted,
    })))
}
