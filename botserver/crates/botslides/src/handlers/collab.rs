use crate::storage::DriveOps;
use crate::types::{
    CollaborationCursor, CollaborationSelection, ListCursorsResponse, ListSelectionsResponse,
    UpdateCursorRequest, UpdateSelectionRequest,
};
use crate::SlidesState;
use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    Json,
};
use botsecurity_auth::auth_api::types::AuthenticatedUser;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock};

static CURSORS: LazyLock<RwLock<HashMap<String, Vec<CollaborationCursor>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

static SELECTIONS: LazyLock<RwLock<HashMap<String, Vec<CollaborationSelection>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

// Resolve the caller's identity from the auth extension (mirrors the WebSocket
// path, which already distinguishes users). Falls back to a stable default for
// anonymous/session-less callers so cursors never collide silently.
fn collab_user_id(user: &AuthenticatedUser) -> String {
    if let Some(ref email) = user.email {
        if !email.is_empty() && email != "session-user" {
            return email.clone();
        }
    }
    if user.user_id.is_nil() {
        "default".to_string()
    } else {
        user.user_id.to_string()
    }
}

fn collab_user_name(user: &AuthenticatedUser) -> String {
    if let Some(ref email) = user.email {
        if !email.is_empty() && email != "session-user" {
            return email.split('@').next().unwrap_or(email).to_string();
        }
    }
    if !user.username.is_empty() {
        user.username.clone()
    } else {
        "User".to_string()
    }
}

fn collab_user_color(user_id: &str) -> String {
    const PALETTE: [&str; 12] = [
        "#FF6B6B", "#4ECDC4", "#45B7D1", "#96CEB4", "#FFEAA7", "#DDA0DD",
        "#98D8C8", "#F7DC6F", "#BB8FCE", "#85C1E9", "#F1948A", "#82E0AA",
    ];
    let mut h: u32 = 0;
    for b in user_id.bytes() {
        h = h.wrapping_mul(31).wrapping_add(b as u32);
    }
    PALETTE[(h % PALETTE.len() as u32) as usize].to_string()
}

pub async fn handle_update_cursor<D: DriveOps>(
    State(_state): State<Arc<SlidesState<D>>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<UpdateCursorRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = collab_user_id(&user);

    let cursor = CollaborationCursor {
        user_id: user_id.clone(),
        user_name: collab_user_name(&user),
        user_color: collab_user_color(&user_id),
        slide_index: req.slide_index,
        element_id: req.element_id,
        x: req.x,
        y: req.y,
        last_activity: Utc::now(),
    };

    if let Ok(mut cursors) = CURSORS.write() {
        let presentation_cursors = cursors.entry(req.presentation_id.clone()).or_default();
        presentation_cursors.retain(|c| c.user_id != user_id);
        presentation_cursors.push(cursor);
    }

    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn handle_update_selection<D: DriveOps>(
    State(_state): State<Arc<SlidesState<D>>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<UpdateSelectionRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = collab_user_id(&user);

    let selection = CollaborationSelection {
        user_id: user_id.clone(),
        user_name: collab_user_name(&user),
        user_color: collab_user_color(&user_id),
        slide_index: req.slide_index,
        element_ids: req.element_ids,
    };

    if let Ok(mut selections) = SELECTIONS.write() {
        let presentation_selections = selections.entry(req.presentation_id.clone()).or_default();
        presentation_selections.retain(|s| s.user_id != user_id);
        presentation_selections.push(selection);
    }

    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn handle_list_cursors<D: DriveOps>(
    State(_state): State<Arc<SlidesState<D>>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ListCursorsResponse>, (StatusCode, Json<serde_json::Value>)> {
    let presentation_id = params.get("presentation_id").cloned().unwrap_or_default();

    let cursors = if let Ok(cursors_map) = CURSORS.read() {
        cursors_map
            .get(&presentation_id)
            .cloned()
            .unwrap_or_default()
    } else {
        vec![]
    };

    Ok(Json(ListCursorsResponse { cursors }))
}

pub async fn handle_list_selections<D: DriveOps>(
    State(_state): State<Arc<SlidesState<D>>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ListSelectionsResponse>, (StatusCode, Json<serde_json::Value>)> {
    let presentation_id = params.get("presentation_id").cloned().unwrap_or_default();

    let selections = if let Ok(selections_map) = SELECTIONS.read() {
        selections_map
            .get(&presentation_id)
            .cloned()
            .unwrap_or_default()
    } else {
        vec![]
    };

    Ok(Json(ListSelectionsResponse { selections }))
}
