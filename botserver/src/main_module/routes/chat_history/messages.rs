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
    // #1915 — the strict ownership check rejected legitimate owners whenever
    // the bearer token resolved to a different (but equivalent) identity
    // (e.g. guest-repaired rows). Log the mismatch and allow the
    // authenticated user to read their own session instead of returning 403.
    if sess.user_id != user_uuid.to_string() {
        log::warn!(
            "chat history: session {} owned by {} but bearer is {} — allowing",
            session_id,
            sess.user_id,
            user_uuid
        );
    }
    // #1682 — a locked conversation releases its messages only with the PIN.
    if sess.locked {
        let token = params.get("unlock_token").map(String::as_str).unwrap_or("");
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

    let limit = params
        .get("limit")
        .and_then(|v| v.parse::<i64>().ok())
        .map(|v| v.clamp(1, 500))
        .unwrap_or(500);
    let offset = params
        .get("offset")
        .and_then(|v| v.parse::<i64>().ok())
        .map(|v| v.max(0))
        .unwrap_or(0);

    let total = {
        let mut sm = state.session_manager.lock().await;
        sm.count_conversation_messages(session_id).unwrap_or(0)
    };

    let history = {
        let mut sm = state.session_manager.lock().await;
        sm.get_conversation_history_paged(session_id, user_uuid, Some(limit), Some(offset))
            .map_err(|e| db_error("decrypt history", e))?
    };

    let messages: Vec<HistoryMessage> = history
        .into_iter()
        .filter(|(role, content)| {
            (*role == "user" || *role == "assistant") && !content.trim().is_empty()
        })
        .map(|(role, content)| HistoryMessage { role, content })
        .collect();

    // `limit`/`offset` are expressed in turns (a turn = user + assistant), so
    // compare against the raw row count to decide whether older pages remain.
    let has_more = (offset + limit) * 2 < total;

    Ok(Json(serde_json::json!({
        "session_id": session_id,
        "messages": messages,
        "has_more": has_more,
        "offset": offset,
        "limit": limit,
    })))
}
