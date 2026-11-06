//! Cell-level requests: updates, formatting, export, sharing and validation.

use serde::{Deserialize, Serialize};

use super::super::core::*;

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
    /// Print preview only: orientation/scale selected in the dialog. They are
    /// ignored by the data exporters and default to portrait / 100% in PDF.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub orientation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<String>,
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
