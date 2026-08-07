//! Campaign run control (#731): pause/resume/stop + durable event monitor.
//!
//! Per-campaign flags are persisted in `marketing_campaigns` so the durable
//! worker honours them across restarts; global worker control is in memory.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

use crate::schema::marketing_campaigns;
use crate::state::AppState;

/// Applies a run-control action to a single campaign. The flag is persisted in
/// `marketing_campaigns` so the durable worker honours it across restarts.
pub async fn control_campaign(
    State(state): State<Arc<AppState>>,
    Path((id, action)): Path<(Uuid, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let exists: bool = diesel::select(diesel::dsl::exists(
        marketing_campaigns::table.filter(marketing_campaigns::id.eq(id)),
    ))
    .get_result(&mut conn)
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;
    if !exists {
        return Err((StatusCode::NOT_FOUND, "Campaign not found".to_string()));
    }

    let (pause, stop, status) = match action.as_str() {
        "pause" => (Some(true), None, Some("paused".to_string())),
        "resume" => (Some(false), None, Some("scheduled".to_string())),
        "stop" => (Some(false), Some(true), Some("stopped".to_string())),
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Unknown action: {action}"),
            ))
        }
    };

    let now = chrono::Utc::now();
    let completed_at = if action == "stop" {
        Some(now)
    } else if action == "resume" {
        None
    } else {
        marketing_campaigns::table
            .filter(marketing_campaigns::id.eq(id))
            .select(marketing_campaigns::completed_at)
            .first::<Option<DateTime<Utc>>>(&mut conn)
            .unwrap_or(None)
    };

    diesel::update(marketing_campaigns::table.filter(marketing_campaigns::id.eq(id)))
        .set((
            marketing_campaigns::pause_requested.eq(pause),
            marketing_campaigns::stop_requested.eq(stop),
            marketing_campaigns::status.eq(status.clone()),
            marketing_campaigns::completed_at.eq(completed_at),
            marketing_campaigns::updated_at.eq(now),
        ))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

    if let Some(state_ref) = state.worker.as_ref() {
        state_ref
            .log_event(id, "all", &action, None, status.as_deref(), None)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }

    Ok(Json(serde_json::json!({
        "campaign_id": id,
        "action": action,
        "status": status,
    })))
}

/// Global worker control. Pause/resume/stop apply to every active campaign and
/// are held in memory (they do not need to survive a restart).
pub async fn control_worker(
    State(state): State<Arc<AppState>>,
    Path(action): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let Some(worker) = state.worker.as_ref() else {
        return Err((StatusCode::SERVICE_UNAVAILABLE, "Worker not running".to_string()));
    };
    match action.as_str() {
        "pause" => worker.set_global_pause(true),
        "resume" => worker.set_global_pause(false),
        // "start" clears both the stop latch and pause so the worker resumes.
        "start" | "reset" => {
            worker.set_global_stop(false);
            worker.set_global_pause(false);
        }
        "stop" => worker.set_global_stop(true),
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Unknown action: {action}"),
            ))
        }
    }
    Ok(Json(serde_json::json!({
        "global_control": action,
        "status": "ok",
    })))
}

/// Realtime campaign event log used by the marketing monitor.
pub async fn campaign_events(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct EventRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        channel: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Text)]
        event_type: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        recipient_email: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        status: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        error_message: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        created_at: chrono::DateTime<Utc>,
    }

    let events: Vec<EventRow> = diesel::sql_query(
        "SELECT id, channel::varchar, event_type::varchar, recipient_email::varchar, \
                status::varchar, error_message::text, created_at \
         FROM marketing_campaign_events \
         WHERE campaign_id = $1 \
         ORDER BY created_at DESC LIMIT 200",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .load(&mut conn)
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    let rows: Vec<serde_json::Value> = events
        .into_iter()
        .map(|e| {
            serde_json::json!({
                "id": e.id,
                "channel": e.channel,
                "event_type": e.event_type,
                "recipient_email": e.recipient_email,
                "status": e.status,
                "error_message": e.error_message,
                "created_at": e.created_at,
            })
        })
        .collect();

    Ok(Json(serde_json::json!({ "events": rows })))
}
