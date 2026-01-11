use crate::sheet::types::{FormulaResult, Worksheet};
use chrono::{Datelike, Local, NaiveDate};

pub fn evaluate_formula(formula: &str, worksheet: &Worksheet) -> FormulaResult {
    if !formula.starts_with('=') {
        return FormulaResult {
            value: formula.to_string(),
            error: None,
        };
    }

    let expr = formula[1..].to_uppercase();

    let evaluators: Vec<fn(&str, &Worksheet) -> Option<String>> = vec![
        evaluate_sum,
        evaluate_average,
        evaluate_count,
        evaluate_counta,
        evaluate_countblank,
        evaluate_countif,
        evaluate_sumif,
        evaluate_averageif,
        evaluate_max,
        evaluate_min,
        evaluate_if,
        evaluate_iferror,
        evaluate_vlookup,
        evaluate_hlookup,
        evaluate_index_match,
        evaluate_concatenate,
        evaluate_left,
        evaluate_right,
        evaluate_mid,
        evaluate_len,
        evaluate_trim,
        evaluate_upper,
        evaluate_lower,
        evaluate_proper,
        evaluate_substitute,
        evaluate_round,
        evaluate_roundup,
        evaluate_rounddown,
        evaluate_abs,
        evaluate_sqrt,
        evaluate_power,
        evaluate_mod_formula,
        evaluate_and,
        evaluate_or,
        evaluate_not,
        evaluate_today,
        evaluate_now,
        evaluate_date,
        evaluate_year,
        evaluate_month,
        evaluate_day,
        evaluate_datedif,
        evaluate_arithmetic,
    ];

    for evaluator in evaluators {
        if let Some(result) = evaluator(&expr, worksheet) {
            return FormulaResult {
                value: result,
                error: None,
            };
        }
    }

    FormulaResult {
        value: "#ERROR!".to_string(),
        error: Some("Invalid formula".to_string()),
    }
}

fn evaluate_sum(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("SUM(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[4..expr.len() - 1];
    let values = get_range_values(inner, worksheet);
    let sum: f64 = values.iter().sum();
    Some(format_number(sum))
}

fn evaluate_average(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("AVERAGE(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[8..expr.len() - 1];
    let values = get_range_values(inner, worksheet);
    if values.is_empty() {
        return Some("#DIV/0!".to_string());
    }
    let avg = values.iter().sum::<f64>() / values.len() as f64;
    Some(format_number(avg))
}

fn evaluate_count(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("COUNT(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[6..expr.len() - 1];
    let values = get_range_values(inner, worksheet);
    Some(values.len().to_string())
}

fn evaluate_counta(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("COUNTA(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[7..expr.len() - 1];
    let count = get_range_string_values(inner, worksheet)
        .iter()
        .filter(|v| !v.is_empty())
        .count();
    Some(count.to_string())
}

fn evaluate_countblank(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("COUNTBLANK(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[11..expr.len() - 1];
    let (start, end) = parse_range(inner)?;
    let mut count = 0;
    for row in start.0..=end.0 {
        for col in start.1..=end.1 {
            let key = format!("{},{}", row, col);
            let is_blank = worksheet
                .data
                .get(&key)
                .and_then(|c| c.value.as_ref())
                .map(|v| v.is_empty())
                .unwrap_or(true);
            if is_blank {
                count += 1;
            }
        }
    }
    Some(count.to_string())
}

fn evaluate_countif(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("COUNTIF(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[8..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    if parts.len() != 2 {
        return None;
    }
    let range = parts[0].trim();
    let criteria = parts[1].trim().trim_matches('"');
    let values = get_range_string_values(range, worksheet);
    let count = count_matching(&values, criteria);
    Some(count.to_string())
}

fn evaluate_sumif(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("SUMIF(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[6..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    if parts.len() < 2 {
        return None;
    }
    let criteria_range = parts[0].trim();
    let criteria = parts[1].trim().trim_matches('"');
    let sum_range = if parts.len() > 2 {
        parts[2].trim()
    } else {
        criteria_range
    };

    let criteria_values = get_range_string_values(criteria_range, worksheet);
    let sum_values = get_range_values(sum_range, worksheet);

    let mut sum = 0.0;
    for (i, cv) in criteria_values.iter().enumerate() {
        if matches_criteria(cv, criteria) {
            if let Some(sv) = sum_values.get(i) {
                sum += sv;
            }
        }
    }
    Some(format_number(sum))
}

fn evaluate_averageif(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("AVERAGEIF(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[10..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    if parts.len() < 2 {
        return None;
    }
    let criteria_range = parts[0].trim();
    let criteria = parts[1].trim().trim_matches('"');
    let avg_range = if parts.len() > 2 {
        parts[2].trim()
    } else {
        criteria_range
    };

    let criteria_values = get_range_string_values(criteria_range, worksheet);
    let avg_values = get_range_values(avg_range, worksheet);

    let mut sum = 0.0;
    let mut count = 0;
    for (i, cv) in criteria_values.iter().enumerate() {
        if matches_criteria(cv, criteria) {
            if let Some(av) = avg_values.get(i) {
                sum += av;
                count += 1;
            }
        }
    }
    if count == 0 {
        return Some("#DIV/0!".to_string());
    }
    Some(format_number(sum / count as f64))
}

fn evaluate_max(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("MAX(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[4..expr.len() - 1];
    let values = get_range_values(inner, worksheet);
    values
        .iter()
        .cloned()
        .fold(None, |max, v| match max {
            None => Some(v),
            Some(m) => Some(if v > m { v } else { m }),
        })
        .map(format_number)
}

fn evaluate_min(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("MIN(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[4..expr.len() - 1];
    let values = get_range_values(inner, worksheet);
    values
        .iter()
        .cloned()
        .fold(None, |min, v| match min {
            None => Some(v),
            Some(m) => Some(if v < m { v } else { m }),
        })
        .map(format_number)
}

fn evaluate_if(expr: &str, worksheet: &Worksheet) -> Option<String> {
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

fn evaluate_iferror(expr: &str, worksheet: &Worksheet) -> Option<String> {
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

    let result = evaluate_formula(&format!("={}", value_expr), worksheet);
    if result.error.is_some() || result.value.starts_with('#') {
        Some(error_value.to_string())
    } else {
        Some(result.value)
    }
}

fn evaluate_vlookup(expr: &str, worksheet: &Worksheet) -> Option<String> {
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
    let col_index: usize = parts[2].trim().parse().ok()?;

    let (start, end) = parse_range(table_range)?;
    for row in start.0..=end.0 {
        let key = format!("{},{}", row, start.1);
        let cell_value = worksheet
            .data
            .get(&key)
            .and_then(|c| c.value.clone())
            .unwrap_or_default();
        if cell_value.eq_ignore_ascii_case(lookup_value) {
            let result_col = start.1 + col_index as u32 - 1;
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

fn evaluate_hlookup(expr: &str, worksheet: &Worksheet) -> Option<String> {
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
    let row_index: usize = parts[2].trim().parse().ok()?;

    let (start, end) = parse_range(table_range)?;
    for col in start.1..=end.1 {
        let key = format!("{},{}", start.0, col);
        let cell_value = worksheet
            .data
            .get(&key)
            .and_then(|c| c.value.clone())
            .unwrap_or_default();
        if cell_value.eq_ignore_ascii_case(lookup_value) {
            let result_row = start.0 + row_index as u32 - 1;
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

fn evaluate_index_match(expr: &str, worksheet: &Worksheet) -> Option<String> {
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
    let target_row = start.0 + row_num - 1;
    let target_col = start.1 + col_num - 1;
    let key = format!("{},{}", target_row, target_col);
    Some(
        worksheet
            .data
            .get(&key)
            .and_then(|c| c.value.clone())
            .unwrap_or_default(),
    )
}

fn evaluate_concatenate(expr: &str, worksheet: &Worksheet) -> Option<String> {
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

fn evaluate_left(expr: &str, worksheet: &Worksheet) -> Option<String> {
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

fn evaluate_right(expr: &str, worksheet: &Worksheet) -> Option<String> {
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

fn evaluate_mid(expr: &str, worksheet: &Worksheet) -> Option<String> {
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

fn evaluate_len(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("LEN(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[4..expr.len() - 1];
    let text = resolve_cell_value(inner.trim().trim_matches('"'), worksheet);
    Some(text.chars().count().to_string())
}

fn evaluate_trim(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("TRIM(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[5..expr.len() - 1];
    let text = resolve_cell_value(inner.trim().trim_matches('"'), worksheet);
    Some(text.split_whitespace().collect::<Vec<_>>().join(" "))
}

fn evaluate_upper(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("UPPER(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[6..expr.len() - 1];
    let text = resolve_cell_value(inner.trim().trim_matches('"'), worksheet);
    Some(text.to_uppercase())
}

fn evaluate_lower(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("LOWER(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[6..expr.len() - 1];
    let text = resolve_cell_value(inner.trim().trim_matches('"'), worksheet);
    Some(text.to_lowercase())
}

fn evaluate_proper(expr: &str, worksheet: &Worksheet) -> Option<String> {
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

fn evaluate_substitute(expr: &str, worksheet: &Worksheet) -> Option<String> {
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

fn evaluate_round(expr: &str, worksheet: &Worksheet) -> Option<String> {
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

fn evaluate_roundup(expr: &str, worksheet: &Worksheet) -> Option<String> {
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

fn evaluate_rounddown(expr: &str, worksheet: &Worksheet) -> Option<String> {
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

fn evaluate_abs(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("ABS(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[4..expr.len() - 1];
    let num: f64 = resolve_cell_value(inner.trim(), worksheet).parse().ok()?;
    Some(format_number(num.abs()))
}

fn evaluate_sqrt(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("SQRT(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[5..expr.len() - 1];
    let num: f64 = resolve_cell_value(inner.trim(), worksheet).parse().ok()?;
    if num < 0.0 {
        return Some("#NUM!".to_string());
    }
    Some(format_number(num.sqrt()))
}

fn evaluate_power(expr: &str, worksheet: &Worksheet) -> Option<String> {
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

fn evaluate_mod_formula(expr: &str, worksheet: &Worksheet) -> Option<String> {
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

fn evaluate_and(expr: &str, worksheet: &Worksheet) -> Option<String> {
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

fn evaluate_or(expr: &str, worksheet: &Worksheet) -> Option<String> {
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

fn evaluate_not(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("NOT(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[4..expr.len() - 1];
    let result = !evaluate_condition(inner.trim(), worksheet);
    Some(result.to_string().to_uppercase())
}

fn evaluate_today(_expr: &str, _worksheet: &Worksheet) -> Option<String> {
    if _expr != "TODAY()" {
        return None;
    }
    let today = Local::now().date_naive();
    Some(today.format("%Y-%m-%d").to_string())
}

fn evaluate_now(_expr: &str, _worksheet: &Worksheet) -> Option<String> {
    if _expr != "NOW()" {
        return None;
    }
    let now = Local::now();
    Some(now.format("%Y-%m-%d %H:%M:%S").to_string())
}

fn evaluate_date(expr: &str, _worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("DATE(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[5..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    if parts.len() != 3 {
        return None;
    }
    let year: i32 = parts[0].trim().parse().ok()?;
    let month: u32 = parts[1].trim().parse().ok()?;
    let day: u32 = parts[2].trim().parse().ok()?;
    let date = NaiveDate::from_ymd_opt(year, month, day)?;
    Some(date.format("%Y-%m-%d").to_string())
}

fn evaluate_year(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("YEAR(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[5..expr.len() - 1];
    let date_str = resolve_cell_value(inner.trim().trim_matches('"'), worksheet);
    let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").ok()?;
    Some(date.year().to_string())
}

fn evaluate_month(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("MONTH(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[6..expr.len() - 1];
    let date_str = resolve_cell_value(inner.trim().trim_matches('"'), worksheet);
    let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").ok()?;
    Some(date.month().to_string())
}

fn evaluate_day(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("DAY(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[4..expr.len() - 1];
    let date_str = resolve_cell_value(inner.trim().trim_matches('"'), worksheet);
    let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").ok()?;
    Some(date.day().to_string())
}

fn evaluate_datedif(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("DATEDIF(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[8..expr.len() - 1];
    let parts: Vec<&str> = split_args(inner);
    if parts.len() != 3 {
        return None;
    }
    let start_str = resolve_cell_value(parts[0].trim().trim_matches('"'), worksheet);
    let end_str = resolve_cell_value(parts[1].trim().trim_matches('"'), worksheet);
    let unit = parts[2].trim().trim_matches('"').to_uppercase();

    let start_date = NaiveDate::parse_from_str(&start_str, "%Y-%m-%d").ok()?;
    let end_date = NaiveDate::parse_from_str(&end_str, "%Y-%m-%d").ok()?;

    let diff = end_date.signed_duration_since(start_date);
    let result = match unit.as_str() {
        "D" => diff.num_days(),
        "M" => {
            let months = (end_date.year() - start_date.year()) * 12
                + end_date.month() as i32
                - start_date.month() as i32;
            i64::from(months)
        }
        "Y" => i64::from(end_date.year() - start_date.year()),
        _ => return Some("#VALUE!".to_string()),
    };
    Some(result.to_string())
}

fn evaluate_arithmetic(expr: &str, worksheet: &Worksheet) -> Option<String> {
    let resolved = resolve_cell_references(expr, worksheet);
    eval_simple_arithmetic(&resolved).map(format_number)
}

pub fn resolve_cell_references(expr: &str, worksheet: &Worksheet) -> String {
    let mut result = expr.to_string();
    let re = regex::Regex::new(r"([A-Z]+)(\d+)").ok();

    if let Some(regex) = re {
        for cap in regex.captures_iter(expr) {
            if let (Some(col_match), Some(row_match)) = (cap.get(1), cap.get(2)) {
                let col = col_name_to_index(col_match.as_str());
                let row: u32 = row_match.as_str().parse().unwrap_or(1) - 1;
                let key = format!("{},{}", row, col);

                let value = worksheet
                    .data
                    .get(&key)
                    .and_then(|c| c.value.clone())
                    .unwrap_or_else(|| "0".to_string());

                let cell_ref = format!("{}{}", col_match.as_str(), row_match.as_str());
                result = result.replace(&cell_ref, &value);
            }
        }
    }
    result
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

pub fn get_range_values(range: &str, worksheet: &Worksheet) -> Vec<f64> {
    let parts: Vec<&str> = range.split(':').collect();
    if parts.len() != 2 {
        if let Ok(val) = resolve_cell_value(range.trim(), worksheet).parse::<f64>() {
            return vec![val];
        }
        return Vec::new();
    }
    let (start, end) = match parse_range(range) {
        Some(r) => r,
        None => return Vec::new(),
    };
    let mut values = Vec::new();
    for row in start.0..=end.0 {
        for col in start.1..=end.1 {
            let key = format!("{},{}", row, col);
            if let Some(cell) = worksheet.data.get(&key) {
                if let Some(ref value) = cell.value {
                    if let Ok(num) = value.parse::<f64>() {
                        values.push(num);
                    }
                }
            }
        }
    }
    values
}

pub fn get_range_string_values(range: &str, worksheet: &Worksheet) -> Vec<String> {
    let (start, end) = match parse_range(range) {
        Some(r) => r,
        None => return Vec::new(),
    };
    let mut values = Vec::new();
    for row in start.0..=end.0 {
        for col in start.1..=end.1 {
            let key = format!("{},{}", row, col);
            let value = worksheet
                .data
                .get(&key)
                .and_then(|c| c.value.clone())
                .unwrap_or_default();
            values.push(value);
        }
    }
    values
}

pub fn parse_range(range: &str) -> Option<((u32, u32), (u32, u32))> {
    let parts: Vec<&str> = range.split(':').collect();
    if parts.len() != 2 {
        return None;
    }
    let start = parse_cell_ref(parts[0].trim())?;
    let end = parse_cell_ref(parts[1].trim())?;
    Some((start, end))
}

pub fn parse_cell_ref(cell_ref: &str) -> Option<(u32, u32)> {
    let cell_ref = cell_ref.trim().to_uppercase();
    let mut col_str = String::new();
    let mut row_str = String::new();
    for ch in cell_ref.chars() {
        if ch.is_ascii_alphabetic() {
            col_str.push(ch);
        } else if ch.is_ascii_digit() {
            row_str.push(ch);
        }
    }
    if col_str.is_empty() || row_str.is_empty() {
        return None;
    }
    let col = col_name_to_index(&col_str);
    let row: u32 = row_str.parse::<u32>().ok()? - 1;
    Some((row, col))
}

pub fn col_name_to_index(name: &str) -> u32 {
    let mut col: u32 = 0;
    for ch in name.chars() {
        col = col * 26 + (ch as u32 - 'A' as u32 + 1);
    }
    col - 1
}

pub fn format_number(num: f64) -> String {
    if num.fract() == 0.0 {
        format!("{}", num as i64)
    } else {
        format!("{:.6}", num)
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}

pub fn resolve_cell_value(value: &str, worksheet: &Worksheet) -> String {
    if let Some((row, col)) = parse_cell_ref(value) {
        let key = format!("{},{}", row, col);
        worksheet
            .data
            .get(&key)
            .and_then(|c| c.value.clone())
            .unwrap_or_default()
    } else {
        value.to_string()
    }
}

pub fn split_args(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0;
    let mut start = 0;
    for (i, ch) in s.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    if start < s.len() {
        parts.push(&s[start..]);
    }
    parts
}

fn evaluate_condition(condition: &str, worksheet: &Worksheet) -> bool {
    let condition = condition.trim();
    if condition.eq_ignore_ascii_case("TRUE") {
        return true;
    }
    if condition.eq_ignore_ascii_case("FALSE") {
        return false;
    }

    let operators = [">=", "<=", "<>", "!=", "=", ">", "<"];
    for op in &operators {
        if let Some(pos) = condition.find(op) {
            let left = resolve_cell_value(condition[..pos].trim(), worksheet);
            let right = resolve_cell_value(condition[pos + op.len()..].trim().trim_matches('"'), worksheet);

            let left_num = left.parse::<f64>().ok();
            let right_num = right.parse::<f64>().ok();

            return match (*op, left_num, right_num) {
                (">=", Some(l), Some(r)) => l >= r,
                ("<=", Some(l), Some(r)) => l <= r,
                ("<>" | "!=", Some(l), Some(r)) => (l - r).abs() > f64::EPSILON,
                ("<>" | "!=", _, _) => left != right,
                ("=", Some(l), Some(r)) => (l - r).abs() < f64::EPSILON,
                ("=", _, _) => left.eq_ignore_ascii_case(&right),
                (">", Some(l), Some(r)) => l > r,
                ("<", Some(l), Some(r)) => l < r,
                _ => false,
            };
        }
    }
    false
}

fn matches_criteria(value: &str, criteria: &str) -> bool {
    if criteria.starts_with(">=") {
        if let (Ok(v), Ok(c)) = (value.parse::<f64>(), criteria[2..].parse::<f64>()) {
            return v >= c;
        }
    } else if criteria.starts_with("<=") {
        if let (Ok(v), Ok(c)) = (value.parse::<f64>(), criteria[2..].parse::<f64>()) {
            return v <= c;
        }
    } else if criteria.starts_with("<>") || criteria.starts_with("!=") {
        let c = &criteria[2..];
        return !value.eq_ignore_ascii_case(c);
    } else if criteria.starts_with('>') {
        if let (Ok(v), Ok(c)) = (value.parse::<f64>(), criteria[1..].parse::<f64>()) {
            return v > c;
        }
    } else if criteria.starts_with('<') {
        if let (Ok(v), Ok(c)) = (value.parse::<f64>(), criteria[1..].parse::<f64>()) {
            return v < c;
        }
    } else if criteria.starts_with('=') {
        return value.eq_ignore_ascii_case(&criteria[1..]);
    } else if criteria.contains('*') || criteria.contains('?') {
        let pattern = criteria.replace('*', ".*").replace('?', ".");
        if let Ok(re) = regex::Regex::new(&format!("^{}$", pattern)) {
            return re.is_match(value);
        }
    }
    value.eq_ignore_ascii_case(criteria)
}

fn count_matching(values: &[String], criteria: &str) -> usize {
    values.iter().filter(|v| matches_criteria(v, criteria)).count()
}
