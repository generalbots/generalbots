use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::Json;
use botcore::shared::state::AppState;
use serde::Serialize;
use uuid::Uuid;

use super::{bearer_user, db_error, HandlerError};
use crate::main_module::routes::chat_history_lock::{unlock_token_valid, verify_pin};

#[derive(Serialize)]
struct HistoryMessage {
    role: String,
    content: String,
}

/// Returns the decrypted messages of one conversation. The session must
/// belong to the authenticated user.
pub async fn handle_chat_history_messages(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<Uuid>,
    Query(params): Query<HashMap<String, String>>,
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
    if sess.user_id != user_uuid {
        return Err((StatusCode::FORBIDDEN, "Not your conversation".to_string()));
    }
    // #1682 — a locked conversation releases its messages only with the PIN.
    if sess.locked {
        let token = params
            .get("unlock_token")
            .map(String::as_str)
            .unwrap_or("");
        let token_ok = !token.is_empty() && unlock_token_valid(token, session_id, user_uuid);
        let supplied = params.get("pin").map(String::as_str).unwrap_or("");
        let pin_ok = sess
            .lock_pin_hash
            .as_deref()
            .is_some_and(|hash| verify_pin(supplied, hash));
        if !token_ok && !pin_ok {
            return Err((
                StatusCode::LOCKED,
                "This conversation is locked".to_string(),
            ));
        }
    }

    let history = {
        let mut sm = state.session_manager.lock().await;
        sm.get_conversation_history(session_id, user_uuid, Some(500))
            .map_err(|e| db_error("decrypt history", e))?
    };

    let messages: Vec<HistoryMessage> = history
        .into_iter()
        .filter(|(role, content)| {
            (*role == "user" || *role == "assistant") && !content.trim().is_empty()
        })
        .map(|(role, content)| HistoryMessage { role, content })
        .collect();

    Ok(Json(serde_json::json!({
        "session_id": session_id,
        "messages": messages,
    })))
}
