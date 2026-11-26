//! User memory API (GB7-16).
//!
//! Settings → Memory lists, edits and deletes these rows and turns memory on
//! or off; the chat pipeline reads them into every conversation. Values are
//! sealed per person (`memory_store`). Entries with no bot id apply to every
//! bot.

use std::sync::Arc;

use axum::extract::{Json, Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json as AxumJson;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use botcore::shared::state::AppState;

use super::memory_store;
use super::settings_api::{get_conn, resolve_user_id};

/// Rows the list returns at most.
const LIST_LIMIT: i64 = 500;

#[derive(Debug, Deserialize)]
pub struct MemoryUpsert {
    /// Existing key to replace; a new entry gets a key from its value.
    #[serde(default)]
    pub key: Option<String>,
    pub value: String,
    #[serde(default)]
    pub bot_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct MemorySwitch {
    pub enabled: bool,
}

type Reply = (StatusCode, AxumJson<Value>);

fn error(status: StatusCode, message: &str) -> Reply {
    (status, AxumJson(json!({ "error": message })))
}

type Conn = diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>;

fn person(state: &Arc<AppState>, headers: &HeaderMap) -> Result<(Conn, Uuid), Reply> {
    let user_id = resolve_user_id(state, headers)
        .map_err(|(status, _)| error(status, "A valid session token is required"))?;
    let conn = get_conn(state)
        .ok_or_else(|| error(StatusCode::INTERNAL_SERVER_ERROR, "Database unavailable"))?;
    Ok((conn, user_id))
}

fn entry_json(entry: memory_store::MemoryEntry) -> Value {
    json!({
        "key": entry.key,
        "value": entry.value,
        "bot_id": entry.bot_id,
        "updated_at": entry.updated_at,
    })
}

/// GET /api/user/memory → `{ memories: [{key, value, bot_id, updated_at}], enabled }`
pub async fn list(State(state): State<Arc<AppState>>, headers: HeaderMap) -> impl IntoResponse {
    let (mut conn, user_id) = match person(&state, &headers) {
        Ok(found) => found,
        Err(reply) => return reply,
    };
    let enabled = memory_store::is_enabled(&mut conn, user_id);
    match memory_store::list(&mut conn, user_id, None, LIST_LIMIT) {
        Ok(rows) => {
            let memories: Vec<Value> = rows.into_iter().map(entry_json).collect();
            (StatusCode::OK, AxumJson(json!({ "memories": memories, "enabled": enabled })))
        }
        Err(failure) => {
            log::warn!("user memory list failed: {failure}");
            error(StatusCode::INTERNAL_SERVER_ERROR, "Memory could not be read")
        }
    }
}

/// PUT /api/user/memory `{ key?, value, bot_id? }` → `{ success, key }`
pub async fn upsert(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<MemoryUpsert>,
) -> impl IntoResponse {
    let value = body.value.trim();
    if value.is_empty() || value.chars().count() > memory_store::MAX_VALUE_CHARS {
        return error(StatusCode::BAD_REQUEST, "value must be 1-2000 characters");
    }
    let given_key = body.key.as_deref().map(str::trim).filter(|key| !key.is_empty());
    if given_key.is_some_and(|key| key.len() > 128) {
        return error(StatusCode::BAD_REQUEST, "key is limited to 128 characters");
    }
    let (mut conn, user_id) = match person(&state, &headers) {
        Ok(found) => found,
        Err(reply) => return reply,
    };
    let key = given_key.map_or_else(|| memory_store::fact_key(user_id, value), str::to_string);
    match memory_store::save(&mut conn, user_id, body.bot_id, &key, value) {
        Ok(_) => (StatusCode::OK, AxumJson(json!({ "success": true, "key": key }))),
        Err(failure) => {
            log::warn!("user memory upsert failed: {failure}");
            error(StatusCode::INTERNAL_SERVER_ERROR, "Memory could not be saved")
        }
    }
}

/// DELETE /api/user/memory/:key → `{ success, removed }`
pub async fn delete(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(key): Path<String>,
) -> impl IntoResponse {
    let (mut conn, user_id) = match person(&state, &headers) {
        Ok(found) => found,
        Err(reply) => return reply,
    };
    match memory_store::delete_keys(&mut conn, user_id, None, &[key]) {
        Ok(count) => (StatusCode::OK, AxumJson(json!({ "success": true, "removed": count }))),
        Err(failure) => {
            log::warn!("user memory delete failed: {failure}");
            error(StatusCode::INTERNAL_SERVER_ERROR, "Memory could not be deleted")
        }
    }
}

/// DELETE /api/user/memory → `{ success, removed }` (every entry of the person)
pub async fn clear(State(state): State<Arc<AppState>>, headers: HeaderMap) -> impl IntoResponse {
    let (mut conn, user_id) = match person(&state, &headers) {
        Ok(found) => found,
        Err(reply) => return reply,
    };
    match memory_store::clear(&mut conn, user_id) {
        Ok(count) => (StatusCode::OK, AxumJson(json!({ "success": true, "removed": count }))),
        Err(failure) => {
            log::warn!("user memory clear failed: {failure}");
            error(StatusCode::INTERNAL_SERVER_ERROR, "Memory could not be deleted")
        }
    }
}

/// GET /api/user/memory-settings → `{ enabled }`
pub async fn get_switch(State(state): State<Arc<AppState>>, headers: HeaderMap) -> impl IntoResponse {
    let (mut conn, user_id) = match person(&state, &headers) {
        Ok(found) => found,
        Err(reply) => return reply,
    };
    let enabled = memory_store::is_enabled(&mut conn, user_id);
    (StatusCode::OK, AxumJson(json!({ "enabled": enabled })))
}

/// PUT /api/user/memory-settings `{ enabled }` → `{ success, enabled }`
pub async fn put_switch(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<MemorySwitch>,
) -> impl IntoResponse {
    let (mut conn, user_id) = match person(&state, &headers) {
        Ok(found) => found,
        Err(reply) => return reply,
    };
    match memory_store::set_enabled(&mut conn, user_id, body.enabled) {
        Ok(_) => (StatusCode::OK, AxumJson(json!({ "success": true, "enabled": body.enabled }))),
        Err(failure) => {
            log::warn!("user memory switch failed: {failure}");
            error(StatusCode::INTERNAL_SERVER_ERROR, "Memory setting could not be saved")
        }
    }
}
