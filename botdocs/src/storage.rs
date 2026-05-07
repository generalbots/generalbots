pub use crate::storage_core::{
    cache_document_bytes, count_words, create_new_document, decode_html_entities,
    escape_html, generate_doc_id, get_cached_document_bytes, get_current_user_id,
    get_user_docs_path, html_to_paragraphs, load_docx_from_drive, paragraphs_to_html,
    parse_html_to_paragraphs, remove_from_cache, strip_html,
};
pub use crate::storage_docx::{convert_docx_to_html, convert_html_to_docx};
pub use crate::storage_drive::{
    delete_document_from_drive, list_documents_from_drive, load_document_from_drive,
    save_document, save_document_as_docx, save_document_to_drive,
};
