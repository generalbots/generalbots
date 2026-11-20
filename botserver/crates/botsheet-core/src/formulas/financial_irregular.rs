//! Extended financial functions (#2434 parity, #2572 differential corpus).
//!
//! Irregular cash flows, depreciation and rate conversions share the same
//! error taxonomy and seed-based root finder as the core financial family.

use chrono::NaiveDate;

use crate::engine::{date_to_serial, format_number};
use crate::types::{DateSystem, Worksheet};

use super::financial::{guard, raw_args, root_of, run};
use super::numeric_args::{at, error_literal, finite, scalar, split_call_args, values};

fn date_scalar(raw: &str, worksheet: &Worksheet) -> Result<f64, String> {
    let text = raw.trim();
    if let Some(error) = error_literal(text) {
        return Err(error);
    }
    let unquoted = text
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(text);
    let resolved = super::helpers::resolve_cell_value(unquoted, worksheet);
    if let Ok(serial) = resolved.trim().parse::<f64>() {
        return Ok(serial);
    }
    let date = NaiveDate::parse_from_str(resolved.trim(), "%Y-%m-%d")
        .map_err(|_| "#VALUE!".to_string())?;
    let serial = date_to_serial(date.year(), date.month(), date.day(), DateSystem::Excel1900)
        .ok_or_else(|| "#NUM!".to_string())?;
    Ok(serial as f64)
}

fn date_numbers(raw: &str, worksheet: &Worksheet) -> Result<Vec<f64>, String> {
    let text = raw.trim();
    if let Some(inner) = text.strip_prefix('[').and_then(|value| value.strip_suffix(']')) {
        let mut result = Vec::new();
        for part in split_call_args(inner) {
            result.push(date_scalar(part, worksheet)?);
        }
        return Ok(result);
    }
    if text.contains(':') {
        let strings = super::refs::get_range_string_values(text, worksheet);
        if strings.is_empty() {
            return Err("#VALUE!".to_string());
        }
        let mut result = Vec::with_capacity(strings.len());
        for value in &strings {
            result.push(date_scalar(value, worksheet)?);
        }
        return Ok(result);
    }
    Ok(vec![date_scalar(text, worksheet)?])
}

fn irregular_npv(rate: f64, flows: &[f64], dates: &[f64]) -> Result<f64, String> {
    if rate <= -1.0 || flows.len() < 2 || flows.len() != dates.len() {
        return Err("#NUM!".to_string());
    }
    let base = dates[0];
    let mut total = 0.0;
    for (flow, date) in flows.iter().zip(dates.iter()) {
        total += flow / (1.0 + rate).powf((date - base) / 365.0);
    }
    finite(total)
}

pub fn evaluate_xnpv(expr: &str, worksheet: &Worksheet) -> Option<String> {
    guard(expr, "XNPV")?;
    let raw = raw_args(expr, "XNPV")?;
    let rate = match scalar(raw.first().copied().unwrap_or(""), worksheet) {
        Ok(rate) => rate,
        Err(error) => return Some(error),
    };
    let flows = match values(raw.get(1).copied().unwrap_or(""), worksheet) {
        Ok(flows) => flows,
        Err(error) => return Some(error),
    };
    let dates = match date_numbers(raw.get(2).copied().unwrap_or(""), worksheet) {
        Ok(dates) => dates,
        Err(error) => return Some(error),
    };
    Some(match irregular_npv(rate, &flows, &dates) {
        Ok(value) => format_number(value),
        Err(error) => error,
    })
}

pub fn evaluate_xirr(expr: &str, worksheet: &Worksheet) -> Option<String> {
    guard(expr, "XIRR")?;
    let raw = raw_args(expr, "XIRR")?;
    let flows = match values(raw.first().copied().unwrap_or(""), worksheet) {
        Ok(flows) => flows,
        Err(error) => return Some(error),
    };
    let dates = match date_numbers(raw.get(1).copied().unwrap_or(""), worksheet) {
        Ok(dates) => dates,
        Err(error) => return Some(error),
    };
    let guess = if raw.len() > 2 {
        match scalar(raw[2], worksheet) {
            Ok(value) if value.is_finite() => value,
            Ok(_) => 0.1,
            Err(error) => return Some(error),
        }
    } else {
        0.1
    };
    let result = root_of(
        |rate| irregular_npv(rate, &flows, &dates).ok(),
        guess,
    );
    Some(match result {
        Ok(value) => format_number(value),
        Err(error) => error,
    })
}

pub fn evaluate_sln(expr: &str, worksheet: &Worksheet) -> Option<String> {
    run(expr, "SLN", worksheet, |arguments| {
        let life = at(&arguments, 2);
        if life == 0.0 {
            return Err("#DIV/0!".to_string());
        }
        finite((at(&arguments, 0) - at(&arguments, 1)) / life)
    })
}

pub fn evaluate_syd(expr: &str, worksheet: &Worksheet) -> Option<String> {
    run(expr, "SYD", worksheet, |arguments| {
        let (cost, salvage) = (at(&arguments, 0), at(&arguments, 1));
        let (life, period) = (at(&arguments, 2), at(&arguments, 3));
        if life <= 0.0 || period < 1.0 || period > life {
            return Err("#NUM!".to_string());
        }
        finite((cost - salvage) * (life - period + 1.0) * 2.0 / (life * (life + 1.0)))
    })
}

pub fn evaluate_db(expr: &str, worksheet: &Worksheet) -> Option<String> {
    run(expr, "DB", worksheet, |arguments| {
        let (cost, salvage) = (at(&arguments, 0), at(&arguments, 1));
        let (life, period) = (at(&arguments, 2), at(&arguments, 3));
        let month = arguments.get(4).copied().unwrap_or(12.0);
        if life <= 0.0 || period < 1.0 || month < 1.0 || month > 12.0 || cost <= 0.0 || salvage < 0.0 {
            return Err("#NUM!".to_string());
        }
        let rate = ((1.0 - (salvage / cost).powf(1.0 / life)) * 1000.0).round() / 1000.0;
        let first = cost * rate * month / 12.0;
        if period == 1.0 {
            return finite(first);
        }
        if period == life + 1.0 {
            let remaining = cost - first - cost * rate * (life - 1.0);
            return finite(remaining * rate * (12.0 - month) / 12.0);
        }
        if period > life + 1.0 {
            return Err("#NUM!".to_string());
        }
        finite((cost - first) * rate * (1.0 - rate).powf(period - 2.0))
    })
}

pub fn evaluate_ddb(expr: &str, worksheet: &Worksheet) -> Option<String> {
    run(expr, "DDB", worksheet, |arguments| {
        let (cost, salvage) = (at(&arguments, 0), at(&arguments, 1));
        let (life, period) = (at(&arguments, 2), at(&arguments, 3));
        let factor = arguments.get(4).copied().unwrap_or(2.0);
        if life <= 0.0 || period < 1.0 || period > life || factor <= 0.0 {
            return Err("#NUM!".to_string());
        }
        let last = i32::try_from(period.floor()).unwrap_or(i32::MAX);
        let mut total = 0.0;
        let mut depreciation = 0.0;
        for step in 1..=last {
            let allowance = ((cost - total) * factor / life).min(cost - salvage - total);
            depreciation = allowance.max(0.0);
            total += depreciation;
        }
        finite(depreciation)
    })
}

pub fn evaluate_effect(expr: &str, worksheet: &Worksheet) -> Option<String> {
    run(expr, "EFFECT", worksheet, |arguments| {
        let (nominal, periods) = (at(&arguments, 0), at(&arguments, 1));
        if nominal <= 0.0 || periods < 1.0 {
            return Err("#NUM!".to_string());
        }
        finite((1.0 + nominal / periods).powf(periods) - 1.0)
    })
}

pub fn evaluate_nominal(expr: &str, worksheet: &Worksheet) -> Option<String> {
    run(expr, "NOMINAL", worksheet, |arguments| {
        let (effective, periods) = (at(&arguments, 0), at(&arguments, 1));
        if effective <= 0.0 || periods < 1.0 {
            return Err("#NUM!".to_string());
        }
        finite(periods * ((1.0 + effective).powf(1.0 / periods) - 1.0))
    })
}
