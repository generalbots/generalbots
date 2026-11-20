//! TEXTJOIN and CONCAT parity with the local preview (#2572).

use crate::types::Worksheet;

use super::financial::raw_args;
use super::helpers::resolve_cell_value;
use super::numeric_args::{error_literal, split_call_args};

fn unquote(raw: &str, worksheet: &Worksheet) -> String {
    let text = raw.trim();
    let unquoted = text
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(text);
    resolve_cell_value(unquoted, worksheet)
}

fn text_values(raw: &str, worksheet: &Worksheet) -> Result<Vec<String>, String> {
    let text = raw.trim();
    if let Some(error) = error_literal(text) {
        return Err(error);
    }
    if let Some(inner) = text.strip_prefix('[').and_then(|value| value.strip_suffix(']')) {
        return Ok(split_call_args(inner)
            .into_iter()
            .map(|part| unquote(part, worksheet))
            .collect());
    }
    if text.contains(':') {
        let values = super::refs::get_range_string_values(text, worksheet);
        if values.is_empty() {
            return Err("#REF!".to_string());
        }
        return Ok(values);
    }
    Ok(vec![unquote(text, worksheet)])
}

pub fn evaluate_textjoin(expr: &str, worksheet: &Worksheet) -> Option<String> {
    let raw = raw_args(expr, "TEXTJOIN")?;
    let delimiter = unquote(raw.first().copied().unwrap_or(""), worksheet);
    let ignore = raw
        .get(1)
        .map(|value| value.trim().eq_ignore_ascii_case("TRUE"))
        .unwrap_or(true);
    let mut parts = Vec::new();
    for argument in raw.iter().skip(2) {
        match text_values(argument, worksheet) {
            Ok(values) => parts.extend(values),
            Err(error) => return Some(error),
        }
    }
    let parts: Vec<String> = if ignore {
        parts.into_iter().filter(|value| !value.is_empty()).collect()
    } else {
        parts
    };
    Some(parts.join(&delimiter))
}

pub fn evaluate_concat(expr: &str, worksheet: &Worksheet) -> Option<String> {
    let raw = raw_args(expr, "CONCAT")?;
    let mut parts = Vec::new();
    for argument in raw {
        match text_values(argument, worksheet) {
            Ok(values) => parts.extend(values),
            Err(error) => return Some(error),
        }
    }
    Some(parts.concat())
}
