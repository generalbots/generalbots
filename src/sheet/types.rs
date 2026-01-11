use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
