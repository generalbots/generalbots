use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post},
    Router,
};
use argon2::password_hash::{
    rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
};
use argon2::Argon2;
use diesel::prelude::*;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use botcore::shared::state::AppState;

use super::anonymous_auth::{resolve_chat_user_uuid, resolve_session_user};

const DEFAULT_SESSION_LIMIT: i64 = 50;
const TITLE_MAX_CHARS: usize = 80;
/// Idle lifetime of an unlock grant; the conversation re-locks itself after
/// this window without further reads (#1682).
const UNLOCK_TTL_SECS: i64 = 300;

#[derive(Clone)]
struct UnlockGrant {
    session_id: Uuid,
    user_id: Uuid,
    expires_at: chrono::DateTime<chrono::Utc>,
}

fn unlock_grants() -> &'static std::sync::Mutex<HashMap<String, UnlockGrant>> {
    static GRANTS: std::sync::OnceLock<std::sync::Mutex<HashMap<String, UnlockGrant>>> =
        std::sync::OnceLock::new();
    GRANTS.get_or_init(|| std::sync::Mutex::new(HashMap::new()))
}

/// Validates an unlock token for one session/user and extends its idle window.
fn unlock_token_valid(token: &str, session_id: Uuid, user_id: Uuid) -> bool {
    let Ok(mut grants) = unlock_grants().lock() else {
        return false;
    };
    let now = chrono::Utc::now();
    grants.retain(|_, grant| grant.expires_at > now);
    match grants.get_mut(token) {
        Some(grant) if grant.session_id == session_id && grant.user_id == user_id => {
            grant.expires_at = now + chrono::Duration::seconds(UNLOCK_TTL_SECS);
            true
        }
        _ => false,
    }
}

type HandlerError = (StatusCode, String);

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

fn bearer_user(request: &axum::extract::Request) -> Result<Uuid, HandlerError> {
    let (user_raw, _roles, is_authenticated) = resolve_session_user(request);
    if !is_authenticated || user_raw.is_empty() {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Authentication required".to_string(),
        ));
    }
    Ok(resolve_chat_user_uuid(&user_raw))
}

fn db_error(context: &str, e: String) -> HandlerError {
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

#[derive(serde::Deserialize)]
struct LockSessionBody {
    pin: String,
}

fn pin_is_valid(pin: &str) -> bool {
    (4..=8).contains(&pin.len()) && pin.chars().all(|character| character.is_ascii_digit())
}

fn hash_pin(pin: &str) -> Result<String, HandlerError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(pin.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|error| {
            log::error!("PIN hash failed: {error}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Could not store the conversation PIN".to_string(),
            )
        })
}

fn verify_pin(pin: &str, hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(pin.as_bytes(), &parsed)
        .is_ok()
}

async fn read_lock_body(
    request: axum::extract::Request,
) -> Result<LockSessionBody, HandlerError> {
    let bytes = axum::body::to_bytes(request.into_body(), 16 * 1024)
        .await
        .map_err(|_| (StatusCode::BAD_REQUEST, "bad body".to_string()))?;
    let body: LockSessionBody = serde_json::from_slice(&bytes)
        .map_err(|_| (StatusCode::BAD_REQUEST, "bad json".to_string()))?;
    if !pin_is_valid(&body.pin) {
        return Err((
            StatusCode::BAD_REQUEST,
            "PIN must be 4-8 digits".to_string(),
        ));
    }
    Ok(body)
}

fn set_session_lock(
    state: &Arc<AppState>,
    session_id: Uuid,
    user_uuid: Uuid,
    pin_hash: Option<&str>,
) -> Result<bool, HandlerError> {
    use botcoresession::schema::user_sessions;
    let mut conn = state.conn.get().map_err(|error| {
        log::error!("Chat lock DB pool error: {error}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Database unavailable".to_string(),
        )
    })?;
    let updated = diesel::update(
        user_sessions::table
            .filter(user_sessions::id.eq(session_id))
            .filter(user_sessions::user_id.eq(user_uuid.to_string())),
    )
    .set((
        user_sessions::locked.eq(pin_hash.is_some()),
        user_sessions::lock_pin_hash.eq(pin_hash),
        user_sessions::updated_at.eq(chrono::Utc::now()),
    ))
    .execute(&mut conn)
    .map_err(|error| db_error("set session lock", error.to_string()))?;
    Ok(updated > 0)
}

/// Locks one conversation behind a PIN (#1682). The PIN is Argon2-hashed; the
/// content itself stays encrypted at rest and hidden from every listing.
pub async fn handle_lock_chat_history_session(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<Uuid>,
    request: axum::extract::Request,
) -> Result<Json<serde_json::Value>, HandlerError> {
    let user_uuid = bearer_user(&request)?;
    let body = read_lock_body(request).await?;
    let hash = hash_pin(&body.pin)?;
    let updated = set_session_lock(&state, session_id, user_uuid, Some(&hash))?;
    if !updated {
        return Err((StatusCode::NOT_FOUND, "Conversation not found".to_string()));
    }
    Ok(Json(serde_json::json!({
        "locked": true,
        "session_id": session_id,
    })))
}

/// Unlocks a conversation after verifying the PIN. The listing stays filtered;
/// the client passes the PIN when fetching the messages.
pub async fn handle_unlock_chat_history_session(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<Uuid>,
    request: axum::extract::Request,
) -> Result<Json<serde_json::Value>, HandlerError> {
    let user_uuid = bearer_user(&request)?;
    let body = read_lock_body(request).await?;
    let session = {
        let mut sm = state.session_manager.lock().await;
        sm.get_session_by_id(session_id)
            .map_err(|error| db_error("load session", error))?
    };
    let Some(sess) = session else {
        return Err((StatusCode::NOT_FOUND, "Conversation not found".to_string()));
    };
    if sess.user_id != user_uuid {
        return Err((StatusCode::FORBIDDEN, "Not your conversation".to_string()));
    }
    let ok = sess
        .lock_pin_hash
        .as_deref()
        .is_some_and(|hash| verify_pin(&body.pin, hash));
    if !ok {
        return Err((StatusCode::FORBIDDEN, "Incorrect PIN".to_string()));
    }
    // The conversation stays locked in storage; the PIN only mints a
    // short-lived grant for reading it (idle re-lock).
    let token = Uuid::new_v4().to_string();
    if let Ok(mut grants) = unlock_grants().lock() {
        let now = chrono::Utc::now();
        grants.retain(|_, grant| grant.expires_at > now);
        grants.insert(
            token.clone(),
            UnlockGrant {
                session_id,
                user_id: user_uuid,
                expires_at: now + chrono::Duration::seconds(UNLOCK_TTL_SECS),
            },
        );
    }
    Ok(Json(serde_json::json!({
        "unlocked": true,
        "session_id": session_id,
        "unlock_token": token,
        "expires_in": UNLOCK_TTL_SECS,
    })))
}

pub fn configure_chat_history_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/chat/history/sessions", get(handle_chat_history_sessions))
        .route(
            "/api/chat/history/sessions/:session_id/messages",
            get(handle_chat_history_messages),
        )
        .route(
            "/api/chat/history/sessions/:session_id/lock",
            post(handle_lock_chat_history_session),
        )
        .route(
            "/api/chat/history/sessions/:session_id/unlock",
            post(handle_unlock_chat_history_session),
        )
        .route(
            "/api/chat/history/sessions/:session_id",
            delete(handle_delete_chat_history_session).put(handle_rename_chat_history_session),
        )
}
