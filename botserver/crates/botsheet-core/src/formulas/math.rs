use crate::types::Worksheet;

use super::{format_number, resolve_cell_value, split_args};

pub fn evaluate_sum(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("SUM(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[4..expr.len() - 1];
    let values = super::get_range_values(inner, worksheet);
    let sum: f64 = values.iter().sum();
    Some(format_number(sum))
}

pub fn evaluate_average(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("AVERAGE(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[8..expr.len() - 1];
    let values = super::get_range_values(inner, worksheet);
    if values.is_empty() {
        return Some("#DIV/0!".to_string());
    }
    let avg = values.iter().sum::<f64>() / values.len() as f64;
    Some(format_number(avg))
}

pub fn evaluate_count(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("COUNT(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[6..expr.len() - 1];
    let values = super::get_range_values(inner, worksheet);
    Some(values.len().to_string())
}

pub fn evaluate_max(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("MAX(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[4..expr.len() - 1];
    let values = super::get_range_values(inner, worksheet);
    values
        .iter()
        .cloned()
        .fold(None, |max, v| match max {
            None => Some(v),
            Some(m) => Some(if v > m { v } else { m }),
        })
        .map(format_number)
}

pub fn evaluate_min(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("MIN(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[4..expr.len() - 1];
    let values = super::get_range_values(inner, worksheet);
    values
        .iter()
        .cloned()
        .fold(None, |min, v| match min {
            None => Some(v),
            Some(m) => Some(if v < m { v } else { m }),
        })
        .map(format_number)
}

pub fn evaluate_round(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("ROUND(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[6..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    let num: f64 = resolve_cell_value(parts[0].trim(), worksheet)
        .parse()
        .ok()?;
    let decimals: i32 = if parts.len() > 1 {
        parts[1].trim().parse().unwrap_or(0)
    } else {
        0
    };
    let factor = 10_f64.powi(decimals);
    Some(format_number((num * factor).round() / factor))
}

pub fn evaluate_roundup(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("ROUNDUP(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[8..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    let num: f64 = resolve_cell_value(parts[0].trim(), worksheet)
        .parse()
        .ok()?;
    let decimals: i32 = if parts.len() > 1 {
        parts[1].trim().parse().unwrap_or(0)
    } else {
        0
    };
    let factor = 10_f64.powi(decimals);
    Some(format_number((num * factor).ceil() / factor))
}

pub fn evaluate_rounddown(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("ROUNDDOWN(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[10..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    let num: f64 = resolve_cell_value(parts[0].trim(), worksheet)
        .parse()
        .ok()?;
    let decimals: i32 = if parts.len() > 1 {
        parts[1].trim().parse().unwrap_or(0)
    } else {
        0
    };
    let factor = 10_f64.powi(decimals);
    Some(format_number((num * factor).floor() / factor))
}

pub fn evaluate_abs(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("ABS(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[4..expr.len() - 1];
    let num: f64 = resolve_cell_value(inner.trim(), worksheet)
        .parse()
        .ok()?;
    Some(format_number(num.abs()))
}

pub fn evaluate_sqrt(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("SQRT(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[5..expr.len() - 1];
    let num: f64 = resolve_cell_value(inner.trim(), worksheet)
        .parse()
        .ok()?;
    if num < 0.0 {
        return Some("#NUM!".to_string());
    }
    Some(format_number(num.sqrt()))
}

pub fn evaluate_power(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("POWER(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[6..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    if parts.len() != 2 {
        return None;
    }
    let base: f64 = resolve_cell_value(parts[0].trim(), worksheet)
        .parse()
        .ok()?;
    let exp: f64 = resolve_cell_value(parts[1].trim(), worksheet)
        .parse()
        .ok()?;
    Some(format_number(base.powf(exp)))
}

pub fn evaluate_mod_formula(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("MOD(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[4..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    if parts.len() != 2 {
        return None;
    }
    let number: f64 = resolve_cell_value(parts[0].trim(), worksheet)
        .parse()
        .ok()?;
    let divisor: f64 = resolve_cell_value(parts[1].trim(), worksheet)
        .parse()
        .ok()?;
    if divisor == 0.0 {
        return Some("#DIV/0!".to_string());
    }
    Some(format_number(number % divisor))
}

pub fn evaluate_arithmetic(expr: &str, worksheet: &Worksheet) -> Option<String> {
    let resolved = super::resolve_cell_references(expr, worksheet);
    eval_simple_arithmetic(&resolved).map(format_number)
}

fn eval_simple_arithmetic(expr: &str) -> Option<f64> {
    let expr = expr.replace(' ', "");
    if let Ok(num) = expr.parse::<f64>() {
        return Some(num);
    }
    if let Some(pos) = expr.rfind('+') {
        if pos > 0 {
            let left = eval_simple_arithmetic(&expr[..pos])?;
            let right = eval_simple_arithmetic(&expr[pos + 1..])?;
            return Some(left + right);
        }
    }
    if let Some(pos) = expr.rfind('-') {
        if pos > 0 {
            let left = eval_simple_arithmetic(&expr[..pos])?;
            let right = eval_simple_arithmetic(&expr[pos + 1..])?;
            return Some(left - right);
        }
    }
    if let Some(pos) = expr.rfind('*') {
        let left = eval_simple_arithmetic(&expr[..pos])?;
        let right = eval_simple_arithmetic(&expr[pos + 1..])?;
        return Some(left * right);
    }
    if let Some(pos) = expr.rfind('/') {
        let left = eval_simple_arithmetic(&expr[..pos])?;
        let right = eval_simple_arithmetic(&expr[pos + 1..])?;
        if right != 0.0 {
            return Some(left / right);
        }
    }
    None
}
