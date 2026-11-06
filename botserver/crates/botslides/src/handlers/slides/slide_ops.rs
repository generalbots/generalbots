//! Slide-level operations: add, delete, duplicate, reorder and notes.

use crate::storage::{create_slide_with_layout, get_current_user_id, DriveOps};
use crate::types::{
    AddSlideRequest, DeleteSlideRequest, DuplicateSlideRequest, ReorderSlidesRequest, SaveResponse,
    UpdateSlideNotesRequest,
};
use crate::SlidesState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

pub async fn handle_add_slide<D: DriveOps>(
    State(state): State<Arc<SlidesState<D>>>,
    Json(req): Json<AddSlideRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut presentation = load_pres!(state, &user_id, &req.presentation_id);

    let new_slide = create_slide_with_layout(&req.layout, &presentation.theme);

    if let Some(position) = req.position {
        if position <= presentation.slides.len() {
            presentation.slides.insert(position, new_slide);
        } else {
            presentation.slides.push(new_slide);
        }
    } else {
        presentation.slides.push(new_slide);
    }

    presentation.updated_at = Utc::now();
    save_pres!(state, &user_id, &presentation);

    Ok(Json(SaveResponse {
        id: req.presentation_id,
        success: true,
        message: Some("Slide added".to_string()),
    }))
}

pub async fn handle_delete_slide<D: DriveOps>(
    State(state): State<Arc<SlidesState<D>>>,
    Json(req): Json<DeleteSlideRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut presentation = load_pres!(state, &user_id, &req.presentation_id);

    if req.slide_index >= presentation.slides.len() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid slide index" })),
        ));
    }

    presentation.slides.remove(req.slide_index);
    presentation.updated_at = Utc::now();
    save_pres!(state, &user_id, &presentation);

    Ok(Json(SaveResponse {
        id: req.presentation_id,
        success: true,
        message: Some("Slide deleted".to_string()),
    }))
}

pub async fn handle_duplicate_slide<D: DriveOps>(
    State(state): State<Arc<SlidesState<D>>>,
    Json(req): Json<DuplicateSlideRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut presentation = load_pres!(state, &user_id, &req.presentation_id);

    if req.slide_index >= presentation.slides.len() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid slide index" })),
        ));
    }

    let mut duplicated = presentation.slides[req.slide_index].clone();
    duplicated.id = Uuid::new_v4().to_string();
    presentation.slides.insert(req.slide_index + 1, duplicated);
    presentation.updated_at = Utc::now();
    save_pres!(state, &user_id, &presentation);

    Ok(Json(SaveResponse {
        id: req.presentation_id,
        success: true,
        message: Some("Slide duplicated".to_string()),
    }))
}

pub async fn handle_reorder_slides<D: DriveOps>(
    State(state): State<Arc<SlidesState<D>>>,
    Json(req): Json<ReorderSlidesRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut presentation = load_pres!(state, &user_id, &req.presentation_id);

    if req.slide_order.len() != presentation.slides.len() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid slide order" })),
        ));
    }

    let old_slides = presentation.slides.clone();
    presentation.slides = req
        .slide_order
        .iter()
        .filter_map(|&idx| old_slides.get(idx).cloned())
        .collect();

    presentation.updated_at = Utc::now();
    save_pres!(state, &user_id, &presentation);

    Ok(Json(SaveResponse {
        id: req.presentation_id,
        success: true,
        message: Some("Slides reordered".to_string()),
    }))
}

pub async fn handle_update_slide_notes<D: DriveOps>(
    State(state): State<Arc<SlidesState<D>>>,
    Json(req): Json<UpdateSlideNotesRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut presentation = load_pres!(state, &user_id, &req.presentation_id);

    if req.slide_index >= presentation.slides.len() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid slide index" })),
        ));
    }

    presentation.slides[req.slide_index].notes = Some(req.notes);
    presentation.updated_at = Utc::now();
    save_pres!(state, &user_id, &presentation);

    Ok(Json(SaveResponse {
        id: req.presentation_id,
        success: true,
        message: Some("Slide notes updated".to_string()),
    }))
}
