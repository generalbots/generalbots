pub mod collaboration;
pub mod handlers;
pub mod ooxml;
pub mod storage;
pub mod types;
pub mod utils;

use crate::shared::state::AppState;
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

pub use collaboration::{
    handle_docs_websocket, handle_get_collaborators, handle_get_mentions, handle_get_presence,
    handle_get_selections, handle_get_typing,
};
pub use handlers::{
    handle_accept_reject_all, handle_accept_reject_change, handle_add_comment, handle_add_endnote,
    handle_add_footnote, handle_ai_custom, handle_ai_expand, handle_ai_improve, handle_ai_simplify,
    handle_ai_summarize, handle_ai_translate, handle_apply_style, handle_autosave,
    handle_compare_documents, handle_create_style, handle_delete_comment, handle_delete_document,
    handle_delete_endnote, handle_delete_footnote, handle_delete_style, handle_docs_ai,
    handle_docs_get_by_id, handle_docs_save, handle_enable_track_changes, handle_export_docx,
    handle_export_html, handle_export_md, handle_export_pdf, handle_export_txt,
    handle_generate_toc, handle_get_document, handle_get_outline, handle_import_document,
    handle_list_comments, handle_list_documents, handle_list_endnotes, handle_list_footnotes,
    handle_list_styles, handle_list_track_changes, handle_new_document, handle_reply_comment,
    handle_resolve_comment, handle_save_document, handle_search_documents, handle_template_blank,
    handle_template_letter, handle_template_meeting, handle_template_report, handle_update_endnote,
    handle_update_footnote, handle_update_style, handle_update_toc,
};
pub use types::{
    AiRequest, AiResponse, Collaborator, CollabMessage, CommentReply, ComparisonSummary, Document,
    DocumentComment, DocumentComparison, DocumentDiff, DocumentMetadata, DocumentStyle, Endnote,
    Footnote, OutlineItem, SaveRequest, SaveResponse, SearchQuery, TableOfContents, TocEntry,
    TrackChange,
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
        .route("/api/docs/import", post(handle_import_document))
        .route("/api/docs/comment", post(handle_add_comment))
        .route("/api/docs/comment/reply", post(handle_reply_comment))
        .route("/api/docs/comment/resolve", post(handle_resolve_comment))
        .route("/api/docs/comment/delete", post(handle_delete_comment))
        .route("/api/docs/comments", get(handle_list_comments))
        .route("/api/docs/track-changes/enable", post(handle_enable_track_changes))
        .route("/api/docs/track-changes/accept-reject", post(handle_accept_reject_change))
        .route("/api/docs/track-changes/accept-reject-all", post(handle_accept_reject_all))
        .route("/api/docs/track-changes", get(handle_list_track_changes))
        .route("/api/docs/toc/generate", post(handle_generate_toc))
        .route("/api/docs/toc/update", post(handle_update_toc))
        .route("/api/docs/footnote", post(handle_add_footnote))
        .route("/api/docs/footnote/update", post(handle_update_footnote))
        .route("/api/docs/footnote/delete", post(handle_delete_footnote))
        .route("/api/docs/footnotes", get(handle_list_footnotes))
        .route("/api/docs/endnote", post(handle_add_endnote))
        .route("/api/docs/endnote/update", post(handle_update_endnote))
        .route("/api/docs/endnote/delete", post(handle_delete_endnote))
        .route("/api/docs/endnotes", get(handle_list_endnotes))
        .route("/api/docs/style", post(handle_create_style))
        .route("/api/docs/style/update", post(handle_update_style))
        .route("/api/docs/style/delete", post(handle_delete_style))
        .route("/api/docs/style/apply", post(handle_apply_style))
        .route("/api/docs/styles", get(handle_list_styles))
        .route("/api/docs/outline", post(handle_get_outline))
        .route("/api/docs/compare", post(handle_compare_documents))
        .route("/api/docs/:doc_id/collaborators", get(handle_get_collaborators))
        .route("/api/docs/:doc_id/presence", get(handle_get_presence))
        .route("/api/docs/:doc_id/typing", get(handle_get_typing))
        .route("/api/docs/:doc_id/selections", get(handle_get_selections))
        .route("/api/docs/mentions/:user_id", get(handle_get_mentions))
        .route("/ws/docs/:doc_id", get(handle_docs_websocket))
}
