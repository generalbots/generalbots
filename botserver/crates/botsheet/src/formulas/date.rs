use chrono::{Datelike, Local, NaiveDate};

use crate::types::Worksheet;

use super::helpers::{resolve_cell_value, split_args};

pub fn evaluate_today(_expr: &str, _worksheet: &Worksheet) -> Option<String> {
    if _expr != "TODAY()" {
        return None;
    }
    let today = Local::now().date_naive();
    Some(today.format("%Y-%m-%d").to_string())
}

pub fn evaluate_now(_expr: &str, _worksheet: &Worksheet) -> Option<String> {
    if _expr != "NOW()" {
        return None;
    }
    let now = Local::now();
    Some(now.format("%Y-%m-%d %H:%M:%S").to_string())
}

pub fn evaluate_date(expr: &str, _worksheet: &Worksheet) -> Option<String> {
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

pub fn evaluate_year(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("YEAR(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[5..expr.len() - 1];
    let date_str = resolve_cell_value(inner.trim().trim_matches('"'), worksheet);
    let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").ok()?;
    Some(date.year().to_string())
}

pub fn evaluate_month(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("MONTH(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[6..expr.len() - 1];
    let date_str = resolve_cell_value(inner.trim().trim_matches('"'), worksheet);
    let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").ok()?;
    Some(date.month().to_string())
}

pub fn evaluate_day(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("DAY(") || !expr.ends_with(')') {
        return None;
    }
    let inner = &expr[4..expr.len() - 1];
    let date_str = resolve_cell_value(inner.trim().trim_matches('"'), worksheet);
    let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").ok()?;
    Some(date.day().to_string())
}

pub fn evaluate_datedif(expr: &str, worksheet: &Worksheet) -> Option<String> {
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
