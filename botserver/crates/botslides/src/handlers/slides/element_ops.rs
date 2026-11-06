//! Element operations: add, update (including animations) and delete.

use super::wire::{presentation_id_of, slide_index_of, wire_element};
use crate::collaboration::broadcast_slide_change;
use crate::storage::{get_current_user_id, DriveOps};
use crate::types::{DeleteElementRequest, SaveResponse};
use crate::SlidesState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use chrono::Utc;
use std::sync::Arc;

pub async fn handle_add_element<D: DriveOps>(
    State(state): State<Arc<SlidesState<D>>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let presentation_id = presentation_id_of(&payload);
    let slide_index = slide_index_of(&payload);
    let user_id = get_current_user_id();
    let mut presentation = load_pres!(state, &user_id, &presentation_id);

    if slide_index >= presentation.slides.len() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid slide index" })),
        ));
    }

    // Bare order payload: {elements: ["el_1", ...]} (the canvas posts the DOM
    // order after elements move); unknown ids keep their relative position.
    if let Some(order) = payload.get("elements").and_then(|value| value.as_array()) {
        let ids: Vec<String> = order
            .iter()
            .filter_map(|value| value.as_str().map(str::to_string))
            .collect();
        let slide = &mut presentation.slides[slide_index];
        slide.elements.sort_by_key(|element| {
            ids.iter()
                .position(|id| id == &element.id)
                .unwrap_or(usize::MAX)
        });
        presentation.updated_at = Utc::now();
        save_pres!(state, &user_id, &presentation);
        return Ok(Json(SaveResponse {
            id: presentation_id,
            success: true,
            message: Some("Slide order saved".to_string()),
        }));
    }

    let element = wire_element(&payload).ok_or_else(|| {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({ "error": "element payload required" })),
        )
    })?;
    let element_id = element.id.clone();
    presentation.slides[slide_index].elements.push(element);
    presentation.updated_at = Utc::now();
    save_pres!(state, &user_id, &presentation);

    broadcast_slide_change(
        &presentation_id,
        &user_id,
        "User",
        "element_added",
        Some(slide_index),
        Some(&element_id),
        None,
    )
    .await;

    Ok(Json(SaveResponse {
        id: presentation_id,
        success: true,
        message: Some("Element added".to_string()),
    }))
}

pub async fn handle_update_element<D: DriveOps>(
    State(state): State<Arc<SlidesState<D>>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let presentation_id = presentation_id_of(&payload);
    let slide_index = slide_index_of(&payload);
    let user_id = get_current_user_id();
    let mut presentation = load_pres!(state, &user_id, &presentation_id);

    if slide_index >= presentation.slides.len() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid slide index" })),
        ));
    }

    // The animation form posts action=add_animation without an element_type;
    // it used to fall through to wire_element and append an invisible 0×0
    // element while the animation was never stored.
    if payload.get("action").and_then(|value| value.as_str()) == Some("add_animation") {
        let element_id = payload
            .get("element_id")
            .and_then(|value| value.as_str())
            .unwrap_or("");
        if element_id.is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "element_id is required for an animation" })),
            ));
        }
        let animation = crate::types::Animation {
            animation_type: payload
                .get("animation_type")
                .and_then(|value| value.as_str())
                .unwrap_or("fade_in")
                .to_string(),
            trigger: payload
                .get("trigger")
                .and_then(|value| value.as_str())
                .unwrap_or("on_click")
                .to_string(),
            duration: payload.get("duration").and_then(|value| value.as_f64()).unwrap_or(0.5),
            delay: payload.get("delay").and_then(|value| value.as_f64()).unwrap_or(0.0),
            direction: payload
                .get("direction")
                .and_then(|value| value.as_str())
                .map(str::to_string),
        };
        let slide = &mut presentation.slides[slide_index];
        match slide.elements.iter_mut().find(|item| item.id == element_id) {
            Some(existing) => existing.animations.push(animation),
            None => {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({ "error": "Element not found" })),
                ))
            }
        }
        presentation.updated_at = Utc::now();
        save_pres!(state, &user_id, &presentation);
        return Ok(Json(SaveResponse {
            id: presentation_id.clone(),
            success: true,
            message: Some("Animation added".to_string()),
        }));
    }

    // Canvas drag/resize and inline text edits target an existing element
    // with a partial payload; merge those fields so the element type and
    // table/chart content survive instead of being rebuilt as a 0×0 shape.
    let partial_update = payload.get("element_type").is_none()
        && payload.get("element_id").is_some();
    if partial_update {
        let element_id = payload
            .get("element_id")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_string();
        let number = |key: &str| payload.get(key).and_then(|value| value.as_f64()).unwrap_or(0.0);
        let slide = &mut presentation.slides[slide_index];
        if let Some(existing) = slide.elements.iter_mut().find(|item| item.id == element_id) {
            if payload.get("x").is_some() { existing.x = number("x"); }
            if payload.get("y").is_some() { existing.y = number("y"); }
            if payload.get("width").is_some() { existing.width = number("width"); }
            if payload.get("height").is_some() { existing.height = number("height"); }
            if payload.get("rotation").is_some() { existing.rotation = number("rotation"); }
            if let Some(text) = payload.get("text").and_then(|value| value.as_str()) {
                existing.content.text = Some(text.to_string());
            }
            presentation.updated_at = Utc::now();
            save_pres!(state, &user_id, &presentation);
            return Ok(Json(SaveResponse {
                id: presentation_id.clone(),
                success: true,
                message: Some("Element updated".to_string()),
            }));
        }
    }
    let element = wire_element(&payload).ok_or_else(|| {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({ "error": "element payload required" })),
        )
    })?;
    let element_id = element.id.clone();
    let slide = &mut presentation.slides[slide_index];
    if let Some(position) = slide.elements.iter().position(|existing| existing.id == element.id) {
        slide.elements[position] = element;
    } else {
        slide.elements.push(element);
    }

    presentation.updated_at = Utc::now();
    save_pres!(state, &user_id, &presentation);

    broadcast_slide_change(
        &presentation_id,
        &user_id,
        "User",
        "element_updated",
        Some(slide_index),
        Some(&element_id),
        None,
    )
    .await;

    Ok(Json(SaveResponse {
        id: presentation_id,
        success: true,
        message: Some("Element updated".to_string()),
    }))
}

pub async fn handle_delete_element<D: DriveOps>(
    State(state): State<Arc<SlidesState<D>>>,
    Json(req): Json<DeleteElementRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut presentation = load_pres!(state, &user_id, &req.presentation_id);

    if req.slide_index >= presentation.slides.len() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid slide index" })),
        ));
    }

    presentation.slides[req.slide_index]
        .elements
        .retain(|e| e.id != req.element_id);
    presentation.updated_at = Utc::now();
    save_pres!(state, &user_id, &presentation);

    Ok(Json(SaveResponse {
        id: req.presentation_id,
        success: true,
        message: Some("Element deleted".to_string()),
    }))
}
