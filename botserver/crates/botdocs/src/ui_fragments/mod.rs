pub mod panels;
pub mod sidebars;
pub mod toolbar;
pub mod modals;

use axum::{routing::{get, post}, Router};

use panels::{
    handle_toc_panel, handle_comments_panel, handle_track_changes_panel,
    handle_footnotes_panel, handle_endnotes_panel, handle_styles_panel,
    handle_outline_panel, handle_ai_panel as handle_ai_panel_inner,
};
use sidebars::{
    handle_doc_list_sidebar, handle_search_sidebar, handle_recent_sidebar,
    handle_document_view,
};
use toolbar::{
    handle_share_form, handle_ai_form, handle_find_replace_form, handle_print_form,
    handle_footnote_form, handle_endnote_form, handle_track_changes_form,
    handle_compare_form, handle_template_form,
};
use modals::{
    handle_share_modal, handle_ai_modal, handle_find_replace_modal,
    handle_footnote_modal, handle_endnote_modal, handle_track_changes_modal,
    handle_compare_modal, handle_template_modal, handle_comment_reply_modal,
    handle_style_modal,
};

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
        r##"<div class="dc-error" role="alert" style="padding:12px;border:1px solid #f87171;background:#7f1d1d;color:#fecaca;border-radius:6px;margin:8px 0;">
<strong>Erro:</strong> {}</div>"##,
        html_escape(msg)
    )
}

pub fn empty_fragment(text: &str) -> String {
    format!(
        r##"<div class="dc-empty" style="padding:24px;text-align:center;color:#94a3b8;font-style:italic;">
{}</div>"##,
        html_escape(text)
    )
}

pub fn configure<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route("/suite/docs/fragments/doc-list", get(handle_doc_list_sidebar))
        .route("/suite/docs/fragments/search", get(handle_search_sidebar))
        .route("/suite/docs/fragments/recent", get(handle_recent_sidebar))
        .route("/suite/docs/fragments/view", post(handle_document_view))
        .route("/suite/docs/fragments/toc", post(handle_toc_panel))
        .route("/suite/docs/fragments/comments", post(handle_comments_panel))
        .route("/suite/docs/fragments/track-changes", post(handle_track_changes_panel))
        .route("/suite/docs/fragments/footnotes", post(handle_footnotes_panel))
        .route("/suite/docs/fragments/endnotes", post(handle_endnotes_panel))
        .route("/suite/docs/fragments/styles", post(handle_styles_panel))
        .route("/suite/docs/fragments/outline", post(handle_outline_panel))
        .route("/suite/docs/fragments/ai", post(handle_ai_panel_inner))
        .route("/suite/docs/forms/share", post(handle_share_form))
        .route("/suite/docs/forms/ai", post(handle_ai_form))
        .route("/suite/docs/forms/find-replace", post(handle_find_replace_form))
        .route("/suite/docs/forms/print", post(handle_print_form))
        .route("/suite/docs/forms/footnote", post(handle_footnote_form))
        .route("/suite/docs/forms/endnote", post(handle_endnote_form))
        .route("/suite/docs/forms/track-changes", post(handle_track_changes_form))
        .route("/suite/docs/forms/compare", post(handle_compare_form))
        .route("/suite/docs/forms/template", post(handle_template_form))
        .route("/suite/docs/modals/share", post(handle_share_modal))
        .route("/suite/docs/modals/ai", post(handle_ai_modal))
        .route("/suite/docs/modals/find-replace", post(handle_find_replace_modal))
        .route("/suite/docs/modals/footnote", post(handle_footnote_modal))
        .route("/suite/docs/modals/endnote", post(handle_endnote_modal))
        .route("/suite/docs/modals/track-changes", post(handle_track_changes_modal))
        .route("/suite/docs/modals/compare", post(handle_compare_modal))
        .route("/suite/docs/modals/template", post(handle_template_modal))
        .route("/suite/docs/modals/comment-reply", post(handle_comment_reply_modal))
        .route("/suite/docs/modals/style", post(handle_style_modal))
        .route("/suite/docs/health", get(health))
}

async fn health() -> &'static str { "ok" }

pub fn render_metadata_card(m: &crate::types_core::DocumentMetadata) -> String {
    let storage = m.storage_type.clone();
    let storage_badge = match storage.as_str() {
        "drive" => r##"<span style="color:#10b981;font-size:10px;background:#064e3b;padding:2px 6px;border-radius:3px;">☁ Drive</span>"##,
        "local" => r##"<span style="color:#94a3b8;font-size:10px;background:#1e293b;padding:2px 6px;border-radius:3px;">💾 Local</span>"##,
        _ => r##"<span style="color:#94a3b8;font-size:10px;background:#1e293b;padding:2px 6px;border-radius:3px;">📄</span>"##,
    };
    format!(
        r##"<div class="dc-metadata-card" hx-get="/suite/docs/fragments/view" hx-vals='{{"id":"{id}","action":"load"}}' hx-target="#doc-content" hx-swap="innerHTML" style="padding:12px;border:1px solid #334155;border-radius:6px;background:#1e293b;cursor:pointer;">
<div style="display:flex;justify-content:space-between;align-items:start;gap:8px;">
<div style="font-weight:600;color:#f8fafc;flex:1;">{title}</div>
{storage_badge}
</div>
<div style="font-size:12px;color:#94a3b8;margin-top:4px;">{words} palavras • Atualizado {updated}</div>
</div>"##,
        id = html_escape(&m.id),
        title = html_escape(&m.title),
        words = m.word_count,
        updated = m.updated_at.format("%d/%m/%Y %H:%M"),
        storage_badge = storage_badge
    )
}
