use crate::types::Spreadsheet;
use botsheet_core::engine::value::CellValue;

use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

/// Maps umya sheet state to the model visibility string (`None` = visible).
fn sheet_visibility(sheet: &umya_spreadsheet::Worksheet) -> Option<String> {
    use umya_spreadsheet::structs::SheetStateValues;
    match sheet.get_state() {
        SheetStateValues::Hidden => Some("hidden".to_string()),
        SheetStateValues::VeryHidden => Some("veryHidden".to_string()),
        SheetStateValues::Visible => None,
    }
}

pub fn create_new_spreadsheet(owner_id: &str) -> Spreadsheet {
    Spreadsheet {
        id: Uuid::new_v4().to_string(),
        name: "Untitled Spreadsheet".to_string(),
        owner_id: owner_id.to_string(),
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
        }],
        created_at: Utc::now(),
        updated_at: Utc::now(),
        named_ranges: None,
        external_links: None,
        source_bucket: None,
        source_path: None,
        source_bytes: None,
        acl: HashMap::new(),
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
                        typed: None,
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
    }])
}

pub fn parse_xlsx_to_worksheets(
    bytes: &[u8],
    ext: &str,
) -> Result<Vec<crate::types::Worksheet>, String> {
    if ext == "ods" {
        return super::import_ods::parse_ods_to_worksheets(bytes);
    }

    if ext == "xlsx" || ext == "xlsm" || ext == "xls" {
        if let Ok(workbook) = umya_spreadsheet::reader::xlsx::read_reader(std::io::Cursor::new(bytes), true) {
            let mut worksheets = Vec::new();

            for sheet in workbook.get_sheet_collection() {
                let mut data: HashMap<String, crate::types::CellData> = HashMap::new();

                // Iterate actual cells (umya's map), not the 1..=max bounding box.
                for ((col, row), cell) in sheet.get_collection_to_hashmap() {
                    let cell = cell.as_ref();
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

                    // Format code (e.g. "$#,##0.00") so the type survives round-trip.
                    let number_format = cell.get_style().get_number_format()
                        .map(|nf| nf.get_format_code().to_string())
                        .filter(|c| c != "General" && !c.is_empty());

                    // Typed value (#781/#785): keeps the real cell type for
                    // format rendering, sort and filter. A text cell whose
                    // content is numeric-looking (`0123`, a phone number, a
                    // zip code) must stay `Text` — parsing it back from the
                    // display string would turn it into a number and drop a
                    // leading zero.
                    let typed = match cell.get_raw_value() {
                        umya_spreadsheet::structs::CellRawValue::Numeric(n) => {
                            CellValue::Number(*n)
                        }
                        umya_spreadsheet::structs::CellRawValue::Bool(b) => CellValue::Bool(*b),
                        umya_spreadsheet::structs::CellRawValue::Error(_) => {
                            CellValue::Error(value.clone())
                        }
                        umya_spreadsheet::structs::CellRawValue::String(s) => {
                            CellValue::Text(s.to_string())
                        }
                        umya_spreadsheet::structs::CellRawValue::RichText(_) => {
                            CellValue::Text(value.clone())
                        }
                        umya_spreadsheet::structs::CellRawValue::Empty => CellValue::Empty,
                        umya_spreadsheet::structs::CellRawValue::Lazy(_) => CellValue::parse(&value),
                    };

                    data.insert(
                        key,
                        crate::types::CellData {
                            value: if value.is_empty() {
                                None
                            } else {
                                Some(value)
                            },
                            typed: Some(typed),
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
                let layout = super::xlsx_layout::extract_layout(sheet);
                let hyperlinks = super::xlsx_layout::extract_hyperlinks(sheet);
                let structures = super::xlsx_layout::extract_structures(sheet);
                worksheets.push(crate::types::Worksheet {
                    name: sheet.get_name().to_string(),
                    data,
                    column_widths: layout.column_widths,
                    row_heights: layout.row_heights,
                    frozen_rows: layout.frozen_rows,
                    frozen_cols: layout.frozen_cols,
                    merged_cells: layout.merged_cells,
                    filters: None,
                    hidden_rows: layout.hidden_rows,
                    validations: super::xlsx_rules::extract_validations(sheet),
                    conditional_formats: super::xlsx_rules::extract_conditional_formats(sheet),
                    charts: None,
                    comments: None,
                    protection: None,
                    array_formulas: None,
                    tables: structures.tables,
                    hidden_columns: layout.hidden_columns,
                    sheet_state: sheet_visibility(sheet),
                    hyperlinks,
                    print_setup: super::xlsx_layout::extract_print_setup(sheet),
                    autofilter: structures.autofilter,
                    row_page_breaks: structures.row_page_breaks,
                    column_page_breaks: structures.column_page_breaks,
                    images: super::xlsx_layout::extract_images(sheet),
                    print_areas: None,
                    rich_text: None,
                });
            }

            if !worksheets.is_empty() {
                #[cfg(feature = "xlsx")]
                if let Ok(format_map) = super::format_codes::extract_cell_format_codes(bytes) {
                    super::format_codes::apply_format_codes(&mut worksheets, &format_map);
                }

                // umya drops charts on read; extract from the raw package.
                #[cfg(feature = "xlsx")]
                match super::chart_read::extract_charts(bytes, &worksheets) {
                    Ok(charts_by_ws) => {
                        for (ws_index, charts) in charts_by_ws.into_iter().enumerate() {
                            if !charts.is_empty() {
                                if let Some(ws) = worksheets.get_mut(ws_index) {
                                    ws.charts = Some(charts);
                                }
                            }
                        }
                    }
                    Err(e) => log::warn!("chart extraction skipped: {e}"),
                }

                // Print areas + titles live in workbook defined names (#788 E11).
                let print_areas = super::xlsx_layout::extract_print_areas(&workbook);
                for (sheet_index, areas) in print_areas {
                    if let Some(ws) = worksheets.get_mut(sheet_index) {
                        ws.print_areas = Some(areas);
                    }
                }
                let print_titles = super::xlsx_layout::extract_print_titles(&workbook);
                for (sheet_index, titles) in print_titles {
                    if let Some(ws) = worksheets.get_mut(sheet_index) {
                        let setup = ws.print_setup.get_or_insert_with(Default::default);
                        setup.print_titles = Some(titles);
                    }
                }

                // Rich-text runs (E6): umya flattens them, so recover from the
                // raw sharedStrings.xml + sheet parts.
                let rich_text_by_sheet = super::xlsx_rich_text::extract_rich_text_all(bytes);
                for (sheet_index, rich_text) in rich_text_by_sheet {
                    if let Some(ws) = worksheets.get_mut(sheet_index) {
                        ws.rich_text = Some(rich_text);
                    }
                }

                // Cell comments / notes (gap 27): recover from xl/commentsN.xml.
                let comments_by_sheet = super::xlsx_comments::extract_comments(bytes);
                for (sheet_index, comments) in comments_by_sheet {
                    if let Some(ws) = worksheets.get_mut(sheet_index) {
                        for key in comments.keys() {
                            if let Some(cell) = ws.data.get_mut(key) {
                                cell.has_comment = Some(true);
                            }
                        }
                        ws.comments = Some(comments);
                    }
                }

                return Ok(worksheets);
            }
        }
    }

    Err("Failed to parse spreadsheet".to_string())
}

pub fn detect_spreadsheet_format(bytes: &[u8]) -> &'static str {
    if bytes.len() >= 4 {
        if &bytes[0..4] == b"PK\x03\x04" {
            let content_str = String::from_utf8_lossy(&bytes[0..4096.min(bytes.len())]);
            if content_str.contains("xl/workbook.bin") {
                return "xlsb";
            }
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
    owner_id: &str,
) -> Result<Spreadsheet, String> {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    let detected = detect_spreadsheet_format(bytes);

    let worksheets = match detected {
        "xlsx" | "xlsm" => parse_xlsx_to_worksheets(bytes, "xlsx")?,
        "xls" => super::import_binary::parse_binary_to_worksheets(bytes, "xls")?,
        "xlsb" => super::import_binary::parse_binary_to_worksheets(bytes, "xlsb")?,
        "ods" => super::import_ods::parse_ods_to_worksheets(bytes)?,
        "csv" => parse_csv_to_worksheets(bytes, b',', "Sheet1")?,
        "tsv" => parse_csv_to_worksheets(bytes, b'\t', "Sheet1")?,
        _ => {
            if ext == "csv" {
                parse_csv_to_worksheets(bytes, b',', "Sheet1")?
            } else if ext == "tsv" || ext == "txt" {
                parse_csv_to_worksheets(bytes, b'\t', "Sheet1")?
            } else if ext == "ods" {
                super::import_ods::parse_ods_to_worksheets(bytes)?
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
        owner_id: owner_id.to_string(),
        worksheets,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        named_ranges: None,
        external_links: super::xlsx_external_links::extract_external_links(bytes),
        source_bucket: None,
        source_path: None,
        source_bytes: None,
        acl: HashMap::new(),
    })
}
