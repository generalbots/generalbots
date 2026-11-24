use serde_json::{Map, Value};

pub(super) fn cell_str(v: &str) -> crate::types::CellData {
    crate::types::CellData {
        value: Some(v.to_string()),
        typed: Some(botsheet_core::engine::CellValue::parse(v)),
        formula: None,
        style: None,
        format: None,
        note: None,
        locked: None,
        has_comment: None,
        array_formula_id: None,
    }
}

pub(super) fn build_pivot_worksheet(
    row_keys_v: &[String],
    col_keys_v: &[String],
    value_specs: &[(u32, String)],
    cells: &Map<String, Value>,
    row_totals: &Map<String, Value>,
    col_totals: &Map<String, Value>,
    grand_total: f64,
) -> crate::types::Worksheet {
    use std::collections::HashMap;
    let mut data: HashMap<String, crate::types::CellData> = HashMap::new();
    let mut col = 0u32;
    data.insert(format!("0,{col}"), cell_str("Row"));
    col += 1;
    if col_keys_v.is_empty() {
        for (_, agg) in value_specs {
            data.insert(format!("0,{col}"), cell_str(agg));
            col += 1;
        }
    } else {
        for ck in col_keys_v {
            let label = if ck.is_empty() {
                "(blank)".to_string()
            } else {
                ck.replace('\0', " / ")
            };
            for (_, agg) in value_specs {
                let head = if value_specs.len() > 1 {
                    format!("{label} ({agg})")
                } else {
                    label.clone()
                };
                data.insert(format!("0,{col}"), cell_str(&head));
                col += 1;
            }
        }
    }
    data.insert(format!("0,{col}"), cell_str("Total"));

    for (ri, rk) in row_keys_v.iter().enumerate() {
        let r = (ri + 1) as u32;
        let rlabel = if rk.is_empty() {
            "(blank)".to_string()
        } else {
            rk.replace('\0', " / ")
        };
        data.insert(format!("{r},0"), cell_str(&rlabel));
        let mut c = 1u32;
        let ckeys: Vec<String> = if col_keys_v.is_empty() {
            vec![String::new()]
        } else {
            col_keys_v.to_vec()
        };
        for ck in &ckeys {
            for i in 0..value_specs.len() {
                let key = format!("{rk}\x00{ck}\x00{i}");
                let val = match cells.get(&key).and_then(|v| v.as_f64()) {
                    Some(n) => n.to_string(),
                    None => String::new(),
                };
                data.insert(format!("{r},{c}"), cell_str(&val));
                c += 1;
            }
        }
        let rt = match row_totals.get(rk).and_then(|v| v.as_f64()) {
            Some(n) => n.to_string(),
            None => String::new(),
        };
        data.insert(format!("{r},{c}"), cell_str(&rt));
    }
    let total_row = (row_keys_v.len() + 1) as u32;
    data.insert(format!("{total_row},0"), cell_str("Total"));
    let mut c = 1u32;
    let ckeys: Vec<String> = if col_keys_v.is_empty() {
        vec![String::new()]
    } else {
        col_keys_v.to_vec()
    };
    for ck in &ckeys {
        for _ in 0..value_specs.len() {
            let ct = match col_totals.get(ck).and_then(|v| v.as_f64()) {
                Some(n) => n.to_string(),
                None => String::new(),
            };
            data.insert(format!("{total_row},{c}"), cell_str(&ct));
            c += 1;
        }
    }
    data.insert(
        format!("{total_row},{c}"),
        cell_str(&grand_total.to_string()),
    );

    crate::types::Worksheet {
        name: "Pivot".to_string(),
        data,
        column_widths: None,
        row_heights: None,
        frozen_rows: None,
        frozen_cols: None,
        merged_cells: None,
        filters: None,
        hidden_rows: None,
        hidden_columns: None,
        validations: None,
        conditional_formats: None,
        charts: None,
        comments: None,
        protection: None,
        array_formulas: None,
        tables: None,
        sheet_state: None,
        hyperlinks: None,
        print_setup: None,
        autofilter: None,
        row_page_breaks: None,
        column_page_breaks: None,
        images: None,
        print_areas: None,
        rich_text: None,
    }
}
