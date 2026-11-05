use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get},
    Router,
};
use diesel::prelude::*;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use botcore::shared::state::AppState;

use super::anonymous_auth::{resolve_chat_user_uuid, resolve_session_user};
use super::chat_history_lock::{unlock_token_valid, verify_pin};

const DEFAULT_SESSION_LIMIT: i64 = 50;
const TITLE_MAX_CHARS: usize = 80;

pub(super) type HandlerError = (StatusCode, String);

#[derive(Serialize)]
struct HistorySession {
    session_id: Uuid,
    bot_id: Uuid,
    title: String,
    created_at: String,
    updated_at: String,
    locked: bool,
}

#[derive(Serialize)]
struct HistoryMessage {
    role: String,
    content: String,
}

pub(super) fn bearer_user(request: &axum::extract::Request) -> Result<Uuid, HandlerError> {
    let (user_raw, _roles, is_authenticated) = resolve_session_user(request);
    if !is_authenticated || user_raw.is_empty() {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Authentication required".to_string(),
        ));
    }
    Ok(resolve_chat_user_uuid(&user_raw))
}

pub(super) fn db_error(context: &str, e: String) -> HandlerError {
    log::error!("Chat history error ({context}): {e}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Failed to load chat history".to_string(),
    )
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
        let populated: std::collections::HashSet<Uuid> =
            with_messages.into_iter().collect();
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
            format!("{}…", trimmed.chars().take(TITLE_MAX_CHARS).collect::<String>())
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

#[derive(serde::Deserialize)]
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
    let body: RenameSessionBody =
        serde_json::from_slice(&bytes).map_err(|_| (StatusCode::BAD_REQUEST, "bad json".to_string()))?;
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

fn escape_preview_text(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// `GET /api/chat/attachments/preview?bot=&name=&access_token=`
///
/// User uploads are staged in the bot's Drive `inbox/`; the chat iframe
/// cannot attach an Authorization header, so the same suite token accepted by
/// `/api/agent/vm/file` is accepted as a query parameter. Office packages are
/// converted to sanitized, script-free HTML with `office_html`.
pub async fn handle_attachment_preview(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
    request: axum::extract::Request,
) -> Result<axum::response::Response, HandlerError> {
    let token = params
        .get("access_token")
        .cloned()
        .or_else(|| {
            request
                .headers()
                .get(axum::http::header::AUTHORIZATION)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.strip_prefix("Bearer "))
                .map(str::to_string)
        })
        .unwrap_or_default();
    let (user_raw, _roles) = super::anonymous_auth::resolve_token_user(&token)
        .ok_or((StatusCode::UNAUTHORIZED, "Authentication required".to_string()))?;
    let user_uuid = resolve_chat_user_uuid(&user_raw);

    let name = params.get("name").cloned().unwrap_or_default();
    let bot_name = params.get("bot").cloned().unwrap_or_default();
    if name.is_empty()
        || name.contains('/')
        || name.contains('\\')
        || name.contains("..")
        || bot_name.is_empty()
    {
        return Err((StatusCode::BAD_REQUEST, "invalid attachment name".to_string()));
    }

    let mut conn = state
        .conn
        .get()
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB pool: {error}")))?;
    let bot_id: Uuid = botschema::bots::table
        .filter(botschema::bots::name.eq(&bot_name))
        .select(botschema::bots::id)
        .first(&mut *conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Bot not found".to_string()))?;

    #[derive(diesel::QueryableByName)]
    struct CountRow {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        total: i64,
    }
    let sessions = diesel::sql_query(
        "SELECT count(*) AS total FROM user_sessions WHERE user_id = $1 AND bot_id = $2",
    )
    // `user_sessions.user_id` stores the UUID as text.
    .bind::<diesel::sql_types::Text, _>(user_uuid.to_string())
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .get_result::<CountRow>(&mut *conn)
    .map_err(|error| {
        log::error!("attachment preview session lookup failed: {error}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Could not verify the conversation".to_string(),
        )
    })?
    .total;
    if sessions == 0 {
        return Err((StatusCode::FORBIDDEN, "No access to this bot".to_string()));
    }

    let location = botbasic_core::utils::bot_drive_location_for_name(&mut conn, &bot_name);
    let key = location.key_for(&format!("inbox/{name}"));
    let drive = state
        .drive
        .as_ref()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "Drive is not configured".to_string()))?;
    let bytes = drive
        .get_object(&location.bucket, &key)
        .await
        .map_err(|error| {
            log::warn!("attachment preview: {key} not found: {error}");
            (StatusCode::NOT_FOUND, "Attachment not found".to_string())
        })?;

    let extension = name.rsplit('.').next().unwrap_or("").to_lowercase();
    let (content_type, body, sandboxed) = match extension.as_str() {
        "docx" | "xlsx" | "pptx" => {
            let html = botcore::kb::document_processor::office_html(&name, &bytes)
                .and_then(|result| result.ok())
                .ok_or((
                    StatusCode::UNSUPPORTED_MEDIA_TYPE,
                    "This Office file could not be converted for preview".to_string(),
                ))?;
            ("text/html; charset=utf-8", html.into_bytes(), true)
        }
        "txt" | "md" | "csv" | "json" | "log" => {
            // Plain-text uploads get the same preview surface as documents;
            // the body is escaped and capped so a large log cannot bloat the
            // conversation (#1693).
            let text: String = String::from_utf8_lossy(&bytes).chars().take(200_000).collect();
            let escaped = escape_preview_text(&text);
            let html = format!(
                "<!doctype html><html><head><meta charset=\"utf-8\">\
                 <style>body{{margin:0;padding:12px;font:13px/1.5 ui-monospace,Menlo,monospace;\
                 white-space:pre-wrap;word-break:break-word;color:#0f172a;background:#fff}}</style>\
                 </head><body>{escaped}</body></html>"
            );
            ("text/html; charset=utf-8", html.into_bytes(), true)
        }
        "pdf" => ("application/pdf", bytes, false),
        "png" => ("image/png", bytes, false),
        "jpg" | "jpeg" => ("image/jpeg", bytes, false),
        "gif" => ("image/gif", bytes, false),
        "webp" => ("image/webp", bytes, false),
        _ => {
            return Err((
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "No inline preview for this file type".to_string(),
            ))
        }
    };

    let csp = if sandboxed {
        "default-src 'none'; style-src 'unsafe-inline'; img-src data:; frame-ancestors 'self'"
    } else {
        "default-src 'none'; img-src 'self' data:; frame-ancestors 'self'"
    };
    axum::response::Response::builder()
        .header(axum::http::header::CONTENT_TYPE, content_type)
        .header(axum::http::header::CACHE_CONTROL, "no-store")
        .header(axum::http::header::X_FRAME_OPTIONS, "SAMEORIGIN")
        .header(axum::http::header::CONTENT_SECURITY_POLICY, csp)
        .body(axum::body::Body::from(body))
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, format!("response: {error}")))
}

pub fn configure_chat_history_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/chat/history/sessions", get(handle_chat_history_sessions))
        .route(
            "/api/chat/history/sessions/:session_id/messages",
            get(handle_chat_history_messages),
        )
        .merge(super::chat_history_lock::router())
        .route(
            "/api/chat/attachments/preview",
            get(handle_attachment_preview),
        )
        .route(
            "/api/chat/history/sessions/:session_id",
            delete(handle_delete_chat_history_session).put(handle_rename_chat_history_session),
        )
}
