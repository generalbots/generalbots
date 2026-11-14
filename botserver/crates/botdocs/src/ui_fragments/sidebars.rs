use super::{empty_fragment, err_fragment, html_escape, render_metadata_card};
use crate::auth::DocUser;
use crate::state::DocState;
use crate::types_core::DocumentMetadata;
use axum::{
    Json,
    extract::{Query, State},
    response::Html,
};
use botuifragments::ui_fragments::{sidebar_shell, sidebar_shell_plain};

/// Server-rendered document view resolved by id (metadata cards carry only
/// the id; the full Document is loaded from Drive here).
pub async fn handle_document_view_by_id(
    State(state): State<std::sync::Arc<DocState>>,
    user: DocUser,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Html<String> {
    let doc_id = params.get("id").cloned().unwrap_or_default();
    if doc_id.is_empty() {
        return Html(err_fragment("id ausente"));
    }
    let user_id = user.id();
    match crate::storage_drive::load_document_from_drive(&state, user_id, &doc_id).await {
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
        return Html(empty_fragment(
            "Nenhum documento. Crie um novo para começar.",
        ));
    }
    let mut body = String::new();
    for m in &items {
        body.push_str(&render_metadata_card(m));
    }
    let action = r##"<button hx-get="/suite/docs/fragments/new" hx-target="#docs-content" hx-swap="innerHTML" style="background:#10b981;color:white;border:none;padding:4px 10px;border-radius:4px;cursor:pointer;font-size:12px;">+ Novo</button>"##;
    Html(sidebar_shell(
        "dc",
        "doc-list-sidebar",
        "Meus Documentos",
        action,
        &body,
    ))
}

pub async fn handle_search_sidebar(Json(items): Json<Vec<DocumentMetadata>>) -> Html<String> {
    if items.is_empty() {
        return Html(empty_fragment("Nenhum resultado para esta busca."));
    }
    let mut body = String::new();
    for m in &items {
        body.push_str(&render_metadata_card(m));
    }
    body.push_str(&format!(
        r##"<div style="padding:8px;color:#94a3b8;font-size:12px;text-align:center;">{} documento(s) encontrado(s)</div>"##,
        items.len()
    ));
    Html(sidebar_shell_plain(
        "dc",
        "search-sidebar",
        "Resultados da Busca",
        &body,
    ))
}

pub async fn handle_recent_sidebar(Json(items): Json<Vec<DocumentMetadata>>) -> Html<String> {
    let recent: Vec<&DocumentMetadata> = items.iter().take(5).collect();
    if recent.is_empty() {
        return Html(empty_fragment("Nenhum documento recente."));
    }
    let mut body = String::new();
    for m in recent {
        body.push_str(&render_metadata_card(m));
    }
    Html(sidebar_shell_plain(
        "dc",
        "recent-sidebar",
        "Recentes",
        &body,
    ))
}

pub async fn handle_document_view(Json(payload): Json<serde_json::Value>) -> Html<String> {
    let title = payload
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Documento sem título");
    let content = payload
        .get("content")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("");
    let word_count = content.split_whitespace().count();
    Html(format!(
        r##"<div class="dc-view" id="doc-view" data-id="{id}" data-title="{title}">
<div class="dc-view-header" style="padding:16px;border-bottom:1px solid #334155;display:flex;justify-content:space-between;align-items:center;background:#0f172a;">
<div>
<h2 style="margin:0;color:#f8fafc;">{title}</h2>
<div style="font-size:12px;color:#94a3b8;margin-top:4px;">{words} palavras • ID: {id}</div>
</div>
<div style="display:flex;gap:8px;">
<button hx-post="/suite/docs/modals/ai" hx-vals='{{"id":"{id}"}}' hx-target="#docs-modal-container" hx-swap="innerHTML" style="background:#6366f1;color:white;border:none;padding:6px 12px;border-radius:4px;cursor:pointer;font-size:12px;">✨ IA</button>
<button hx-post="/suite/docs/modals/share" hx-vals='{{"id":"{id}"}}' hx-target="#docs-modal-container" hx-swap="innerHTML" style="background:#3b82f6;color:white;border:none;padding:6px 12px;border-radius:4px;cursor:pointer;font-size:12px;">🔗 Compartilhar</button>
<button hx-post="/suite/docs/modals/find-replace" hx-vals='{{"id":"{id}"}}' hx-target="#docs-modal-container" hx-swap="innerHTML" style="background:#0f172a;color:#f8fafc;border:1px solid #334155;padding:6px 12px;border-radius:4px;cursor:pointer;font-size:12px;">🔍 Buscar</button>
</div>
</div>
<!-- content is the document body authored by this editor; the JSON load path
     already renders it as HTML, so escaping it here showed raw tags. -->
<article class="dc-content docs-doc-view" id="doc-content" data-doc-id="{id}" contenteditable="true" style="padding:48px 64px;line-height:1.8;color:#cbd5e1;font-size:15px;background:#0f172a;min-height:60vh;outline:none;font-family:Georgia,serif;">{content}</article>
</div>"##,
        id = html_escape(id),
        title = html_escape(title),
        content = content,
        words = word_count
    ))
}

/// GET /suite/docs/fragments/new — creates an unsaved document view.
/// The id is a fresh UUID and the first autosave persists it, so an empty
/// document does not appear in the sidebar until it has content.
pub async fn handle_new_document_view(user: DocUser) -> Html<String> {
    let doc = crate::storage::create_new_document(user.id());
    handle_document_view(Json(serde_json::json!({
        "id": doc.id,
        "title": doc.title,
        "content": doc.content,
    })))
    .await
}
