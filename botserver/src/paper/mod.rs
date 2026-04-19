// Paper module - document management system
// This module has been split into submodules for better organization

pub mod ai_handlers;
pub mod auth;
pub mod export;
pub mod handlers;
pub mod llm;
pub mod models;
pub mod storage;
pub mod templates;
pub mod utils;

// Re-export public types and functions for backward compatibility
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

use axum::{routing::{get, post}, Router};
use std::sync::Arc;

use crate::core::shared::state::AppState;
use crate::core::urls::ApiUrls;

pub fn configure_paper_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(ApiUrls::PAPER_NEW, post(handle_new_document))
        .route(ApiUrls::PAPER_LIST, get(handle_list_documents))
        .route(ApiUrls::PAPER_SEARCH, get(handle_search_documents))
        .route(ApiUrls::PAPER_SAVE, post(handle_save_document))
        .route(ApiUrls::PAPER_AUTOSAVE, post(handle_autosave))
        .route(ApiUrls::PAPER_BY_ID, get(handle_get_document))
        .route(ApiUrls::PAPER_DELETE, post(handle_delete_document))
        .route(ApiUrls::PAPER_TEMPLATE_BLANK, post(handle_template_blank))
        .route(ApiUrls::PAPER_TEMPLATE_MEETING, post(handle_template_meeting))
        .route(ApiUrls::PAPER_TEMPLATE_TODO, post(handle_template_todo))
        .route(
            ApiUrls::PAPER_TEMPLATE_RESEARCH,
            post(handle_template_research),
        )
        .route(ApiUrls::PAPER_TEMPLATE_REPORT, post(handle_template_report))
        .route(ApiUrls::PAPER_TEMPLATE_LETTER, post(handle_template_letter))
        .route(ApiUrls::PAPER_AI_SUMMARIZE, post(handle_ai_summarize))
        .route(ApiUrls::PAPER_AI_EXPAND, post(handle_ai_expand))
        .route(ApiUrls::PAPER_AI_IMPROVE, post(handle_ai_improve))
        .route(ApiUrls::PAPER_AI_SIMPLIFY, post(handle_ai_simplify))
        .route(ApiUrls::PAPER_AI_TRANSLATE, post(handle_ai_translate))
        .route(ApiUrls::PAPER_AI_CUSTOM, post(handle_ai_custom))
        .route(ApiUrls::PAPER_EXPORT_PDF, get(handle_export_pdf))
        .route(ApiUrls::PAPER_EXPORT_DOCX, get(handle_export_docx))
        .route(ApiUrls::PAPER_EXPORT_MD, get(handle_export_md))
        .route(ApiUrls::PAPER_EXPORT_HTML, get(handle_export_html))
        .route(ApiUrls::PAPER_EXPORT_TXT, get(handle_export_txt))
}
