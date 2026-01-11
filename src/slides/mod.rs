pub mod collaboration;
pub mod handlers;
pub mod storage;
pub mod types;
pub mod utils;

use crate::shared::state::AppState;
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

pub use collaboration::{handle_get_collaborators, handle_slides_websocket};
pub use handlers::{
    handle_add_element, handle_add_slide, handle_apply_theme, handle_delete_element,
    handle_delete_presentation, handle_delete_slide, handle_duplicate_slide,
    handle_export_presentation, handle_get_presentation_by_id, handle_list_presentations,
    handle_load_presentation, handle_new_presentation, handle_reorder_slides,
    handle_save_presentation, handle_search_presentations, handle_slides_ai,
    handle_update_element, handle_update_slide_notes,
};
pub use types::{
    Animation, ChartData, ChartDataset, Collaborator, ElementContent, ElementStyle,
    GradientStop, GradientStyle, Presentation, PresentationMetadata, PresentationTheme,
    SaveResponse, ShadowStyle, Slide, SlideBackground, SlideElement, SlideMessage,
    SlideTransition, TableCell, TableData, ThemeColors, ThemeFonts,
};

pub fn configure_slides_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/slides/list", get(handle_list_presentations))
        .route("/api/slides/search", get(handle_search_presentations))
        .route("/api/slides/load", get(handle_load_presentation))
        .route("/api/slides/save", post(handle_save_presentation))
        .route("/api/slides/delete", post(handle_delete_presentation))
        .route("/api/slides/new", get(handle_new_presentation))
        .route("/api/slides/ai", post(handle_slides_ai))
        .route("/api/slides/:id", get(handle_get_presentation_by_id))
        .route("/api/slides/:id/collaborators", get(handle_get_collaborators))
        .route("/api/slides/slide/add", post(handle_add_slide))
        .route("/api/slides/slide/delete", post(handle_delete_slide))
        .route("/api/slides/slide/duplicate", post(handle_duplicate_slide))
        .route("/api/slides/slide/reorder", post(handle_reorder_slides))
        .route("/api/slides/slide/notes", post(handle_update_slide_notes))
        .route("/api/slides/element/add", post(handle_add_element))
        .route("/api/slides/element/update", post(handle_update_element))
        .route("/api/slides/element/delete", post(handle_delete_element))
        .route("/api/slides/theme", post(handle_apply_theme))
        .route("/api/slides/export", post(handle_export_presentation))
        .route("/ws/slides/:presentation_id", get(handle_slides_websocket))
}
