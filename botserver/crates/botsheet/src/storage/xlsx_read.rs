use crate::types::{CellData, MergedCell, NamedRange, SheetProtection, Spreadsheet, Worksheet};
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

    let raw_name = file_path.split('/').next_back().unwrap_or("Untitled");
    let file_name = raw_name
        .strip_suffix(".xlsx")
        .or_else(|| raw_name.strip_suffix(".xlsm"))
        .or_else(|| raw_name.strip_suffix(".xls"))
        .unwrap_or(raw_name);

    let mut worksheets = Vec::new();

    for sheet in workbook.get_sheet_collection() {
        let mut data: HashMap<String, CellData> = HashMap::new();
        let mut column_widths: HashMap<u32, u32> = HashMap::new();
        let mut row_heights: HashMap<u32, u32> = HashMap::new();

        let (max_col, max_row) = sheet.get_highest_column_and_row();

        for row in 1..=max_row {
            for col in 1..=max_col {
                if let Some(cell) = sheet.get_cell((col, row)) {
                    let value = cell.get_value().to_string();
                    let formula = if cell.get_formula().is_empty() {
                        None
                    } else {
                        Some(format!("={}", cell.get_formula()))
                    };

                    if value.is_empty() && formula.is_none() {
                        continue;
                    }

                    let key = format!("{},{}", row - 1, col - 1);
                    let style = super::xlsx_write::extract_cell_style(cell);

                    let note = sheet
                        .get_comments()
                        .iter()
                        .find(|c| {
                            let coord = c.get_coordinate();
                            coord.get_col_num() == &col && coord.get_row_num() == &row
                        })
                        .and_then(|c| {
                            c.get_text()
                                .get_rich_text()
                                .map(|rt| rt.get_text().to_string())
                        });

                    let cell_value = value.clone();
                    let has_comment = note.is_some();
                    data.insert(
                        key,
                        CellData {
                            value: Some(cell_value),
                                typed: None,
                            formula,
                            style,
                            format: None,
                            note,
                            locked: None,
                            has_comment: has_comment.then_some(true),
                            array_formula_id: None,
                        },
                    );
                }
            }
        }

        for col in 1..=max_col {
            let col_letter = super::xlsx_write::get_col_letter(col);
            if let Some(dim) = sheet.get_column_dimension(&col_letter) {
                let width = *dim.get_width();
                if width > 0.0 {
                    column_widths.insert(col, width.round() as u32);
                }
            }
        }

        for row in 1..=max_row {
            if let Some(dim) = sheet.get_row_dimension(&row) {
                let height = *dim.get_height();
                if height > 0.0 {
                    row_heights.insert(row, height.round() as u32);
                }
            }
        }

        let merged_cells: Vec<MergedCell> = sheet
            .get_merge_cells()
            .iter()
            .filter_map(|mc| {
                let range = mc.get_range().to_string();
                parse_merge_range(&range)
            })
            .collect();

        let frozen_rows = sheet
            .get_sheets_views()
            .get_sheet_view_list()
            .first()
            .and_then(|v| v.get_pane())
            .map(|p| *p.get_vertical_split() as u32)
            .filter(|&v| v > 0);

        let frozen_cols = sheet
            .get_sheets_views()
            .get_sheet_view_list()
            .first()
            .and_then(|v| v.get_pane())
            .map(|p| *p.get_horizontal_split() as u32)
            .filter(|&v| v > 0);

        let sheet_name = sheet.get_name().to_string();

        // Sheet protection (#788): a protected sheet arrives with its flags,
        // so the structured model can enforce and re-emit them.
        let protection = sheet
            .get_sheet_protection()
            .filter(|p| *p.get_sheet())
            .map(|p| SheetProtection {
                protected: true,
                password_hash: {
                    #[cfg(feature = "xlsx")]
                    {
                        let raw = p.get_password_raw();
                        if raw.is_empty() {
                            None
                        } else {
                            Some(raw.to_string())
                        }
                    }
                    #[cfg(not(feature = "xlsx"))]
                    {
                        let _ = p;
                        None
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
            });

        worksheets.push(Worksheet {
            name: sheet_name,
            data,
            column_widths: if column_widths.is_empty() {
                None
            } else {
                Some(column_widths)
            },
            row_heights: if row_heights.is_empty() {
                None
            } else {
                Some(row_heights)
            },
            frozen_rows,
            frozen_cols,
            merged_cells: if merged_cells.is_empty() {
                None
            } else {
                Some(merged_cells)
            },
            filters: None,
            hidden_rows: None,
            validations: None,
            conditional_formats: None,
            charts: None,
            comments: None,
            protection,
            array_formulas: None,
            tables: None,
        });
    }

    // Defined names (#788): workbook-scope names (e.g. `TaxRate = Sheet1!$B$2`)
    // land in the structured model so the round-trip keeps them.
    let mut named_ranges: Vec<NamedRange> = Vec::new();
    let defined_names = workbook.get_defined_names();
    for name in defined_names {
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

    #[cfg(feature = "xlsx")]
    if let Ok(format_map) = super::format_codes::extract_cell_format_codes(bytes) {
        super::format_codes::apply_format_codes(&mut worksheets, &format_map);
    }

    let spreadsheet = Spreadsheet {
        named_ranges: if named_ranges.is_empty() {
            None
        } else {
            Some(named_ranges)
        },
        external_links: None,
        id: Uuid::new_v4().to_string(),
        name: file_name.to_string(),
        owner_id: user_id.to_string(),
        worksheets,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    source_bucket: None,
    source_path: None,
        source_bytes: None,
    acl: HashMap::new(),
    };

    Ok((spreadsheet, workbook))
}

fn parse_merge_range(range: &str) -> Option<MergedCell> {
    let parts: Vec<&str> = range.split(':').collect();
    if parts.len() != 2 {
        return None;
    }

    let start = parse_cell_ref(parts[0])?;
    let end = parse_cell_ref(parts[1])?;

    Some(MergedCell {
        start_row: start.0,
        start_col: start.1,
        end_row: end.0,
        end_col: end.1,
    })
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
