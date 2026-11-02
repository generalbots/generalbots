//! User memory API (GB6-16).
//!
//! Settings → Memory edits these rows; the chat pipeline reads them into the
//! system prompt. Entries with no bot id apply to every bot.

use std::sync::Arc;

use axum::extract::{Json, Path, State};
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use axum::Json as AxumJson;
use diesel::prelude::*;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use botcore::shared::state::AppState;
use botschema::chat_preferences;

use super::settings_api::{get_conn, resolve_user_id};

#[derive(Debug, Deserialize)]
pub struct MemoryUpsert {
    pub key: String,
    pub value: String,
    #[serde(default)]
    pub bot_id: Option<Uuid>,
}

fn auth_error(status: axum::http::StatusCode) -> (axum::http::StatusCode, AxumJson<serde_json::Value>) {
    (
        status,
        AxumJson(json!({ "error": "A valid session token is required" })),
    )
}

/// GET /api/user/memory
pub async fn list(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let mut conn = match get_conn(&state) {
        Some(conn) => conn,
        None => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                AxumJson(json!({ "error": "Database unavailable" })),
            )
        }
    };
    let user_id = match resolve_user_id(&state, &headers) {
        Ok(id) => id,
        Err((status, _)) => return auth_error(status),
    };
    let rows = chat_preferences::table
        .filter(chat_preferences::user_id.eq(user_id))
        .order(chat_preferences::updated_at.desc())
        .select((
            chat_preferences::memory_key,
            chat_preferences::memory_value,
            chat_preferences::bot_id,
            chat_preferences::updated_at,
        ))
        .load::<(String, String, Option<Uuid>, chrono::DateTime<chrono::Utc>)>(&mut conn);
    match rows {
        Ok(rows) => {
            let memories: Vec<serde_json::Value> = rows
                .into_iter()
                .map(|(key, value, bot_id, updated_at)| {
                    json!({
                        "key": key,
                        "value": value,
                        "bot_id": bot_id,
                        "updated_at": updated_at,
                    })
                })
                .collect();
            (
                axum::http::StatusCode::OK,
                AxumJson(json!({ "memories": memories })),
            )
        }
        Err(error) => {
            log::warn!("user memory list failed: {error}");
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                AxumJson(json!({ "error": "Memory could not be read" })),
            )
        }
    }
}

/// PUT /api/user/memory
pub async fn upsert(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<MemoryUpsert>,
) -> impl IntoResponse {
    let key = body.key.trim();
    if key.is_empty() || key.len() > 128 {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            AxumJson(json!({ "error": "key must be 1-128 characters" })),
        );
    }
    if body.value.len() > 2000 {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            AxumJson(json!({ "error": "value is limited to 2000 characters" })),
        );
    }
    let mut conn = match get_conn(&state) {
        Some(conn) => conn,
        None => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                AxumJson(json!({ "error": "Database unavailable" })),
            )
        }
    };
    let user_id = match resolve_user_id(&state, &headers) {
        Ok(id) => id,
        Err((status, _)) => return auth_error(status),
    };
    // The unique index is expression-based (NULL bot id normalised), so the
    // upsert must name the same expression rather than a constraint.
    let result = diesel::sql_query(
        "INSERT INTO chat_preferences (user_id, bot_id, memory_key, memory_value) \
         VALUES ($1, $2, $3, $4) \
         ON CONFLICT (user_id, COALESCE(bot_id, '00000000-0000-0000-0000-000000000000'::uuid), memory_key) \
         DO UPDATE SET memory_value = EXCLUDED.memory_value, updated_at = NOW()",
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(body.bot_id)
    .bind::<diesel::sql_types::Text, _>(key)
    .bind::<diesel::sql_types::Text, _>(body.value.trim())
    .execute(&mut conn);
    match result {
        Ok(_) => (
            axum::http::StatusCode::OK,
            AxumJson(json!({ "success": true, "key": key })),
        ),
        Err(error) => {
            log::warn!("user memory upsert failed: {error}");
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                AxumJson(json!({ "error": "Memory could not be saved" })),
            )
        }
    }
}

/// DELETE /api/user/memory/:key
pub async fn delete(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(key): Path<String>,
) -> impl IntoResponse {
    let mut conn = match get_conn(&state) {
        Some(conn) => conn,
        None => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                AxumJson(json!({ "error": "Database unavailable" })),
            )
        }
    };
    let user_id = match resolve_user_id(&state, &headers) {
        Ok(id) => id,
        Err((status, _)) => return auth_error(status),
    };
    let removed = diesel::delete(
        chat_preferences::table
            .filter(chat_preferences::user_id.eq(user_id))
            .filter(chat_preferences::memory_key.eq(&key)),
    )
    .execute(&mut conn);
    match removed {
        Ok(count) => (
            axum::http::StatusCode::OK,
            AxumJson(json!({ "success": true, "removed": count })),
        ),
        Err(error) => {
            log::warn!("user memory delete failed: {error}");
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                AxumJson(json!({ "error": "Memory could not be deleted" })),
            )
        }
    }
}
