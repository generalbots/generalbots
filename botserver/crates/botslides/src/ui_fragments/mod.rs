pub mod panels;
pub mod sidebars;
pub mod toolbar;
pub mod modals;
pub mod themes;

use axum::{routing::{get, post}, Router};

use panels::{
    handle_slide_list_panel, handle_theme_panel, handle_transitions_panel,
    handle_animations_panel, handle_media_panel, handle_presenter_panel,
    handle_collaborators_panel, handle_slide_sorter,
};
use sidebars::{
    handle_presentation_list_sidebar, handle_search_sidebar, handle_recent_sidebar,
    handle_slide_thumbnails, handle_presentation_view,
        handle_presentation_view_by_id,
};
use toolbar::{
    handle_share_form, handle_ai_form, handle_new_slide_form, handle_insert_element_form,
    handle_add_text_form, handle_add_image_form, handle_add_shape_form, handle_add_chart_form,
    handle_add_table_form, handle_notes_form, handle_background_form,
    handle_transition_form, handle_animation_form, handle_export_form, handle_master_form,
};
use modals::{
    handle_share_modal, handle_ai_modal, handle_new_slide_modal, handle_insert_element_modal,
    handle_add_text_modal, handle_add_image_modal, handle_add_shape_modal, handle_add_chart_modal,
    handle_add_table_modal, handle_notes_modal, handle_background_modal,
    handle_transition_modal, handle_animation_modal, handle_export_modal, handle_master_modal,
};
use themes::handle_theme_picker;

pub fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

pub fn err_fragment(msg: &str) -> String {
    format!(
        r##"<div class="sl-error" role="alert" style="padding:12px;border:1px solid #f87171;background:#7f1d1d;color:#fecaca;border-radius:6px;margin:8px 0;">
<strong>Erro:</strong> {}</div>"##,
        html_escape(msg)
    )
}

pub fn empty_fragment(text: &str) -> String {
    format!(
        r##"<div class="sl-empty" style="padding:24px;text-align:center;color:#94a3b8;font-style:italic;">
{}</div>"##,
        html_escape(text)
    )
}

pub fn configure<D: crate::storage::DriveOps + Send + Sync + 'static>() -> Router<std::sync::Arc<crate::SlidesState<D>>> {
    Router::new()
        .route("/suite/slides/fragments/presentation-list", get(handle_presentation_list_sidebar))
        .route("/suite/slides/fragments/search", get(handle_search_sidebar))
        .route("/suite/slides/fragments/recent", get(handle_recent_sidebar))
        .route("/suite/slides/fragments/view", post(handle_presentation_view))
        .route("/suite/slides/fragments/view-by-id", get(handle_presentation_view_by_id::<D>))
        .route("/suite/slides/fragments/thumbnails", post(handle_slide_thumbnails))
        .route("/suite/slides/fragments/slide-list", post(handle_slide_list_panel))
        .route("/suite/slides/fragments/theme", post(handle_theme_panel))
        .route("/suite/slides/fragments/themes-picker", post(handle_theme_picker))
        .route("/suite/slides/fragments/transitions", post(handle_transitions_panel))
        .route("/suite/slides/fragments/animations", post(handle_animations_panel))
        .route("/suite/slides/fragments/media", post(handle_media_panel))
        .route("/suite/slides/fragments/presenter", post(handle_presenter_panel))
        .route("/suite/slides/fragments/collaborators", post(handle_collaborators_panel))
        .route("/suite/slides/fragments/sorter", post(handle_slide_sorter))
        .route("/suite/slides/forms/share", post(handle_share_form))
        .route("/suite/slides/forms/ai", post(handle_ai_form))
        .route("/suite/slides/forms/new-slide", post(handle_new_slide_form))
        .route("/suite/slides/forms/insert-element", post(handle_insert_element_form))
        .route("/suite/slides/forms/add-text", post(handle_add_text_form))
        .route("/suite/slides/forms/add-image", post(handle_add_image_form))
        .route("/suite/slides/forms/add-shape", post(handle_add_shape_form))
        .route("/suite/slides/forms/add-chart", post(handle_add_chart_form))
        .route("/suite/slides/forms/add-table", post(handle_add_table_form))
        .route("/suite/slides/forms/notes", post(handle_notes_form))
        .route("/suite/slides/forms/background", post(handle_background_form))
        .route("/suite/slides/forms/transition", post(handle_transition_form))
        .route("/suite/slides/forms/animation", post(handle_animation_form))
        .route("/suite/slides/forms/export", post(handle_export_form))
        .route("/suite/slides/forms/master", post(handle_master_form))
        .route("/suite/slides/modals/share", post(handle_share_modal))
        .route("/suite/slides/modals/ai", post(handle_ai_modal))
        .route("/suite/slides/modals/new-slide", post(handle_new_slide_modal))
        .route("/suite/slides/modals/insert-element", post(handle_insert_element_modal))
        .route("/suite/slides/modals/add-text", post(handle_add_text_modal))
        .route("/suite/slides/modals/add-image", post(handle_add_image_modal))
        .route("/suite/slides/modals/add-shape", post(handle_add_shape_modal))
        .route("/suite/slides/modals/add-chart", post(handle_add_chart_modal))
        .route("/suite/slides/modals/add-table", post(handle_add_table_modal))
        .route("/suite/slides/modals/notes", post(handle_notes_modal))
        .route("/suite/slides/modals/background", post(handle_background_modal))
        .route("/suite/slides/modals/transition", post(handle_transition_modal))
        .route("/suite/slides/modals/animation", post(handle_animation_modal))
        .route("/suite/slides/modals/export", post(handle_export_modal))
        .route("/suite/slides/modals/master", post(handle_master_modal))
        .route("/suite/slides/health", get(health))
}

async fn health() -> &'static str { "ok" }

pub fn render_metadata_card(m: &crate::types::PresentationMetadata) -> String {
    format!(
        r##"<div class="sl-metadata-card" hx-post="/suite/slides/fragments/view" hx-vals='{{"id":"{id}","action":"load"}}' hx-target="#slides-content" hx-swap="innerHTML" style="padding:12px;border:1px solid #334155;border-radius:6px;background:#1e293b;cursor:pointer;">
<div style="font-weight:600;color:#f8fafc;">{name}</div>
<div style="font-size:12px;color:#94a3b8;margin-top:4px;">{slides} slide(s) • Atualizado {updated}</div>
</div>"##,
        id = html_escape(&m.id),
        name = html_escape(&m.name),
        slides = m.slide_count,
        updated = m.updated_at.format("%d/%m/%Y %H:%M")
    )
}
