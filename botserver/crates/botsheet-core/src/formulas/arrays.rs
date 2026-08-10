use crate::types::Worksheet;

use super::helpers::{format_number, split_args};

fn parse_numbers(values: Vec<String>) -> Vec<f64> {
    values.iter().filter_map(|v| v.parse::<f64>().ok()).collect()
}

/// Splits a FILTER criterion on the first comparison operator.
fn split_compare(cond: &str) -> Option<(&str, &str, &str)> {
    for op in [">=", "<=", "<>", "!=", "=", ">", "<"] {
        if let Some(pos) = cond.find(op) {
            let left = cond[..pos].trim();
            let right = cond[pos + op.len()..].trim();
            if left.is_empty() || right.is_empty() {
                return None;
            }
            return Some((left, op, right));
        }
    }
    None
}

fn is_range(text: &str) -> bool {
    super::refs::parse_range(text).is_some()
}

fn truthy(text: &str) -> bool {
    let t = text.trim();
    t.eq_ignore_ascii_case("TRUE") || t.parse::<f64>().map(|n| n != 0.0).unwrap_or(!t.is_empty())
}

fn compare_text(left: &str, op: &str, right: &str) -> bool {
    let a = left.trim().trim_matches('"');
    let b = right.trim().trim_matches('"');
    match (a.parse::<f64>(), b.parse::<f64>()) {
        (Ok(x), Ok(y)) => match op {
            "=" => x == y,
            "<>" | "!=" => x != y,
            "<" => x < y,
            ">" => x > y,
            "<=" => x <= y,
            ">=" => x >= y,
            _ => false,
        },
        _ => match op {
            "=" => a.eq_ignore_ascii_case(b),
            "<>" | "!=" => !a.eq_ignore_ascii_case(b),
            "<" => a.to_lowercase() < b.to_lowercase(),
            ">" => a.to_lowercase() > b.to_lowercase(),
            "<=" => a.to_lowercase() <= b.to_lowercase(),
            ">=" => a.to_lowercase() >= b.to_lowercase(),
            _ => false,
        },
    }
}

/// Builds the per-row include mask for a FILTER criterion. The criterion can
/// be `TRUE`/`1` (keep all), a truthy criterion range, a scalar, or a
/// comparison such as `A1:A3>10` or `B1:B3<>""`.
fn filter_matches(values: &[String], cond: &str, worksheet: &Worksheet) -> Vec<bool> {
    let n = values.len();
    let all = || vec![true; n];
    let none = || vec![false; n];
    let cond = cond.trim();
    if cond == "TRUE" || cond == "1" {
        return all();
    }
    if cond == "FALSE" || cond == "0" {
        return none();
    }
    if let Some((left, op, right)) = split_compare(cond) {
        let lefts = if is_range(left) {
            super::get_range_string_values(left, worksheet)
        } else {
            vec![super::resolve_cell_value(left, worksheet); n]
        };
        let rights = if is_range(right) {
            super::get_range_string_values(right, worksheet)
        } else {
            vec![super::resolve_cell_value(right, worksheet); n]
        };
        return (0..n)
            .map(|i| {
                let l = lefts.get(i).cloned().unwrap_or_default();
                let r = rights.get(i).cloned().unwrap_or_default();
                compare_text(&l, op, &r)
            })
            .collect();
    }
    if is_range(cond) {
        let crit = super::get_range_string_values(cond, worksheet);
        return (0..n)
            .map(|i| crit.get(i).map(|c| truthy(c)).unwrap_or(false))
            .collect();
    }
    // Scalar criterion without an operator: expands to every row.
    if truthy(cond) {
        all()
    } else {
        none()
    }
}

pub fn evaluate_filter(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("FILTER(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[7..expr.len() - 1];
    let parts = split_args(inner);
    if parts.is_empty() {
        return Some("#ERROR!".to_string());
    }
    let range = parts[0].trim();
    let values = super::get_range_string_values(range, worksheet);
    let filtered: Vec<String> = if parts.len() >= 2 {
        let cond = parts[1].trim();
        if cond == "TRUE" || cond == "1" {
            values
        } else {
            let matches = filter_matches(&values, cond, worksheet);
            values
                .into_iter()
                .zip(matches)
                .filter(|(_, m)| *m)
                .map(|(v, _)| v)
                .collect()
        }
    } else {
        values
    };
    Some(filtered.join(","))
}

pub fn evaluate_sort(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("SORT(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[5..expr.len() - 1];
    let parts = split_args(inner);
    if parts.is_empty() {
        return Some("#ERROR!".to_string());
    }
    let mut values = super::get_range_string_values(parts[0].trim(), worksheet);
    let desc = parts.len() > 1 && parts[1].trim().contains("FALSE") || parts.len() > 1 && parts[1].to_uppercase().contains("DESC");
    let nums = parse_numbers(values.clone());
    if nums.len() == values.len() && !values.is_empty() {
        let mut paired: Vec<(f64, String)> = nums.into_iter().zip(values.drain(..)).collect();
        paired.sort_by(|a, b| if desc { b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal) } else { a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal) });
        Some(paired.into_iter().map(|(_, s)| s).collect::<Vec<_>>().join(","))
    } else {
        values.sort_by(|a, b| if desc { b.cmp(a) } else { a.cmp(b) });
        Some(values.join(","))
    }
}

pub fn evaluate_sortby(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("SORTBY(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[7..expr.len() - 1];
    let parts = split_args(inner);
    if parts.len() < 2 {
        return Some("#ERROR!".to_string());
    }
    let values = super::get_range_string_values(parts[0].trim(), worksheet);
    let keys = parse_numbers(super::get_range_string_values(parts[1].trim(), worksheet));
    if keys.len() != values.len() || values.is_empty() {
        return Some(values.join(","));
    }
    let mut paired: Vec<(f64, String)> = keys.into_iter().zip(values).collect();
    paired.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    Some(paired.into_iter().map(|(_, s)| s).collect::<Vec<_>>().join(","))
}

pub fn evaluate_unique(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("UNIQUE(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[7..expr.len() - 1];
    let parts = split_args(inner);
    if parts.is_empty() {
        return Some("#ERROR!".to_string());
    }
    let values = super::get_range_string_values(parts[0].trim(), worksheet);
    let mut seen = std::collections::HashSet::new();
    let unique: Vec<String> = values.into_iter().filter(|v| seen.insert(v.clone())).collect();
    Some(unique.join(","))
}

pub fn evaluate_sequence(expr: &str, _worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("SEQUENCE(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[9..expr.len() - 1];
    let parts = split_args(inner);
    if parts.is_empty() {
        return Some("#ERROR!".to_string());
    }
    let rows: i64 = parts[0].trim().parse().unwrap_or(1);
    let cols: i64 = if parts.len() > 1 { parts[1].trim().parse().unwrap_or(1) } else { 1 };
    let start: i64 = if parts.len() > 2 { parts[2].trim().parse().unwrap_or(1) } else { 1 };
    let step: i64 = if parts.len() > 3 { parts[3].trim().parse().unwrap_or(1) } else { 1 };
    let mut out: Vec<String> = Vec::new();
    let mut n = start;
    for _ in 0..(rows * cols) {
        out.push(n.to_string());
        n += step;
    }
    Some(out.join(","))
}

pub fn evaluate_randarray(expr: &str, _worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("RANDARRAY(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[10..expr.len() - 1];
    let parts = split_args(inner);
    let rows: i64 = parts.first().and_then(|s| s.trim().parse().ok()).unwrap_or(1);
    let cols: i64 = parts.get(1).and_then(|s| s.trim().parse().ok()).unwrap_or(1);
    let min: f64 = parts.get(2).and_then(|s| s.trim().parse().ok()).unwrap_or(0.0);
    let max: f64 = parts.get(3).and_then(|s| s.trim().parse().ok()).unwrap_or(1.0);
    let mut rng = rand::rng();
    let mut out: Vec<String> = Vec::new();
    for _ in 0..(rows * cols) {
        let v: f64 = min + (max - min) * rand::Rng::random::<f64>(&mut rng);
        out.push(format_number(v));
    }
    Some(out.join(","))
}

pub fn evaluate_tocol(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("TOCOL(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[6..expr.len() - 1];
    let parts = split_args(inner);
    if parts.is_empty() {
        return Some("#ERROR!".to_string());
    }
    Some(super::get_range_string_values(parts[0].trim(), worksheet).join(","))
}

pub fn evaluate_torow(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("TOROW(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[6..expr.len() - 1];
    let parts = split_args(inner);
    if parts.is_empty() {
        return Some("#ERROR!".to_string());
    }
    Some(super::get_range_string_values(parts[0].trim(), worksheet).join(","))
}

pub fn evaluate_wrapcols(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("WRAPCOLS(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[9..expr.len() - 1];
    let parts = split_args(inner);
    if parts.len() < 2 {
        return Some("#ERROR!".to_string());
    }
    let values = super::get_range_string_values(parts[0].trim(), worksheet);
    let width: usize = parts[1].trim().parse().unwrap_or(1);
    let mut out: Vec<String> = Vec::new();
    for chunk in values.chunks(width) {
        out.push(chunk.join("|"));
    }
    Some(out.join(","))
}

pub fn evaluate_wraprows(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("WRAPROWS(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[9..expr.len() - 1];
    let parts = split_args(inner);
    if parts.len() < 2 {
        return Some("#ERROR!".to_string());
    }
    let values = super::get_range_string_values(parts[0].trim(), worksheet);
    let height: usize = parts[1].trim().parse().unwrap_or(1);
    let mut out: Vec<String> = Vec::new();
    for chunk in values.chunks(height) {
        out.push(chunk.join("|"));
    }
    Some(out.join(","))
}

pub fn evaluate_hstack(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("HSTACK(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[7..expr.len() - 1];
    let parts = split_args(inner);
    let mut all: Vec<String> = Vec::new();
    for p in &parts {
        all.extend(super::get_range_string_values(p.trim(), worksheet));
    }
    Some(all.join(","))
}

pub fn evaluate_vstack(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("VSTACK(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[7..expr.len() - 1];
    let parts = split_args(inner);
    let mut all: Vec<String> = Vec::new();
    for p in &parts {
        all.extend(super::get_range_string_values(p.trim(), worksheet));
    }
    Some(all.join("|"))
}

pub fn evaluate_chooserows(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("CHOOSEROWS(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[11..expr.len() - 1];
    let parts = split_args(inner);
    if parts.len() < 2 {
        return Some("#ERROR!".to_string());
    }
    let values = super::get_range_string_values(parts[0].trim(), worksheet);
    let indices: Vec<i64> = parts[1..].iter().filter_map(|p| p.trim().parse().ok()).collect();
    let selected: Vec<String> = indices.iter().filter_map(|&i| values.get(i.max(0) as usize).cloned()).collect();
    Some(selected.join(","))
}

pub fn evaluate_choosecols(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("CHOOSECOLS(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[11..expr.len() - 1];
    let parts = split_args(inner);
    if parts.len() < 2 {
        return Some("#ERROR!".to_string());
    }
    let values = super::get_range_string_values(parts[0].trim(), worksheet);
    let indices: Vec<i64> = parts[1..].iter().filter_map(|p| p.trim().parse().ok()).collect();
    let selected: Vec<String> = indices.iter().filter_map(|&i| values.get(i.max(0) as usize).cloned()).collect();
    Some(selected.join(","))
}

pub fn evaluate_take(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("TAKE(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[5..expr.len() - 1];
    let parts = split_args(inner);
    if parts.len() < 2 {
        return Some("#ERROR!".to_string());
    }
    let values = super::get_range_string_values(parts[0].trim(), worksheet);
    let n: i64 = parts[1].trim().parse().unwrap_or(0);
    let take_n = if n < 0 { values.len() as i64 + n } else { n } as usize;
    Some(values.iter().take(take_n).cloned().collect::<Vec<_>>().join(","))
}

pub fn evaluate_drop(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("DROP(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[5..expr.len() - 1];
    let parts = split_args(inner);
    if parts.len() < 2 {
        return Some("#ERROR!".to_string());
    }
    let values = super::get_range_string_values(parts[0].trim(), worksheet);
    let n: usize = parts[1].trim().parse().unwrap_or(0);
    Some(values.iter().skip(n).cloned().collect::<Vec<_>>().join(","))
}

pub fn evaluate_expand(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("EXPAND(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[7..expr.len() - 1];
    let parts = split_args(inner);
    if parts.len() < 3 {
        return Some("#ERROR!".to_string());
    }
    let values = super::get_range_string_values(parts[0].trim(), worksheet);
    let rows: usize = parts[1].trim().parse().unwrap_or(values.len());
    let pad = if parts.len() > 3 { parts[3].trim().trim_matches('"').to_string() } else { "0".to_string() };
    let mut out = values;
    while out.len() < rows {
        out.push(pad.clone());
    }
    Some(out.join(","))
}

pub fn evaluate_trimrange(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("TRIMRANGE(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[10..expr.len() - 1];
    let parts = split_args(inner);
    if parts.is_empty() {
        return Some("#ERROR!".to_string());
    }
    let values = super::get_range_string_values(parts[0].trim(), worksheet);
    let trimmed: Vec<String> = values.into_iter().filter(|v| !v.is_empty()).collect();
    Some(trimmed.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CellData, Worksheet};
    use std::collections::HashMap;

    fn ws_with(values: &[(&str, &str)]) -> Worksheet {
        let mut data = HashMap::new();
        for (key, val) in values {
            data.insert(
                key.to_string(),
                CellData {
                    value: Some(val.to_string()),
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
        Worksheet {
            data,
            ..Worksheet::default()
        }
    }

    fn ev(expr: &str, ws: &Worksheet) -> Option<String> {
        evaluate_filter(expr, ws)
    }

    #[test]
    fn true_criterion_keeps_all() {
        let ws = ws_with(&[("0,0", "1"), ("1,0", "2"), ("2,0", "3")]);
        assert_eq!(ev("FILTER(A1:A3,TRUE)", &ws), Some("1,2,3".to_string()));
    }

    #[test]
    fn comparison_criterion_filters_rows() {
        let ws = ws_with(&[("0,0", "1"), ("1,0", "10"), ("2,0", "3")]);
        assert_eq!(ev("FILTER(A1:A3,A1:A3>2)", &ws), Some("10,3".to_string()));
    }

    #[test]
    fn criterion_range_uses_truthiness() {
        let ws = ws_with(&[("0,0", "a"), ("1,0", "b"), ("2,0", "c"), ("0,1", "TRUE"), ("1,1", "FALSE"), ("2,1", "TRUE")]);
        assert_eq!(ev("FILTER(A1:A3,B1:B3)", &ws), Some("a,c".to_string()));
    }

    #[test]
    fn scalar_criterion_expands() {
        let ws = ws_with(&[("0,0", "1"), ("1,0", "2")]);
        assert_eq!(ev("FILTER(A1:A2,\"x\")", &ws), Some("1,2".to_string()));
    }

    #[test]
    fn not_equals_operator() {
        let ws = ws_with(&[("0,0", "a"), ("1,0", "b"), ("2,0", "a")]);
        assert_eq!(ev("FILTER(A1:A3,A1:A3<>\"a\")", &ws), Some("b".to_string()));
    }
}
