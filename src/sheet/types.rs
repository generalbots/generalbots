use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellComment {
    pub id: String,
    pub author_id: String,
    pub author_name: String,
    pub content: String,
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
pub struct SheetProtection {
    pub protected: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_hash: Option<String>,
    #[serde(default)]
    pub locked_cells: Vec<String>,
    #[serde(default)]
    pub allow_select_locked: bool,
    #[serde(default)]
    pub allow_select_unlocked: bool,
    #[serde(default)]
    pub allow_format_cells: bool,
    #[serde(default)]
    pub allow_format_columns: bool,
    #[serde(default)]
    pub allow_format_rows: bool,
    #[serde(default)]
    pub allow_insert_columns: bool,
    #[serde(default)]
    pub allow_insert_rows: bool,
    #[serde(default)]
    pub allow_insert_hyperlinks: bool,
    #[serde(default)]
    pub allow_delete_columns: bool,
    #[serde(default)]
    pub allow_delete_rows: bool,
    #[serde(default)]
    pub allow_sort: bool,
    #[serde(default)]
    pub allow_filter: bool,
    #[serde(default)]
    pub allow_pivot_tables: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalLink {
    pub id: String,
    pub source_path: String,
    pub link_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_sheet: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_range: Option<String>,
    pub status: String,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArrayFormula {
    pub id: String,
    pub formula: String,
    pub start_row: u32,
    pub start_col: u32,
    pub end_row: u32,
    pub end_col: u32,
    #[serde(default)]
    pub is_dynamic: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedRange {
    pub id: String,
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
pub struct CollabMessage {
    pub msg_type: String,
    pub sheet_id: String,
    pub user_id: String,
    pub user_name: String,
    pub user_color: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub col: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worksheet_index: Option<usize>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collaborator {
    pub id: String,
    pub name: String,
    pub color: String,
    pub cursor_row: Option<u32>,
    pub cursor_col: Option<u32>,
    pub connected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spreadsheet {
    pub id: String,
    pub name: String,
    pub owner_id: String,
    pub worksheets: Vec<Worksheet>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub named_ranges: Option<Vec<NamedRange>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_links: Option<Vec<ExternalLink>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Worksheet {
    pub name: String,
    pub data: HashMap<String, CellData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column_widths: Option<HashMap<u32, u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row_heights: Option<HashMap<u32, u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frozen_rows: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frozen_cols: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merged_cells: Option<Vec<MergedCell>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<HashMap<u32, FilterConfig>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidden_rows: Option<Vec<u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validations: Option<HashMap<String, ValidationRule>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conditional_formats: Option<Vec<ConditionalFormatRule>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charts: Option<Vec<ChartConfig>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<HashMap<String, CellComment>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protection: Option<SheetProtection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub array_formulas: Option<Vec<ArrayFormula>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formula: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<CellStyle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locked: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_comment: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub array_formula_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CellStyle {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_weight: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_style: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_decoration: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_align: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vertical_align: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergedCell {
    pub start_row: u32,
    pub start_col: u32,
    pub end_row: u32,
    pub end_col: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterConfig {
    pub filter_type: String,
    pub values: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value2: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
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
    pub error_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalFormatRule {
    pub id: String,
    pub start_row: u32,
    pub start_col: u32,
    pub end_row: u32,
    pub end_col: u32,
    pub rule_type: String,
    pub condition: String,
    pub style: CellStyle,
    pub priority: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartConfig {
    pub id: String,
    pub chart_type: String,
    pub title: String,
    pub data_range: String,
    pub label_range: String,
    pub position: ChartPosition,
    pub options: ChartOptions,
    pub datasets: Vec<ChartDataset>,
    pub labels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartPosition {
    pub row: u32,
    pub col: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChartOptions {
    pub show_legend: bool,
    pub show_grid: bool,
    pub stacked: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legend_position: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x_axis_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y_axis_title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartDataset {
    pub label: String,
    pub data: Vec<f64>,
    pub color: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadsheetMetadata {
    pub id: String,
    pub name: String,
    pub owner_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub worksheet_count: usize,
}

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
