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
    source_bucket: None,
    source_path: None,
        source_bytes: None,
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

pub fn parse_xlsx_to_worksheets(
    bytes: &[u8],
    ext: &str,
) -> Result<Vec<crate::types::Worksheet>, String> {
    if ext == "ods" {
        return parse_ods_to_worksheets(bytes);
    }

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

                            // Extract number format code (e.g. "$#,##0.00", "yyyy-mm-dd", "0%")
                            // to preserve cell type across round-trip xlsx → JSON → xlsx
                            let number_format = cell.get_style().get_number_format()
                                .map(|nf| nf.get_format_code().to_string())
                                .filter(|c| c != "General" && !c.is_empty());

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
                                    format: number_format,
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
                #[cfg(feature = "xlsx")]
                if let Ok(format_map) = super::format_codes::extract_cell_format_codes(bytes) {
                    super::format_codes::apply_format_codes(&mut worksheets, &format_map);
                }

                return Ok(worksheets);
            }
        }
    }

    Err("Failed to parse spreadsheet".to_string())
}

fn parse_ods_xml(xml_content: &str) -> Result<Vec<crate::types::Worksheet>, String> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_str(xml_content);

    let mut worksheets = Vec::new();
    let mut data: HashMap<String, crate::types::CellData> = HashMap::new();
    let mut sheet_name = String::new();
    let mut current_row: u32 = 0;
    let mut current_col: u32 = 0;
    let mut in_table = false;
    let mut cell_value = String::new();
    let mut in_text_p = false;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                match e.name().as_ref() {
                    b"table:table" => {
                        in_table = true;
                        current_row = 0;
                        data = HashMap::new();
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"table:name" {
                                sheet_name = String::from_utf8_lossy(&attr.value).to_string();
                            }
                        }
                        if sheet_name.is_empty() {
                            sheet_name = "Sheet1".to_string();
                        }
                    }
                    b"table:table-row" if in_table => {
                        current_col = 0;
                    }
                    b"table:table-cell" if in_table => {
                        cell_value.clear();
                    }
                    b"text:p" => {
                        in_text_p = true;
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                match e.name().as_ref() {
                    b"table:table" if in_table => {
                        in_table = false;
                        worksheets.push(crate::types::Worksheet {
                            name: sheet_name.clone(),
                            data: data.clone(),
                            column_widths: None, row_heights: None,
                            frozen_rows: None, frozen_cols: None,
                            merged_cells: None, filters: None,
                            hidden_rows: None, validations: None,
                            conditional_formats: None, charts: None,
                            comments: None, protection: None,
                            array_formulas: None,
                        });
                    }
                    b"table:table-row" if in_table => {
                        current_row += 1;
                    }
                    b"table:table-cell" if in_table => {
                        if !cell_value.is_empty() {
                            let key = format!("{current_row},{current_col}");
                            data.insert(key, crate::types::CellData {
                                value: Some(cell_value.clone()),
                                formula: None, style: None, format: None,
                                note: None, locked: None, has_comment: None,
                                array_formula_id: None,
                            });
                        }
                        current_col += 1;
                    }
                    b"text:p" => {
                        in_text_p = false;
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) => {
                if in_text_p {
                    if let Ok(text) = e.unescape() {
                        cell_value.push_str(&text);
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("XML parse error: {e}")),
            _ => {}
        }
        buf.clear();
    }

    if worksheets.is_empty() {
        worksheets.push(crate::types::Worksheet {
            name: "Sheet1".to_string(),
            data: HashMap::new(),
            column_widths: None, row_heights: None,
            frozen_rows: None, frozen_cols: None,
            merged_cells: None, filters: None,
            hidden_rows: None, validations: None,
            conditional_formats: None, charts: None,
            comments: None, protection: None,
            array_formulas: None,
        });
    }

    Ok(worksheets)
}

pub fn parse_ods_to_worksheets(bytes: &[u8]) -> Result<Vec<crate::types::Worksheet>, String> {
    use std::io::Cursor;
    use zip::ZipArchive;

    let cursor = Cursor::new(bytes);
    let mut zip = ZipArchive::new(cursor)
        .map_err(|e| format!("Failed to open ODS zip: {e}"))?;

    let mut content_xml = zip.by_name("content.xml")
        .map_err(|_| "content.xml not found in ODS".to_string())?;

    let mut xml_string = String::new();
    std::io::Read::read_to_string(&mut content_xml, &mut xml_string)
        .map_err(|e| format!("Failed to read content.xml: {e}"))?;

    parse_ods_xml(&xml_string)
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
        "xlsx" | "xlsm" => parse_xlsx_to_worksheets(bytes, "xlsx")?,
        "xls" => parse_xlsx_to_worksheets(bytes, "xls")?,
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
    source_bucket: None,
    source_path: None,
        source_bytes: None,
    })
}
