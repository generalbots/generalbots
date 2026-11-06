//! Sheet protection and cell locking requests.

use serde::{Deserialize, Serialize};

use super::super::core::*;

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
