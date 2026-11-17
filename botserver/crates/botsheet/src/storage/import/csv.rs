use crate::types::Spreadsheet;
use std::collections::HashMap;

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
