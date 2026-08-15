//! ODS (OpenDocument Spreadsheet) import — split from `import.rs`.
//!
//! Reads `content.xml` from an ODS zip package into the worksheet model. The
//! parser is intentionally conservative: table/row/cell structure and
//! paragraph text are extracted; ODS styles/formulas are not modelled.

use std::collections::HashMap;

pub fn parse_ods_to_worksheets(bytes: &[u8]) -> Result<Vec<crate::types::Worksheet>, String> {
    use std::io::Cursor;
    use zip::ZipArchive;

    let cursor = Cursor::new(bytes);
    let mut zip = ZipArchive::new(cursor)
        .map_err(|e| format!("Failed to open ODS zip: {e}"))?;

    let mut content_xml = zip
        .by_name("content.xml")
        .map_err(|_| "content.xml not found in ODS".to_string())?;

    let mut xml_string = String::new();
    std::io::Read::read_to_string(&mut content_xml, &mut xml_string)
        .map_err(|e| format!("Failed to read content.xml: {e}"))?;

    parse_ods_xml(&xml_string)
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
                    b"table:table-row" if in_table => {
                        current_row += 1;
                    }
                    b"table:table-cell" if in_table => {
                        if !cell_value.is_empty() {
                            let key = format!("{current_row},{current_col}");
                            data.insert(
                                key,
                                crate::types::CellData {
                                    value: Some(cell_value.clone()),
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

    Ok(worksheets)
}
