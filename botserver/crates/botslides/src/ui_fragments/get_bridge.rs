//! GET bridge for the slides shell (#1707).
//!
//! `slides.html` and its modules load every form/modal/panel fragment with
//! `hx-get` (hx-vals become query parameters), while the handlers were
//! registered as POST with a JSON body — each of those calls answered
//! 404/405 and the app rendered as an empty shell. This bridge accepts GET
//! with query parameters, converts them to the `Json` payload the existing
//! handlers expect and dispatches by path. POST callers keep working: the
//! original routes stay registered unchanged.

use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::response::Html;
use axum::Json;

use crate::storage::DriveOps;
use crate::types::Presentation;
use crate::SlidesState;

type Params = HashMap<String, String>;

fn to_json(params: Params) -> Json<serde_json::Value> {
    Json(serde_json::Value::Object(params.into_iter().collect()))
}

/// The deck an empty session starts from: the rest of the app addresses it
/// as `current` (saves and canvas operations use that literal id).
fn blank_presentation() -> Presentation {
    let theme = crate::utils::create_default_theme();
    Presentation {
        id: "current".to_string(),
        name: "Untitled Presentation".to_string(),
        owner_id: crate::storage::get_current_user_id(),
        slides: vec![crate::utils::create_blank_slide(&theme)],
        theme,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

/// Resolves the deck for a fragment load: `new`/empty never touches Drive,
/// any other id loads from Drive and falls back to a blank deck on miss
/// (a missing `gbo` bucket on a fresh install must not break the shell).
pub(super) async fn load_or_blank<D: DriveOps>(state: &Arc<SlidesState<D>>, id: &str) -> Presentation {
    if id.is_empty() || id == "new" {
        return blank_presentation();
    }
    let user_id = crate::storage::get_current_user_id();
    match state.drive.as_ref() {
        Some(drive) => {
            crate::storage::load_presentation_by_id(drive, &user_id, id)
                .await
                .unwrap_or_else(|_| blank_presentation())
        }
        None => blank_presentation(),
    }
}

fn thumbnails_html(pres: &Presentation) -> Html<String> {
    let mut html = String::from(
        r#"<div style="display:flex;flex-direction:column;gap:8px;padding:8px;">"#,
    );
    if pres.slides.is_empty() {
        html.push_str(&super::empty_fragment("Nenhum slide. Adicione um slide para começar."));
    }
    for (index, slide) in pres.slides.iter().enumerate() {
        html.push_str(&format!(
            r#"<div class="sl-thumb" data-index="{index}" style="background:#0f172a;border:1px solid #334155;border-radius:6px;padding:6px;cursor:pointer;">
<div style="font-size:10px;color:#94a3b8;margin-bottom:4px;">Slide {n}</div>
<div style="background:#fff;border-radius:4px;overflow:hidden;aspect-ratio:16/9;">{preview}</div>
</div>"#,
            index = index,
            n = index + 1,
            preview = super::sidebars::render_slide(slide)
        ));
    }
    html.push_str("</div>");
    Html(html)
}

/// GET /suite/slides/*path — see module docs.
pub async fn get_bridge<D: DriveOps + Send + Sync + 'static>(
    State(state): State<Arc<SlidesState<D>>>,
    Path(path): Path<String>,
    Query(params): Query<Params>,
) -> Html<String> {
    let query = params.clone();
    let json = to_json(params);
    match path.as_str() {
        "fragments/presentation-view" => {
            let id = query.get("id").cloned().unwrap_or_default();
            let pres = load_or_blank(&state, &id).await;
            super::sidebars::handle_presentation_view(Json(pres)).await
        }
        "fragments/slide-thumbnails" => {
            let id = query.get("id").cloned().unwrap_or_default();
            let pres = load_or_blank(&state, &id).await;
            thumbnails_html(&pres)
        }
        "fragments/presentation-list" => {
            let items = match state.drive.as_ref() {
                Some(drive) => {
                    crate::storage::list_presentations_from_drive(
                        drive,
                        &crate::storage::get_current_user_id(),
                    )
                    .await
                    .unwrap_or_default()
                }
                None => Vec::new(),
            };
            super::sidebars::handle_presentation_list_sidebar(Json(items)).await
        }
        // "+ Nova": an unsaved deck addressed as `current`, rendered inline
        // instead of dumping the JSON of /api/slides/new into the canvas.
        "fragments/new" => {
            super::sidebars::handle_presentation_view(Json(blank_presentation())).await
        }
        "fragments/themes" => super::themes::handle_theme_picker(json).await,
        "fragments/presenter" => super::panels::handle_presenter_panel(json).await,
        "modals/new-slide" => super::modals::handle_new_slide_modal(json).await,
        "forms/add-text" => super::toolbar::handle_add_text_form(json).await,
        "forms/add-image" => super::toolbar::handle_add_image_form(json).await,
        "forms/add-shape" => super::toolbar::handle_add_shape_form(json).await,
        "forms/add-chart" => super::toolbar::handle_add_chart_form(json).await,
        "forms/add-table" => super::toolbar::handle_add_table_form(json).await,
        "forms/notes" => super::toolbar::handle_notes_form(json).await,
        "forms/background" => super::toolbar::handle_background_form(json).await,
        "forms/transition" => super::toolbar::handle_transition_form(json).await,
        "forms/animation" => super::toolbar::handle_animation_form(json).await,
        "forms/master" => super::toolbar::handle_master_form(json).await,
        "forms/export" => super::toolbar::handle_export_form(json).await,
        _ => Html(super::err_fragment(&format!("route not found: {path}"))),
    }
}
