//! Locked conversations (#1682): PIN hashing, short-lived unlock grants and
//! the lock/unlock endpoints. Split from `chat_history.rs` to keep both files
//! inside the repository's size limit.

use std::collections::HashMap;
use std::sync::Arc;

use argon2::password_hash::{
    rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
};
use argon2::Argon2;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::post,
    Router,
};
use diesel::prelude::*;
use uuid::Uuid;

use botcore::shared::state::AppState;

use super::chat_history::{bearer_user, db_error, HandlerError};

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
pub(super) fn unlock_token_valid(token: &str, session_id: Uuid, user_id: Uuid) -> bool {
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

pub(super) fn verify_pin(pin: &str, hash: &str) -> bool {
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

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/api/chat/history/sessions/:session_id/lock",
            post(handle_lock_chat_history_session),
        )
        .route(
            "/api/chat/history/sessions/:session_id/unlock",
            post(handle_unlock_chat_history_session),
        )
}
