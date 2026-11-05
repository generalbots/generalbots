use crate::collaboration::broadcast_slide_change;
use crate::storage::{
    create_slide_with_layout, get_current_user_id, load_presentation_by_id,
    save_presentation_to_drive, DriveOps,
};
use crate::types::{
    AddSlideRequest, ApplyThemeRequest, ApplyTransitionToAllRequest, DeleteElementRequest,
    DeleteSlideRequest, DuplicateSlideRequest, ElementContent, ElementStyle, ReorderSlidesRequest,
    RemoveTransitionRequest, SaveResponse, SetTransitionRequest, SlideElement,
    UpdateSlideNotesRequest,
};
use crate::SlidesState;
use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

macro_rules! load_pres {
    ($state:expr, $user_id:expr, $pres_id:expr) => {
        match $state.drive.as_ref() {
            Some(drive) => match load_presentation_by_id(drive, $user_id, $pres_id).await {
                Ok(p) => p,
                Err(e) => {
                    return Err((
                        StatusCode::NOT_FOUND,
                        Json(serde_json::json!({ "error": e })),
                    ))
                }
            },
            None => {
                return Err((
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(serde_json::json!({ "error": "Drive not configured" })),
                ))
            }
        }
    };
}

macro_rules! save_pres {
    ($state:expr, $user_id:expr, $presentation:expr) => {
        if let Some(drive) = &$state.drive {
            if let Err(e) = save_presentation_to_drive(drive, $user_id, $presentation).await {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": botsecurity_core::error_sanitizer::log_and_sanitize_json(&e, "botslides::handlers::slides", None)})),
                ));
            }
        }
    };
}

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

/// The canvas posts flat element fields (`slide_id`, `element_id`,
/// `element_type`, x/y/width/height, rotation, fill, shape_type) while this
/// API was typed for a nested SlideElement — every canvas edit 422'd. The
/// helpers below accept both shapes and the bare `elements: [ids]` order
/// payload sent when elements move between slides.
fn presentation_id_of(payload: &serde_json::Value) -> String {
    payload
        .get("presentation_id")
        .and_then(|value| value.as_str())
        .unwrap_or("current")
        .to_string()
}

fn slide_index_of(payload: &serde_json::Value) -> usize {
    payload
        .get("slide_index")
        .and_then(|value| value.as_u64())
        .or_else(|| {
            payload
                .get("slide_id")
                .and_then(|value| value.as_str())
                .and_then(|value| value.parse::<u64>().ok())
        })
        .unwrap_or(0) as usize
}

fn wire_element(payload: &serde_json::Value) -> Option<SlideElement> {
    if let Some(element) = payload.get("element") {
        if let Ok(element) = serde_json::from_value::<SlideElement>(element.clone()) {
            return Some(element);
        }
    }
    let id = payload
        .get("element_id")
        .and_then(|value| value.as_str())?
        .to_string();
    let number = |key: &str| payload.get(key).and_then(|value| value.as_f64()).unwrap_or(0.0);
    let text = |key: &str| {
        payload
            .get(key)
            .and_then(|value| value.as_str())
            .map(str::to_string)
    };
    Some(SlideElement {
        id,
        element_type: text("element_type").unwrap_or_else(|| "shape".to_string()),
        x: number("x"),
        y: number("y"),
        width: number("width"),
        height: number("height"),
        rotation: number("rotation"),
        content: ElementContent {
            text: text("text"),
            html: None,
            src: text("src"),
            shape_type: text("shape_type"),
            chart_data: None,
            table_data: None,
        },
        style: ElementStyle {
            fill: text("fill"),
            stroke: text("stroke"),
            stroke_width: None,
            opacity: None,
            shadow: None,
            font_family: None,
            font_size: None,
            font_weight: None,
            font_style: None,
            text_align: None,
            vertical_align: None,
            locked: false,
        },
        animations: Vec::new(),
        z_index: 0,
        locked: false,
    })
}

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

pub async fn handle_apply_theme<D: DriveOps>(
    State(state): State<Arc<SlidesState<D>>>,
    Json(req): Json<ApplyThemeRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut presentation = load_pres!(state, &user_id, &req.presentation_id);

    presentation.theme = req.theme;
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

    presentation.slides[req.slide_index].transition_config = Some(req.transition);
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
