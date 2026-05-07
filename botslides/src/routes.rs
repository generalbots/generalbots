use crate::collaboration::{
    handle_get_collaborators, handle_get_mentions, handle_get_presence, handle_get_selections,
    handle_get_typing, handle_slides_websocket,
};
use crate::handlers::{
    handle_add_element, handle_add_media, handle_add_slide, handle_apply_theme,
    handle_apply_transition_to_all, handle_delete_element, handle_delete_media,
    handle_delete_presentation, handle_delete_slide, handle_duplicate_slide, handle_end_presenter,
    handle_export_presentation, handle_get_presentation_by_id, handle_get_presenter_notes,
    handle_import_presentation, handle_list_cursors, handle_list_media,
    handle_list_presentations, handle_list_selections, handle_load_presentation,
    handle_new_presentation, handle_remove_transition, handle_reorder_slides,
    handle_save_presentation, handle_search_presentations, handle_set_transition,
    handle_slides_ai, handle_start_presenter, handle_update_cursor, handle_update_element,
    handle_update_media, handle_update_presenter, handle_update_selection,
    handle_update_slide_notes,
};
use crate::storage::DriveOps;
use crate::SlidesState;
use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;

pub fn configure_slides_routes<D: DriveOps + Send + Sync + 'static>() -> Router<Arc<SlidesState<D>>> {
    Router::new()
        .route("/api/slides/list", get(handle_list_presentations::<D>))
        .route("/api/slides/search", get(handle_search_presentations::<D>))
        .route("/api/slides/load", get(handle_load_presentation::<D>))
        .route("/api/slides/save", post(handle_save_presentation::<D>))
        .route("/api/slides/delete", post(handle_delete_presentation::<D>))
        .route("/api/slides/new", get(handle_new_presentation::<D>))
        .route("/api/slides/ai", post(handle_slides_ai::<D>))
        .route("/api/slides/:id", get(handle_get_presentation_by_id::<D>))
        .route(
            "/api/slides/:id/collaborators",
            get(handle_get_collaborators),
        )
        .route("/api/slides/slide/add", post(handle_add_slide::<D>))
        .route(
            "/api/slides/slide/delete",
            post(handle_delete_slide::<D>),
        )
        .route(
            "/api/slides/slide/duplicate",
            post(handle_duplicate_slide::<D>),
        )
        .route(
            "/api/slides/slide/reorder",
            post(handle_reorder_slides::<D>),
        )
        .route(
            "/api/slides/slide/notes",
            post(handle_update_slide_notes::<D>),
        )
        .route(
            "/api/slides/element/add",
            post(handle_add_element::<D>),
        )
        .route(
            "/api/slides/element/update",
            post(handle_update_element::<D>),
        )
        .route(
            "/api/slides/element/delete",
            post(handle_delete_element::<D>),
        )
        .route("/api/slides/theme", post(handle_apply_theme::<D>))
        .route(
            "/api/slides/export",
            post(handle_export_presentation::<D>),
        )
        .route(
            "/api/slides/import",
            post(handle_import_presentation::<D>),
        )
        .route("/api/slides/cursor", post(handle_update_cursor::<D>))
        .route(
            "/api/slides/selection",
            post(handle_update_selection::<D>),
        )
        .route("/api/slides/cursors", get(handle_list_cursors::<D>))
        .route(
            "/api/slides/selections",
            get(handle_list_selections::<D>),
        )
        .route(
            "/api/slides/transition",
            post(handle_set_transition::<D>),
        )
        .route(
            "/api/slides/transition/all",
            post(handle_apply_transition_to_all::<D>),
        )
        .route(
            "/api/slides/transition/remove",
            post(handle_remove_transition::<D>),
        )
        .route("/api/slides/media", post(handle_add_media::<D>))
        .route(
            "/api/slides/media/update",
            post(handle_update_media::<D>),
        )
        .route(
            "/api/slides/media/delete",
            post(handle_delete_media::<D>),
        )
        .route("/api/slides/media/list", get(handle_list_media::<D>))
        .route(
            "/api/slides/presenter/start",
            post(handle_start_presenter::<D>),
        )
        .route(
            "/api/slides/presenter/update",
            post(handle_update_presenter::<D>),
        )
        .route(
            "/api/slides/presenter/end",
            post(handle_end_presenter::<D>),
        )
        .route(
            "/api/slides/presenter/notes",
            get(handle_get_presenter_notes::<D>),
        )
        .route(
            "/api/slides/:presentation_id/presence",
            get(handle_get_presence),
        )
        .route(
            "/api/slides/:presentation_id/typing",
            get(handle_get_typing),
        )
        .route(
            "/api/slides/:presentation_id/selections",
            get(handle_get_selections),
        )
        .route(
            "/api/slides/mentions/:user_id",
            get(handle_get_mentions),
        )
        .route(
            "/ws/slides/:presentation_id",
            get(handle_slides_websocket::<D>),
        )
}
