//! Theme, master and transition operations.

use crate::storage::{get_current_user_id, DriveOps};
use crate::types::{
    ApplyThemeRequest, ApplyTransitionToAllRequest, RemoveTransitionRequest, SaveResponse,
    SetTransitionRequest, TransitionConfig,
};
use crate::SlidesState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use chrono::Utc;
use std::sync::Arc;

pub async fn handle_apply_theme<D: DriveOps>(
    State(state): State<Arc<SlidesState<D>>>,
    Json(req): Json<ApplyThemeRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut presentation = load_pres!(state, &user_id, &req.presentation_id);

    // The master picker posts to the same route; it changes each slide's
    // layout and must leave the current theme untouched.
    if req.action.as_deref() == Some("set_master") {
        let master = req.master_id.unwrap_or_else(|| "default".to_string());
        for slide in &mut presentation.slides {
            slide.layout = master.clone();
        }
        presentation.updated_at = Utc::now();
        save_pres!(state, &user_id, &presentation);
        return Ok(Json(SaveResponse {
            id: req.presentation_id,
            success: true,
            message: Some("Master applied".to_string()),
        }));
    }

    let theme = match req.theme {
        Some(theme) => theme,
        None => match req.theme_name.as_deref() {
            Some(name) => crate::utils::theme_from_name(name),
            None => crate::utils::create_default_theme(),
        },
    };
    presentation.theme = theme;
    presentation.updated_at = Utc::now();
    save_pres!(state, &user_id, &presentation);

    Ok(Json(SaveResponse {
        id: req.presentation_id,
        success: true,
        message: Some("Theme applied".to_string()),
    }))
}

pub async fn handle_set_transition<D: DriveOps>(
    State(state): State<Arc<SlidesState<D>>>,
    Json(req): Json<SetTransitionRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut presentation = load_pres!(state, &user_id, &req.presentation_id);

    if req.slide_index >= presentation.slides.len() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid slide index" })),
        ));
    }

    let transition = req.transition.unwrap_or_else(|| TransitionConfig {
        transition_type: req.transition_type.unwrap_or_else(|| "none".to_string()),
        duration: req.duration.unwrap_or(500.0),
        direction: req.direction,
        easing: None,
        advance_on_click: true,
        advance_after_time: None,
        sound: None,
    });
    presentation.slides[req.slide_index].transition_config = Some(transition);
    presentation.updated_at = Utc::now();
    save_pres!(state, &user_id, &presentation);

    Ok(Json(SaveResponse {
        id: req.presentation_id,
        success: true,
        message: Some("Transition set".to_string()),
    }))
}

pub async fn handle_apply_transition_to_all<D: DriveOps>(
    State(state): State<Arc<SlidesState<D>>>,
    Json(req): Json<ApplyTransitionToAllRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut presentation = load_pres!(state, &user_id, &req.presentation_id);

    for slide in presentation.slides.iter_mut() {
        slide.transition_config = Some(req.transition.clone());
    }
    presentation.updated_at = Utc::now();
    save_pres!(state, &user_id, &presentation);

    Ok(Json(SaveResponse {
        id: req.presentation_id,
        success: true,
        message: Some("Transition applied to all slides".to_string()),
    }))
}

pub async fn handle_remove_transition<D: DriveOps>(
    State(state): State<Arc<SlidesState<D>>>,
    Json(req): Json<RemoveTransitionRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut presentation = load_pres!(state, &user_id, &req.presentation_id);

    if req.slide_index >= presentation.slides.len() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid slide index" })),
        ));
    }

    presentation.slides[req.slide_index].transition_config = None;
    presentation.slides[req.slide_index].transition = None;
    presentation.updated_at = Utc::now();
    save_pres!(state, &user_id, &presentation);

    Ok(Json(SaveResponse {
        id: req.presentation_id,
        success: true,
        message: Some("Transition removed".to_string()),
    }))
}
