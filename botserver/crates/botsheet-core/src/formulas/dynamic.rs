use crate::types::Worksheet;

use super::helpers::{format_number, resolve_cell_value, split_args};

pub fn evaluate_let(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("LET(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[4..expr.len() - 1];
    let parts = split_args(inner);
    if parts.len() < 3 || parts.len().is_multiple_of(2) {
        return Some("#ERROR!".to_string());
    }
    let mut bindings: Vec<(String, String)> = Vec::new();
    for chunk in parts.chunks_exact(2) {
        let name = chunk[0].trim().to_string();
        let val_expr = chunk[1].trim();
        let val = if val_expr.starts_with('=') {
            super::evaluate_formula(val_expr, worksheet).value
        } else {
            resolve_cell_value(val_expr, worksheet)
        };
        bindings.push((name, val));
    }
    let calc = parts[parts.len() - 1].trim();
    let substituted = apply_bindings(calc, &bindings);
    if substituted.starts_with('=') {
        Some(super::evaluate_formula(&substituted, worksheet).value)
    } else {
        Some(resolve_cell_value(&substituted, worksheet))
    }
}

pub fn evaluate_lambda(expr: &str, _worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("LAMBDA(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[7..expr.len() - 1];
    let parts = split_args(inner);
    if parts.len() < 2 {
        return Some("#ERROR!".to_string());
    }
    Some(format!("λ({})", parts[0].trim()))
}

pub fn evaluate_map(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("MAP(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[4..expr.len() - 1];
    let parts = split_args(inner);
    if parts.len() < 2 {
        return Some("#ERROR!".to_string());
    }
    let lambda = parts.last().unwrap().trim();
    let mut values: Vec<String> = Vec::new();
    for p in &parts[..parts.len() - 1] {
        values.extend(super::get_range_string_values(p.trim(), worksheet));
    }
    let mut out: Vec<String> = Vec::new();
    for v in &values {
        let mut bindings: Vec<(String, String)> = Vec::new();
        if lambda.starts_with("LAMBDA(") {
            let l_inner = &lambda[7..lambda.len() - 1];
            let l_parts = split_args(l_inner);
            if !l_parts.is_empty() {
                let param_names = l_parts[0].trim();
                let calc = l_parts[1..].join(",");
                for name in param_names.split(',') {
                    bindings.push((name.trim().to_string(), v.clone()));
                }
                let sub = apply_bindings(&calc, &bindings);
                out.push(if sub.starts_with('=') {
                    super::evaluate_formula(&sub, worksheet).value
                } else {
                    resolve_cell_value(&sub, worksheet)
                });
            }
        } else {
            out.push(resolve_cell_value(lambda, worksheet));
        }
    }
    Some(out.join(","))
}

pub fn evaluate_reduce(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("REDUCE(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[7..expr.len() - 1];
    let parts = split_args(inner);
    if parts.len() < 3 {
        return Some("#ERROR!".to_string());
    }
    let initial = resolve_cell_value(parts[0].trim(), worksheet);
    let values = super::get_range_string_values(parts[1].trim(), worksheet);
    let lambda = parts[2].trim();
    let mut acc = initial;
    for v in values {
        let sub = lambda
            .replace("[accumulator]", &acc)
            .replace("[value]", &v)
            .replace("[acc]", &acc)
            .replace("[val]", &v);
        acc = if sub.starts_with('=') {
            super::evaluate_formula(&sub, worksheet).value
        } else {
            resolve_cell_value(&sub, worksheet)
        };
    }
    Some(acc)
}

pub fn evaluate_byrow(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("BYROW(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[6..expr.len() - 1];
    let parts = split_args(inner);
    if parts.len() < 2 {
        return Some("#ERROR!".to_string());
    }
    let range = parts[0].trim();
    let lambda = parts[1].trim();
    let rows = collect_rows(range, worksheet);
    let mut out: Vec<String> = Vec::new();
    for row in rows {
        let sub = lambda.replace("[row]", &row.join(","));
        let val = if sub.starts_with('=') {
            super::evaluate_formula(&sub, worksheet).value
        } else {
            resolve_cell_value(&sub, worksheet)
        };
        out.push(val);
    }
    Some(out.join(","))
}

pub fn evaluate_bycol(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("BYCOL(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[6..expr.len() - 1];
    let parts = split_args(inner);
    if parts.len() < 2 {
        return Some("#ERROR!".to_string());
    }
    let range = parts[0].trim();
    let lambda = parts[1].trim();
    let cols = collect_cols(range, worksheet);
    let mut out: Vec<String> = Vec::new();
    for col in cols {
        let sub = lambda.replace("[col]", &col.join(","));
        let val = if sub.starts_with('=') {
            super::evaluate_formula(&sub, worksheet).value
        } else {
            resolve_cell_value(&sub, worksheet)
        };
        out.push(val);
    }
    Some(out.join(","))
}

pub fn evaluate_makearray(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("MAKEARRAY(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[10..expr.len() - 1];
    let parts = split_args(inner);
    if parts.len() < 3 {
        return Some("#ERROR!".to_string());
    }
    let rows: i64 = parts[0].trim().parse().unwrap_or(0);
    let cols: i64 = parts[1].trim().parse().unwrap_or(0);
    let lambda = parts[2].trim();
    let mut out: Vec<String> = Vec::new();
    for r in 0..rows {
        for c in 0..cols {
            let sub = lambda.replace("[row]", &r.to_string()).replace("[col]", &c.to_string());
            let v = if sub.starts_with('=') {
                super::evaluate_formula(&sub, worksheet).value
            } else {
                resolve_cell_value(&sub, worksheet)
            };
            out.push(v);
        }
    }
    Some(out.join(","))
}

fn apply_bindings(calc: &str, bindings: &[(String, String)]) -> String {
    let mut out = calc.to_string();
    for (name, val) in bindings {
        out = out.replace(name, val);
    }
    out
}

fn collect_rows(range: &str, worksheet: &Worksheet) -> Vec<Vec<String>> {
    if let Some((start, end)) = super::parse_range(range).map(|(s, e)| super::refs::clamp_range(s, e, worksheet)) {
        let mut out = Vec::new();
        for r in start.0..=end.0 {
            let mut row = Vec::new();
            for c in start.1..=end.1 {
                let key = format!("{},{}", r, c);
                if let Some(cell) = worksheet.data.get(&key) {
                    row.push(cell.value.clone().unwrap_or_default());
                } else {
                    row.push(String::new());
                }
            }
            out.push(row);
        }
        out
    } else {
        Vec::new()
    }
}

fn collect_cols(range: &str, worksheet: &Worksheet) -> Vec<Vec<String>> {
    if let Some((start, end)) = super::parse_range(range).map(|(s, e)| super::refs::clamp_range(s, e, worksheet)) {
        let mut out = Vec::new();
        for c in start.1..=end.1 {
            let mut col = Vec::new();
            for r in start.0..=end.0 {
                let key = format!("{},{}", r, c);
                if let Some(cell) = worksheet.data.get(&key) {
                    col.push(cell.value.clone().unwrap_or_default());
                } else {
                    col.push(String::new());
                }
            }
            out.push(col);
        }
        out
    } else {
        Vec::new()
    }
}

pub fn evaluate_reduce_arithmetic(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("REDUCE_OP(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[11..expr.len() - 1];
    let parts = split_args(inner);
    if parts.len() < 2 {
        return None;
    }
    let op = parts[0].trim().trim_matches('"');
    let values = super::get_range_values(parts[1].trim(), worksheet);
    let acc = if parts.len() > 2 {
        parts[2].trim().parse().unwrap_or(0.0)
    } else {
        0.0
    };
    let result = match op.to_uppercase().as_str() {
        "SUM" | "+" => values.iter().fold(acc, |a, b| a + b),
        "PRODUCT" | "*" => values.iter().fold(if acc == 0.0 { 1.0 } else { acc }, |a, b| a * b),
        "MAX" => values.iter().fold(acc, |a, b| a.max(*b)),
        "MIN" => values.iter().fold(acc, |a, b| a.min(*b)),
        "AND" => values.iter().fold(if acc == 0.0 { 1.0 } else { acc }, |a, b| if *b != 0.0 && a != 0.0 { 1.0 } else { 0.0 }),
        "OR" => values.iter().fold(acc, |a, b| if *b != 0.0 || a != 0.0 { 1.0 } else { 0.0 }),
        _ => return Some("#ERROR!".to_string()),
    };
    Some(format_number(result))
}
