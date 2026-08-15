//! Drive-open entry point (#788).
//!
//! Shares the worksheet extraction with the Drive-open handler
//! (`import::parse_xlsx_to_worksheets`) so every path reads the same fidelity
//! surface (cells, layout, tables, autofilter, hyperlinks, validation,
//! conditional formats, images, print setup, page breaks, rich text, comments
//! and charts), then layers the umya-only extras — per-sheet protection and
//! workbook defined names — on top.

use crate::types::{NamedRange, SheetProtection, Spreadsheet};
use chrono::Utc;
use std::collections::HashMap;
use std::io::Cursor;
use uuid::Uuid;

pub fn load_xlsx_from_bytes(
    bytes: &[u8],
    user_id: &str,
    file_path: &str,
) -> Result<(Spreadsheet, umya_spreadsheet::Spreadsheet), String> {
    let cursor = Cursor::new(bytes);
    let workbook = umya_spreadsheet::reader::xlsx::read_reader(cursor, true)
        .map_err(|e| format!("Failed to parse xlsx: {e}"))?;

    // Single source of truth for worksheet extraction (iterates umya's
    // actual-cell map, not the bounding box — see `import.rs`).
    let mut worksheets = super::import::parse_xlsx_to_worksheets(bytes, "xlsx")?;

    // umya-only extras: per-sheet protection, keyed by worksheet index (both
    // paths iterate `get_sheet_collection()` in document order).
    for (index, sheet) in workbook.get_sheet_collection().iter().enumerate() {
        if let Some(ws) = worksheets.get_mut(index) {
            ws.protection = extract_protection(sheet);
        }
    }

    let named_ranges = extract_named_ranges(&workbook);

    let raw_name = file_path.split('/').next_back().unwrap_or("Untitled");
    let file_name = raw_name
        .strip_suffix(".xlsx")
        .or_else(|| raw_name.strip_suffix(".xlsm"))
        .or_else(|| raw_name.strip_suffix(".xls"))
        .unwrap_or(raw_name);

    let spreadsheet = Spreadsheet {
        id: Uuid::new_v4().to_string(),
        name: file_name.to_string(),
        owner_id: user_id.to_string(),
        worksheets,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        named_ranges,
        external_links: super::xlsx_external_links::extract_external_links(bytes),
        source_bucket: None,
        source_path: None,
        source_bytes: None,
        acl: HashMap::new(),
    };

    Ok((spreadsheet, workbook))
}

/// Reads a sheet's protection flags into the structured model (#788).
fn extract_protection(sheet: &umya_spreadsheet::Worksheet) -> Option<SheetProtection> {
    let p = sheet.get_sheet_protection().filter(|p| *p.get_sheet())?;
    Some(SheetProtection {
        protected: true,
        password_hash: {
            let raw = p.get_password_raw();
            if raw.is_empty() {
                None
            } else {
                Some(raw.to_string())
            }
        },
        locked_cells: Vec::new(),
        allow_select_locked: !*p.get_select_locked_cells(),
        allow_select_unlocked: !*p.get_select_unlocked_cells(),
        allow_format_cells: *p.get_format_cells(),
        allow_format_columns: *p.get_format_columns(),
        allow_format_rows: *p.get_format_rows(),
        allow_insert_columns: *p.get_insert_columns(),
        allow_insert_rows: *p.get_insert_rows(),
        allow_insert_hyperlinks: *p.get_insert_hyperlinks(),
        allow_delete_columns: *p.get_delete_columns(),
        allow_delete_rows: *p.get_delete_rows(),
        allow_sort: *p.get_sort(),
        allow_filter: *p.get_auto_filter(),
        allow_pivot_tables: *p.get_pivot_tables(),
    })
}

/// Maps workbook-scope defined names (e.g. `TaxRate = Sheet1!$B$2`) into the
/// structured model so the round-trip keeps them (#788).
fn extract_named_ranges(workbook: &umya_spreadsheet::Spreadsheet) -> Option<Vec<NamedRange>> {
    let mut named_ranges = Vec::new();
    for name in workbook.get_defined_names() {
        let Some((start_row, start_col, end_row, end_col)) = parse_a1_address(&name.get_address())
        else {
            continue;
        };
        named_ranges.push(NamedRange {
            id: Uuid::new_v4().to_string(),
            name: name.get_name().to_string(),
            scope: "workbook".to_string(),
            worksheet_index: None,
            start_row,
            start_col,
            end_row,
            end_col,
            comment: None,
        });
    }
    if named_ranges.is_empty() {
        None
    } else {
        Some(named_ranges)
    }
}

fn parse_cell_ref(cell_ref: &str) -> Option<(u32, u32)> {
    let mut col_str = String::new();
    let mut row_str = String::new();

    for c in cell_ref.chars() {
        if c.is_ascii_alphabetic() {
            col_str.push(c.to_ascii_uppercase());
        } else if c.is_ascii_digit() {
            row_str.push(c);
        }
    }

    let col = col_str
        .chars()
        .fold(0u32, |acc, c| acc * 26 + (c as u32 - 'A' as u32 + 1));

    let row: u32 = row_str.parse().ok()?;

    Some((row.saturating_sub(1), col.saturating_sub(1)))
}

/// Parses an A1-style address (`Sheet1!$B$2`, `B2`, `B2:D5`) into zero-based
/// row/col bounds. Used for defined names (#788); sheet qualifiers and `$`
/// anchors are ignored. Returns `None` for non-rectangular addresses (formulas,
/// 3-D refs, whole-column refs).
fn parse_a1_address(address: &str) -> Option<(u32, u32, u32, u32)> {
    let cell_part = address.split('!').next_back().unwrap_or(address);
    let cell_part = cell_part.replace('$', "");
    // Any structural character outside A1 cell syntax (parens, operators,
    // commas, quotes, spaces, braces) means the "address" is really a formula
    // or a non-rectangular ref — reject it rather than mis-parse it (#788).
    if !cell_part
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == ':' || c == '_' || c == '.')
    {
        return None;
    }
    let parts: Vec<&str> = cell_part.split(':').collect();
    if parts.len() > 2 {
        return None;
    }
    let first = parts[0];
    let last = parts.get(1).copied().unwrap_or(parts[0]);
    if !first.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
        || !last.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
    {
        return None;
    }
    let (r1, c1) = parse_cell_ref(first)?;
    let (r2_abs, c2_abs) = parse_cell_ref(last)?;
    let (r1, r2, c1, c2) = if parts.len() == 1 {
        (r1, r1, c1, c1)
    } else {
        (r1, r2_abs, c1, c2_abs)
    };
    let (start_row, end_row) = (r1.min(r2), r1.max(r2));
    let (start_col, end_col) = (c1.min(c2), c1.max(c2));
    Some((start_row, start_col, end_row, end_col))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_single_cell_a1() {
        assert_eq!(parse_a1_address("B2"), Some((1, 1, 1, 1)));
    }

    #[test]
    fn parses_anchored_sheet_qualified() {
        assert_eq!(parse_a1_address("Sheet1!$B$2"), Some((1, 1, 1, 1)));
    }

    #[test]
    fn parses_rectangular_range() {
        assert_eq!(parse_a1_address("B2:D5"), Some((1, 1, 4, 3)));
    }

    #[test]
    fn rejects_non_rectangular() {
        assert!(parse_a1_address("SUM(B2:D5)").is_none());
    }
}
