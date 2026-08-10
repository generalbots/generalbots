use crate::types::Worksheet;

use super::helpers::{format_number, resolve_cell_value, split_args};

pub fn evaluate_concatenate(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("CONCATENATE(") && !expr.starts_with("CONCAT(") {
        return None;
    }
    if !expr.ends_with(')') {
        return None;
    }
    let start_idx = if expr.starts_with("CONCATENATE(") {
        12
    } else {
        7
    };
    let inner = &expr[start_idx..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    let result: String = parts
        .iter()
        .map(|p| {
            let trimmed = p.trim().trim_matches('"');
            resolve_cell_value(trimmed, worksheet)
        })
        .collect();
    Some(result)
}

pub fn evaluate_left(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("LEFT(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[5..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    if parts.is_empty() {
        return None;
    }
    let text = resolve_cell_value(parts[0].trim().trim_matches('"'), worksheet);
    let num_chars: usize = if parts.len() > 1 {
        parts[1].trim().parse().unwrap_or(1)
    } else {
        1
    };
    Some(text.chars().take(num_chars).collect())
}

pub fn evaluate_right(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("RIGHT(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[6..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    if parts.is_empty() {
        return None;
    }
    let text = resolve_cell_value(parts[0].trim().trim_matches('"'), worksheet);
    let num_chars: usize = if parts.len() > 1 {
        parts[1].trim().parse().unwrap_or(1)
    } else {
        1
    };
    let len = text.chars().count();
    let skip = len.saturating_sub(num_chars);
    Some(text.chars().skip(skip).collect())
}

pub fn evaluate_mid(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("MID(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[4..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    if parts.len() < 3 {
        return None;
    }
    let text = resolve_cell_value(parts[0].trim().trim_matches('"'), worksheet);
    let start_pos: usize = parts[1].trim().parse().unwrap_or(1);
    let num_chars: usize = parts[2].trim().parse().unwrap_or(1);
    Some(
        text.chars()
            .skip(start_pos.saturating_sub(1))
            .take(num_chars)
            .collect(),
    )
}

pub fn evaluate_len(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("LEN(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[4..expr.len() - 1];
    let text = resolve_cell_value(inner.trim().trim_matches('"'), worksheet);
    Some(text.chars().count().to_string())
}

pub fn evaluate_trim(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("TRIM(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[5..expr.len() - 1];
    let text = resolve_cell_value(inner.trim().trim_matches('"'), worksheet);
    Some(text.split_whitespace().collect::<Vec<_>>().join(" "))
}

pub fn evaluate_upper(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("UPPER(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[6..expr.len() - 1];
    let text = resolve_cell_value(inner.trim().trim_matches('"'), worksheet);
    Some(text.to_uppercase())
}

pub fn evaluate_lower(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("LOWER(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[6..expr.len() - 1];
    let text = resolve_cell_value(inner.trim().trim_matches('"'), worksheet);
    Some(text.to_lowercase())
}

pub fn evaluate_proper(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("PROPER(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[7..expr.len() - 1];
    let text = resolve_cell_value(inner.trim().trim_matches('"'), worksheet);
    let result: String = text
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    let mut result = first.to_uppercase().to_string();
                    result.push_str(&chars.as_str().to_lowercase());
                    result
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    Some(result)
}

pub fn evaluate_substitute(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("SUBSTITUTE(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[11..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    if parts.len() < 3 {
        return None;
    }
    let text = resolve_cell_value(parts[0].trim().trim_matches('"'), worksheet);
    let old_text = parts[1].trim().trim_matches('"');
    let new_text = parts[2].trim().trim_matches('"');
    Some(text.replace(old_text, new_text))
}


pub fn evaluate_replace(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("REPLACE(") || !expr.ends_with(')') { return None; }
    let inner = &expr[8..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    if parts.len() < 4 { return None; }
    let text = resolve_cell_value(parts[0].trim().trim_matches('"'), worksheet);
    let start: usize = parts[1].trim().parse().unwrap_or(1);
    let num_chars: usize = parts[2].trim().parse().unwrap_or(0);
    let new_text = parts[3].trim().trim_matches('"');
    let mut chars: Vec<char> = text.chars().collect();
    let s_idx = start.saturating_sub(1);
    if s_idx >= chars.len() { return Some(text); }
    let e_idx = (s_idx + num_chars).min(chars.len());
    chars.splice(s_idx..e_idx, new_text.chars());
    Some(chars.into_iter().collect())
}

pub fn evaluate_find(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("FIND(") || !expr.ends_with(')') { return None; }
    let inner = &expr[5..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    if parts.len() < 2 { return None; }
    let find_text = parts[0].trim().trim_matches('"');
    let text = resolve_cell_value(parts[1].trim().trim_matches('"'), worksheet);
    let start: usize = if parts.len() > 2 { parts[2].trim().parse().unwrap_or(1) } else { 1 };
    let bytes: Vec<(usize, char)> = text.char_indices().collect();
    if start == 0 || start > bytes.len() { return Some("#VALUE!".to_string()); }
    let idx = bytes[start - 1..].iter().position(|(_, c)| *c == find_text.chars().next().unwrap_or(' '));
    if let Some(rel) = idx {
        Some((start + rel).to_string())
    } else {
        Some("#VALUE!".to_string())
    }
}

pub fn evaluate_search(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("SEARCH(") || !expr.ends_with(')') { return None; }
    let inner = &expr[7..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    if parts.len() < 2 { return None; }
    let needle = parts[0].trim().trim_matches('"').to_lowercase();
    let haystack = resolve_cell_value(parts[1].trim().trim_matches('"'), worksheet).to_lowercase();
    let pos = haystack.find(&needle);
    match pos {
        Some(byte_idx) => Some((text_chars_before(&haystack, byte_idx) + 1).to_string()),
        None => Some("#VALUE!".to_string()),
    }
}

fn text_chars_before(s: &str, byte_pos: usize) -> usize {
    s[..byte_pos].chars().count()
}

pub fn evaluate_exact(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("EXACT(") || !expr.ends_with(')') { return None; }
    let inner = &expr[6..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    if parts.len() != 2 { return None; }
    let a = resolve_cell_value(parts[0].trim().trim_matches('"'), worksheet);
    let b = resolve_cell_value(parts[1].trim().trim_matches('"'), worksheet);
    Some(if a == b { "TRUE" } else { "FALSE" }.to_string())
}

pub fn evaluate_rept(expr: &str, _worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("REPT(") || !expr.ends_with(')') { return None; }
    let inner = &expr[5..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    if parts.len() != 2 { return None; }
    let text = parts[0].trim().trim_matches('"');
    let count: usize = parts[1].trim().parse().unwrap_or(0);
    Some(text.repeat(count))
}

pub fn evaluate_text(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("TEXT(") || !expr.ends_with(')') { return None; }
    let inner = &expr[5..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    if parts.len() != 2 { return None; }
    let val = resolve_cell_value(parts[0].trim(), worksheet);
    let fmt = parts[1].trim().trim_matches('"');
    let cell = crate::engine::value::CellValue::parse(&val);
    Some(crate::engine::formats::apply_format(&cell, fmt))
}

pub fn evaluate_value(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("VALUE(") || !expr.ends_with(')') { return None; }
    let inner = &expr[6..expr.len() - 1];
    let val = resolve_cell_value(inner.trim().trim_matches('"'), worksheet);
    let n: f64 = val.parse().ok()?;
    Some(format_number(n))
}
