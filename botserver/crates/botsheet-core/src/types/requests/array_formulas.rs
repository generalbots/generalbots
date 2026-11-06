//! Array formula requests.

use serde::{Deserialize, Serialize};

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
