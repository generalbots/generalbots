use super::{empty_fragment, err_fragment, html_escape, render_metadata_card};
use axum::{extract::{Query, State}, response::Html, Json};
use crate::state::DocState;
use crate::types_core::DocumentMetadata;

/// Server-rendered document view resolved by id (metadata cards carry only
/// the id; the full Document is loaded from Drive here).
pub async fn handle_document_view_by_id(
    State(state): State<std::sync::Arc<DocState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Html<String> {
    let doc_id = params.get("id").cloned().unwrap_or_default();
    if doc_id.is_empty() {
        return Html(err_fragment("id ausente"));
    }
    let user_id = crate::storage::get_current_user_id();
    match crate::storage_drive::load_document_from_drive(&state, &user_id, &doc_id).await {
        Ok(Some(doc)) => {
            handle_document_view(Json(serde_json::json!({
                "id": doc.id,
                "title": doc.title,
                "content": doc.content,
            })))
            .await
        }
        _ => Html(err_fragment("Documento não encontrado")),
    }
}

pub async fn handle_doc_list_sidebar(Json(items): Json<Vec<DocumentMetadata>>) -> Html<String> {
    if items.is_empty() {
        return Html(empty_fragment("Nenhum documento. Crie um novo para começar."));
    }
    let mut html = String::from(
        r##"<div class="dc-sidebar" id="doc-list-sidebar">
<div class="dc-sidebar-header" style="display:flex;justify-content:space-between;align-items:center;padding:12px;border-bottom:1px solid #334155;">
<h3 style="margin:0;color:#f8fafc;font-size:14px;">Meus Documentos</h3>
<button hx-get="/api/docs/new" hx-target="#doc-content" hx-swap="innerHTML" style="background:#10b981;color:white;border:none;padding:4px 10px;border-radius:4px;cursor:pointer;font-size:12px;">+ Novo</button>
</div>
<div class="dc-sidebar-body" style="padding:8px;display:flex;flex-direction:column;gap:8px;max-height:calc(100vh - 200px);overflow-y:auto;">"##
    );
    for m in &items {
        html.push_str(&render_metadata_card(m));
    }
    html.push_str("</div></div>");
    Html(html)
}

pub async fn handle_search_sidebar(Json(items): Json<Vec<DocumentMetadata>>) -> Html<String> {
    if items.is_empty() {
        return Html(empty_fragment("Nenhum resultado para esta busca."));
    }
    let mut html = String::from(
        r##"<div class="dc-sidebar" id="search-sidebar">
<div class="dc-sidebar-header" style="padding:12px;border-bottom:1px solid #334155;">
<h3 style="margin:0;color:#f8fafc;font-size:14px;">Resultados da Busca</h3>
</div>
<div class="dc-sidebar-body" style="padding:8px;display:flex;flex-direction:column;gap:8px;">"##
    );
    for m in &items {
        html.push_str(&render_metadata_card(m));
    }
    html.push_str(&format!(
        r##"<div style="padding:8px;color:#94a3b8;font-size:12px;text-align:center;">{} documento(s) encontrado(s)</div>"##,
        items.len()
    ));
    html.push_str("</div></div>");
    Html(html)
}

pub async fn handle_recent_sidebar(Json(items): Json<Vec<DocumentMetadata>>) -> Html<String> {
    let recent: Vec<&DocumentMetadata> = items.iter().take(5).collect();
    if recent.is_empty() {
        return Html(empty_fragment("Nenhum documento recente."));
    }
    let mut html = String::from(
        r##"<div class="dc-sidebar" id="recent-sidebar">
<div class="dc-sidebar-header" style="padding:12px;border-bottom:1px solid #334155;">
<h3 style="margin:0;color:#f8fafc;font-size:14px;">Recentes</h3>
</div>
<div class="dc-sidebar-body" style="padding:8px;display:flex;flex-direction:column;gap:8px;">"##
    );
    for m in recent {
        html.push_str(&render_metadata_card(m));
    }
    html.push_str("</div></div>");
    Html(html)
}

pub async fn handle_document_view(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let title = payload.get("title").and_then(|v| v.as_str()).unwrap_or("Documento sem título");
    let content = payload.get("content").and_then(|v| v.as_str()).unwrap_or("");
    let id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("");
    let word_count = content.split_whitespace().count();
    Html(format!(
        r##"<div class="dc-view" id="doc-view" data-id="{id}">
<div class="dc-view-header" style="padding:16px;border-bottom:1px solid #334155;display:flex;justify-content:space-between;align-items:center;background:#0f172a;">
<div>
<h2 style="margin:0;color:#f8fafc;">{title}</h2>
<div style="font-size:12px;color:#94a3b8;margin-top:4px;">{words} palavras • ID: {id}</div>
</div>
<div style="display:flex;gap:8px;">
<button hx-post="/suite/docs/modals/ai" hx-vals='{{"id":"{id}"}}' hx-target="#modal-container" hx-swap="innerHTML" style="background:#6366f1;color:white;border:none;padding:6px 12px;border-radius:4px;cursor:pointer;font-size:12px;">✨ IA</button>
<button hx-post="/suite/docs/modals/share" hx-vals='{{"id":"{id}"}}' hx-target="#modal-container" hx-swap="innerHTML" style="background:#3b82f6;color:white;border:none;padding:6px 12px;border-radius:4px;cursor:pointer;font-size:12px;">🔗 Compartilhar</button>
<button hx-post="/suite/docs/modals/find-replace" hx-vals='{{"id":"{id}"}}' hx-target="#modal-container" hx-swap="innerHTML" style="background:#0f172a;color:#f8fafc;border:1px solid #334155;padding:6px 12px;border-radius:4px;cursor:pointer;font-size:12px;">🔍 Buscar</button>
</div>
</div>
<article class="dc-content" id="doc-content" contenteditable="true" hx-post="/api/docs/autosave" hx-trigger="blur changed" hx-vals='{{"id":"{id}"}}' hx-swap="none" style="padding:48px 64px;line-height:1.8;color:#cbd5e1;font-size:15px;background:#0f172a;min-height:60vh;outline:none;font-family:Georgia,serif;">{content}</article>
</div>"##,
        id = html_escape(id),
        title = html_escape(title),
        content = html_escape(content),
        words = word_count
    ))
}
