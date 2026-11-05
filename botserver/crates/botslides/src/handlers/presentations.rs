use crate::storage::{
    create_new_presentation, delete_presentation_from_drive, get_current_user_id,
    list_presentations_from_drive, load_presentation_by_id, load_presentation_from_drive,
    save_presentation_to_drive, DriveOps,
};
use crate::types::{
    LoadQuery, Presentation, PresentationMetadata, SavePresentationRequest, SaveResponse,
    SearchQuery,
};
use crate::SlidesState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

pub async fn handle_new_presentation<D: DriveOps>(
    State(_state): State<Arc<SlidesState<D>>>,
) -> Result<Json<Presentation>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(create_new_presentation()))
}

pub async fn handle_list_presentations<D: DriveOps>(
    State(state): State<Arc<SlidesState<D>>>,
) -> Result<Json<Vec<PresentationMetadata>>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    let presentations = match &state.drive {
        Some(drive) => match list_presentations_from_drive(drive, &user_id).await {
            Ok(p) => p,
            Err(e) => {
                log::error!("Failed to list presentations: {e}");
                Vec::new()
            }
        },
        None => Vec::new(),
    };

    Ok(Json(presentations))
}

pub async fn handle_search_presentations<D: DriveOps>(
    State(state): State<Arc<SlidesState<D>>>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<PresentationMetadata>>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

let presentations = match &state.drive {
    Some(drive) => list_presentations_from_drive(drive, &user_id)
        .await
        .unwrap_or_default(),
    None => Vec::new(),
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

pub async fn handle_load_presentation<D: DriveOps>(
    State(state): State<Arc<SlidesState<D>>>,
    Query(query): Query<LoadQuery>,
) -> Result<Json<Presentation>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    let drive = state.drive.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Drive not configured" })),
        )
    })?;

    match load_presentation_from_drive(drive, &user_id, &query.id).await {
        Ok(presentation) => Ok(Json(presentation)),
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

/// `dir/sample.pptx` → `dir/sample.gbslides.pptx`; a path without an extension
/// gets `.gbslides.pptx` appended. Save-back never overwrites the source.
fn sibling_pptx_path(path: &str) -> String {
    let (dir, name) = match path.rsplit_once('/') {
        Some((dir, name)) => (format!("{dir}/"), name),
        None => (String::new(), path),
    };
    let stem = name.rsplit_once('.').map(|(stem, _)| stem).unwrap_or(name);
    format!("{dir}{stem}.gbslides.pptx")
}

pub async fn handle_save_presentation<D: DriveOps>(
    State(state): State<Arc<SlidesState<D>>>,
    Json(req): Json<SavePresentationRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let presentation_id = req
        .id
        .filter(|id| !id.is_empty())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // Always start from the stored deck when it exists: that preserves the
    // import source identity (needed for save-back to the .pptx) and the
    // creation timestamp even when the client posts a full slides+theme
    // payload (#1710).
    let mut presentation = match state.drive.as_ref() {
        Some(drive) => load_presentation_by_id(drive, &user_id, &presentation_id)
            .await
            .ok(),
        None => None,
    }
    .unwrap_or_else(|| {
        let mut fresh = create_new_presentation();
        fresh.id = presentation_id.clone();
        fresh
    });
    if let Some(name) = req.name {
        presentation.name = name;
    }
    if let Some(slides) = req.slides {
        presentation.slides = slides;
    }
    if let Some(theme) = req.theme {
        presentation.theme = theme;
    }
    presentation.updated_at = Utc::now();

    if let Some(drive) = &state.drive {
        if let Err(e) = save_presentation_to_drive(drive, &user_id, &presentation).await {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": botsecurity_core::error_sanitizer::log_and_sanitize_json(&e, "botslides::handlers::presentations", None)})),
            ));
        }
        // Save-back: a deck imported from a Drive .pptx writes the exported
        // presentation next to the source as `{stem}.gbslides.pptx` — the same
        // parity the sheet app gives (a `.gbsheet.xlsx` sibling). The original
        // object is never overwritten: the exporter models text, shapes,
        // images and notes, not every source feature (tables), so replacing
        // the upload could silently drop content (#1710). A failure here never
        // discards the deck JSON saved above.
        if let (Some(bucket), Some(path)) = (
            presentation.source_bucket.as_deref(),
            presentation.source_path.as_deref(),
        ) {
            match crate::pptx::export_to_pptx(&presentation) {
                Ok(bytes) => {
                    let sibling = sibling_pptx_path(path);
                    if let Err(e) = drive
                        .put_object(
                            bucket,
                            &sibling,
                            bytes,
                            "application/vnd.openxmlformats-officedocument.presentationml.presentation",
                        )
                        .await
                    {
                        log::warn!("slides save-back to {bucket}/{sibling} failed: {e}");
                    }
                }
                Err(e) => log::warn!("slides save-back export failed for {bucket}/{path}: {e}"),
            }
        }
    }

    Ok(Json(SaveResponse {
        id: presentation_id,
        success: true,
        message: Some("Presentation saved successfully".to_string()),
    }))
}

pub async fn handle_delete_presentation<D: DriveOps>(
    State(state): State<Arc<SlidesState<D>>>,
    Json(req): Json<LoadQuery>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    if let Some(drive) = &state.drive {
        if let Err(e) = delete_presentation_from_drive(drive, &user_id, &req.id).await {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": botsecurity_core::error_sanitizer::log_and_sanitize_json(&e, "botslides::handlers::presentations", None)})),
            ));
        }
    }

    Ok(Json(SaveResponse {
        id: req.id.unwrap_or_default(),
        success: true,
        message: Some("Presentation deleted".to_string()),
    }))
}

pub async fn handle_get_presentation_by_id<D: DriveOps>(
    State(state): State<Arc<SlidesState<D>>>,
    Path(presentation_id): Path<String>,
) -> Result<Json<Presentation>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    let drive = state.drive.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Drive not configured" })),
        )
    })?;

    match load_presentation_by_id(drive, &user_id, &presentation_id).await {
        Ok(presentation) => Ok(Json(presentation)),
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::sibling_pptx_path;

    #[test]
    fn save_back_uses_a_sibling_path() {
        assert_eq!(
            sibling_pptx_path("destacados/review.pptx"),
            "destacados/review.gbslides.pptx"
        );
        assert_eq!(sibling_pptx_path("review.pptx"), "review.gbslides.pptx");
        assert_eq!(
            sibling_pptx_path("folder/noext"),
            "folder/noext.gbslides.pptx"
        );
        assert_eq!(
            sibling_pptx_path("folder/two.dots.pptx"),
            "folder/two.dots.gbslides.pptx"
        );
    }
}
