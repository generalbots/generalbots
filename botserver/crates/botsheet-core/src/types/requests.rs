use serde::{Deserialize, Serialize};

use super::core::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveRequest {
    pub id: Option<String>,
    pub name: String,
    pub worksheets: Vec<Worksheet>,
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
    pub sheet_id: String,
    pub worksheet_index: usize,
    pub start_row: u32,
    pub start_col: u32,
    pub end_row: u32,
    pub end_col: u32,
    pub style: CellStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRequest {
    pub id: String,
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareRequest {
    pub sheet_id: String,
    pub email: String,
    pub permission: String,
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
    pub sheet_id: String,
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
    pub sheet_id: String,
    pub worksheet_index: usize,
    pub start_row: u32,
    pub start_col: u32,
    pub end_row: u32,
    pub end_col: u32,
    pub rule_type: String,
    pub condition: String,
    pub style: CellStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataValidationRequest {
    pub sheet_id: String,
    pub worksheet_index: usize,
    pub start_row: u32,
    pub start_col: u32,
    pub end_row: u32,
    pub end_col: u32,
    pub validation_type: String,
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
    pub sheet_id: String,
    pub range_id: String,
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

#[derive(Debug, Deserialize)]
pub struct SheetAiRequest {
    pub command: String,
    #[serde(default)]
    pub selection: Option<serde_json::Value>,
    #[serde(default)]
    pub active_cell: Option<serde_json::Value>,
    #[serde(default)]
    pub sheet_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SheetAiResponse {
    pub response: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}
