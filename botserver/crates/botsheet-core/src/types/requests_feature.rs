//! Request/response DTOs for AI, pivot and layout features (split from
//! [`super::requests`] to respect the file-size ceiling).

use serde::{Deserialize, Serialize};

use super::core::*;

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

#[derive(Debug, Clone, Deserialize)]
pub struct PivotFieldAgg {
    pub field: String,
    #[serde(default = "default_agg")]
    pub agg: String,
}

fn default_agg() -> String {
    "SUM".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct PivotRequest {
    pub sheet_id: String,
    #[serde(default)]
    pub source_range: Option<String>,
    #[serde(default)]
    pub rows: Vec<String>,
    #[serde(default)]
    pub cols: Vec<String>,
    #[serde(default)]
    pub values: Vec<PivotFieldAgg>,
    #[serde(default)]
    pub filter: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PivotResult {
    pub result: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct NamedRangesExportResponse {
    pub csv: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct NamedRangesImportResponse {
    pub added: u32,
    pub updated: u32,
    pub errors: Vec<String>,
    pub entries: Vec<super::core::NamedRange>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RangeRequest {
    pub sheet_id: String,
    pub worksheet_index: usize,
    pub start_row: u32,
    pub start_col: u32,
    pub end_row: u32,
    pub end_col: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct RangeResponse {
    pub cells: std::collections::HashMap<String, super::core::CellData>,
    pub total_rows: u32,
    pub total_cols: u32,
    pub range_start: String,
    pub range_end: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorksheetMetaResponse {
    pub total_rows: u32,
    pub total_cols: u32,
    pub name: String,
    pub frozen_rows: u32,
    pub frozen_cols: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResizeRequest {
    pub sheet_id: String,
    pub worksheet_index: usize,
    #[serde(default)]
    pub row: Option<u32>,
    #[serde(default)]
    pub col: Option<u32>,
    #[serde(default)]
    pub height: Option<u32>,
    #[serde(default)]
    pub width: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasteRequest {
    pub sheet_id: String,
    pub worksheet_index: usize,
    /// Target cell for the top-left of the pasted block.
    pub start_row: u32,
    pub start_col: u32,
    /// Raw HTML table fragment from the clipboard (Excel/Sheets flavor).
    pub html: String,
    /// Paste Special mode: "all" | "values" | "formats".
    #[serde(default)]
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableRequest {
    pub sheet_id: String,
    pub worksheet_index: usize,
    pub start_row: u32,
    pub start_col: u32,
    pub end_row: u32,
    pub end_col: u32,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TableResponse {
    pub id: String,
    pub name: String,
    pub start_row: u32,
    pub start_col: u32,
    pub end_row: u32,
    pub end_col: u32,
}
