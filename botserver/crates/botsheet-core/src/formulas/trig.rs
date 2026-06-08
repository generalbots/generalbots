use crate::types::Worksheet;

use super::helpers::resolve_cell_value;

pub fn evaluate_sin(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("SIN(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[4..expr.len() - 1];
    let num: f64 = resolve_cell_value(inner.trim(), worksheet).parse().ok()?;
    Some(super::format_number(num.sin()))
}

pub fn evaluate_cos(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("COS(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[4..expr.len() - 1];
    let num: f64 = resolve_cell_value(inner.trim(), worksheet).parse().ok()?;
    Some(super::format_number(num.cos()))
}

pub fn evaluate_tan(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("TAN(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[4..expr.len() - 1];
    let num: f64 = resolve_cell_value(inner.trim(), worksheet).parse().ok()?;
    Some(super::format_number(num.tan()))
}

pub fn evaluate_asin(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("ASIN(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[5..expr.len() - 1];
    let num: f64 = resolve_cell_value(inner.trim(), worksheet).parse().ok()?;
    Some(super::format_number(num.asin()))
}

pub fn evaluate_acos(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("ACOS(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[5..expr.len() - 1];
    let num: f64 = resolve_cell_value(inner.trim(), worksheet).parse().ok()?;
    Some(super::format_number(num.acos()))
}

pub fn evaluate_atan(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("ATAN(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[5..expr.len() - 1];
    let num: f64 = resolve_cell_value(inner.trim(), worksheet).parse().ok()?;
    Some(super::format_number(num.atan()))
}

pub fn evaluate_atan2(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("ATAN2(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[6..expr.len() - 1];
    let parts: Vec<&str> = super::split_args(inner);
    let y: f64 = resolve_cell_value(parts[0].trim(), worksheet).parse().ok()?;
    let x: f64 = resolve_cell_value(parts[1].trim(), worksheet).parse().ok()?;
    Some(super::format_number(y.atan2(x)))
}
