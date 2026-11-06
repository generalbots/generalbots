use serde::{Deserialize, Serialize};

use super::core::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveRequest {
    pub id: Option<String>,
    /// Document title. Optional so the save endpoint can serve both full-state
    /// saves and rename/flush actions that operate on an existing session.
    pub name: Option<String>,
    /// Full worksheet set. Empty when the client only wants the session
    /// flushed or the document renamed (the session already holds live state).
    #[serde(default)]
    pub worksheets: Vec<Worksheet>,
    /// Explicit action: `rename` renames the document in its session;
    /// anything else is a full save (or a session flush when `worksheets`
    /// is empty and the document already exists).
    pub action: Option<String>,
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
pub struct SearchQuery {
    pub q: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellUpdateRequest {
    pub sheet_id: String,
    pub worksheet_index: usize,
    pub row: u32,
    pub col: u32,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatRequest {
    #[serde(alias = "id")]
    pub sheet_id: String,
    #[serde(default)]
    pub worksheet_index: usize,
    #[serde(default)]
    pub start_row: u32,
    #[serde(default)]
    pub start_col: u32,
    #[serde(default)]
    pub end_row: u32,
    #[serde(default)]
    pub end_col: u32,
    #[serde(default)]
    pub style: CellStyle,
    /// Toolbar shorthand: "A1:A100"; overrides the explicit bounds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<String>,
    /// Custom number format ("0.00", "R$ #,##0.00"); written to CellData.format.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRequest {
    pub id: String,
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareRequest {
    // The sheet modal posts the workbook as `id`; the API calls it sheet_id.
    #[serde(alias = "id")]
    pub sheet_id: String,
    pub email: String,
    pub permission: String,
    /// Explicit grant key (user id). When absent the grant is keyed by email.
    #[serde(default)]
    pub user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveResponse {
    pub id: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormulaResult {
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormulaRequest {
    pub sheet_id: String,
    pub worksheet_index: usize,
    pub formula: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeCellsRequest {
    pub sheet_id: String,
    pub worksheet_index: usize,
    pub start_row: u32,
    pub start_col: u32,
    pub end_row: u32,
    pub end_col: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreezePanesRequest {
    pub sheet_id: String,
    pub worksheet_index: usize,
    pub frozen_rows: u32,
    pub frozen_cols: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortRequest {
    pub sheet_id: String,
    pub worksheet_index: usize,
    pub start_row: u32,
    pub start_col: u32,
    pub end_row: u32,
    pub end_col: u32,
    pub sort_col: u32,
    pub ascending: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterRequest {
    pub sheet_id: String,
    pub worksheet_index: usize,
    pub col: u32,
    pub filter_type: String,
    #[serde(default)]
    pub values: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value2: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartRequest {
    // The chart modal posts the workbook as `id` and never sends an index.
    #[serde(alias = "id")]
    pub sheet_id: String,
    #[serde(default)]
    pub worksheet_index: usize,
    pub chart_type: String,
    pub data_range: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_range: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<ChartPosition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalFormatRequest {
    #[serde(alias = "id")]
    pub sheet_id: String,
    #[serde(default)]
    pub worksheet_index: usize,
    #[serde(default)]
    pub start_row: u32,
    #[serde(default)]
    pub start_col: u32,
    #[serde(default)]
    pub end_row: u32,
    #[serde(default)]
    pub end_col: u32,
    /// Toolbar form shorthand: "A1:D10". Overrides the explicit bounds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<String>,
    #[serde(default)]
    pub rule_type: String,
    #[serde(default)]
    pub condition: String,
    #[serde(default)]
    pub style: CellStyle,
    /// Toolbar form comparison value; becomes `condition` when it is empty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// Toolbar form fill colour; merged into `style.background`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bg_color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataValidationRequest {
    #[serde(alias = "id")]
    pub sheet_id: String,
    #[serde(default)]
    pub worksheet_index: usize,
    #[serde(default)]
    pub start_row: u32,
    #[serde(default)]
    pub start_col: u32,
    #[serde(default)]
    pub end_row: u32,
    #[serde(default)]
    pub end_col: u32,
    /// Toolbar form shorthand: "A1" or "A1:B4". Overrides the explicit bounds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<String>,
    #[serde(default)]
    pub validation_type: String,
    /// Toolbar form name for `validation_type` (list/number/date/...).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// Toolbar form comma-separated list for the `list` validation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub values: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value2: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_values: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindReplaceRequest {
    // The toolbar posts the workbook as `id`; the API calls it sheet_id.
    #[serde(alias = "id")]
    pub sheet_id: String,
    #[serde(default)]
    pub worksheet_index: usize,
    pub find: String,
    #[serde(default)]
    pub replace: String,
    #[serde(default)]
    pub match_case: bool,
}

/// Resolves the toolbar `range` shorthand against explicit bounds.
pub fn bounds_from_request(
    range: Option<&str>,
    start_row: u32,
    start_col: u32,
    end_row: u32,
    end_col: u32,
) -> (u32, u32, u32, u32) {
    match range.and_then(crate::formulas::refs::parse_range) {
        Some(((start_r, start_c), (end_r, end_c))) => (start_r, start_c, end_r, end_c),
        None => (start_row, start_col, end_row, end_col),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateCellRequest {
    pub sheet_id: String,
    pub worksheet_index: usize,
    pub row: u32,
    pub col: u32,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClearFilterRequest {
    pub sheet_id: String,
    pub worksheet_index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub col: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteChartRequest {
    pub sheet_id: String,
    pub worksheet_index: usize,
    pub chart_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddNoteRequest {
    pub sheet_id: String,
    pub worksheet_index: usize,
    pub row: u32,
    pub col: u32,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddCommentRequest {
    pub sheet_id: String,
    pub worksheet_index: usize,
    pub row: u32,
    pub col: u32,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplyCommentRequest {
    pub sheet_id: String,
    pub worksheet_index: usize,
    pub row: u32,
    pub col: u32,
    pub comment_id: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveCommentRequest {
    pub sheet_id: String,
    pub worksheet_index: usize,
    pub row: u32,
    pub col: u32,
    pub comment_id: String,
    pub resolved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteCommentRequest {
    pub sheet_id: String,
    pub worksheet_index: usize,
    pub row: u32,
    pub col: u32,
    pub comment_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectSheetRequest {
    pub sheet_id: String,
    pub worksheet_index: usize,
    pub protection: SheetProtection,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnprotectSheetRequest {
    pub sheet_id: String,
    pub worksheet_index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockCellsRequest {
    pub sheet_id: String,
    pub worksheet_index: usize,
    pub start_row: u32,
    pub start_col: u32,
    pub end_row: u32,
    pub end_col: u32,
    pub locked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddExternalLinkRequest {
    pub sheet_id: String,
    pub source_path: String,
    pub link_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_sheet: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_range: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshExternalLinkRequest {
    pub sheet_id: String,
    pub link_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveExternalLinkRequest {
    pub sheet_id: String,
    pub link_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArrayFormulaRequest {
    pub sheet_id: String,
    pub worksheet_index: usize,
    pub formula: String,
    pub start_row: u32,
    pub start_col: u32,
    pub end_row: u32,
    pub end_col: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteArrayFormulaRequest {
    pub sheet_id: String,
    pub worksheet_index: usize,
    pub array_formula_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNamedRangeRequest {
    pub sheet_id: String,
    pub name: String,
    pub scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worksheet_index: Option<usize>,
    pub start_row: u32,
    pub start_col: u32,
    pub end_row: u32,
    pub end_col: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateNamedRangeRequest {
    pub sheet_id: String,
    pub range_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_row: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_col: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_row: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_col: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteNamedRangeRequest {
    #[serde(alias = "id")]
    pub sheet_id: String,
    #[serde(default)]
    pub range_id: String,
    /// The ranges panel knows the display name, not the generated id.
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListNamedRangesRequest {
    pub sheet_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListNamedRangesResponse {
    pub ranges: Vec<NamedRange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListExternalLinksResponse {
    pub links: Vec<ExternalLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListCommentsRequest {
    pub sheet_id: String,
    pub worksheet_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListCommentsResponse {
    pub comments: Vec<CommentWithLocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentWithLocation {
    pub row: u32,
    pub col: u32,
    pub comment: CellComment,
}
