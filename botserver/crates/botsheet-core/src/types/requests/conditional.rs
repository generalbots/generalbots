//! Conditional formatting and data validation requests.

use serde::{Deserialize, Serialize};

use super::super::core::*;

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
