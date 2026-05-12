use crate::types::Worksheet;

use super::helpers::{parse_range, split_args};

pub fn evaluate_vlookup(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("VLOOKUP(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[8..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    if parts.len() < 3 {
        return None;
    }
    let lookup_value = parts[0].trim().trim_matches('"');
    let table_range = parts[1].trim();
    let col_index: usize = parts[2].trim().parse().ok()?;

    let (start, end) = parse_range(table_range)?;
    for row in start.0..=end.0 {
        let key = format!("{},{}", row, start.1);
        let cell_value = worksheet
            .data
            .get(&key)
            .and_then(|c| c.value.clone())
            .unwrap_or_default();
        if cell_value.eq_ignore_ascii_case(lookup_value) {
            let result_col = start.1 + col_index as u32 - 1;
            if result_col > end.1 {
                return Some("#REF!".to_string());
            }
            let result_key = format!("{},{}", row, result_col);
            return Some(
                worksheet
                    .data
                    .get(&result_key)
                    .and_then(|c| c.value.clone())
                    .unwrap_or_default(),
            );
        }
    }
    Some("#N/A".to_string())
}

pub fn evaluate_hlookup(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("HLOOKUP(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[8..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    if parts.len() < 3 {
        return None;
    }
    let lookup_value = parts[0].trim().trim_matches('"');
    let table_range = parts[1].trim();
    let row_index: usize = parts[2].trim().parse().ok()?;

    let (start, end) = parse_range(table_range)?;
    for col in start.1..=end.1 {
        let key = format!("{},{}", start.0, col);
        let cell_value = worksheet
            .data
            .get(&key)
            .and_then(|c| c.value.clone())
            .unwrap_or_default();
        if cell_value.eq_ignore_ascii_case(lookup_value) {
            let result_row = start.0 + row_index as u32 - 1;
            if result_row > end.0 {
                return Some("#REF!".to_string());
            }
            let result_key = format!("{},{}", result_row, col);
            return Some(
                worksheet
                    .data
                    .get(&result_key)
                    .and_then(|c| c.value.clone())
                    .unwrap_or_default(),
            );
        }
    }
    Some("#N/A".to_string())
}

pub fn evaluate_index_match(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("INDEX(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[6..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    if parts.len() < 2 {
        return None;
    }
    let range = parts[0].trim();
    let row_num: u32 = parts[1].trim().parse().ok()?;
    let col_num: u32 = if parts.len() > 2 {
        parts[2].trim().parse().ok()?
    } else {
        1
    };

    let (start, _end) = parse_range(range)?;
    let target_row = start.0 + row_num - 1;
    let target_col = start.1 + col_num - 1;
    let key = format!("{},{}", target_row, target_col);
    Some(
        worksheet
            .data
            .get(&key)
            .and_then(|c| c.value.clone())
            .unwrap_or_default(),
    )
}
