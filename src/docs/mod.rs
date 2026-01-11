pub mod collaboration;
pub mod handlers;
pub mod storage;
pub mod types;
pub mod utils;

use crate::shared::state::AppState;
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

pub use collaboration::handle_docs_websocket;
pub use handlers::{
    handle_ai_custom, handle_ai_expand, handle_ai_improve, handle_ai_simplify, handle_ai_summarize,
    handle_ai_translate, handle_autosave, handle_delete_document, handle_docs_ai, handle_docs_get_by_id,
    handle_docs_save, handle_export_docx, handle_export_html, handle_export_md, handle_export_pdf,
    handle_export_txt, handle_get_document, handle_list_documents, handle_new_document,
    handle_save_document, handle_search_documents, handle_template_blank, handle_template_letter,
    handle_template_meeting, handle_template_report,
};
pub use types::{
    AiRequest, AiResponse, Collaborator, CollabMessage, Document, DocumentMetadata, SaveRequest,
    SaveResponse, SearchQuery,
};

pub fn configure_docs_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/docs/list", get(handle_list_documents))
        .route("/api/docs/search", get(handle_search_documents))
        .route("/api/docs/load", get(handle_get_document))
        .route("/api/docs/save", post(handle_docs_save))
        .route("/api/docs/autosave", post(handle_autosave))
        .route("/api/docs/delete", post(handle_delete_document))
        .route("/api/docs/new", get(handle_new_document))
        .route("/api/docs/ai", post(handle_docs_ai))
        .route("/api/docs/:id", get(handle_docs_get_by_id))
        .route("/api/docs/template/blank", get(handle_template_blank))
        .route("/api/docs/template/meeting", get(handle_template_meeting))
        .route("/api/docs/template/report", get(handle_template_report))
        .route("/api/docs/template/letter", get(handle_template_letter))
        .route("/api/docs/ai/summarize", post(handle_ai_summarize))
        .route("/api/docs/ai/expand", post(handle_ai_expand))
        .route("/api/docs/ai/improve", post(handle_ai_improve))
        .route("/api/docs/ai/simplify", post(handle_ai_simplify))
        .route("/api/docs/ai/translate", post(handle_ai_translate))
        .route("/api/docs/ai/custom", post(handle_ai_custom))
        .route("/api/docs/export/pdf", get(handle_export_pdf))
        .route("/api/docs/export/docx", get(handle_export_docx))
        .route("/api/docs/export/md", get(handle_export_md))
        .route("/api/docs/export/html", get(handle_export_html))
        .route("/api/docs/export/txt", get(handle_export_txt))
        .route("/ws/docs/:doc_id", get(handle_docs_websocket))
}
