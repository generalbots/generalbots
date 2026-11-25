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
    /// The raw database message_index, needed by truncate (#2562).
    message_index: i32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    attachments: Vec<HistoryAttachment>,
    #[serde(skip_serializing_if = "String::is_empty")]
    reasoning: String,
    /// The turn's structured events (`memory_event`, `agent_enable_request`,
    /// `files_produced`), so a reload renders the same blocks.
    #[serde(flatten)]
    events: serde_json::Map<String, serde_json::Value>,
}

#[derive(Serialize)]
struct HistoryAttachment {
    name: String,
    #[serde(rename = "type")]
    mime_type: String,
    size: usize,
    drive_path: String,
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
    if sess.user_id.to_string() != user_uuid.to_string() {
        // #3006 — CodeQL rust/cleartext-logging: botlog scrubs the identifiers
        // while the message keeps the mismatch traceable.
        botlog::safe_warn!(
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

    // #2562 — use the raw variant so metadata (reasoning, attachments) is
    // still present for decoding.  The model-facing `get_conversation_history`
    // strips the trailer; here we need it intact.
    let raw_history = {
        let mut sm = state.session_manager.lock().await;
        sm.get_conversation_history_raw(session_id, user_uuid, Some(limit), Some(offset))
            .map_err(|e| db_error("decrypt history", e))?
    };

    let mut msg_seq: i32 = 0;
    let messages: Vec<HistoryMessage> = raw_history
        .into_iter()
        .filter(|(role, _content, _idx)| *role == "user" || *role == "assistant")
        .map(|(role, raw_content, _db_idx)| {
            // Decode metadata (reasoning, attachments) from the raw stored
            // content.  The separator is stripped, returning visible text.
            let (visible, meta) =
                crate::core::bot::pipeline::llm::message_meta::decode(&raw_content);
            let attachments = if role == "user" {
                if let Some(ref m) = meta {
                    if !m.attachments.is_empty() {
                        m.attachments
                            .iter()
                            .map(|a| HistoryAttachment {
                                name: a.name.clone(),
                                mime_type: a.mime_type.clone(),
                                size: a.size,
                                drive_path: a.drive_path.clone(),
                            })
                            .collect()
                    } else {
                        parse_attachment_markers(visible)
                    }
                } else {
                    parse_attachment_markers(visible)
                }
            } else {
                Vec::new()
            };
            let reasoning = meta
                .as_ref()
                .map(|m| m.reasoning.clone())
                .unwrap_or_default();
            let events = meta.as_ref().map(|m| m.events.clone()).unwrap_or_default();
            let idx = msg_seq;
            msg_seq += 1;
            HistoryMessage {
                role,
                content: visible.to_string(),
                message_index: idx,
                attachments,
                reasoning,
                events,
            }
        })
        .collect();

    // `limit`/`offset` are expressed in turns (a turn = user + assistant), so
    // compare against the raw row count to decide whether older pages remain.
    let has_more = (offset + limit) * 2 < total;

    // #2562 — the `include_activities` param asks for the persisted run events
    // (gb.activity custom events) alongside the messages, so a reopened
    // conversation can show attachments, KB searches and tool runs.
    let activities = if params
        .get("include_activities")
        .is_some_and(|v| v == "1" || v == "true")
    {
        load_activities(&state.conn, session_id)
    } else {
        Vec::new()
    };

    Ok(Json(serde_json::json!({
        "session_id": session_id,
        "messages": messages,
        "has_more": has_more,
        "offset": offset,
        "limit": limit,
        "activities": activities,
    })))
}

/// Loads persisted `gb.activity` events for a session from `run_events`,
/// merges start/end pairs into one object per activity (with both timestamps
/// and duration), and returns them ordered by first-seen time.
fn load_activities(
    pool: &botcore::shared::utils::DbPool,
    session_id: Uuid,
) -> Vec<serde_json::Value> {
    use diesel::RunQueryDsl;

    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            log::warn!("activities: pool error: {e}");
            return Vec::new();
        }
    };
    let thread = session_id.to_string();
    let rows: Vec<ActivityRow> = match diesel::sql_query(
        "SELECT ts::timestamp AS ts, payload \
         FROM run_events \
         WHERE thread_id = $1 AND event_type = 'CUSTOM' \
         ORDER BY seq ASC \
         LIMIT 400",
    )
    .bind::<diesel::sql_types::Text, _>(&thread)
    .load::<ActivityRow>(&mut conn)
    {
        Ok(rows) => rows,
        Err(e) => {
            log::warn!("activities: query error: {e}");
            return Vec::new();
        }
    };
    let pairs: Vec<(crate::core::bot::pipeline::activity::Activity, String)> = rows
        .into_iter()
        .filter(|r| {
            r.payload
                .get("name")
                .and_then(|n| n.as_str())
                .is_some_and(|n| n == "gb.activity")
        })
        .filter_map(|r| {
            let inner = r.payload.get("payload")?.clone();
            let act: crate::core::bot::pipeline::activity::Activity =
                serde_json::from_value(inner).ok()?;
            Some((act, r.ts.to_string()))
        })
        .collect();
    crate::core::bot::pipeline::activity::merge_activity_pairs(pairs)
}

/// Extracts structured attachment info from the `[User attached a file stored
/// at inbox/fname; ...]` markers in persisted user messages.
fn parse_attachment_markers(content: &str) -> Vec<HistoryAttachment> {
    const MARKER: &str = "[User attached a file stored at ";
    let mut attachments = Vec::new();
    let mut search = content;
    while let Some(start) = search.find(MARKER) {
        let rest = &search[start + MARKER.len()..];
        let end = rest.find(';').unwrap_or(rest.len());
        let path = rest[..end].trim();
        if let Some(name) = path.strip_prefix("inbox/") {
            let mime = crate::core::bot::pipeline::activity::mime_from_name(name);
            attachments.push(HistoryAttachment {
                name: name.to_string(),
                mime_type: mime,
                size: 0,
                drive_path: path.to_string(),
            });
        }
        search = &rest[end..];
    }
    attachments
}

#[derive(diesel::QueryableByName)]
struct ActivityRow {
    #[diesel(sql_type = diesel::sql_types::Timestamp)]
    ts: chrono::NaiveDateTime,
    #[diesel(sql_type = diesel::sql_types::Jsonb)]
    payload: serde_json::Value,
}
