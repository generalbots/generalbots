use chrono::{Datelike, Local, NaiveDate};

use crate::engine::date_serial::{date_to_serial, serial_to_date_parts};
use crate::types::{DateSystem, Worksheet};

use super::financial::raw_args;
use super::helpers::{resolve_cell_value, split_args};
use super::numeric_args::{scalar, split_call_args};

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


pub fn evaluate_hour(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("HOUR(") || !expr.ends_with(')') { return None; }
    let inner = &expr[5..expr.len() - 1];
    let num: f64 = resolve_cell_value(inner.trim(), worksheet).parse().ok()?;
    let secs = (num * 86400.0) as i64;
    let h = ((secs / 3600) % 24 + 24) % 24;
    Some(h.to_string())
}

pub fn evaluate_minute(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("MINUTE(") || !expr.ends_with(')') { return None; }
    let inner = &expr[7..expr.len() - 1];
    let num: f64 = resolve_cell_value(inner.trim(), worksheet).parse().ok()?;
    let secs = (num * 86400.0) as i64;
    let m = ((secs / 60) % 60 + 60) % 60;
    Some(m.to_string())
}

pub fn evaluate_second(expr: &str, worksheet: &Worksheet) -> Option<String> {
    if !expr.starts_with("SECOND(") || !expr.ends_with(')') { return None; }
    let inner = &expr[7..expr.len() - 1];
    let num: f64 = resolve_cell_value(inner.trim(), worksheet).parse().ok()?;
    let secs = (num * 86400.0) as i64;
    let s = (secs % 60 + 60) % 60;
    Some(s.to_string())
}

fn date_from(raw: &str, worksheet: &Worksheet) -> Option<NaiveDate> {
    let text = raw.trim();
    let unquoted = text
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(text);
    let resolved = resolve_cell_value(unquoted, worksheet);
    let resolved = resolved.trim();
    if let Ok(serial) = resolved.parse::<f64>() {
        let (year, month, day) = serial_to_date_parts(serial, DateSystem::Excel1900)?;
        return NaiveDate::from_ymd_opt(year, month, day);
    }
    NaiveDate::parse_from_str(resolved, "%Y-%m-%d").ok()
}

fn serial_of(date: NaiveDate) -> String {
    match date_to_serial(date.year(), date.month(), date.day(), DateSystem::Excel1900) {
        Some(serial) => serial.to_string(),
        None => "#NUM!".to_string(),
    }
}

fn add_months(date: NaiveDate, months: i32) -> Option<NaiveDate> {
    let total = i64::from(date.year()) * 12 + i64::from(date.month()) - 1 + i64::from(months);
    let year = i32::try_from(total.div_euclid(12)).ok()?;
    let month = u32::try_from(total.rem_euclid(12) + 1).ok()?;
    let last_day = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)?
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)?
    }
    .signed_duration_since(NaiveDate::from_ymd_opt(year, month, 1)?)
    .num_days();
    let day = date.day().min(u32::try_from(last_day).ok()?);
    NaiveDate::from_ymd_opt(year, month, day)
}

fn holiday_dates(raw: &[&str], worksheet: &Worksheet) -> Vec<NaiveDate> {
    let mut dates = Vec::new();
    for argument in raw {
        let text = argument.trim();
        if let Some(inner) = text.strip_prefix('[').and_then(|value| value.strip_suffix(']')) {
            for part in split_call_args(inner) {
                if let Some(date) = date_from(part, worksheet) {
                    dates.push(date);
                }
            }
        } else if text.contains(':') {
            for value in super::refs::get_range_string_values(text, worksheet) {
                if let Some(date) = date_from(&value, worksheet) {
                    dates.push(date);
                }
            }
        } else if let Some(date) = date_from(text, worksheet) {
            dates.push(date);
        }
    }
    dates
}

fn network_days(expr: &str, name: &str, worksheet: &Worksheet) -> Option<String> {
    let raw = raw_args(expr, name)?;
    let start = date_from(raw.first().copied().unwrap_or(""), worksheet)?;
    let end = date_from(raw.get(1).copied().unwrap_or(""), worksheet)?;
    let holidays = holiday_dates(&raw[2..], worksheet);
    let mut current = start;
    let mut count = 0i64;
    while current <= end {
        let weekday = current.weekday().num_days_from_monday();
        if weekday < 5 && !holidays.contains(&current) {
            count += 1;
        }
        current = current.succ_opt()?;
    }
    Some(count.to_string())
}

pub fn evaluate_networkdays(expr: &str, worksheet: &Worksheet) -> Option<String> {
    network_days(expr, "NETWORKDAYS", worksheet)
}

pub fn evaluate_networkdays_intl(expr: &str, worksheet: &Worksheet) -> Option<String> {
    network_days(expr, "NETWORKDAYS.INTL", worksheet)
}

pub fn evaluate_edate(expr: &str, worksheet: &Worksheet) -> Option<String> {
    let raw = raw_args(expr, "EDATE")?;
    let start = date_from(raw.first().copied().unwrap_or(""), worksheet)?;
    let months = scalar(raw.get(1).copied().unwrap_or(""), worksheet).ok()?;
    let result = add_months(start, months as i32)?;
    Some(serial_of(result))
}

pub fn evaluate_eomonth(expr: &str, worksheet: &Worksheet) -> Option<String> {
    let raw = raw_args(expr, "EOMONTH")?;
    let start = date_from(raw.first().copied().unwrap_or(""), worksheet)?;
    let months = scalar(raw.get(1).copied().unwrap_or(""), worksheet).ok()?;
    let first = add_months(start, months as i32)?;
    let next = if first.month() == 12 {
        NaiveDate::from_ymd_opt(first.year() + 1, 1, 1)?
    } else {
        NaiveDate::from_ymd_opt(first.year(), first.month() + 1, 1)?
    };
    Some(serial_of(next.pred_opt()?))
}

pub fn evaluate_yearfrac(expr: &str, worksheet: &Worksheet) -> Option<String> {
    let raw = raw_args(expr, "YEARFRAC")?;
    let start = date_from(raw.first().copied().unwrap_or(""), worksheet)?;
    let end = date_from(raw.get(1).copied().unwrap_or(""), worksheet)?;
    let basis = scalar(raw.get(2).copied().unwrap_or("1"), worksheet).ok()? as i32;
    let days = end.signed_duration_since(start).num_days() as f64;
    let result = match basis {
        1 => days / 365.0,
        _ => days / 365.0,
    };
    Some(crate::engine::format_number(result))
}

pub fn evaluate_weeknum(expr: &str, worksheet: &Worksheet) -> Option<String> {
    let raw = raw_args(expr, "WEEKNUM")?;
    let date = date_from(raw.first().copied().unwrap_or(""), worksheet)?;
    let return_type = scalar(raw.get(1).copied().unwrap_or("1"), worksheet).ok()? as i32;
    if return_type == 21 {
        return Some(date.iso_week().week().to_string());
    }
    let jan1 = NaiveDate::from_ymd_opt(date.year(), 1, 1)?;
    let value =
        date_to_serial(date.year(), date.month(), date.day(), DateSystem::Excel1900)? as f64;
    let jan1_serial =
        date_to_serial(jan1.year(), jan1.month(), jan1.day(), DateSystem::Excel1900)? as f64;
    let week_start = if return_type == 2 { 1i64 } else { 0i64 };
    let offset = (i64::from(jan1.weekday().num_days_from_sunday()) - week_start + 7) % 7;
    let week = ((value - jan1_serial + offset as f64) / 7.0).floor() as i64 + 1;
    Some(week.to_string())
}

pub fn evaluate_isoweeknum(expr: &str, worksheet: &Worksheet) -> Option<String> {
    let raw = raw_args(expr, "ISOWEEKNUM")?;
    let date = date_from(raw.first().copied().unwrap_or(""), worksheet)?;
    Some(date.iso_week().week().to_string())
}
