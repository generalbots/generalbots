pub mod collaboration;
pub mod handlers;
pub mod ooxml;
pub mod storage;
pub mod types;
pub mod utils;

use crate::shared::state::AppState;
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

pub use collaboration::{
    handle_get_collaborators, handle_get_mentions, handle_get_presence, handle_get_selections,
    handle_get_typing, handle_slides_websocket,
};
pub use handlers::{
    handle_add_element, handle_add_media, handle_add_slide, handle_apply_theme,
    handle_apply_transition_to_all, handle_delete_element, handle_delete_media,
    handle_delete_presentation, handle_delete_slide, handle_duplicate_slide,
    handle_end_presenter, handle_export_presentation, handle_get_presentation_by_id,
    handle_get_presenter_notes, handle_import_presentation, handle_list_cursors, handle_list_media,
    handle_list_presentations, handle_list_selections, handle_load_presentation,
    handle_new_presentation, handle_remove_transition, handle_reorder_slides,
    handle_save_presentation, handle_search_presentations, handle_set_transition,
    handle_slides_ai, handle_start_presenter, handle_update_cursor, handle_update_element,
    handle_update_media, handle_update_presenter, handle_update_selection,
    handle_update_slide_notes,
};
pub use types::{
    Animation, ChartData, ChartDataset, Collaborator, CollaborationCursor,
    CollaborationSelection, ElementContent, ElementStyle, GradientStop, GradientStyle,
    MediaElement, Presentation, PresentationMetadata, PresentationTheme, PresenterSession,
    PresenterViewSettings, SaveResponse, ShadowStyle, Slide, SlideBackground, SlideElement,
    SlideMessage, SlideTransition, TableCell, TableData, ThemeColors, ThemeFonts,
    TransitionConfig, TransitionSound,
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
        .route("/api/slides/import", post(handle_import_presentation))
        .route("/api/slides/cursor", post(handle_update_cursor))
        .route("/api/slides/selection", post(handle_update_selection))
        .route("/api/slides/cursors", get(handle_list_cursors))
        .route("/api/slides/selections", get(handle_list_selections))
        .route("/api/slides/transition", post(handle_set_transition))
        .route("/api/slides/transition/all", post(handle_apply_transition_to_all))
        .route("/api/slides/transition/remove", post(handle_remove_transition))
        .route("/api/slides/media", post(handle_add_media))
        .route("/api/slides/media/update", post(handle_update_media))
        .route("/api/slides/media/delete", post(handle_delete_media))
        .route("/api/slides/media/list", get(handle_list_media))
        .route("/api/slides/presenter/start", post(handle_start_presenter))
        .route("/api/slides/presenter/update", post(handle_update_presenter))
        .route("/api/slides/presenter/end", post(handle_end_presenter))
        .route("/api/slides/presenter/notes", get(handle_get_presenter_notes))
        .route("/api/slides/:presentation_id/presence", get(handle_get_presence))
        .route("/api/slides/:presentation_id/typing", get(handle_get_typing))
        .route("/api/slides/:presentation_id/selections", get(handle_get_selections))
        .route("/api/slides/mentions/:user_id", get(handle_get_mentions))
        .route("/ws/slides/:presentation_id", get(handle_slides_websocket))
}
