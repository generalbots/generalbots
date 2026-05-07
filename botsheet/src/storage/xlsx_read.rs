use crate::types::{CellData, MergedCell, Spreadsheet, Worksheet};
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

    let raw_name = file_path.split('/').last().unwrap_or("Untitled");
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
            protection: None,
            array_formulas: None,
        });
    }

    let spreadsheet = Spreadsheet {
        named_ranges: None,
        external_links: None,
        id: Uuid::new_v4().to_string(),
        name: file_name.to_string(),
        owner_id: user_id.to_string(),
        worksheets,
        created_at: Utc::now(),
        updated_at: Utc::now(),
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
