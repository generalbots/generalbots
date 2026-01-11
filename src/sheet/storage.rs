use crate::shared::state::AppState;
use crate::sheet::types::{CellData, CellStyle, MergedCell, Spreadsheet, SpreadsheetMetadata, Worksheet};
use chrono::Utc;
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;
use umya_spreadsheet::Spreadsheet as UmyaSpreadsheet;
use uuid::Uuid;

pub fn get_user_sheets_path(user_id: &str) -> String {
    format!("users/{}/sheets", user_id)
}

pub fn get_current_user_id() -> String {
    "default-user".to_string()
}

fn extract_id_from_path(path: &str) -> String {
    path.split('/')
        .last()
        .unwrap_or("")
        .trim_end_matches(".json")
        .trim_end_matches(".xlsx")
        .to_string()
}

pub async fn save_sheet_to_drive(
    state: &Arc<AppState>,
    user_id: &str,
    sheet: &Spreadsheet,
) -> Result<(), String> {
    let drive = state
        .drive
        .as_ref()
        .ok_or_else(|| "Drive not available".to_string())?;

    let path = format!("{}/{}.json", get_user_sheets_path(user_id), sheet.id);
    let content =
        serde_json::to_string_pretty(sheet).map_err(|e| format!("Serialization error: {e}"))?;

    drive
        .put_object()
        .bucket("gbo")
        .key(&path)
        .body(content.into_bytes().into())
        .content_type("application/json")
        .send()
        .await
        .map_err(|e| format!("Failed to save sheet: {e}"))?;

    Ok(())
}

pub async fn save_sheet_as_xlsx(
    state: &Arc<AppState>,
    user_id: &str,
    sheet: &Spreadsheet,
) -> Result<Vec<u8>, String> {
    let xlsx_bytes = convert_to_xlsx(sheet)?;

    let drive = state
        .drive
        .as_ref()
        .ok_or_else(|| "Drive not available".to_string())?;

    let path = format!("{}/{}.xlsx", get_user_sheets_path(user_id), sheet.id);

    drive
        .put_object()
        .bucket("gbo")
        .key(&path)
        .body(xlsx_bytes.clone().into())
        .content_type("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
        .send()
        .await
        .map_err(|e| format!("Failed to save xlsx: {e}"))?;

    Ok(xlsx_bytes)
}

pub fn convert_to_xlsx(sheet: &Spreadsheet) -> Result<Vec<u8>, String> {
    let mut workbook = umya_spreadsheet::new_file();

    for (ws_idx, worksheet) in sheet.worksheets.iter().enumerate() {
        let umya_sheet = if ws_idx == 0 {
            workbook.get_sheet_mut(&0).ok_or("Failed to get first sheet")?
        } else {
            workbook.new_sheet(&worksheet.name).map_err(|e| format!("Failed to create sheet: {e}"))?
        };

        umya_sheet.set_name(&worksheet.name);

        for (key, cell_data) in &worksheet.data {
            let parts: Vec<&str> = key.split(',').collect();
            if parts.len() != 2 {
                continue;
            }

            let row: u32 = parts[0].parse().unwrap_or(0) + 1;
            let col: u32 = parts[1].parse().unwrap_or(0) + 1;

            if row == 0 || col == 0 {
                continue;
            }

            let cell = umya_sheet.get_cell_mut((col, row));

            if let Some(ref formula) = cell_data.formula {
                let formula_str = if formula.starts_with('=') {
                    &formula[1..]
                } else {
                    formula.as_str()
                };
                cell.set_formula(formula_str);
            } else if let Some(ref value) = cell_data.value {
                if let Ok(num) = value.parse::<f64>() {
                    cell.set_value_number(num);
                } else if value.eq_ignore_ascii_case("true") || value.eq_ignore_ascii_case("false") {
                    cell.set_value_bool(value.eq_ignore_ascii_case("true"));
                } else {
                    cell.set_value_string(value);
                }
            }

            if let Some(ref style) = cell_data.style {
                apply_umya_style(cell, style);
            }
        }

        if let Some(ref widths) = worksheet.column_widths {
            for (col_idx, width) in widths {
                let col_letter = get_col_letter(*col_idx);
                if let Some(col_dim) = umya_sheet.get_column_dimension_mut(&col_letter) {
                    col_dim.set_width(*width as f64);
                }
            }
        }

        if let Some(ref heights) = worksheet.row_heights {
            for (row_idx, height) in heights {
                if let Some(row_dim) = umya_sheet.get_row_dimension_mut(row_idx) {
                    row_dim.set_height(*height as f64);
                }
            }
        }

        if let Some(ref merged) = worksheet.merged_cells {
            for merge in merged {
                let start_col = get_col_letter(merge.start_col + 1);
                let end_col = get_col_letter(merge.end_col + 1);
                let range = format!("{}{}:{}{}", start_col, merge.start_row + 1, end_col, merge.end_row + 1);
                let _ = umya_sheet.add_merge_cells(&range);
            }
        }

        if let Some(frozen_rows) = worksheet.frozen_rows {
            if frozen_rows > 0 {
                let sheet_views = umya_sheet.get_sheet_views_mut();
                if let Some(view) = sheet_views.get_sheet_view_list_mut().first_mut() {
                    let pane = view.get_pane_mut();
                    pane.set_y_split(frozen_rows as f64);
                    pane.set_state(umya_spreadsheet::structs::PaneStateValues::Frozen);
                }
            }
        }

        if let Some(frozen_cols) = worksheet.frozen_cols {
            if frozen_cols > 0 {
                let sheet_views = umya_sheet.get_sheet_views_mut();
                if let Some(view) = sheet_views.get_sheet_view_list_mut().first_mut() {
                    let pane = view.get_pane_mut();
                    pane.set_x_split(frozen_cols as f64);
                    pane.set_state(umya_spreadsheet::structs::PaneStateValues::Frozen);
                }
            }
        }
    }

    let mut buf = Cursor::new(Vec::new());
    umya_spreadsheet::writer::xlsx::write_writer(&workbook, &mut buf)
        .map_err(|e| format!("Failed to write xlsx: {e}"))?;

    Ok(buf.into_inner())
}

fn apply_umya_style(cell: &mut umya_spreadsheet::Cell, style: &CellStyle) {
    let cell_style = cell.get_style_mut();

    if let Some(ref weight) = style.font_weight {
        if weight == "bold" {
            cell_style.get_font_mut().set_bold(true);
        }
    }

    if let Some(ref font_style) = style.font_style {
        if font_style == "italic" {
            cell_style.get_font_mut().set_italic(true);
        }
    }

    if let Some(ref decoration) = style.text_decoration {
        if decoration.contains("underline") {
            cell_style.get_font_mut().set_underline("single");
        }
        if decoration.contains("line-through") {
            cell_style.get_font_mut().set_strikethrough(true);
        }
    }

    if let Some(size) = style.font_size {
        cell_style.get_font_mut().set_size(size as f64);
    }

    if let Some(ref font) = style.font_family {
        cell_style.get_font_mut().set_name(font);
    }

    if let Some(ref color) = style.color {
        let color_str = color.trim_start_matches('#');
        cell_style.get_font_mut().get_color_mut().set_argb(&format!("FF{color_str}"));
    }

    if let Some(ref bg) = style.background {
        let bg_str = bg.trim_start_matches('#');
        cell_style.get_fill_mut().get_pattern_fill_mut()
            .get_foreground_color_mut().set_argb(&format!("FF{bg_str}"));
        cell_style.get_fill_mut().get_pattern_fill_mut()
            .set_pattern_type(umya_spreadsheet::structs::PatternValues::Solid);
    }

    if let Some(ref align) = style.text_align {
        let h_align = match align.as_str() {
            "left" => umya_spreadsheet::structs::HorizontalAlignmentValues::Left,
            "center" => umya_spreadsheet::structs::HorizontalAlignmentValues::Center,
            "right" => umya_spreadsheet::structs::HorizontalAlignmentValues::Right,
            _ => umya_spreadsheet::structs::HorizontalAlignmentValues::Left,
        };
        cell_style.get_alignment_mut().set_horizontal(h_align);
    }

    if let Some(ref v_align) = style.vertical_align {
        let v = match v_align.as_str() {
            "top" => umya_spreadsheet::structs::VerticalAlignmentValues::Top,
            "middle" => umya_spreadsheet::structs::VerticalAlignmentValues::Center,
            "bottom" => umya_spreadsheet::structs::VerticalAlignmentValues::Bottom,
            _ => umya_spreadsheet::structs::VerticalAlignmentValues::Center,
        };
        cell_style.get_alignment_mut().set_vertical(v);
    }
}

fn get_col_letter(col: u32) -> String {
    let mut result = String::new();
    let mut n = col;
    while n > 0 {
        n -= 1;
        result.insert(0, (b'A' + (n % 26) as u8) as char);
        n /= 26;
    }
    result
}

pub async fn load_xlsx_from_drive(
    state: &Arc<AppState>,
    user_id: &str,
    file_path: &str,
) -> Result<(Spreadsheet, UmyaSpreadsheet), String> {
    let drive = state
        .drive
        .as_ref()
        .ok_or_else(|| "Drive not available".to_string())?;

    let result = drive
        .get_object()
        .bucket("gbo")
        .key(file_path)
        .send()
        .await
        .map_err(|e| format!("Failed to load file: {e}"))?;

    let bytes = result
        .body
        .collect()
        .await
        .map_err(|e| format!("Failed to read file: {e}"))?
        .into_bytes();

    load_xlsx_from_bytes(&bytes, user_id, file_path)
}

pub fn load_xlsx_from_bytes(
    bytes: &[u8],
    user_id: &str,
    file_path: &str,
) -> Result<(Spreadsheet, UmyaSpreadsheet), String> {
    let cursor = Cursor::new(bytes);
    let workbook = umya_spreadsheet::reader::xlsx::read_reader(cursor, true)
        .map_err(|e| format!("Failed to parse xlsx: {e}"))?;

    let file_name = file_path
        .split('/')
        .last()
        .unwrap_or("Untitled")
        .trim_end_matches(".xlsx")
        .trim_end_matches(".xlsm")
        .trim_end_matches(".xls");

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
                    let style = extract_cell_style(cell);

                    let note = sheet.get_comments()
                        .iter()
                        .find(|c| {
                            let coord = c.get_coordinate();
                            coord.get_col_num() == &col && coord.get_row_num() == &row
                        })
                        .map(|c| c.get_text().get_value().to_string());

                    data.insert(
                        key,
                        CellData {
                            value: if value.is_empty() { None } else { Some(value) },
                            formula,
                            style,
                            format: None,
                            note,
                        },
                    );
                }
            }
        }

        for col in 1..=max_col {
            let col_letter = get_col_letter(col);
            if let Some(dim) = sheet.get_column_dimension(&col_letter) {
                if let Some(width) = dim.get_width() {
                    column_widths.insert(col, width.round() as u32);
                }
            }
        }

        for row in 1..=max_row {
            if let Some(dim) = sheet.get_row_dimension(&row) {
                if let Some(height) = dim.get_height() {
                    row_heights.insert(row, height.round() as u32);
                }
            }
        }

        let merged_cells: Vec<MergedCell> = sheet.get_merge_cells()
            .iter()
            .filter_map(|mc| {
                let range = mc.get_range().get_range();
                parse_merge_range(&range)
            })
            .collect();

        let frozen_rows = sheet.get_sheet_views()
            .get_sheet_view_list()
            .first()
            .and_then(|v| v.get_pane().get_y_split())
            .map(|y| y as u32);

        let frozen_cols = sheet.get_sheet_views()
            .get_sheet_view_list()
            .first()
            .and_then(|v| v.get_pane().get_x_split())
            .map(|x| x as u32);

        worksheets.push(Worksheet {
            name: sheet.get_name().to_string(),
            data,
            column_widths: if column_widths.is_empty() { None } else { Some(column_widths) },
            row_heights: if row_heights.is_empty() { None } else { Some(row_heights) },
            frozen_rows,
            frozen_cols,
            merged_cells: if merged_cells.is_empty() { None } else { Some(merged_cells) },
            filters: None,
            hidden_rows: None,
            validations: None,
            conditional_formats: None,
            charts: None,
        });
    }

    let spreadsheet = Spreadsheet {
        id: Uuid::new_v4().to_string(),
        name: file_name.to_string(),
        owner_id: user_id.to_string(),
        worksheets,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    Ok((spreadsheet, workbook))
}

fn extract_cell_style(cell: &umya_spreadsheet::Cell) -> Option<CellStyle> {
    let style = cell.get_style();
    let font = style.get_font();
    let fill = style.get_fill();
    let alignment = style.get_alignment();

    let font_weight = if font.get_bold() { Some("bold".to_string()) } else { None };
    let font_style = if font.get_italic() { Some("italic".to_string()) } else { None };

    let underline = font.get_underline();
    let strikethrough = font.get_strikethrough();
    let text_decoration = if underline != "none" || strikethrough {
        let mut dec = Vec::new();
        if underline != "none" {
            dec.push("underline");
        }
        if strikethrough {
            dec.push("line-through");
        }
        Some(dec.join(" "))
    } else {
        None
    };

    let font_size = Some(font.get_size().round() as u32);
    let font_family = Some(font.get_name().to_string());

    let color = font.get_color().get_argb().map(|c| {
        let s = c.to_string();
        if s.len() >= 8 {
            format!("#{}", &s[2..])
        } else {
            format!("#{s}")
        }
    });

    let background = fill.get_pattern_fill().get_foreground_color().get_argb().map(|c| {
        let s = c.to_string();
        if s.len() >= 8 {
            format!("#{}", &s[2..])
        } else {
            format!("#{s}")
        }
    });

    let text_align = match alignment.get_horizontal().to_string().as_str() {
        "left" => Some("left".to_string()),
        "center" => Some("center".to_string()),
        "right" => Some("right".to_string()),
        _ => None,
    };

    let vertical_align = match alignment.get_vertical().to_string().as_str() {
        "top" => Some("top".to_string()),
        "center" => Some("middle".to_string()),
        "bottom" => Some("bottom".to_string()),
        _ => None,
    };

    if font_weight.is_some() || font_style.is_some() || text_decoration.is_some()
        || color.is_some() || background.is_some() || text_align.is_some() {
        Some(CellStyle {
            font_family,
            font_size,
            font_weight,
            font_style,
            text_decoration,
            color,
            background,
            text_align,
            vertical_align,
            border: None,
        })
    } else {
        None
    }
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

    let col = col_str.chars().fold(0u32, |acc, c| {
        acc * 26 + (c as u32 - 'A' as u32 + 1)
    });

    let row: u32 = row_str.parse().ok()?;

    Some((row.saturating_sub(1), col.saturating_sub(1)))
}

pub async fn update_xlsx_cell(
    workbook: &mut UmyaSpreadsheet,
    sheet_name: &str,
    row: u32,
    col: u32,
    value: Option<&str>,
    formula: Option<&str>,
    style: Option<&CellStyle>,
) -> Result<(), String> {
    let sheet = workbook
        .get_sheet_by_name_mut(sheet_name)
        .ok_or_else(|| format!("Sheet '{sheet_name}' not found"))?;

    let cell = sheet.get_cell_mut((col + 1, row + 1));

    if let Some(f) = formula {
        let formula_str = if f.starts_with('=') { &f[1..] } else { f };
        cell.set_formula(formula_str);
    } else if let Some(v) = value {
        if let Ok(num) = v.parse::<f64>() {
            cell.set_value_number(num);
        } else if v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("false") {
            cell.set_value_bool(v.eq_ignore_ascii_case("true"));
        } else {
            cell.set_value_string(v);
        }
    } else {
        cell.set_value_string("");
    }

    if let Some(s) = style {
        apply_umya_style(cell, s);
    }

    Ok(())
}

pub async fn save_workbook_to_drive(
    state: &Arc<AppState>,
    user_id: &str,
    sheet_id: &str,
    workbook: &UmyaSpreadsheet,
) -> Result<(), String> {
    let drive = state
        .drive
        .as_ref()
        .ok_or_else(|| "Drive not available".to_string())?;

    let path = format!("{}/{}.xlsx", get_user_sheets_path(user_id), sheet_id);

    let mut buf = Cursor::new(Vec::new());
    umya_spreadsheet::writer::xlsx::write_writer(workbook, &mut buf)
        .map_err(|e| format!("Failed to write xlsx: {e}"))?;

    drive
        .put_object()
        .bucket("gbo")
        .key(&path)
        .body(buf.into_inner().into())
        .content_type("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
        .send()
        .await
        .map_err(|e| format!("Failed to save xlsx: {e}"))?;

    Ok(())
}

pub async fn load_sheet_from_drive(
    state: &Arc<AppState>,
    user_id: &str,
    sheet_id: &Option<String>,
) -> Result<Spreadsheet, String> {
    let sheet_id = sheet_id
        .as_ref()
        .ok_or_else(|| "Sheet ID is required".to_string())?;

    let drive = state
        .drive
        .as_ref()
        .ok_or_else(|| "Drive not available".to_string())?;

    let path = format!("{}/{}.json", get_user_sheets_path(user_id), sheet_id);

    let result = drive
        .get_object()
        .bucket("gbo")
        .key(&path)
        .send()
        .await
        .map_err(|e| format!("Failed to load sheet: {e}"))?;

    let bytes = result
        .body
        .collect()
        .await
        .map_err(|e| format!("Failed to read sheet: {e}"))?
        .into_bytes();

    let sheet: Spreadsheet =
        serde_json::from_slice(&bytes).map_err(|e| format!("Failed to parse sheet: {e}"))?;

    Ok(sheet)
}

pub async fn load_sheet_by_id(
    state: &Arc<AppState>,
    user_id: &str,
    sheet_id: &str,
) -> Result<Spreadsheet, String> {
    let drive = state
        .drive
        .as_ref()
        .ok_or_else(|| "Drive not available".to_string())?;

    let path = format!("{}/{}.json", get_user_sheets_path(user_id), sheet_id);

    let result = drive
        .get_object()
        .bucket("gbo")
        .key(&path)
        .send()
        .await
        .map_err(|e| format!("Failed to load sheet: {e}"))?;

    let bytes = result
        .body
        .collect()
        .await
        .map_err(|e| format!("Failed to read sheet: {e}"))?
        .into_bytes();

    let sheet: Spreadsheet =
        serde_json::from_slice(&bytes).map_err(|e| format!("Failed to parse sheet: {e}"))?;

    Ok(sheet)
}

pub async fn list_sheets_from_drive(
    state: &Arc<AppState>,
    user_id: &str,
) -> Result<Vec<SpreadsheetMetadata>, String> {
    let drive = state
        .drive
        .as_ref()
        .ok_or_else(|| "Drive not available".to_string())?;

    let prefix = format!("{}/", get_user_sheets_path(user_id));

    let result = drive
        .list_objects_v2()
        .bucket("gbo")
        .prefix(&prefix)
        .send()
        .await
        .map_err(|e| format!("Failed to list sheets: {e}"))?;

    let mut sheets = Vec::new();

    if let Some(contents) = result.contents {
        for obj in contents {
            if let Some(key) = obj.key {
                if key.ends_with(".json") {
                    let id = extract_id_from_path(&key);
                    if let Ok(sheet) = load_sheet_by_id(state, user_id, &id).await {
                        sheets.push(SpreadsheetMetadata {
                            id: sheet.id,
                            name: sheet.name,
                            owner_id: sheet.owner_id,
                            created_at: sheet.created_at,
                            updated_at: sheet.updated_at,
                            worksheet_count: sheet.worksheets.len(),
                        });
                    }
                }
            }
        }
    }

    sheets.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

    Ok(sheets)
}

pub async fn delete_sheet_from_drive(
    state: &Arc<AppState>,
    user_id: &str,
    sheet_id: &Option<String>,
) -> Result<(), String> {
    let sheet_id = sheet_id
        .as_ref()
        .ok_or_else(|| "Sheet ID is required".to_string())?;

    let drive = state
        .drive
        .as_ref()
        .ok_or_else(|| "Drive not available".to_string())?;

    let json_path = format!("{}/{}.json", get_user_sheets_path(user_id), sheet_id);
    let xlsx_path = format!("{}/{}.xlsx", get_user_sheets_path(user_id), sheet_id);

    let _ = drive
        .delete_object()
        .bucket("gbo")
        .key(&json_path)
        .send()
        .await;

    let _ = drive
        .delete_object()
        .bucket("gbo")
        .key(&xlsx_path)
        .send()
        .await;

    Ok(())
}

pub fn parse_csv_to_worksheets(
    bytes: &[u8],
    delimiter: u8,
    sheet_name: &str,
) -> Result<Vec<Worksheet>, String> {
    let content = String::from_utf8_lossy(bytes);
    let mut data: HashMap<String, CellData> = HashMap::new();

    for (row_idx, line) in content.lines().enumerate() {
        let cols: Vec<&str> = if delimiter == b'\t' {
            line.split('\t').collect()
        } else {
            line.split(',').collect()
        };

        for (col_idx, value) in cols.iter().enumerate() {
            let clean_value = value.trim().trim_matches('"').to_string();
            if !clean_value.is_empty() {
                let key = format!("{row_idx},{col_idx}");
                data.insert(
                    key,
                    CellData {
                        value: Some(clean_value),
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
    }

    Ok(vec![Worksheet {
        name: sheet_name.to_string(),
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
    }])
}

pub fn parse_excel_to_worksheets(bytes: &[u8], ext: &str) -> Result<Vec<Worksheet>, String> {
    if ext == "xlsx" || ext == "xlsm" || ext == "xls" {
        let cursor = Cursor::new(bytes);
        if let Ok(workbook) = umya_spreadsheet::reader::xlsx::read_reader(cursor, true) {
            let mut worksheets = Vec::new();

            for sheet in workbook.get_sheet_collection() {
                let mut data: HashMap<String, CellData> = HashMap::new();
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
                            let style = extract_cell_style(cell);

                            data.insert(
                                key,
                                CellData {
                                    value: if value.is_empty() { None } else { Some(value) },
                                    formula,
                                    style,
                                    format: None,
                                    note: None,
                                    locked: None,
                                    has_comment: None,
                                    array_formula_id: None,
                                },
                            );
                        }
                    }
                }

                worksheets.push(Worksheet {
                    name: sheet.get_name().to_string(),
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
                });
            }

            if !worksheets.is_empty() {
                return Ok(worksheets);
            }
        }
    }

    Err("Failed to parse spreadsheet".to_string())
}

pub fn parse_ods_to_worksheets(bytes: &[u8]) -> Result<Vec<Worksheet>, String> {
    let content = String::from_utf8_lossy(bytes);
    let mut worksheets = Vec::new();
    let mut current_sheet_name = "Sheet1".to_string();
    let mut data: HashMap<String, CellData> = HashMap::new();
    let mut row_idx = 0u32;

    let mut in_table = false;
    let mut in_row = false;
    let mut col_idx = 0u32;

    let chars: Vec<char> = content.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '<' {
            let mut tag = String::new();
            i += 1;
            while i < chars.len() && chars[i] != '>' {
                tag.push(chars[i]);
                i += 1;
            }

            if tag.starts_with("table:table ") {
                if let Some(name_start) = tag.find("table:name=\"") {
                    let name_part = &tag[name_start + 12..];
                    if let Some(name_end) = name_part.find('"') {
                        current_sheet_name = name_part[..name_end].to_string();
                    }
                }
                in_table = true;
                data.clear();
                row_idx = 0;
            } else if tag == "/table:table" {
                if in_table && !data.is_empty() {
                    worksheets.push(Worksheet {
                        name: current_sheet_name.clone(),
                        data: data.clone(),
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
                    });
                }
                in_table = false;
            } else if tag.starts_with("table:table-row") && !tag.ends_with('/') {
                in_row = true;
                col_idx = 0;
            } else if tag == "/table:table-row" {
                in_row = false;
                row_idx += 1;
            } else if tag.starts_with("table:table-cell") {
                let mut cell_value = String::new();
                let mut has_formula = false;
                let mut formula = String::new();

                if tag.contains("table:formula=") {
                    has_formula = true;
                    if let Some(f_start) = tag.find("table:formula=\"") {
                        let f_part = &tag[f_start + 15..];
                        if let Some(f_end) = f_part.find('"') {
                            formula = f_part[..f_end].to_string();
                        }
                    }
                }

                if tag.contains("office:value=") {
                    if let Some(v_start) = tag.find("office:value=\"") {
                        let v_part = &tag[v_start + 14..];
                        if let Some(v_end) = v_part.find('"') {
                            cell_value = v_part[..v_end].to_string();
                        }
                    }
                }

                i += 1;
                let mut text_depth = 0;
                while i < chars.len() {
                    if chars[i] == '<' {
                        let mut inner_tag = String::new();
                        i += 1;
                        while i < chars.len() && chars[i] != '>' {
                            inner_tag.push(chars[i]);
                            i += 1;
                        }
                        if inner_tag.starts_with("text:p") {
                            text_depth += 1;
                        } else if inner_tag == "/text:p" {
                            text_depth -= 1;
                        } else if inner_tag == "/table:table-cell" {
                            break;
                        }
                    } else if text_depth > 0 {
                        cell_value.push(chars[i]);
                    }
                    i += 1;
                }

                if !cell_value.is_empty() || has_formula {
                    let key = format!("{row_idx},{col_idx}");
                    data.insert(key, CellData {
                        value: if cell_value.is_empty() { None } else { Some(cell_value) },
                        formula: if has_formula { Some(formula) } else { None },
                        style: None,
                        format: None,
                        note: None,
                        locked: None,
                        has_comment: None,
                        array_formula_id: None,
                    });
                }

                col_idx += 1;
            }
        }
        i += 1;
    }

    if worksheets.is_empty() {
        worksheets.push(Worksheet {
            name: "Sheet1".to_string(),
            data: HashMap::new(),
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
        });
    }

    Ok(worksheets)
}

pub fn detect_spreadsheet_format(bytes: &[u8]) -> &'static str {
    if bytes.len() >= 4 {
        if &bytes[0..4] == b"PK\x03\x04" {
            let content_str = String::from_utf8_lossy(&bytes[0..500.min(bytes.len())]);
            if content_str.contains("xl/") || content_str.contains("[Content_Types].xml") {
                return "xlsx";
            }
            if content_str.contains("content.xml") || content_str.contains("mimetype") {
                return "ods";
            }
            return "zip";
        }
        if bytes[0] == 0xD0 && bytes[1] == 0xCF {
            return "xls";
        }
    }

    let text = String::from_utf8_lossy(&bytes[0..100.min(bytes.len())]);
    if text.contains('\t') && text.lines().count() > 1 {
        return "tsv";
    }
    if text.contains(',') && text.lines().count() > 1 {
        return "csv";
    }

    "unknown"
}

pub fn import_spreadsheet_bytes(bytes: &[u8], filename: &str) -> Result<Spreadsheet, String> {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    let detected = detect_spreadsheet_format(bytes);

    let worksheets = match detected {
        "xlsx" | "xlsm" => parse_excel_to_worksheets(bytes, "xlsx")?,
        "xls" => parse_excel_to_worksheets(bytes, "xls")?,
        "ods" => parse_ods_to_worksheets(bytes)?,
        "csv" => parse_csv_to_worksheets(bytes, b',', "Sheet1")?,
        "tsv" => parse_csv_to_worksheets(bytes, b'\t', "Sheet1")?,
        _ => {
            if ext == "csv" {
                parse_csv_to_worksheets(bytes, b',', "Sheet1")?
            } else if ext == "tsv" || ext == "txt" {
                parse_csv_to_worksheets(bytes, b'\t', "Sheet1")?
            } else if ext == "ods" {
                parse_ods_to_worksheets(bytes)?
            } else {
                return Err(format!("Unsupported format: {detected}"));
            }
        }
    };

    let name = filename.rsplit('/').next().unwrap_or(filename)
        .trim_end_matches(&format!(".{ext}"))
        .to_string();

    Ok(Spreadsheet {
        id: Uuid::new_v4().to_string(),
        name,
        owner_id: get_current_user_id(),
        worksheets,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        named_ranges: None,
        external_links: None,
    })
}

pub fn create_new_spreadsheet() -> Spreadsheet {
    Spreadsheet {
        id: Uuid::new_v4().to_string(),
        name: "Untitled Spreadsheet".to_string(),
        owner_id: get_current_user_id(),
        worksheets: vec![Worksheet {
            name: "Sheet1".to_string(),
            data: HashMap::new(),
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
        }],
        created_at: Utc::now(),
        updated_at: Utc::now(),
        named_ranges: None,
        external_links: None,
    }
}
