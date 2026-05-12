use crate::types::Worksheet;

use super::helpers::{evaluate_condition, split_args};

pub fn evaluate_if(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("IF(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[3..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    if parts.len() < 2 {
        return None;
    }
    let condition = parts[0].trim();
    let true_value = parts[1].trim().trim_matches('"');
    let false_value = if parts.len() > 2 {
        parts[2].trim().trim_matches('"')
    } else {
        "FALSE"
    };
    if evaluate_condition(condition, worksheet) {
        Some(true_value.to_string())
    } else {
        Some(false_value.to_string())
    }
}

pub fn evaluate_iferror(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("IFERROR(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[8..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    if parts.len() != 2 {
        return None;
    }
    let value_expr = parts[0].trim();
    let error_value = parts[1].trim().trim_matches('"');

    let result = super::evaluate_formula(&format!("={}", value_expr), worksheet);
    if result.error.is_some() || result.value.starts_with('#') {
        Some(error_value.to_string())
    } else {
        Some(result.value)
    }
}

pub fn evaluate_and(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("AND(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[4..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    let result = parts
        .iter()
        .all(|p| evaluate_condition(p.trim(), worksheet));
    Some(result.to_string().to_uppercase())
}

pub fn evaluate_or(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("OR(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[3..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    let result = parts
        .iter()
        .any(|p| evaluate_condition(p.trim(), worksheet));
    Some(result.to_string().to_uppercase())
}

pub fn evaluate_not(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("NOT(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[4..expr.len() - 1];
    let result = !evaluate_condition(inner.trim(), worksheet);
    Some(result.to_string().to_uppercase())
}
