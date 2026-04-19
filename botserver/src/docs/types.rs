use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackChange {
    pub id: String,
    pub change_type: String,
    pub author_id: String,
    pub author_name: String,
    pub timestamp: DateTime<Utc>,
    pub original_text: Option<String>,
    pub new_text: Option<String>,
    pub position: usize,
    pub length: usize,
    pub accepted: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentComment {
    pub id: String,
    pub author_id: String,
    pub author_name: String,
    pub content: String,
    pub position: usize,
    pub length: usize,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub replies: Vec<CommentReply>,
    #[serde(default)]
    pub resolved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentReply {
    pub id: String,
    pub author_id: String,
    pub author_name: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableOfContents {
    pub id: String,
    pub title: String,
    pub entries: Vec<TocEntry>,
    pub max_level: u32,
    pub show_page_numbers: bool,
    pub use_hyperlinks: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TocEntry {
    pub id: String,
    pub text: String,
    pub level: u32,
    pub page_number: Option<u32>,
    pub position: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Footnote {
    pub id: String,
    pub reference_mark: String,
    pub content: String,
    pub position: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endnote {
    pub id: String,
    pub reference_mark: String,
    pub content: String,
    pub position: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentStyle {
    pub id: String,
    pub name: String,
    pub style_type: String,
    pub based_on: Option<String>,
    pub next_style: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_weight: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_style: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_spacing: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub space_before: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub space_after: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_align: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indent_left: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indent_right: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indent_first_line: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlineItem {
    pub id: String,
    pub text: String,
    pub level: u32,
    pub position: usize,
    pub length: usize,
    pub style_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentComparison {
    pub id: String,
    pub original_doc_id: String,
    pub modified_doc_id: String,
    pub created_at: DateTime<Utc>,
    pub differences: Vec<DocumentDiff>,
    pub summary: ComparisonSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentDiff {
    pub diff_type: String,
    pub position: usize,
    pub original_text: Option<String>,
    pub modified_text: Option<String>,
    pub length: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonSummary {
    pub insertions: u32,
    pub deletions: u32,
    pub modifications: u32,
    pub total_changes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollabMessage {
    pub msg_type: String,
    pub doc_id: String,
    pub user_id: String,
    pub user_name: String,
    pub user_color: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collaborator {
    pub id: String,
    pub name: String,
    pub color: String,
    pub cursor_position: Option<usize>,
    pub selection_length: Option<usize>,
    pub connected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub title: String,
    pub content: String,
    pub owner_id: String,
    pub storage_path: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub collaborators: Vec<String>,
    #[serde(default)]
    pub version: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track_changes: Option<Vec<TrackChange>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<Vec<DocumentComment>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub footnotes: Option<Vec<Footnote>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endnotes: Option<Vec<Endnote>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub styles: Option<Vec<DocumentStyle>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub toc: Option<TableOfContents>,
    #[serde(default)]
    pub track_changes_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub id: String,
    pub title: String,
    pub owner_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub word_count: usize,
    pub storage_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveRequest {
    pub id: Option<String>,
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveResponse {
    pub id: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_text: Option<String>,
    pub prompt: String,
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translate_lang: Option<String>,
    pub document_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiResponse {
    pub result: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportQuery {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct DocsAiRequest {
    pub command: String,
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub extra: Option<String>,
    #[serde(default)]
    pub selected_text: Option<String>,
    #[serde(default)]
    pub doc_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DocsAiResponse {
    pub response: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocsSaveRequest {
    pub id: Option<String>,
    pub title: String,
    pub content: String,
    #[serde(default)]
    pub drive_source: Option<DriveSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveSource {
    pub bucket: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocsSaveResponse {
    pub id: String,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadQuery {
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadFromDriveRequest {
    pub bucket: String,
    pub path: String,
}

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
