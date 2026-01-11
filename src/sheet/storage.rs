use crate::shared::state::AppState;
use crate::sheet::types::{CellData, Spreadsheet, SpreadsheetMetadata, Worksheet};
use calamine::{Data, Reader, Xlsx};
use chrono::Utc;
use rust_xlsxwriter::{Workbook, Format};
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;
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
    let mut workbook = Workbook::new();

    for worksheet in &sheet.worksheets {
        let ws = workbook.add_worksheet();
        ws.set_name(&worksheet.name).map_err(|e| format!("Failed to set sheet name: {e}"))?;

        for (key, cell_data) in &worksheet.data {
            let parts: Vec<&str> = key.split(',').collect();
            if parts.len() != 2 {
                continue;
            }

            let row: u32 = parts[0].parse().unwrap_or(0);
            let col: u16 = parts[1].parse().unwrap_or(0);

            let mut format = Format::new();

            if let Some(style) = &cell_data.style {
                if let Some(ref weight) = style.font_weight {
                    if weight == "bold" {
                        format = format.set_bold();
                    }
                }
                if let Some(ref font_style) = style.font_style {
                    if font_style == "italic" {
                        format = format.set_italic();
                    }
                }
                if let Some(size) = style.font_size {
                    format = format.set_font_size(size as f64);
                }
                if let Some(ref font) = style.font_family {
                    format = format.set_font_name(font);
                }
            }

            if let Some(ref formula) = cell_data.formula {
                let formula_str = if formula.starts_with('=') {
                    &formula[1..]
                } else {
                    formula
                };
                let _ = ws.write_formula_with_format(row, col, formula_str, &format);
            } else if let Some(ref value) = cell_data.value {
                if let Ok(num) = value.parse::<f64>() {
                    let _ = ws.write_number_with_format(row, col, num, &format);
                } else {
                    let _ = ws.write_string_with_format(row, col, value, &format);
                }
            }
        }

        if let Some(widths) = &worksheet.column_widths {
            for (col_idx, width) in widths {
                let _ = ws.set_column_width(*col_idx as u16, *width as f64);
            }
        }

        if let Some(heights) = &worksheet.row_heights {
            for (row_idx, height) in heights {
                let _ = ws.set_row_height(*row_idx, *height as f64);
            }
        }

        if let Some(merged) = &worksheet.merged_cells {
            for merge in merged {
                let _ = ws.merge_range(
                    merge.start_row,
                    merge.start_col as u16,
                    merge.end_row,
                    merge.end_col as u16,
                    "",
                    &Format::new(),
                );
            }
        }
    }

    let buf = workbook.save_to_buffer().map_err(|e| format!("Failed to write xlsx: {e}"))?;
    Ok(buf)
}

pub async fn load_xlsx_from_drive(
    state: &Arc<AppState>,
    _user_id: &str,
    file_path: &str,
) -> Result<Spreadsheet, String> {
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

    load_xlsx_from_bytes(&bytes, file_path)
}

pub fn load_xlsx_from_bytes(bytes: &[u8], file_path: &str) -> Result<Spreadsheet, String> {
    let file_name = file_path
        .split('/')
        .last()
        .unwrap_or("Untitled")
        .trim_end_matches(".xlsx")
        .trim_end_matches(".xlsm")
        .trim_end_matches(".xls");

    let worksheets = parse_excel_to_worksheets(bytes, "xlsx")?;

    Ok(Spreadsheet {
        id: Uuid::new_v4().to_string(),
        name: file_name.to_string(),
        owner_id: get_current_user_id(),
        worksheets,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    })
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
    }])
}

pub fn parse_excel_to_worksheets(bytes: &[u8], _ext: &str) -> Result<Vec<Worksheet>, String> {
    let cursor = Cursor::new(bytes);
    let mut workbook: Xlsx<_> =
        Reader::new(cursor).map_err(|e| format!("Failed to parse spreadsheet: {e}"))?;

    let sheet_names: Vec<String> = workbook.sheet_names().to_vec();
    let mut worksheets = Vec::new();

    for sheet_name in sheet_names {
        let range = workbook
            .worksheet_range(&sheet_name)
            .map_err(|e| format!("Failed to read sheet {sheet_name}: {e}"))?;

        let mut data: HashMap<String, CellData> = HashMap::new();

        for (row_idx, row) in range.rows().enumerate() {
            for (col_idx, cell) in row.iter().enumerate() {
                let value = match cell {
                    Data::Empty => continue,
                    Data::String(s) => s.clone(),
                    Data::Int(i) => i.to_string(),
                    Data::Float(f) => f.to_string(),
                    Data::Bool(b) => b.to_string(),
                    Data::DateTime(dt) => dt.to_string(),
                    Data::Error(e) => format!("#ERR:{e:?}"),
                    Data::DateTimeIso(s) => s.clone(),
                    Data::DurationIso(s) => s.clone(),
                };

                let key = format!("{row_idx},{col_idx}");
                data.insert(
                    key,
                    CellData {
                        value: Some(value),
                        formula: None,
                        style: None,
                        format: None,
                        note: None,
                    },
                );
            }
        }

        worksheets.push(Worksheet {
            name: sheet_name,
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
        });
    }

    if worksheets.is_empty() {
        return Err("Spreadsheet has no sheets".to_string());
    }

    Ok(worksheets)
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
        }],
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}
