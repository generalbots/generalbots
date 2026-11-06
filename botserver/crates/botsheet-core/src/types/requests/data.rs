//! Sort and filter requests plus the toolbar range resolver.

use serde::{Deserialize, Serialize};

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
pub struct ClearFilterRequest {
    pub sheet_id: String,
    pub worksheet_index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub col: Option<u32>,
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
