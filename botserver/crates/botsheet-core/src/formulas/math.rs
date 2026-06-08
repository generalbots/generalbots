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


pub fn evaluate_product(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("PRODUCT(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[8..expr.len() - 1];
    let values = super::get_range_values(inner, worksheet);
    if values.is_empty() {
        return Some("0".to_string());
    }
    let product: f64 = values.iter().product();
    Some(format_number(product))
}

pub fn evaluate_stdev(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("STDEV(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[6..expr.len() - 1];
    let values = super::get_range_values(inner, worksheet);
    if values.len() < 2 {
        return Some("#DIV/0!".to_string());
    }
    let mean: f64 = values.iter().sum::<f64>() / values.len() as f64;
    let variance: f64 = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (values.len() - 1) as f64;
    Some(format_number(variance.sqrt()))
}

pub fn evaluate_stdevp(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("STDEVP(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[7..expr.len() - 1];
    let values = super::get_range_values(inner, worksheet);
    if values.is_empty() {
        return Some("#DIV/0!".to_string());
    }
    let mean: f64 = values.iter().sum::<f64>() / values.len() as f64;
    let variance: f64 = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
    Some(format_number(variance.sqrt()))
}

pub fn evaluate_median(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("MEDIAN(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[7..expr.len() - 1];
    let mut values = super::get_range_values(inner, worksheet);
    if values.is_empty() {
        return Some("#NUM!".to_string());
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = values.len();
    let median = if n % 2 == 0 {
        (values[n / 2 - 1] + values[n / 2]) / 2.0
    } else {
        values[n / 2]
    };
    Some(format_number(median))
}

pub fn evaluate_ceiling(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("CEILING(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[8..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    let num: f64 = resolve_cell_value(parts[0].trim(), worksheet).parse().ok()?;
    let sig: f64 = if parts.len() > 1 {
        parts[1].trim().parse().unwrap_or(1.0)
    } else {
        1.0
    };
    if sig == 0.0 { return Some("0".to_string()); }
    Some(format_number((num / sig).ceil() * sig))
}

pub fn evaluate_floor(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("FLOOR(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[6..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    let num: f64 = resolve_cell_value(parts[0].trim(), worksheet).parse().ok()?;
    let sig: f64 = if parts.len() > 1 {
        parts[1].trim().parse().unwrap_or(1.0)
    } else {
        1.0
    };
    if sig == 0.0 { return Some("0".to_string()); }
    Some(format_number((num / sig).floor() * sig))
}

pub fn evaluate_int(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("INT(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[4..expr.len() - 1];
    let num: f64 = resolve_cell_value(inner.trim(), worksheet).parse().ok()?;
    Some(format_number(num.floor()))
}

pub fn evaluate_exp(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("EXP(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[4..expr.len() - 1];
    let num: f64 = resolve_cell_value(inner.trim(), worksheet).parse().ok()?;
    Some(format_number(num.exp()))
}

pub fn evaluate_ln(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("LN(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[3..expr.len() - 1];
    let num: f64 = resolve_cell_value(inner.trim(), worksheet).parse().ok()?;
    if num <= 0.0 { return Some("#NUM!".to_string()); }
    Some(format_number(num.ln()))
}

pub fn evaluate_log(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("LOG(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[4..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    let num: f64 = resolve_cell_value(parts[0].trim(), worksheet).parse().ok()?;
    let base: f64 = if parts.len() > 1 {
        parts[1].trim().parse().unwrap_or(10.0)
    } else {
        10.0
    };
    if num <= 0.0 || base <= 0.0 || base == 1.0 { return Some("#NUM!".to_string()); }
    Some(format_number(num.log(base)))
}

pub fn evaluate_log10(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("LOG10(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[6..expr.len() - 1];
    let num: f64 = resolve_cell_value(inner.trim(), worksheet).parse().ok()?;
    if num <= 0.0 { return Some("#NUM!".to_string()); }
    Some(format_number(num.log10()))
}

pub fn evaluate_sign(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("SIGN(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[5..expr.len() - 1];
    let num: f64 = resolve_cell_value(inner.trim(), worksheet).parse().ok()?;
    let s = if num > 0.0 { 1.0 } else if num < 0.0 { -1.0 } else { 0.0 };
    Some(format_number(s))
}

pub fn evaluate_pi(_expr: &str, _worksheet: &Worksheet) -> Option<String> {
    if !_expr.starts_with("PI(") || !_expr.ends_with(')') {
        return None;
    }
    Some(format_number(std::f64::consts::PI))
}

pub fn evaluate_rand(_expr: &str, _worksheet: &Worksheet) -> Option<String> {
    if !_expr.starts_with("RAND(") || !_expr.ends_with(')') {
        return None;
    }
    Some(format_number(rand_simple()))
}

pub fn evaluate_randbetween(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("RANDBETWEEN(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[12..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    let lo: i64 = resolve_cell_value(parts[0].trim(), worksheet).parse().ok()?;
    let hi: i64 = resolve_cell_value(parts[1].trim(), worksheet).parse().ok()?;
    if lo > hi { return Some("#NUM!".to_string()); }
    let n = rand_simple() * (hi - lo + 1) as f64;
    Some(((lo as f64) + n.floor()).to_string())
}

fn rand_simple() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.subsec_nanos()).unwrap_or(0);
    let mixed = (nanos ^ 0x9E3779B9) as u64;
    (mixed % 1_000_000) as f64 / 1_000_000.0
}


