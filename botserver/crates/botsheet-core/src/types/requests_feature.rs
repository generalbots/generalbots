//! Request/response DTOs for AI, pivot and layout features (split from
//! [`super::requests`] to respect the file-size ceiling).

use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct PivotFieldAgg {
    pub field: String,
    #[serde(default = "default_agg")]
    pub agg: String,
}

fn default_agg() -> String {
    "SUM".to_string()
}

fn default_sheet_id() -> String {
    String::new()
}

fn de_string_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v = serde_json::Value::deserialize(deserializer)?;
    Ok(value_to_string_vec(&v))
}

fn de_value_specs<'de, D>(deserializer: D) -> Result<Vec<PivotFieldAgg>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v = serde_json::Value::deserialize(deserializer)?;
    Ok(value_to_agg_specs(&v))
}

fn value_to_string_vec(v: &serde_json::Value) -> Vec<String> {
    match v {
        serde_json::Value::String(s) => s
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(ToString::to_string)
            .collect(),
        serde_json::Value::Array(arr) => arr
            .iter()
            .flat_map(|item| value_to_string_vec(item))
            .collect(),
        serde_json::Value::Object(map) => map
            .get("field")
            .or_else(|| map.get("name"))
            .and_then(|x| x.as_str())
            .map(|s| vec![s.trim().to_string()])
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

fn value_to_agg_specs(v: &serde_json::Value) -> Vec<PivotFieldAgg> {
    match v {
        serde_json::Value::Array(arr) => arr.iter().flat_map(value_to_agg_specs).collect(),
        serde_json::Value::String(s) => s
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|field| PivotFieldAgg {
                field: field.to_string(),
                agg: default_agg(),
            })
            .collect(),
        serde_json::Value::Object(map) => {
            let field = map
                .get("field")
                .or_else(|| map.get("name"))
                .or_else(|| map.get("column"))
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            if field.is_empty() {
                return Vec::new();
            }
            let agg = map
                .get("agg")
                .or_else(|| map.get("aggregation"))
                .or_else(|| map.get("function"))
                .and_then(|x| x.as_str())
                .unwrap_or("SUM")
                .trim()
                .to_ascii_uppercase();
            vec![PivotFieldAgg {
                field,
                agg: if agg.is_empty() { default_agg() } else { agg },
            }]
        }
        _ => Vec::new(),
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PivotRequest {
    #[serde(default = "default_sheet_id", alias = "file_id", alias = "id")]
    pub sheet_id: String,
    #[serde(default, alias = "range", alias = "data_range")]
    pub source_range: Option<String>,
    #[serde(default, deserialize_with = "de_string_vec")]
    pub rows: Vec<String>,
    #[serde(default, deserialize_with = "de_string_vec")]
    pub cols: Vec<String>,
    #[serde(default, deserialize_with = "de_value_specs")]
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
