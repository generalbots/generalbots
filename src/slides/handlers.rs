use crate::shared::state::AppState;
use crate::slides::collaboration::broadcast_slide_change;
use crate::slides::storage::{
    create_new_presentation, create_slide_with_layout, delete_presentation_from_drive,
    get_current_user_id, list_presentations_from_drive, load_presentation_by_id,
    load_presentation_from_drive, save_presentation_to_drive,
};
use crate::slides::types::{
    AddElementRequest, AddSlideRequest, ApplyThemeRequest, DeleteElementRequest,
    DeleteSlideRequest, DuplicateSlideRequest, ExportRequest, LoadQuery, Presentation,
    PresentationMetadata, ReorderSlidesRequest, SavePresentationRequest, SaveResponse, SearchQuery,
    SlidesAiRequest, SlidesAiResponse, UpdateElementRequest, UpdateSlideNotesRequest,
};
use crate::slides::utils::export_to_html;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use log::error;
use std::sync::Arc;
use uuid::Uuid;

pub async fn handle_slides_ai(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<SlidesAiRequest>,
) -> impl IntoResponse {
    let command = req.command.to_lowercase();

    let response = if command.contains("add") && command.contains("slide") {
        "I've added a new slide to your presentation."
    } else if command.contains("duplicate") {
        "I've duplicated the current slide."
    } else if command.contains("delete") || command.contains("remove") {
        "I've removed the slide from your presentation."
    } else if command.contains("text") || command.contains("title") {
        "I've added a text box to your slide. Click to edit."
    } else if command.contains("image") || command.contains("picture") {
        "I've added an image placeholder. Click to upload an image."
    } else if command.contains("shape") {
        "I've added a shape to your slide. You can resize and move it."
    } else if command.contains("chart") {
        "I've added a chart. Click to edit the data."
    } else if command.contains("table") {
        "I've added a table. Click cells to edit."
    } else if command.contains("theme") || command.contains("design") {
        "I can help you change the theme. Choose from the Design menu."
    } else if command.contains("animate") || command.contains("animation") {
        "I've added an animation to the selected element."
    } else if command.contains("transition") {
        "I've applied a transition effect to this slide."
    } else if command.contains("help") {
        "I can help you with:\n• Add/duplicate/delete slides\n• Insert text, images, shapes\n• Add charts and tables\n• Apply themes and animations\n• Set slide transitions"
    } else {
        "I understand you want help with your presentation. Try commands like 'add slide', 'insert image', 'add chart', or 'apply animation'."
    };

    Json(SlidesAiResponse {
        response: response.to_string(),
        action: None,
        data: None,
    })
}

pub async fn handle_new_presentation(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<Presentation>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(create_new_presentation()))
}

pub async fn handle_list_presentations(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<PresentationMetadata>>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    match list_presentations_from_drive(&state, &user_id).await {
        Ok(presentations) => Ok(Json(presentations)),
        Err(e) => {
            error!("Failed to list presentations: {}", e);
            Ok(Json(Vec::new()))
        }
    }
}

pub async fn handle_search_presentations(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<PresentationMetadata>>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    let presentations = match list_presentations_from_drive(&state, &user_id).await {
        Ok(p) => p,
        Err(_) => Vec::new(),
    };

    let filtered = if let Some(q) = query.q {
        let q_lower = q.to_lowercase();
        presentations
            .into_iter()
            .filter(|p| p.name.to_lowercase().contains(&q_lower))
            .collect()
    } else {
        presentations
    };

    Ok(Json(filtered))
}

pub async fn handle_load_presentation(
    State(state): State<Arc<AppState>>,
    Query(query): Query<LoadQuery>,
) -> Result<Json<Presentation>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    match load_presentation_from_drive(&state, &user_id, &query.id).await {
        Ok(presentation) => Ok(Json(presentation)),
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

pub async fn handle_save_presentation(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SavePresentationRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let presentation_id = req.id.unwrap_or_else(|| Uuid::new_v4().to_string());

    let presentation = Presentation {
        id: presentation_id.clone(),
        name: req.name,
        owner_id: user_id.clone(),
        slides: req.slides,
        theme: req.theme,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    if let Err(e) = save_presentation_to_drive(&state, &user_id, &presentation).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(SaveResponse {
        id: presentation_id,
        success: true,
        message: Some("Presentation saved successfully".to_string()),
    }))
}

pub async fn handle_delete_presentation(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoadQuery>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    if let Err(e) = delete_presentation_from_drive(&state, &user_id, &req.id).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(SaveResponse {
        id: req.id.unwrap_or_default(),
        success: true,
        message: Some("Presentation deleted".to_string()),
    }))
}

pub async fn handle_get_presentation_by_id(
    State(state): State<Arc<AppState>>,
    Path(presentation_id): Path<String>,
) -> Result<Json<Presentation>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    match load_presentation_by_id(&state, &user_id, &presentation_id).await {
        Ok(presentation) => Ok(Json(presentation)),
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

pub async fn handle_add_slide(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddSlideRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut presentation = match load_presentation_by_id(&state, &user_id, &req.presentation_id).await
    {
        Ok(p) => p,
        Err(e) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e })),
            ))
        }
    };

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

    if let Err(e) = save_presentation_to_drive(&state, &user_id, &presentation).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(SaveResponse {
        id: req.presentation_id,
        success: true,
        message: Some("Slide added".to_string()),
    }))
}

pub async fn handle_delete_slide(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DeleteSlideRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut presentation = match load_presentation_by_id(&state, &user_id, &req.presentation_id).await
    {
        Ok(p) => p,
        Err(e) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e })),
            ))
        }
    };

    if req.slide_index >= presentation.slides.len() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid slide index" })),
        ));
    }

    presentation.slides.remove(req.slide_index);
    presentation.updated_at = Utc::now();

    if let Err(e) = save_presentation_to_drive(&state, &user_id, &presentation).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(SaveResponse {
        id: req.presentation_id,
        success: true,
        message: Some("Slide deleted".to_string()),
    }))
}

pub async fn handle_duplicate_slide(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DuplicateSlideRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut presentation = match load_presentation_by_id(&state, &user_id, &req.presentation_id).await
    {
        Ok(p) => p,
        Err(e) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e })),
            ))
        }
    };

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

    if let Err(e) = save_presentation_to_drive(&state, &user_id, &presentation).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(SaveResponse {
        id: req.presentation_id,
        success: true,
        message: Some("Slide duplicated".to_string()),
    }))
}

pub async fn handle_reorder_slides(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ReorderSlidesRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut presentation = match load_presentation_by_id(&state, &user_id, &req.presentation_id).await
    {
        Ok(p) => p,
        Err(e) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e })),
            ))
        }
    };

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

    if let Err(e) = save_presentation_to_drive(&state, &user_id, &presentation).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(SaveResponse {
        id: req.presentation_id,
        success: true,
        message: Some("Slides reordered".to_string()),
    }))
}

pub async fn handle_update_slide_notes(
    State(state): State<Arc<AppState>>,
    Json(req): Json<UpdateSlideNotesRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut presentation = match load_presentation_by_id(&state, &user_id, &req.presentation_id).await
    {
        Ok(p) => p,
        Err(e) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e })),
            ))
        }
    };

    if req.slide_index >= presentation.slides.len() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid slide index" })),
        ));
    }

    presentation.slides[req.slide_index].notes = Some(req.notes);
    presentation.updated_at = Utc::now();

    if let Err(e) = save_presentation_to_drive(&state, &user_id, &presentation).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(SaveResponse {
        id: req.presentation_id,
        success: true,
        message: Some("Slide notes updated".to_string()),
    }))
}

pub async fn handle_add_element(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddElementRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut presentation = match load_presentation_by_id(&state, &user_id, &req.presentation_id).await
    {
        Ok(p) => p,
        Err(e) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e })),
            ))
        }
    };

    if req.slide_index >= presentation.slides.len() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid slide index" })),
        ));
    }

    presentation.slides[req.slide_index].elements.push(req.element);
    presentation.updated_at = Utc::now();

    if let Err(e) = save_presentation_to_drive(&state, &user_id, &presentation).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    broadcast_slide_change(
        &req.presentation_id,
        &user_id,
        "User",
        "element_added",
        Some(req.slide_index),
        None,
        None,
    )
    .await;

    Ok(Json(SaveResponse {
        id: req.presentation_id,
        success: true,
        message: Some("Element added".to_string()),
    }))
}

pub async fn handle_update_element(
    State(state): State<Arc<AppState>>,
    Json(req): Json<UpdateElementRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut presentation = match load_presentation_by_id(&state, &user_id, &req.presentation_id).await
    {
        Ok(p) => p,
        Err(e) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e })),
            ))
        }
    };

    if req.slide_index >= presentation.slides.len() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid slide index" })),
        ));
    }

    let slide = &mut presentation.slides[req.slide_index];
    if let Some(pos) = slide.elements.iter().position(|e| e.id == req.element.id) {
        slide.elements[pos] = req.element.clone();
    } else {
        slide.elements.push(req.element.clone());
    }

    presentation.updated_at = Utc::now();

    if let Err(e) = save_presentation_to_drive(&state, &user_id, &presentation).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    broadcast_slide_change(
        &req.presentation_id,
        &user_id,
        "User",
        "element_updated",
        Some(req.slide_index),
        Some(&req.element.id),
        None,
    )
    .await;

    Ok(Json(SaveResponse {
        id: req.presentation_id,
        success: true,
        message: Some("Element updated".to_string()),
    }))
}

pub async fn handle_delete_element(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DeleteElementRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut presentation = match load_presentation_by_id(&state, &user_id, &req.presentation_id).await
    {
        Ok(p) => p,
        Err(e) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e })),
            ))
        }
    };

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

    if let Err(e) = save_presentation_to_drive(&state, &user_id, &presentation).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(SaveResponse {
        id: req.presentation_id,
        success: true,
        message: Some("Element deleted".to_string()),
    }))
}

pub async fn handle_apply_theme(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ApplyThemeRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut presentation = match load_presentation_by_id(&state, &user_id, &req.presentation_id).await
    {
        Ok(p) => p,
        Err(e) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e })),
            ))
        }
    };

    presentation.theme = req.theme;
    presentation.updated_at = Utc::now();

    if let Err(e) = save_presentation_to_drive(&state, &user_id, &presentation).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(SaveResponse {
        id: req.presentation_id,
        success: true,
        message: Some("Theme applied".to_string()),
    }))
}

pub async fn handle_export_presentation(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ExportRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    let presentation = match load_presentation_by_id(&state, &user_id, &req.id).await {
        Ok(p) => p,
        Err(e) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e })),
            ))
        }
    };

    match req.format.as_str() {
        "html" => {
            let html = export_to_html(&presentation);
            Ok(([(axum::http::header::CONTENT_TYPE, "text/html")], html))
        }
        "json" => {
            let json = serde_json::to_string_pretty(&presentation).unwrap_or_default();
            Ok(([(axum::http::header::CONTENT_TYPE, "application/json")], json))
        }
        "pptx" => {
            Ok((
                [(
                    axum::http::header::CONTENT_TYPE,
                    "application/vnd.openxmlformats-officedocument.presentationml.presentation",
                )],
                "PPTX export not yet implemented".to_string(),
            ))
        }
        _ => Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Unsupported format" })),
        )),
    }
}
