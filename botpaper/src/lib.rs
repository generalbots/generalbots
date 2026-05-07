pub mod schema;
pub mod state;
pub mod models;
pub mod auth;
pub mod storage;
pub mod llm;
pub mod handlers;
pub mod ai_handlers;
pub mod export;
pub mod templates;
pub mod utils;

pub use models::*;
pub use auth::get_current_user;
pub use storage::{
    delete_document_from_drive, list_documents_from_drive, load_document_from_drive,
    save_document_to_drive,
};
pub use llm::call_llm;
pub use handlers::{
    handle_autosave, handle_delete_document, handle_get_document, handle_list_documents,
    handle_new_document, handle_save_document, handle_search_documents,
};
pub use templates::{
    handle_template_blank, handle_template_letter, handle_template_meeting,
    handle_template_report, handle_template_research, handle_template_todo,
};
pub use ai_handlers::{
    handle_ai_custom, handle_ai_expand, handle_ai_improve, handle_ai_simplify,
    handle_ai_summarize, handle_ai_translate,
};
pub use export::{
    handle_export_docx, handle_export_html, handle_export_md, handle_export_pdf,
    handle_export_txt,
};
pub use utils::{
    format_ai_response, format_document_content, format_document_list_item, format_error,
    format_relative_time, html_escape, markdown_to_html, strip_markdown,
};

use axum::{routing::get, routing::post, Router};
use std::sync::Arc;

use crate::state::PaperState;

pub const PAPER_NEW: &str = "/paper/new";
pub const PAPER_LIST: &str = "/paper/list";
pub const PAPER_SEARCH: &str = "/paper/search";
pub const PAPER_SAVE: &str = "/paper/save";
pub const PAPER_AUTOSAVE: &str = "/paper/autosave";
pub const PAPER_BY_ID: &str = "/paper/:id";
pub const PAPER_DELETE: &str = "/paper/delete/:id";
pub const PAPER_TEMPLATE_BLANK: &str = "/paper/template/blank";
pub const PAPER_TEMPLATE_MEETING: &str = "/paper/template/meeting";
pub const PAPER_TEMPLATE_TODO: &str = "/paper/template/todo";
pub const PAPER_TEMPLATE_RESEARCH: &str = "/paper/template/research";
pub const PAPER_TEMPLATE_REPORT: &str = "/paper/template/report";
pub const PAPER_TEMPLATE_LETTER: &str = "/paper/template/letter";
pub const PAPER_AI_SUMMARIZE: &str = "/paper/ai/summarize";
pub const PAPER_AI_EXPAND: &str = "/paper/ai/expand";
pub const PAPER_AI_IMPROVE: &str = "/paper/ai/improve";
pub const PAPER_AI_SIMPLIFY: &str = "/paper/ai/simplify";
pub const PAPER_AI_TRANSLATE: &str = "/paper/ai/translate";
pub const PAPER_AI_CUSTOM: &str = "/paper/ai/custom";
pub const PAPER_EXPORT_PDF: &str = "/paper/export/pdf";
pub const PAPER_EXPORT_DOCX: &str = "/paper/export/docx";
pub const PAPER_EXPORT_MD: &str = "/paper/export/md";
pub const PAPER_EXPORT_HTML: &str = "/paper/export/html";
pub const PAPER_EXPORT_TXT: &str = "/paper/export/txt";

pub fn configure_paper_routes() -> Router<Arc<PaperState>> {
    Router::new()
        .route(PAPER_NEW, post(handle_new_document))
        .route(PAPER_LIST, get(handle_list_documents))
        .route(PAPER_SEARCH, get(handle_search_documents))
        .route(PAPER_SAVE, post(handle_save_document))
        .route(PAPER_AUTOSAVE, post(handle_autosave))
        .route(PAPER_BY_ID, get(handle_get_document))
        .route(PAPER_DELETE, post(handle_delete_document))
        .route(PAPER_TEMPLATE_BLANK, post(handle_template_blank))
        .route(PAPER_TEMPLATE_MEETING, post(handle_template_meeting))
        .route(PAPER_TEMPLATE_TODO, post(handle_template_todo))
        .route(PAPER_TEMPLATE_RESEARCH, post(handle_template_research))
        .route(PAPER_TEMPLATE_REPORT, post(handle_template_report))
        .route(PAPER_TEMPLATE_LETTER, post(handle_template_letter))
        .route(PAPER_AI_SUMMARIZE, post(handle_ai_summarize))
        .route(PAPER_AI_EXPAND, post(handle_ai_expand))
        .route(PAPER_AI_IMPROVE, post(handle_ai_improve))
        .route(PAPER_AI_SIMPLIFY, post(handle_ai_simplify))
        .route(PAPER_AI_TRANSLATE, post(handle_ai_translate))
        .route(PAPER_AI_CUSTOM, post(handle_ai_custom))
        .route(PAPER_EXPORT_PDF, get(handle_export_pdf))
        .route(PAPER_EXPORT_DOCX, get(handle_export_docx))
        .route(PAPER_EXPORT_MD, get(handle_export_md))
        .route(PAPER_EXPORT_HTML, get(handle_export_html))
        .route(PAPER_EXPORT_TXT, get(handle_export_txt))
}
