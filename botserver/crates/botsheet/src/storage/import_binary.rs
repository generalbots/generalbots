//! Legacy binary workbook import (`.xls` BIFF8 / `.xlsb`) via calamine (E9).
//!
//! calamine reads cell values and formulas but not styles, merges, charts or
//! layout, so those features are absent from the model for these formats. The
//! loops iterate `used_cells()` — O(used cells), not the bounding box.

use botsheet_core::engine::value::CellValue;
use std::collections::HashMap;

/// Reads a `.xls` or `.xlsb` workbook into the worksheet model.
pub fn parse_binary_to_worksheets(
    bytes: &[u8],
    format: &str,
) -> Result<Vec<crate::types::Worksheet>, String> {
    use calamine::{open_workbook_from_rs, Xls, Xlsb};
    use std::io::Cursor;

    let mut worksheets = Vec::new();
    let cursor = Cursor::new(bytes);

    if format == "xls" {
        let mut wb: Xls<_> = open_workbook_from_rs(cursor)
            .map_err(|e| format!("Failed to read .xls: {e}"))?;
        push_sheets(&mut worksheets, &mut wb);
    } else {
        let mut wb: Xlsb<_> = open_workbook_from_rs(cursor)
            .map_err(|e| format!("Failed to read .xlsb: {e}"))?;
        push_sheets(&mut worksheets, &mut wb);
    }

    if worksheets.is_empty() {
        return Err("No worksheets found".to_string());
    }
    Ok(worksheets)
}

fn push_sheets<R, RS>(worksheets: &mut Vec<crate::types::Worksheet>, wb: &mut R)
where
    RS: std::io::Read + std::io::Seek,
    R: calamine::Reader<RS>,
{
    for name in wb.sheet_names() {
        let mut data: HashMap<String, crate::types::CellData> = HashMap::new();
        if let Ok(range) = wb.worksheet_range(&name) {
            for (row, col, cell) in range.used_cells() {
                let (value, typed) = calamine_cell(cell);
                data.insert(
                    format!("{row},{col}"),
                    crate::types::CellData {
                        value,
                        typed,
                        formula: None,
                        style: None,
                        format: None,
                        note: None,
                        locked: None,
                        has_comment: None,
                        array_formula_id: None,
                    },
                );
            }
        }
        // Formulas (E9): calamine returns formula strings without the leading
        // `=`; merge them onto the values above.
        if let Ok(formulas) = wb.worksheet_formula(&name) {
            for (row, col, formula) in formulas.used_cells() {
                if formula.is_empty() {
                    continue;
                }
                let entry = data.entry(format!("{row},{col}")).or_insert_with(empty_cell);
                entry.formula = Some(format!("={formula}"));
            }
        }
        worksheets.push(crate::types::Worksheet {
            name,
            data,
            column_widths: None,
            row_heights: None,
            frozen_rows: None,
            frozen_cols: None,
            merged_cells: None,
            filters: None,
            hidden_rows: None,
            validations: None,
            conditional_formats: None,
            charts: None,
            comments: None,
            protection: None,
            array_formulas: None,
            tables: None,
            hidden_columns: None,
            sheet_state: None,
            hyperlinks: None,
            print_setup: None,
            autofilter: None,
            row_page_breaks: None,
            column_page_breaks: None,
            images: None,
            print_areas: None,
            rich_text: None,
        });
    }
}

/// A blank cell for formula-only entries (no cached value).
fn empty_cell() -> crate::types::CellData {
    crate::types::CellData {
        value: None,
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

/// Maps a calamine `Data` cell to the model's display string + typed value.
fn calamine_cell(cell: &calamine::Data) -> (Option<String>, Option<CellValue>) {
    use calamine::Data;
    match cell {
        Data::Int(i) => (Some(i.to_string()), Some(CellValue::Number(*i as f64))),
        Data::Float(f) => (Some(f.to_string()), Some(CellValue::Number(*f))),
        Data::String(s) => (Some(s.clone()), Some(CellValue::Text(s.clone()))),
        Data::Bool(b) => (
            Some(if *b { "TRUE".to_string() } else { "FALSE".to_string() }),
            Some(CellValue::Bool(*b)),
        ),
        Data::DateTime(dt) => (
            Some(dt.to_string()),
            Some(CellValue::Number(dt.as_f64())),
        ),
        Data::DateTimeIso(s) | Data::DurationIso(s) => {
            (Some(s.clone()), Some(CellValue::Text(s.clone())))
        }
        Data::Error(e) => (Some(e.to_string()), Some(CellValue::Error(e.to_string()))),
        Data::Empty => (None, None),
    }
}
