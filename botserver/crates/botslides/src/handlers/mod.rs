pub mod ai;
pub mod collab;
pub mod export;
pub mod import;
pub mod media;
pub mod presenter;
pub mod presentations;
pub mod slides;

pub use ai::handle_slides_ai;
pub use collab::{
    handle_list_cursors, handle_list_selections, handle_update_cursor, handle_update_selection,
};
pub use export::handle_export_presentation;
pub use import::handle_import_presentation;
pub use media::{
    handle_add_media, handle_delete_media, handle_list_media, handle_update_media,
};
pub use presenter::{
    handle_end_presenter, handle_get_presenter_notes, handle_start_presenter,
    handle_update_presenter,
};
pub use presentations::{
    handle_delete_presentation, handle_get_presentation_by_id, handle_list_presentations,
    handle_load_presentation, handle_new_presentation, handle_save_presentation,
    handle_search_presentations,
};
pub use slides::{
    handle_add_element, handle_add_slide, handle_apply_theme,
    handle_apply_transition_to_all, handle_delete_element, handle_delete_slide,
    handle_duplicate_slide, handle_remove_transition, handle_reorder_slides,
    handle_set_transition, handle_update_element, handle_update_slide_notes,
};
