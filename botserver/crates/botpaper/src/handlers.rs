use crate::state::PaperState;
use crate::PAPER_BY_ID;
use crate::PAPER_LIST;
use crate::PAPER_NEW;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use super::auth::PaperUser;
use super::models::SaveRequest;
use super::storage::{
    delete_document_from_drive, list_documents_from_drive, load_document_from_drive,
    save_document_to_drive,
};
use super::utils::{
    format_document_content, format_document_list_item, format_error, format_relative_time,
    html_escape,
};

pub async fn handle_new_document(
    State(state): State<Arc<PaperState>>,
    user: PaperUser,
) -> impl IntoResponse {
    let user_identifier = user.owner_id().to_string();
    let user_id = user_identifier.clone();

    let doc_id = Uuid::new_v4().to_string();
    let title = "Untitled".to_string();
    let content = String::new();

    if let Err(e) =
        save_document_to_drive(&state, &user_identifier, &doc_id, &title, &content, false).await
    {
        log::error!("Failed to save new document: {}", e);
    }

    let mut html = String::new();
    html.push_str("<div class=\"paper-new-created\" data-id=\"");
    html.push_str(&html_escape(&doc_id));
    html.push_str("\">");
    html.push_str(&format_document_list_item(
        &doc_id, &title, "just now", true,
    ));
    html.push_str("<script>");
    html.push_str("htmx.trigger('#paper-list', 'refresh');");
    html.push_str(&format!(
        "htmx.ajax('GET', '{}', {{target: '#editor-content', swap: 'innerHTML'}});",
        PAPER_BY_ID.replace(":id", &html_escape(&doc_id))
    ));
    html.push_str("</script>");
    html.push_str("</div>");

    log::info!("New document created: {} for user {}", doc_id, user_id);
    Html(html)
}

pub async fn handle_list_documents(
    State(state): State<Arc<PaperState>>,
    user: PaperUser,
) -> impl IntoResponse {
    let user_identifier = user.owner_id().to_string();

    let documents = match list_documents_from_drive(&state, &user_identifier).await {
        Ok(docs) => docs,
        Err(e) => {
            log::error!("Failed to list documents: {}", e);
            Vec::new()
        }
    };

    let mut html = String::new();
    html.push_str("<div class=\"paper-list\">");

    if documents.is_empty() {
        html.push_str("<div class=\"paper-empty\">");
        html.push_str("<p>No documents yet</p>");
        html.push_str(&format!(
            "<button class=\"btn-new\" hx-post=\"{}\" hx-target=\"#paper-list\" hx-swap=\"afterbegin\">Create your first document</button>",
            PAPER_NEW
        ));
        html.push_str("</div>");
    } else {
        for doc in documents {
            let time_str = format_relative_time(doc.updated_at);
            let badge = if doc.storage_type == "named" {
                " 📁"
            } else {
                ""
            };
            html.push_str(&format_document_list_item(
                &doc.id,
                &format!("{}{}", doc.title, badge),
                &time_str,
                false,
            ));
        }
    }

    html.push_str("</div>");
    Html(html)
}

pub async fn handle_search_documents(
    State(state): State<Arc<PaperState>>,
    user: PaperUser,
    Query(params): Query<super::models::SearchQuery>,
) -> impl IntoResponse {
    let user_identifier = user.owner_id().to_string();

    let query = params.q.unwrap_or_default().to_lowercase();

    let documents = list_documents_from_drive(&state, &user_identifier)
        .await
        .unwrap_or_default();

    let filtered: Vec<_> = if query.is_empty() {
        documents
    } else {
        documents
            .into_iter()
            .filter(|d| d.title.to_lowercase().contains(&query))
            .collect()
    };

    let mut html = String::new();
    html.push_str("<div class=\"paper-search-results\">");

    if filtered.is_empty() {
        html.push_str("<div class=\"paper-empty\">");
        html.push_str("<p>No documents found</p>");
        html.push_str("</div>");
    } else {
        for doc in filtered {
            let time_str = format_relative_time(doc.updated_at);
            html.push_str(&format_document_list_item(
                &doc.id, &doc.title, &time_str, false,
            ));
        }
    }

    html.push_str("</div>");
    Html(html)
}

pub async fn handle_get_document(
    State(state): State<Arc<PaperState>>,
    user: PaperUser,
    Path(id): Path<String>,
) -> Result<Html<String>, (StatusCode, Html<String>)> {
    let user_identifier = user.owner_id().to_string();

    match load_document_from_drive(&state, &user_identifier, &id).await {
        Ok(Some(doc)) => Ok(Html(format_document_content(&doc.title, &doc.content))),
        // A document that is not under the caller's prefix is a 404 — it used
        // to render an empty editor, which looked like a successful open of
        // somebody else's (or a deleted) document (#2137).
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Html(format_error("Document not found")),
        )),
        Err(e) => {
            log::error!("Failed to load document {}: {}", id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(format_error("Failed to load document")),
            ))
        }
    }
}

pub async fn handle_save_document(
    State(state): State<Arc<PaperState>>,
    user: PaperUser,
    Json(payload): Json<SaveRequest>,
) -> impl IntoResponse {
    let user_identifier = user.owner_id().to_string();

    let doc_id = payload.id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let title = payload.title.unwrap_or_else(|| "Untitled".to_string());
    let content = payload.content.unwrap_or_default();
    let is_named = payload.save_as_named.unwrap_or(false);

    match save_document_to_drive(
        &state, &user_identifier, &doc_id, &title, &content, is_named,
    )
    .await
    {
        Ok(path) => {
            log::info!("Document saved: {} at {}", doc_id, path);
            let mut html = String::new();
            html.push_str("<div class=\"save-success\">");
            html.push_str("<span class=\"save-icon\">*</span>");
            html.push_str("<span>Saved</span>");
            html.push_str("</div>");
            Html(html)
        }
        Err(e) => {
            log::error!("Failed to save document: {}", e);
            Html(format_error("Failed to save document"))
        }
    }
}

pub async fn handle_paper_import(
    State(state): State<Arc<PaperState>>,
    user: PaperUser,
    Json(payload): Json<SaveRequest>,
) -> impl IntoResponse {
    let user_identifier = user.owner_id().to_string();

    let doc_id = Uuid::new_v4().to_string();
    let title = payload.title.unwrap_or_else(|| "Imported from Research".to_string());
    let content = payload.content.unwrap_or_default();

    match save_document_to_drive(
        &state, &user_identifier, &doc_id, &title, &content, false,
    )
    .await
    {
        Ok(path) => {
            log::info!("Document imported to Paper: {} at {}", doc_id, path);
            let mut html = String::new();
            html.push_str("<div class=\"paper-import-success\" data-id=\"");
            html.push_str(&html_escape(&doc_id));
            html.push_str("\">");
            html.push_str("<script>");
            html.push_str("htmx.trigger('#paper-list', 'refresh');");
            html.push_str("</script>");
            html.push_str("</div>");
            Html(html)
        }
        Err(e) => {
            log::error!("Failed to import document: {}", e);
            Html(format_error("Failed to import document"))
        }
    }
}

pub async fn handle_autosave(
    State(state): State<Arc<PaperState>>,
    user: PaperUser,
    Json(payload): Json<SaveRequest>,
) -> impl IntoResponse {
    let user_identifier = user.owner_id().to_string();

    let doc_id = payload.id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let title = payload.title.unwrap_or_else(|| "Untitled".to_string());
    let content = payload.content.unwrap_or_default();

    if let Err(e) =
        save_document_to_drive(&state, &user_identifier, &doc_id, &title, &content, false).await
    {
        log::warn!("Autosave failed for {}: {}", doc_id, e);
    }

    Html("<span class=\"autosave-indicator\">Auto-saved</span>".to_string())
}

pub async fn handle_delete_document(
    State(state): State<Arc<PaperState>>,
    user: PaperUser,
    Path(id): Path<String>,
) -> Result<Html<String>, (StatusCode, Html<String>)> {
    let user_identifier = user.owner_id().to_string();

    match delete_document_from_drive(&state, &user_identifier, &id).await {
        Ok(()) => {
            log::info!("Document deleted: {}", id);
            Ok(Html(format!(
                "<div class=\"delete-success\" hx-trigger=\"load\" hx-get=\"{}\" hx-target=\"#paper-list\" hx-swap=\"innerHTML\"></div>",
                PAPER_LIST
            )))
        }
        Err(e) if e == "Document not found" => Err((
            StatusCode::NOT_FOUND,
            Html(format_error("Document not found")),
        )),
        Err(e) => {
            log::error!("Failed to delete document {}: {}", id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(format_error("Failed to delete document")),
            ))
        }
    }
}
