use crate::storage::DriveOps;
use crate::storage::get_current_user_id;
use crate::types::{
    CollaborationCursor, CollaborationSelection, ListCursorsResponse, ListSelectionsResponse,
    UpdateCursorRequest, UpdateSelectionRequest,
};
use crate::SlidesState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock};

static CURSORS: LazyLock<RwLock<HashMap<String, Vec<CollaborationCursor>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

static SELECTIONS: LazyLock<RwLock<HashMap<String, Vec<CollaborationSelection>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

pub async fn handle_update_cursor<D: DriveOps>(
    State(_state): State<Arc<SlidesState<D>>>,
    Json(req): Json<UpdateCursorRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    let cursor = CollaborationCursor {
        user_id: user_id.clone(),
        user_name: "User".to_string(),
        user_color: "#4285f4".to_string(),
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
    Json(req): Json<UpdateSelectionRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    let selection = CollaborationSelection {
        user_id: user_id.clone(),
        user_name: "User".to_string(),
        user_color: "#4285f4".to_string(),
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
