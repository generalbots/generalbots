//! #2562 — Truncate a conversation's history from a given visible message
//! index, supporting regenerate (drop last assistant, re-ask) and edit
//! (drop from edited user message onward, send edited text).
//!
//! The `from_index` refers to the position in the filtered message list
//! (user + assistant only, empty excluded) — the same indices the history
//! endpoint returns. The server resolves it to the raw DB `message_index`.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Json;
use botcore::shared::state::AppState;
use diesel::prelude::*;
use serde::Deserialize;
use uuid::Uuid;

use super::{bearer_user, db_error, HandlerError};

#[derive(Deserialize)]
pub struct TruncateBody {
    /// The visible message index from which to delete (inclusive). Messages
    /// at this position and later in the filtered (user+assistant) list are
    /// removed from the database.
    pub from_index: i32,
}

/// Truncates a conversation's stored history from the given visible index.
///
/// POST /api/chat/history/sessions/:session_id/truncate
pub async fn handle_truncate(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<Uuid>,
    request: axum::extract::Request,
) -> Result<Json<serde_json::Value>, HandlerError> {
    let user_uuid = bearer_user(&request)?;

    let bytes = axum::body::to_bytes(request.into_body(), 1024)
        .await
        .map_err(|_| (StatusCode::BAD_REQUEST, "bad body".to_string()))?;
    let body: TruncateBody = serde_json::from_slice(&bytes)
        .map_err(|_| (StatusCode::BAD_REQUEST, "bad json: expected { from_index: N }".to_string()))?;

    if body.from_index < 0 {
        return Err((StatusCode::BAD_REQUEST, "from_index must be >= 0".to_string()));
    }

    // Verify ownership.
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

    // Resolve the visible index to the raw DB message_index by loading
    // all row indices and filtering the same way the history endpoint does.
    let deleted = {
        let mut conn = state.conn.get().map_err(|e| {
            db_error("pool", e.to_string())
        })?;
        let all_indices: Vec<(i32, i32)> = botschema::message_history::table
            .filter(botschema::message_history::session_id.eq(session_id))
            .order(botschema::message_history::message_index.asc())
            .select((
                botschema::message_history::role,
                botschema::message_history::message_index,
            ))
            .load::<(i32, i32)>(&mut conn)
            .map_err(|e| db_error("load indices", e.to_string()))?;

        // Keep only user (1) and assistant (2) rows — same filter as the
        // history endpoint.
        let visible: Vec<i32> = all_indices
            .iter()
            .filter(|(role, _)| *role == 1 || *role == 2)
            .map(|(_, idx)| *idx)
            .collect();

        let from = body.from_index as usize;
        if from >= visible.len() {
            return Ok(Json(serde_json::json!({
                "truncated": true,
                "session_id": session_id,
                "from_index": body.from_index,
                "deleted_count": 0,
            })));
        }
        let db_from = visible[from];
        diesel::delete(
            botschema::message_history::table
                .filter(botschema::message_history::session_id.eq(session_id))
                .filter(botschema::message_history::message_index.ge(db_from)),
        )
        .execute(&mut conn)
        .map_err(|e| db_error("truncate", e.to_string()))?
    };

    Ok(Json(serde_json::json!({
        "truncated": true,
        "session_id": session_id,
        "from_index": body.from_index,
        "deleted_count": deleted,
    })))
}
