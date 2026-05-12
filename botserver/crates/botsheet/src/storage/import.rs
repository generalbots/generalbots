use crate::types::Spreadsheet;

use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

pub fn create_new_spreadsheet() -> Spreadsheet {
    Spreadsheet {
        id: Uuid::new_v4().to_string(),
        name: "Untitled Spreadsheet".to_string(),
        owner_id: crate::state::get_current_user_id(),
        worksheets: vec![crate::types::Worksheet {
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

pub fn parse_csv_to_worksheets(
    bytes: &[u8],
    delimiter: u8,
    sheet_name: &str,
) -> Result<Vec<crate::types::Worksheet>, String> {
    let content = String::from_utf8_lossy(bytes);
    let mut data: HashMap<String, crate::types::CellData> = HashMap::new();

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
                    crate::types::CellData {
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

    Ok(vec![crate::types::Worksheet {
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

pub fn parse_excel_to_worksheets(
    bytes: &[u8],
    ext: &str,
) -> Result<Vec<crate::types::Worksheet>, String> {
    if ext == "xlsx" || ext == "xlsm" || ext == "xls" {
        let cursor = std::io::Cursor::new(bytes);
        if let Ok(workbook) = umya_spreadsheet::reader::xlsx::read_reader(cursor, true) {
            let mut worksheets = Vec::new();

            for sheet in workbook.get_sheet_collection() {
                let mut data: HashMap<String, crate::types::CellData> = HashMap::new();
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

                            data.insert(
                                key,
                                crate::types::CellData {
                                    value: if value.is_empty() {
                                        None
                                    } else {
                                        Some(value)
                                    },
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

                worksheets.push(crate::types::Worksheet {
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

pub fn parse_ods_to_worksheets(bytes: &[u8]) -> Result<Vec<crate::types::Worksheet>, String> {
    let content = String::from_utf8_lossy(bytes);
    let mut worksheets = Vec::new();
    let mut current_sheet_name = "Sheet1".to_string();
    let mut data: HashMap<String, crate::types::CellData> = HashMap::new();
    let mut row_idx = 0u32;
    let mut in_table = false;
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
                    worksheets.push(crate::types::Worksheet {
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
                col_idx = 0;
            } else if tag == "/table:table-row" {
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
                    data.insert(
                        key,
                        crate::types::CellData {
                            value: if cell_value.is_empty() {
                                None
                            } else {
                                Some(cell_value)
                            },
                            formula: if has_formula { Some(formula) } else { None },
                            style: None,
                            format: None,
                            note: None,
                            locked: None,
                            has_comment: None,
                            array_formula_id: None,
                        },
                    );
                }

                col_idx += 1;
            }
        }
        i += 1;
    }

    if worksheets.is_empty() {
        worksheets.push(crate::types::Worksheet {
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

pub fn import_spreadsheet_bytes(
    bytes: &[u8],
    filename: &str,
) -> Result<Spreadsheet, String> {
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

    let raw_filename = filename.rsplit('/').next().unwrap_or(filename);
    let suffix = format!(".{ext}");
    let name = raw_filename
        .strip_suffix(&suffix)
        .unwrap_or(raw_filename)
        .to_string();

    Ok(Spreadsheet {
        id: Uuid::new_v4().to_string(),
        name,
        owner_id: crate::state::get_current_user_id(),
        worksheets,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        named_ranges: None,
        external_links: None,
    })
}
