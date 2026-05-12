use crate::types::Worksheet;

use super::helpers::{resolve_cell_value, split_args};

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
