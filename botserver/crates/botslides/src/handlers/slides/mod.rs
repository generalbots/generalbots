//! Slide, element, theme and transition handlers.
//!
//! `load_pres!`/`save_pres!` are declared before the child modules: `macro_rules!`
//! scoping is textual, so every submodule below can expand them. The macros use
//! fully qualified paths, so submodules do not need extra imports for them.

macro_rules! load_pres {
    ($state:expr, $user_id:expr, $pres_id:expr) => {{
        let requested_id = $pres_id.to_string();
        match $state.drive.as_ref() {
            Some(drive) => match $crate::storage::load_presentation_by_id(drive, $user_id, &requested_id).await {
                Ok(p) => p,
                // The blank deck rendered as `current` is not persisted until
                // the first mutation; without this fallback every element,
                // slide and notes call on a fresh deck returned 404.
                Err(_) if requested_id == "current" => {
                    $crate::ui_fragments::get_bridge::blank_presentation($user_id)
                }
                Err(e) => {
                    return Err((
                        axum::http::StatusCode::NOT_FOUND,
                        axum::Json(serde_json::json!({ "error": e })),
                    ))
                }
            },
            None => {
                return Err((
                    axum::http::StatusCode::SERVICE_UNAVAILABLE,
                    axum::Json(serde_json::json!({ "error": "Drive not configured" })),
                ))
            }
        }
    }};
}

macro_rules! save_pres {
    ($state:expr, $user_id:expr, $presentation:expr) => {
        if let Some(drive) = &$state.drive {
            if let Err(e) = $crate::storage::save_presentation_to_drive(drive, $user_id, $presentation).await {
                return Err((
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    axum::Json(serde_json::json!({ "error": botsecurity_core::error_sanitizer::log_and_sanitize_json(&e, "botslides::handlers::slides", None)})),
                ));
            }
        }
    };
}

mod element_ops;
mod slide_ops;
mod theme_ops;
mod wire;

pub use element_ops::{handle_add_element, handle_delete_element, handle_update_element};
pub use slide_ops::{
    handle_add_slide, handle_delete_slide, handle_duplicate_slide, handle_reorder_slides,
    handle_update_slide_notes,
};
pub use theme_ops::{
    handle_apply_theme, handle_apply_transition_to_all, handle_remove_transition,
    handle_set_transition,
};
