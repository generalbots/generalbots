use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::Json;
use botcore::shared::state::AppState;
use diesel::prelude::*;
use serde::Serialize;
use uuid::Uuid;

use super::{bearer_user, db_error, HandlerError};

const DEFAULT_SESSION_LIMIT: i64 = 50;
const TITLE_MAX_CHARS: usize = 80;

#[derive(Serialize)]
struct HistorySession {
    session_id: Uuid,
    bot_id: Uuid,
    title: String,
    created_at: String,
    updated_at: String,
    locked: bool,
}

/// Lists the authenticated user's chat conversations, newest first. An
/// optional `bot_name` narrows the listing to a single bot; `title` is the
/// decrypted first user message of the conversation.
pub async fn handle_chat_history_sessions(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
    request: axum::extract::Request,
) -> Result<Json<serde_json::Value>, HandlerError> {
    let user_uuid = bearer_user(&request)?;
    let limit = params
        .get("limit")
        .and_then(|v| v.parse::<i64>().ok())
        .map(|v| v.clamp(1, 200))
        .unwrap_or(DEFAULT_SESSION_LIMIT);
    let bot_filter = params.get("bot_name").cloned().unwrap_or_default();
    // #1682 — locked conversations never appear in the regular history. The
    // dedicated `locked=1` listing returns ids only (no titles) so the user
    // can pick one to unlock without leaking content.
    let locked_only = params.get("locked").is_some_and(|value| value == "1");

    let sessions = {
        let mut sm = state.session_manager.lock().await;
        sm.get_user_sessions(user_uuid)
            .map_err(|e| db_error("list sessions", e))?
    };

    let mut conn = state.conn.get().map_err(|e| {
        log::error!("Chat history DB pool error: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Database unavailable".to_string(),
        )
    })?;

    let bot_id_filter: Option<Uuid> = if bot_filter.is_empty() {
        None
    } else {
        use botcoresession::schema::bots::dsl::*;
        bots.filter(name.eq(&bot_filter))
            .select(id)
            .first::<Uuid>(&mut conn)
            .optional()
            .map_err(|e| db_error("bot lookup", e.to_string()))?
    };

    let mut filtered: Vec<botlib::models::UserSession> = sessions
        .into_iter()
        .filter(|s| match bot_id_filter {
            Some(bid) => s.bot_id == bid,
            None => true,
        })
        .collect();

    // Only conversations with at least one stored message belong in history.
    // Browsing the app creates empty session rows; without this filter they
    // push real conversations out of the page and show up as noise.
    if !filtered.is_empty() {
        use botcoresession::schema::message_history;
        let ids: Vec<Uuid> = filtered.iter().map(|s| s.id).collect();
        let with_messages: Vec<Uuid> = message_history::table
            .filter(message_history::session_id.eq_any(&ids))
            .select(message_history::session_id)
            .distinct()
            .load::<Uuid>(&mut conn)
            .map_err(|e| db_error("session messages", e.to_string()))?;
        let populated: std::collections::HashSet<Uuid> = with_messages.into_iter().collect();
        filtered.retain(|s| populated.contains(&s.id));
    }

    filtered.retain(|session| session.locked == locked_only);
    filtered.sort_by_key(|a| std::cmp::Reverse(a.updated_at));

    // Build the page AFTER readability is known: a conversation whose first
    // user turn cannot be decrypted (encrypted under a previous master key)
    // or is empty must not occupy one of the `limit` slots shown to the user.
    let mut out: Vec<HistorySession> = Vec::with_capacity(limit as usize);
    for sess in filtered {
        if out.len() >= limit as usize {
            break;
        }
        // Prefer the stored title: it is written from the first user turn when
        // the conversation starts and replaced when the user renames it.
        // Sessions created before titles were persisted fall back to the
        // decrypted first message.
        let stored = sess.title.trim();
        let has_stored_title = !stored.is_empty()
            && !stored.eq_ignore_ascii_case("new conversation")
            && !stored.eq_ignore_ascii_case("nova conversa")
            && !stored.eq_ignore_ascii_case("untitled");
        let title = if locked_only {
            // Never decrypt or expose the title of a locked conversation.
            String::new()
        } else if has_stored_title {
            stored.to_string()
        } else {
            let mut sm = state.session_manager.lock().await;
            match sm.get_first_user_message(sess.id) {
                Ok(Some(first)) => first,
                Ok(None) => continue,
                Err(e) => {
                    log::warn!("Failed to load title for session {}: {e}", sess.id);
                    continue;
                }
            }
        };
        let trimmed = title.trim().replace('\n', " ");
        if trimmed.is_empty() && !locked_only {
            continue;
        }
        let title = if trimmed.chars().count() > TITLE_MAX_CHARS {
            format!(
                "{}…",
                trimmed.chars().take(TITLE_MAX_CHARS).collect::<String>()
            )
        } else {
            trimmed
        };
        out.push(HistorySession {
            session_id: sess.id,
            bot_id: sess.bot_id,
            title,
            created_at: sess.created_at.to_rfc3339(),
            updated_at: sess.updated_at.to_rfc3339(),
            locked: sess.locked,
        });
    }

    Ok(Json(serde_json::json!({ "sessions": out })))
}
