use crate::types::Worksheet;

use super::helpers::{resolve_cell_value, split_args};
use super::refs::{clamp_range, parse_range};

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
    let col_index: u32 = parts[2].trim().parse().ok()?;
    // Column offsets are one-based; offset 0 is an invalid argument, not column -1.
    let col_offset = col_index.checked_sub(1)?;

    let (start, end) = parse_range(table_range).map(|(s, e)| clamp_range(s, e, worksheet))?;
    for row in start.0..=end.0 {
        let key = format!("{},{}", row, start.1);
        let cell_value = worksheet
            .data
            .get(&key)
            .and_then(|c| c.value.clone())
            .unwrap_or_default();
        if cell_value.eq_ignore_ascii_case(lookup_value) {
            let result_col = start.1.checked_add(col_offset)?;
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
    let row_index: u32 = parts[2].trim().parse().ok()?;
    // Row offsets are one-based; offset 0 is an invalid argument, not row -1.
    let row_offset = row_index.checked_sub(1)?;

    let (start, end) = parse_range(table_range).map(|(s, e)| clamp_range(s, e, worksheet))?;
    for col in start.1..=end.1 {
        let key = format!("{},{}", start.0, col);
        let cell_value = worksheet
            .data
            .get(&key)
            .and_then(|c| c.value.clone())
            .unwrap_or_default();
        if cell_value.eq_ignore_ascii_case(lookup_value) {
            let result_row = start.0.checked_add(row_offset)?;
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
    // INDEX offsets are one-based; a zero offset is invalid rather than a wrap-around.
    let target_row = start.0.checked_add(row_num.checked_sub(1)?)?;
    let target_col = start.1.checked_add(col_num.checked_sub(1)?)?;
    let key = format!("{target_row},{target_col}");
    Some(
        worksheet
            .data
            .get(&key)
            .and_then(|c| c.value.clone())
            .unwrap_or_default(),
    )
}


pub fn evaluate_xlookup(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("XLOOKUP(") || !expr.ends_with(')') { return None; }
    let inner = &expr[8..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    if parts.len() < 3 { return None; }
    let needle = parts[0].trim().trim_matches('"');
    let lookup_range = parts[1].trim();
    let return_range = parts[2].trim();
    let lookup_values = super::get_range_string_values(lookup_range, worksheet);
    let return_values = super::get_range_string_values(return_range, worksheet);
    for (i, v) in lookup_values.iter().enumerate() {
        if v == needle {
            return return_values.get(i).cloned();
        }
    }
    None
}

pub fn evaluate_match(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("MATCH(") || !expr.ends_with(')') { return None; }
    let inner = &expr[6..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    if parts.len() < 2 { return None; }
    let needle = parts[0].trim().trim_matches('"');
    let lookup_range = parts[1].trim();
    let lookup_values = super::get_range_string_values(lookup_range, worksheet);
    for (i, v) in lookup_values.iter().enumerate() {
        if v == needle {
            return Some((i + 1).to_string());
        }
    }
    Some("#N/A".to_string())
}

pub fn evaluate_choose(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("CHOOSE(") || !expr.ends_with(')') { return None; }
    let inner = &expr[7..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    if parts.len() < 2 { return None; }
    let idx: usize = parts[0].trim().parse().ok()?;
    if idx == 0 || idx > parts.len() - 1 { return Some("#VALUE!".to_string()); }
    Some(resolve_cell_value(parts[idx].trim().trim_matches('"'), worksheet))
}
