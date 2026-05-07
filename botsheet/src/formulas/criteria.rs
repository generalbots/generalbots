use crate::types::Worksheet;

use super::{parse_range, split_args};
use super::helpers::{count_matching, matches_criteria};

pub fn evaluate_counta(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("COUNTA(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[7..expr.len() - 1];
    let count = super::get_range_string_values(inner, worksheet)
        .iter()
        .filter(|v| !v.is_empty())
        .count();
    Some(count.to_string())
}

pub fn evaluate_countblank(expr: &str, worksheet: &Worksheet) -> Option<String> {
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

pub fn evaluate_countif(expr: &str, worksheet: &Worksheet) -> Option<String> {
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
    let values = super::get_range_string_values(range, worksheet);
    let count = count_matching(&values, criteria);
    Some(count.to_string())
}

pub fn evaluate_sumif(expr: &str, worksheet: &Worksheet) -> Option<String> {
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

    let criteria_values = super::get_range_string_values(criteria_range, worksheet);
    let sum_values = super::get_range_values(sum_range, worksheet);

    let mut sum = 0.0;
    for (i, cv) in criteria_values.iter().enumerate() {
        if matches_criteria(cv, criteria) {
            if let Some(sv) = sum_values.get(i) {
                sum += sv;
            }
        }
    }
    Some(super::format_number(sum))
}

pub fn evaluate_averageif(expr: &str, worksheet: &Worksheet) -> Option<String> {
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

    let criteria_values = super::get_range_string_values(criteria_range, worksheet);
    let avg_values = super::get_range_values(avg_range, worksheet);

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
    Some(super::format_number(sum / count as f64))
}
