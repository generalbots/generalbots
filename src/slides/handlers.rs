use crate::shared::state::AppState;
use crate::slides::collaboration::broadcast_slide_change;
use crate::slides::storage::{
    create_new_presentation, create_slide_with_layout, delete_presentation_from_drive,
    get_current_user_id, list_presentations_from_drive, load_presentation_by_id,
    load_presentation_from_drive, save_presentation_to_drive,
};
use crate::slides::types::{
    AddElementRequest, AddMediaRequest, AddSlideRequest, ApplyThemeRequest,
    ApplyTransitionToAllRequest, CollaborationCursor, CollaborationSelection, DeleteElementRequest,
    DeleteMediaRequest, DeleteSlideRequest, DuplicateSlideRequest, EndPresenterRequest,
    ExportRequest, ListCursorsResponse, ListMediaResponse, ListSelectionsResponse, LoadQuery,
    Presentation, PresentationMetadata, PresenterNotesResponse, PresenterSession,
    PresenterSessionResponse, RemoveTransitionRequest, ReorderSlidesRequest,
    SavePresentationRequest, SaveResponse, SearchQuery, SetTransitionRequest, SlidesAiRequest,
    SlidesAiResponse, StartPresenterRequest, UpdateCursorRequest, UpdateElementRequest,
    UpdateMediaRequest, UpdatePresenterRequest, UpdateSelectionRequest, UpdateSlideNotesRequest,
};
use crate::slides::utils::{create_default_theme, export_to_html, export_to_json, export_to_markdown, export_to_odp_content, export_to_svg, slides_from_markdown};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use log::error;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock};
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
            let json = export_to_json(&presentation);
            Ok(([(axum::http::header::CONTENT_TYPE, "application/json")], json))
        }
        "svg" => {
            let slide_idx = 0;
            if slide_idx < presentation.slides.len() {
                let svg = export_to_svg(&presentation.slides[slide_idx], 960, 540);
                Ok(([(axum::http::header::CONTENT_TYPE, "image/svg+xml")], svg))
            } else {
                Err((
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": "No slides to export" })),
                ))
            }
        }
        "md" | "markdown" => {
            let md = export_to_markdown(&presentation);
            Ok(([(axum::http::header::CONTENT_TYPE, "text/markdown")], md))
        }
        "odp" => {
            let odp = export_to_odp_content(&presentation);
            Ok((
                [(
                    axum::http::header::CONTENT_TYPE,
                    "application/vnd.oasis.opendocument.presentation",
                )],
                odp,
            ))
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

static CURSORS: LazyLock<RwLock<HashMap<String, Vec<CollaborationCursor>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

static SELECTIONS: LazyLock<RwLock<HashMap<String, Vec<CollaborationSelection>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

static PRESENTER_SESSIONS: LazyLock<RwLock<HashMap<String, PresenterSession>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

pub async fn handle_update_cursor(
    State(_state): State<Arc<AppState>>,
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

pub async fn handle_update_selection(
    State(_state): State<Arc<AppState>>,
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

pub async fn handle_list_cursors(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ListCursorsResponse>, (StatusCode, Json<serde_json::Value>)> {
    let presentation_id = params.get("presentation_id").cloned().unwrap_or_default();

    let cursors = if let Ok(cursors_map) = CURSORS.read() {
        cursors_map.get(&presentation_id).cloned().unwrap_or_default()
    } else {
        vec![]
    };

    Ok(Json(ListCursorsResponse { cursors }))
}

pub async fn handle_list_selections(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ListSelectionsResponse>, (StatusCode, Json<serde_json::Value>)> {
    let presentation_id = params.get("presentation_id").cloned().unwrap_or_default();

    let selections = if let Ok(selections_map) = SELECTIONS.read() {
        selections_map.get(&presentation_id).cloned().unwrap_or_default()
    } else {
        vec![]
    };

    Ok(Json(ListSelectionsResponse { selections }))
}

pub async fn handle_set_transition(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SetTransitionRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut presentation = match load_presentation_by_id(&state, &user_id, &req.presentation_id).await {
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

    presentation.slides[req.slide_index].transition_config = Some(req.transition);
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
        message: Some("Transition set".to_string()),
    }))
}

pub async fn handle_apply_transition_to_all(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ApplyTransitionToAllRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut presentation = match load_presentation_by_id(&state, &user_id, &req.presentation_id).await {
        Ok(p) => p,
        Err(e) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e })),
            ))
        }
    };

    for slide in presentation.slides.iter_mut() {
        slide.transition_config = Some(req.transition.clone());
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
        message: Some("Transition applied to all slides".to_string()),
    }))
}

pub async fn handle_remove_transition(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RemoveTransitionRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut presentation = match load_presentation_by_id(&state, &user_id, &req.presentation_id).await {
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

    presentation.slides[req.slide_index].transition_config = None;
    presentation.slides[req.slide_index].transition = None;
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
        message: Some("Transition removed".to_string()),
    }))
}

pub async fn handle_add_media(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddMediaRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut presentation = match load_presentation_by_id(&state, &user_id, &req.presentation_id).await {
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

    let media_list = presentation.slides[req.slide_index].media.get_or_insert_with(Vec::new);
    media_list.push(req.media.clone());
    presentation.updated_at = Utc::now();

    if let Err(e) = save_presentation_to_drive(&state, &user_id, &presentation).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(serde_json::json!({ "success": true, "media": req.media })))
}

pub async fn handle_update_media(
    State(state): State<Arc<AppState>>,
    Json(req): Json<UpdateMediaRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut presentation = match load_presentation_by_id(&state, &user_id, &req.presentation_id).await {
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

    if let Some(media_list) = &mut presentation.slides[req.slide_index].media {
        for media in media_list.iter_mut() {
            if media.id == req.media_id {
                if let Some(autoplay) = req.autoplay {
                    media.autoplay = autoplay;
                }
                if let Some(loop_playback) = req.loop_playback {
                    media.loop_playback = loop_playback;
                }
                if let Some(muted) = req.muted {
                    media.muted = muted;
                }
                if let Some(volume) = req.volume {
                    media.volume = Some(volume);
                }
                if let Some(start_time) = req.start_time {
                    media.start_time = Some(start_time);
                }
                if let Some(end_time) = req.end_time {
                    media.end_time = Some(end_time);
                }
                break;
            }
        }
    }

    presentation.updated_at = Utc::now();
    if let Err(e) = save_presentation_to_drive(&state, &user_id, &presentation).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn handle_delete_media(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DeleteMediaRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut presentation = match load_presentation_by_id(&state, &user_id, &req.presentation_id).await {
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

    if let Some(media_list) = &mut presentation.slides[req.slide_index].media {
        media_list.retain(|m| m.id != req.media_id);
    }

    presentation.updated_at = Utc::now();
    if let Err(e) = save_presentation_to_drive(&state, &user_id, &presentation).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn handle_list_media(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ListMediaResponse>, (StatusCode, Json<serde_json::Value>)> {
    let presentation_id = params.get("presentation_id").cloned().unwrap_or_default();
    let slide_index: usize = params.get("slide_index").and_then(|s| s.parse().ok()).unwrap_or(0);
    let user_id = get_current_user_id();

    let presentation = match load_presentation_by_id(&state, &user_id, &presentation_id).await {
        Ok(p) => p,
        Err(e) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e })),
            ))
        }
    };

    if slide_index >= presentation.slides.len() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid slide index" })),
        ));
    }

    let media = presentation.slides[slide_index].media.clone().unwrap_or_default();
    Ok(Json(ListMediaResponse { media }))
}

pub async fn handle_start_presenter(
    State(state): State<Arc<AppState>>,
    Json(req): Json<StartPresenterRequest>,
) -> Result<Json<PresenterSessionResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    let _presentation = match load_presentation_by_id(&state, &user_id, &req.presentation_id).await {
        Ok(p) => p,
        Err(e) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e })),
            ))
        }
    };

    let session = PresenterSession {
        id: uuid::Uuid::new_v4().to_string(),
        presentation_id: req.presentation_id,
        current_slide: 0,
        started_at: Utc::now(),
        elapsed_time: Some(0),
        is_paused: false,
        settings: req.settings,
    };

    if let Ok(mut sessions) = PRESENTER_SESSIONS.write() {
        sessions.insert(session.id.clone(), session.clone());
    }

    Ok(Json(PresenterSessionResponse { session }))
}

pub async fn handle_update_presenter(
    State(_state): State<Arc<AppState>>,
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

pub async fn handle_end_presenter(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<EndPresenterRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if let Ok(mut sessions) = PRESENTER_SESSIONS.write() {
        sessions.remove(&req.session_id);
    }

    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn handle_get_presenter_notes(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<PresenterNotesResponse>, (StatusCode, Json<serde_json::Value>)> {
    let presentation_id = params.get("presentation_id").cloned().unwrap_or_default();
    let slide_index: usize = params.get("slide_index").and_then(|s| s.parse().ok()).unwrap_or(0);
    let user_id = get_current_user_id();

    let presentation = match load_presentation_by_id(&state, &user_id, &presentation_id).await {
        Ok(p) => p,
        Err(e) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e })),
            ))
        }
    };

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

pub async fn handle_import_presentation(
    State(state): State<Arc<AppState>>,
    mut multipart: axum::extract::Multipart,
) -> Result<Json<Presentation>, (StatusCode, Json<serde_json::Value>)> {
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut filename = "import.pptx".to_string();

    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some("file") {
            filename = field.file_name().unwrap_or("import.pptx").to_string();
            if let Ok(bytes) = field.bytes().await {
                file_bytes = Some(bytes.to_vec());
            }
        }
    }

    let bytes = file_bytes.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "No file uploaded" })),
        )
    })?;

    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    let theme = create_default_theme();

    let slides = match ext.as_str() {
        "md" | "markdown" => {
            let content = String::from_utf8_lossy(&bytes);
            slides_from_markdown(&content)
        }
        "json" => {
            let pres: Result<Presentation, _> = serde_json::from_slice(&bytes);
            match pres {
                Ok(p) => p.slides,
                Err(e) => {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({ "error": format!("Invalid JSON: {}", e) })),
                    ))
                }
            }
        }
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("Unsupported format: {}", ext) })),
            ))
        }
    };

    let name = filename.rsplit('/').next().unwrap_or(&filename)
        .rsplit('.').last().unwrap_or(&filename)
        .to_string();

    let user_id = get_current_user_id();
    let presentation = Presentation {
        id: Uuid::new_v4().to_string(),
        name,
        owner_id: user_id.clone(),
        slides,
        theme,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    if let Err(e) = save_presentation_to_drive(&state, &user_id, &presentation).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(presentation))
}
