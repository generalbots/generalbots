//! Builders shared by the exporter unit tests.

use botsheet_core::engine::value::CellValue;
use botsheet_core::types::{CellData, Spreadsheet, Worksheet};
use chrono::{TimeZone, Utc};

/// A cell with only a display value (no typed value).
pub(super) fn cell(value: &str) -> CellData {
    CellData {
        value: Some(value.to_string()),
        typed: None,
        formula: None,
        style: None,
        format: None,
        note: None,
        locked: None,
        has_comment: None,
        array_formula_id: None,
    }
}

/// A cell with a typed value and the display string derived from it.
pub(super) fn typed_cell(value: &str, typed: CellValue) -> CellData {
    CellData {
        typed: Some(typed),
        ..cell(value)
    }
}

/// A worksheet named `name` holding `cells` as `(row, col, value)`.
pub(super) fn ws(name: &str, cells: &[(u32, u32, &str)]) -> Worksheet {
    let mut worksheet = Worksheet::default();
    worksheet.name = name.to_string();
    for (row, col, value) in cells {
        worksheet
            .data
            .insert(format!("{row},{col}"), cell(value));
    }
    worksheet
}

/// A workbook named `name` holding `worksheets`.
pub(super) fn sheet(name: &str, worksheets: Vec<Worksheet>) -> Spreadsheet {
    let stamp = Utc.with_ymd_and_hms(2026, 1, 2, 3, 4, 5).single().expect("fixed timestamp");
    Spreadsheet {
        id: "sheet-1".to_string(),
        name: name.to_string(),
        owner_id: "owner-1".to_string(),
        worksheets,
        created_at: stamp,
        updated_at: stamp,
        named_ranges: None,
        external_links: None,
        source_bucket: None,
        source_path: None,
        source_bytes: None,
        acl: Default::default(),
    }
}
