use serde::{Deserialize, Serialize};

use crate::types_core::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateResponse {
    pub id: String,
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddCommentRequest {
    pub doc_id: String,
    pub content: String,
    pub position: usize,
    pub length: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplyCommentRequest {
    pub doc_id: String,
    pub comment_id: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveCommentRequest {
    pub doc_id: String,
    pub comment_id: String,
    pub resolved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteCommentRequest {
    pub doc_id: String,
    pub comment_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListCommentsResponse {
    pub comments: Vec<DocumentComment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnableTrackChangesRequest {
    pub doc_id: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptRejectChangeRequest {
    pub doc_id: String,
    pub change_id: String,
    pub accept: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptRejectAllRequest {
    pub doc_id: String,
    pub accept: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListTrackChangesResponse {
    pub changes: Vec<TrackChange>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateTocRequest {
    pub doc_id: String,
    pub max_level: u32,
    pub show_page_numbers: bool,
    pub use_hyperlinks: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTocRequest {
    pub doc_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TocResponse {
    pub toc: TableOfContents,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddFootnoteRequest {
    pub doc_id: String,
    pub content: String,
    pub position: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateFootnoteRequest {
    pub doc_id: String,
    pub footnote_id: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteFootnoteRequest {
    pub doc_id: String,
    pub footnote_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListFootnotesResponse {
    pub footnotes: Vec<Footnote>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddEndnoteRequest {
    pub doc_id: String,
    pub content: String,
    pub position: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEndnoteRequest {
    pub doc_id: String,
    pub endnote_id: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteEndnoteRequest {
    pub doc_id: String,
    pub endnote_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListEndnotesResponse {
    pub endnotes: Vec<Endnote>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStyleRequest {
    pub doc_id: String,
    pub style: DocumentStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStyleRequest {
    pub doc_id: String,
    pub style: DocumentStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteStyleRequest {
    pub doc_id: String,
    pub style_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListStylesResponse {
    pub styles: Vec<DocumentStyle>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyStyleRequest {
    pub doc_id: String,
    pub style_id: String,
    pub position: usize,
    pub length: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetOutlineRequest {
    pub doc_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlineResponse {
    pub items: Vec<OutlineItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareDocumentsRequest {
    pub original_doc_id: String,
    pub modified_doc_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareDocumentsResponse {
    pub comparison: DocumentComparison,
}
