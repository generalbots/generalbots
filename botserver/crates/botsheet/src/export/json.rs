//! JSON export.

use botsheet_core::types::Spreadsheet;

pub fn export_to_json(sheet: &Spreadsheet) -> String {
    serde_json::to_string_pretty(sheet).unwrap_or_default()
}
