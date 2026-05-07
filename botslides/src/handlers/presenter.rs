use crate::storage::{get_current_user_id, load_presentation_by_id, DriveOps};
use crate::types::{
    EndPresenterRequest, PresenterNotesResponse, PresenterSession, PresenterSessionResponse,
    StartPresenterRequest, UpdatePresenterRequest,
};
use crate::SlidesState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock};

static PRESENTER_SESSIONS: LazyLock<RwLock<HashMap<String, PresenterSession>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

pub async fn handle_start_presenter<D: DriveOps>(
    State(state): State<Arc<SlidesState<D>>>,
    Json(req): Json<StartPresenterRequest>,
) -> Result<Json<PresenterSessionResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    let drive = state.drive.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Drive not configured" })),
        )
    })?;

    let _presentation = load_presentation_by_id(drive, &user_id, &req.presentation_id)
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e })),
            )
        })?;

    let session = PresenterSession {
        id: uuid::Uuid::new_v4().to_string(),
        presentation_id: req.presentation_id,
        current_slide: 0,
        started_at: chrono::Utc::now(),
        elapsed_time: Some(0),
        is_paused: false,
        settings: req.settings,
    };

    if let Ok(mut sessions) = PRESENTER_SESSIONS.write() {
        sessions.insert(session.id.clone(), session.clone());
    }

    Ok(Json(PresenterSessionResponse { session }))
}

pub async fn handle_update_presenter<D: DriveOps>(
    State(_state): State<Arc<SlidesState<D>>>,
    Json(req): Json<UpdatePresenterRequest>,
) -> Result<Json<PresenterSessionResponse>, (StatusCode, Json<serde_json::Value>)> {
    let session = if let Ok(mut sessions) = PRESENTER_SESSIONS.write() {
        if let Some(session) = sessions.get_mut(&req.session_id) {
            if let Some(current_slide) = req.current_slide {
                session.current_slide = current_slide;
            }
            if let Some(is_paused) = req.is_paused {
                session.is_paused = is_paused;
            }
            if let Some(settings) = req.settings {
                session.settings = settings;
            }
            session.clone()
        } else {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Session not found" })),
            ));
        }
    } else {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "Failed to access sessions" })),
        ));
    };

    Ok(Json(PresenterSessionResponse { session }))
}

pub async fn handle_end_presenter<D: DriveOps>(
    State(_state): State<Arc<SlidesState<D>>>,
    Json(req): Json<EndPresenterRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if let Ok(mut sessions) = PRESENTER_SESSIONS.write() {
        sessions.remove(&req.session_id);
    }

    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn handle_get_presenter_notes<D: DriveOps>(
    State(state): State<Arc<SlidesState<D>>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<PresenterNotesResponse>, (StatusCode, Json<serde_json::Value>)> {
    let presentation_id = params.get("presentation_id").cloned().unwrap_or_default();
    let slide_index: usize = params
        .get("slide_index")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let user_id = get_current_user_id();

    let drive = state.drive.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Drive not configured" })),
        )
    })?;

    let presentation = load_presentation_by_id(drive, &user_id, &presentation_id)
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e })),
            )
        })?;

    if slide_index >= presentation.slides.len() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid slide index" })),
        ));
    }

    let notes = presentation.slides[slide_index].notes.clone();
    let next_slide_notes = if slide_index + 1 < presentation.slides.len() {
        presentation.slides[slide_index + 1].notes.clone()
    } else {
        None
    };

    Ok(Json(PresenterNotesResponse {
        slide_index,
        notes,
        next_slide_notes,
        next_slide_thumbnail: None,
    }))
}
