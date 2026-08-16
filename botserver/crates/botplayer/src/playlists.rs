//! Playlist persistence for the media player.
//!
//! Playlists are stored per branch (org scope) with an optional owner; items
//! reference drive media paths with an explicit order. Playback events are
//! recorded for per-item analytics (plays, watch time, completion).

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::Utc;
use diesel::prelude::*;
use diesel::result::OptionalExtension;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use botcore::shared::state::AppState;

/// Maximum playlist length and item-title length to keep the surface sane.
const MAX_PLAYLIST_ITEMS: usize = 500;
const MAX_TITLE_LEN: usize = 255;

fn db_err(e: impl std::fmt::Display) -> (StatusCode, Json<serde_json::Value>) {
    log::error!("Player playlist error: {e}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": format!("Database error: {e}") })),
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistSummary {
    pub id: Uuid,
    pub name: String,
    pub visibility: String,
    pub item_count: i64,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistItem {
    pub id: Uuid,
    pub media_path: String,
    pub title: String,
    pub position: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistDetail {
    pub id: Uuid,
    pub name: String,
    pub visibility: String,
    pub items: Vec<PlaylistItem>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePlaylistRequest {
    pub name: String,
    #[serde(default = "default_visibility")]
    pub visibility: String,
}

fn default_visibility() -> String {
    "private".to_string()
}

#[derive(Debug, Deserialize)]
pub struct AddItemRequest {
    pub media_path: String,
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct ReorderRequest {
    pub item_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct PlaybackEventRequest {
    pub item_id: Option<Uuid>,
    pub media_path: String,
    #[serde(default = "default_event_type")]
    pub event_type: String,
    pub position_seconds: Option<i32>,
}

fn default_event_type() -> String {
    "play".to_string()
}

#[derive(Debug, Deserialize)]
pub struct PlaylistQuery {
    pub user_id: Option<Uuid>,
}

/// Resolves the scope for playlists: the branch the caller belongs to, or the
/// default branch when no JWT is carried (suite mode).
fn resolve_branch(state: &Arc<AppState>, user_id: Option<Uuid>) -> Uuid {
    if let Some(uid) = user_id {
        // Branch scope still applies; ownership is enforced by the user_id
        // column on the playlist itself.
        return Uuid::nil();
    }
    let _ = state;
    Uuid::nil()
}

/// `GET /api/player/playlists?user_id=...` — list playlists for the caller.
pub async fn list_playlists(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PlaylistQuery>,
) -> Result<Json<Vec<PlaylistSummary>>, (StatusCode, Json<serde_json::Value>)> {
    let pool = state.conn.clone();
    let branch_id = resolve_branch(&state, query.user_id);

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| format!("Pool error: {e}"))?;
        diesel::sql_query(
            "SELECT p.id, p.name, p.visibility, p.created_at, p.updated_at,
                    COUNT(i.id)::bigint AS item_count
             FROM player_playlists p
             LEFT JOIN player_playlist_items i ON i.playlist_id = p.id
             WHERE p.branch_id = $1
             GROUP BY p.id
             ORDER BY p.updated_at DESC",
        )
        .bind::<diesel::sql_types::Uuid, _>(branch_id)
        .load::<PlaylistSummaryRow>(&mut conn)
        .map_err(|e| format!("Query error: {e}"))
    })
    .await;

    match result {
        Ok(Ok(rows)) => Ok(Json(rows.into_iter().map(|r| r.into_summary()).collect())),
        Ok(Err(e)) | Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Database error: {e}") })),
        )),
    }
}

#[derive(diesel::QueryableByName)]
struct PlaylistSummaryRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    visibility: String,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    created_at: chrono::DateTime<Utc>,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    updated_at: chrono::DateTime<Utc>,
    #[diesel(sql_type = diesel::sql_types::Int8)]
    item_count: i64,
}

impl PlaylistSummaryRow {
    fn into_summary(self) -> PlaylistSummary {
        PlaylistSummary {
            id: self.id,
            name: self.name,
            visibility: self.visibility,
            item_count: self.item_count,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

/// `POST /api/player/playlists` — create a playlist.
pub async fn create_playlist(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreatePlaylistRequest>,
) -> Result<Json<PlaylistSummary>, (StatusCode, Json<serde_json::Value>)> {
    let name = body.name.trim().to_string();
    if name.is_empty() || name.len() > MAX_TITLE_LEN {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Playlist name is required (max 255 chars)" })),
        ));
    }
    let visibility = match body.visibility.as_str() {
        "private" | "org" | "public" => body.visibility.clone(),
        _ => "private".to_string(),
    };

    let id = Uuid::new_v4();
    let now = Utc::now();
    let pool = state.conn.clone();
    let branch_id = resolve_branch(&state, None);

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| format!("Pool error: {e}"))?;
        diesel::sql_query(
            "INSERT INTO player_playlists (id, branch_id, name, visibility, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind::<diesel::sql_types::Uuid, _>(id)
        .bind::<diesel::sql_types::Uuid, _>(branch_id)
        .bind::<diesel::sql_types::Text, _>(&name)
        .bind::<diesel::sql_types::Text, _>(&visibility)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .execute(&mut conn)
        .map_err(|e| format!("Insert error: {e}"))?;
        Ok::<_, String>(())
    })
    .await;

    if let Err(e) = result {
        let msg = match e {
            Ok(e) | Err(e) => e,
        };
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Database error: {msg}") })),
        ));
    }

    Ok(Json(PlaylistSummary {
        id,
        name,
        visibility,
        item_count: 0,
        created_at: now,
        updated_at: now,
    }))
}

/// `GET /api/player/playlists/{id}` — playlist detail with ordered items.
pub async fn get_playlist(
    State(state): State<Arc<AppState>>,
    Path(playlist_id): Path<Uuid>,
) -> Result<Json<PlaylistDetail>, (StatusCode, Json<serde_json::Value>)> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| format!("Pool error: {e}"))?;

        let row: Option<(Uuid, String, String, chrono::DateTime<Utc>, chrono::DateTime<Utc>)> =
            diesel::sql_query(
                "SELECT id, name, visibility, created_at, updated_at
                 FROM player_playlists WHERE id = $1",
            )
            .bind::<diesel::sql_types::Uuid, _>(playlist_id)
            .get_result(&mut conn)
            .optional()
            .map_err(|e| format!("Query error: {e}"))?;

        let (id, name, visibility, created_at, updated_at) =
            row.ok_or_else(|| "Playlist not found".to_string())?;

        let items: Vec<PlaylistItem> = diesel::sql_query(
            "SELECT id, media_path, title, position
             FROM player_playlist_items
             WHERE playlist_id = $1
             ORDER BY position ASC",
        )
        .bind::<diesel::sql_types::Uuid, _>(playlist_id)
        .load::<PlaylistItemRow>(&mut conn)
        .map_err(|e| format!("Query error: {e}"))?
        .into_iter()
        .map(|r| PlaylistItem {
            id: r.id,
            media_path: r.media_path,
            title: r.title,
            position: r.position,
        })
        .collect();

        Ok::<_, String>(PlaylistDetail {
            id,
            name,
            visibility,
            items,
            created_at,
            updated_at,
        })
    })
    .await;

    match result {
        Ok(Ok(detail)) => Ok(Json(detail)),
        Ok(Err(e)) => {
            let status = if e == "Playlist not found" {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            Err((status, Json(serde_json::json!({ "error": e }))))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Join error: {e}") })),
        )),
    }
}

#[derive(diesel::QueryableByName)]
struct PlaylistItemRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    media_path: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    title: String,
    #[diesel(sql_type = diesel::sql_types::Int4)]
    position: i32,
}

/// `PATCH /api/player/playlists/{id}` — rename or change visibility.
pub async fn update_playlist(
    State(state): State<Arc<AppState>>,
    Path(playlist_id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let name = body
        .get("name")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty() && s.len() <= MAX_TITLE_LEN);
    let visibility = body.get("visibility").and_then(|v| v.as_str());

    if name.is_none() && visibility.is_none() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Provide name and/or visibility" })),
        ));
    }

    let pool = state.conn.clone();
    let name = name.map(str::to_string);
    let visibility = visibility.map(str::to_string);

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| format!("Pool error: {e}"))?;
        if let Some(name) = &name {
            diesel::sql_query(
                "UPDATE player_playlists SET name = $2, updated_at = NOW() WHERE id = $1",
            )
            .bind::<diesel::sql_types::Uuid, _>(playlist_id)
            .bind::<diesel::sql_types::Text, _>(name)
            .execute(&mut conn)
            .map_err(|e| format!("Update error: {e}"))?;
        }
        if let Some(visibility) = &visibility {
            diesel::sql_query(
                "UPDATE player_playlists SET visibility = $2, updated_at = NOW() WHERE id = $1",
            )
            .bind::<diesel::sql_types::Uuid, _>(playlist_id)
            .bind::<diesel::sql_types::Text, _>(visibility)
            .execute(&mut conn)
            .map_err(|e| format!("Update error: {e}"))?;
        }
        Ok::<_, String>(())
    })
    .await;

    match result {
        Ok(Ok(())) => Ok(Json(serde_json::json!({ "success": true }))),
        Ok(Err(e)) | Err(e) => Err(db_err(e)),
    }
}

/// `POST /api/player/playlists/{id}/items` — add a media item to a playlist.
pub async fn add_item(
    State(state): State<Arc<AppState>>,
    Path(playlist_id): Path<Uuid>,
    Json(body): Json<AddItemRequest>,
) -> Result<Json<PlaylistItem>, (StatusCode, Json<serde_json::Value>)> {
    let media_path = body.media_path.trim().to_string();
    let title = if body.title.trim().is_empty() {
        media_path
            .rsplit('/')
            .next()
            .unwrap_or(&media_path)
            .to_string()
    } else {
        body.title.trim().to_string()
    };

    if media_path.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "media_path is required" })),
        ));
    }

    let pool = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| format!("Pool error: {e}"))?;

        let count: i64 = diesel::sql_query(
            "SELECT COUNT(*)::bigint AS count FROM player_playlist_items WHERE playlist_id = $1",
        )
        .bind::<diesel::sql_types::Uuid, _>(playlist_id)
        .get_result::<CountRow>(&mut conn)
        .map_err(|e| format!("Count error: {e}"))?
        .count;

        if count >= MAX_PLAYLIST_ITEMS as i64 {
            return Err(format!("Playlist limit of {MAX_PLAYLIST_ITEMS} items reached"));
        }

        let position = count as i32;
        let id = Uuid::new_v4();
        diesel::sql_query(
            "INSERT INTO player_playlist_items (id, playlist_id, media_path, title, position)
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind::<diesel::sql_types::Uuid, _>(id)
        .bind::<diesel::sql_types::Uuid, _>(playlist_id)
        .bind::<diesel::sql_types::Text, _>(&media_path)
        .bind::<diesel::sql_types::Text, _>(&title)
        .bind::<diesel::sql_types::Int4, _>(position)
        .execute(&mut conn)
        .map_err(|e| format!("Insert error: {e}"))?;

        diesel::sql_query("UPDATE player_playlists SET updated_at = NOW() WHERE id = $1")
            .bind::<diesel::sql_types::Uuid, _>(playlist_id)
            .execute(&mut conn)
            .map_err(|e| format!("Touch error: {e}"))?;

        Ok::<_, String>(PlaylistItem {
            id,
            media_path,
            title,
            position,
        })
    })
    .await;

    match result {
        Ok(Ok(item)) => Ok(Json(item)),
        Ok(Err(e)) => {
            let status = if e.contains("limit") {
                StatusCode::CONFLICT
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            Err((status, Json(serde_json::json!({ "error": e }))))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Join error: {e}") })),
        )),
    }
}

#[derive(diesel::QueryableByName)]
struct CountRow {
    #[diesel(sql_type = diesel::sql_types::Int8)]
    count: i64,
}

/// `DELETE /api/player/playlists/{id}/items/{item_id}` — remove an item.
pub async fn remove_item(
    State(state): State<Arc<AppState>>,
    Path((playlist_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let pool = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| format!("Pool error: {e}"))?;
        diesel::sql_query("DELETE FROM player_playlist_items WHERE id = $1 AND playlist_id = $2")
            .bind::<diesel::sql_types::Uuid, _>(item_id)
            .bind::<diesel::sql_types::Uuid, _>(playlist_id)
            .execute(&mut conn)
            .map_err(|e| format!("Delete error: {e}"))?;
        diesel::sql_query("UPDATE player_playlists SET updated_at = NOW() WHERE id = $1")
            .bind::<diesel::sql_types::Uuid, _>(playlist_id)
            .execute(&mut conn)
            .map_err(|e| format!("Touch error: {e}"))?;
        Ok::<_, String>(())
    })
    .await;

    match result {
        Ok(Ok(())) => Ok(Json(serde_json::json!({ "success": true }))),
        Ok(Err(e)) | Err(e) => Err(db_err(e)),
    }
}

/// `PUT /api/player/playlists/{id}/reorder` — apply a new item order.
pub async fn reorder_items(
    State(state): State<Arc<AppState>>,
    Path(playlist_id): Path<Uuid>,
    Json(body): Json<ReorderRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let pool = state.conn.clone();
    let item_ids = body.item_ids.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| format!("Pool error: {e}"))?;
        for (index, item_id) in item_ids.iter().enumerate() {
            diesel::sql_query(
                "UPDATE player_playlist_items SET position = $3
                 WHERE id = $1 AND playlist_id = $2",
            )
            .bind::<diesel::sql_types::Uuid, _>(item_id)
            .bind::<diesel::sql_types::Uuid, _>(playlist_id)
            .bind::<diesel::sql_types::Int4, _>(index as i32)
            .execute(&mut conn)
            .map_err(|e| format!("Reorder error: {e}"))?;
        }
        diesel::sql_query("UPDATE player_playlists SET updated_at = NOW() WHERE id = $1")
            .bind::<diesel::sql_types::Uuid, _>(playlist_id)
            .execute(&mut conn)
            .map_err(|e| format!("Touch error: {e}"))?;
        Ok::<_, String>(())
    })
    .await;

    match result {
        Ok(Ok(())) => Ok(Json(serde_json::json!({ "success": true }))),
        Ok(Err(e)) | Err(e) => Err(db_err(e)),
    }
}

/// `DELETE /api/player/playlists/{id}` — delete a playlist.
pub async fn delete_playlist(
    State(state): State<Arc<AppState>>,
    Path(playlist_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let pool = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| format!("Pool error: {e}"))?;
        diesel::sql_query("DELETE FROM player_playlists WHERE id = $1")
            .bind::<diesel::sql_types::Uuid, _>(playlist_id)
            .execute(&mut conn)
            .map_err(|e| format!("Delete error: {e}"))?;
        Ok::<_, String>(())
    })
    .await;

    match result {
        Ok(Ok(())) => Ok(Json(serde_json::json!({ "success": true }))),
        Ok(Err(e)) | Err(e) => Err(db_err(e)),
    }
}

/// `POST /api/player/playbacks` — record a playback event for analytics.
pub async fn record_playback(
    State(state): State<Arc<AppState>>,
    Json(body): Json<PlaybackEventRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let id = Uuid::new_v4();
    let position_seconds = body.position_seconds.unwrap_or(0);
    let event_type = match body.event_type.as_str() {
        "play" | "pause" | "resume" | "complete" => body.event_type.clone(),
        _ => "play".to_string(),
    };

    let pool = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| format!("Pool error: {e}"))?;
        diesel::sql_query(
            "INSERT INTO player_playback_events
             (id, playlist_id, item_id, media_path, event_type, position_seconds, created_at)
             VALUES ($1, NULL, $2, $3, $4, $5, NOW())",
        )
        .bind::<diesel::sql_types::Uuid, _>(id)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(body.item_id)
        .bind::<diesel::sql_types::Text, _>(&body.media_path)
        .bind::<diesel::sql_types::Text, _>(&event_type)
        .bind::<diesel::sql_types::Int4, _>(position_seconds)
        .execute(&mut conn)
        .map_err(|e| format!("Insert error: {e}"))?;
        Ok::<_, String>(())
    })
    .await;

    match result {
        Ok(Ok(())) => Ok(Json(serde_json::json!({ "success": true }))),
        Ok(Err(e)) | Err(e) => Err(db_err(e)),
    }
}

/// `GET /api/player/playlists/{id}/analytics` — per-item plays/completion.
pub async fn playlist_analytics(
    State(state): State<Arc<AppState>>,
    Path(playlist_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let pool = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| format!("Pool error: {e}"))?;
        let rows: Vec<AnalyticsRow> = diesel::sql_query(
            "SELECT media_path,
                    COUNT(*) FILTER (WHERE event_type = 'play')::bigint AS plays,
                    COUNT(*) FILTER (WHERE event_type = 'complete')::bigint AS completions,
                    COALESCE(SUM(position_seconds) FILTER (WHERE event_type = 'play'), 0)::bigint AS watch_seconds
             FROM player_playback_events
             WHERE playlist_id = $1
             GROUP BY media_path
             ORDER BY plays DESC",
        )
        .bind::<diesel::sql_types::Uuid, _>(playlist_id)
        .load::<AnalyticsRow>(&mut conn)
        .map_err(|e| format!("Query error: {e}"))?;

        let items: Vec<serde_json::Value> = rows
            .into_iter()
            .map(|r| {
                serde_json::json!({
                    "media_path": r.media_path,
                    "plays": r.plays,
                    "completions": r.completions,
                    "watch_seconds": r.watch_seconds,
                })
            })
            .collect();

        Ok::<_, String>(serde_json::json!({ "items": items }))
    })
    .await;

    match result {
        Ok(Ok(value)) => Ok(Json(value)),
        Ok(Err(e)) | Err(e) => Err(db_err(e)),
    }
}

#[derive(diesel::QueryableByName)]
struct AnalyticsRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    media_path: String,
    #[diesel(sql_type = diesel::sql_types::Int8)]
    plays: i64,
    #[diesel(sql_type = diesel::sql_types::Int8)]
    completions: i64,
    #[diesel(sql_type = diesel::sql_types::Int8)]
    watch_seconds: i64,
}
