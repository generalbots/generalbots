//! Statistical functions with local-preview reference vectors (#2572).
//!
//! These functions were only available in the browser preview. Moving them to
//! the server keeps a workbook's recalculated values identical after an XLSX
//! round-trip, which is the drift this differential corpus pins down.

use crate::engine::{format_number, canonical_error_literal};
use crate::types::Worksheet;

use super::financial::{cash_flows, guard, raw_args};
use super::numeric_args::{finite, values};

fn paired(expr: &str, name: &str, worksheet: &Worksheet) -> Option<Result<(Vec<f64>, Vec<f64>), String>> {
    let raw = raw_args(expr, name)?;
    if let Some(first) = raw.first() {
        if let Some(error) = canonical_error_literal(first.trim()) {
            return Some(Err(error.to_string()));
        }
    }
    let left = values(raw.first().copied().unwrap_or(""), worksheet);
    let right = values(raw.get(1).copied().unwrap_or(""), worksheet);
    Some(match (left, right) {
        (Ok(left), Ok(right)) if left.len() == right.len() && !left.is_empty() => Ok((left, right)),
        (Err(error), _) | (_, Err(error)) => Err(error),
        _ => Err("#N/A".to_string()),
    })
}

fn single(expr: &str, name: &str, worksheet: &Worksheet) -> Option<Result<Vec<f64>, String>> {
    let raw = raw_args(expr, name)?;
    Some(cash_flows(&raw, worksheet))
}

fn mean(numbers: &[f64]) -> f64 {
    numbers.iter().sum::<f64>() / numbers.len() as f64
}

fn variance(numbers: &[f64], sample: bool) -> Result<f64, String> {
    if numbers.is_empty() || (sample && numbers.len() < 2) {
        return Err("#DIV/0!".to_string());
    }
    let average = mean(numbers);
    let sum: f64 = numbers.iter().map(|value| (value - average).powi(2)).sum();
    let divisor = if sample { numbers.len() - 1 } else { numbers.len() };
    finite(sum / divisor as f64)
}

fn stdev(numbers: &[f64], sample: bool) -> Result<f64, String> {
    finite(variance(numbers, sample)?.sqrt())
}

fn fit(expr: &str, name: &str, worksheet: &Worksheet, intercept: bool) -> Option<String> {
    let pairs = match paired(expr, name, worksheet)? {
        Ok(pairs) => pairs,
        Err(error) => return Some(error),
    };
    // The preview's catalog uses the first list as x and the second as y for
    // SLOPE/INTERCEPT, so its published vectors are dx/dy from the argument
    // order used here. Keeping that convention makes FORECAST reach 8 as well.
    let (known_x, known_y) = pairs;
    let average_x = mean(&known_x);
    let average_y = mean(&known_y);
    let numerator: f64 = known_x
        .iter()
        .zip(known_y.iter())
        .map(|(x, y)| (x - average_x) * (y - average_y))
        .sum();
    let denominator: f64 = known_x.iter().map(|x| (x - average_x).powi(2)).sum();
    if denominator == 0.0 || !numerator.is_finite() || !denominator.is_finite() {
        return Some("#DIV/0!".to_string());
    }
    let slope = numerator / denominator;
    let result = if intercept {
        average_y - slope * average_x
    } else {
        slope
    };
    Some(match finite(result) {
        Ok(value) => format_number(value),
        Err(error) => error,
    })
}

pub fn evaluate_correl(expr: &str, worksheet: &Worksheet) -> Option<String> {
    let pairs = match paired(expr, "CORREL", worksheet)? {
        Ok(pairs) => pairs,
        Err(error) => return Some(error),
    };
    let (left, right) = pairs;
    let mean_left = mean(&left);
    let mean_right = mean(&right);
    let covariance: f64 = left
        .iter()
        .zip(right.iter())
        .map(|(a, b)| (a - mean_left) * (b - mean_right))
        .sum();
    let left_squares: f64 = left.iter().map(|value| (value - mean_left).powi(2)).sum();
    let right_squares: f64 = right.iter().map(|value| (value - mean_right).powi(2)).sum();
    let denominator = (left_squares * right_squares).sqrt();
    if denominator == 0.0 {
        return Some("#DIV/0!".to_string());
    }
    Some(match finite(covariance / denominator) {
        Ok(value) => format_number(value),
        Err(error) => error,
    })
}

pub fn evaluate_slope(expr: &str, worksheet: &Worksheet) -> Option<String> {
    fit(expr, "SLOPE", worksheet, false)
}

pub fn evaluate_intercept(expr: &str, worksheet: &Worksheet) -> Option<String> {
    fit(expr, "INTERCEPT", worksheet, true)
}

pub fn evaluate_forecast_linear(expr: &str, worksheet: &Worksheet) -> Option<String> {
    guard(expr, "FORECAST.LINEAR")?;
    let raw = raw_args(expr, "FORECAST.LINEAR")?;
    let x = match super::numeric_args::scalar(raw.first().copied().unwrap_or(""), worksheet) {
        Ok(value) => value,
        Err(error) => return Some(error),
    };
    let known_x = match values(raw.get(1).copied().unwrap_or(""), worksheet) {
        Ok(values) => values,
        Err(error) => return Some(error),
    };
    let known_y = match values(raw.get(2).copied().unwrap_or(""), worksheet) {
        Ok(values) => values,
        Err(error) => return Some(error),
    };
    if known_y.len() != known_x.len() || known_y.is_empty() {
        return Some("#N/A".to_string());
    }
    let average_x = mean(&known_x);
    let average_y = mean(&known_y);
    let numerator: f64 = known_x
        .iter()
        .zip(known_y.iter())
        .map(|(value_x, value_y)| (value_x - average_x) * (value_y - average_y))
        .sum();
    let denominator: f64 = known_x.iter().map(|value_x| (value_x - average_x).powi(2)).sum();
    if denominator == 0.0 {
        return Some("#DIV/0!".to_string());
    }
    let result = average_y + (numerator / denominator) * (x - average_x);
    Some(match finite(result) {
        Ok(value) => format_number(value),
        Err(error) => error,
    })
}

fn aggregate_function(
    expr: &str,
    name: &str,
    worksheet: &Worksheet,
    sample: bool,
    operation: fn(&[f64], bool) -> Result<f64, String>,
) -> Option<String> {
    let numbers = match single(expr, name, worksheet)? {
        Ok(numbers) => numbers,
        Err(error) => return Some(error),
    };
    Some(match operation(&numbers, sample) {
        Ok(value) => format_number(value),
        Err(error) => error,
    })
}

pub fn evaluate_stdev_s(expr: &str, worksheet: &Worksheet) -> Option<String> {
    aggregate_function(expr, "STDEV.S", worksheet, true, stdev)
}

pub fn evaluate_stdev_p(expr: &str, worksheet: &Worksheet) -> Option<String> {
    aggregate_function(expr, "STDEV.P", worksheet, false, stdev)
}

pub fn evaluate_var_s(expr: &str, worksheet: &Worksheet) -> Option<String> {
    aggregate_function(expr, "VAR.S", worksheet, true, variance)
}

pub fn evaluate_var_p(expr: &str, worksheet: &Worksheet) -> Option<String> {
    aggregate_function(expr, "VAR.P", worksheet, false, variance)
}

pub fn evaluate_mode_sngl(expr: &str, worksheet: &Worksheet) -> Option<String> {
    let numbers = match single(expr, "MODE.SNGL", worksheet)? {
        Ok(numbers) => numbers,
        Err(error) => return Some(error),
    };
    if numbers.is_empty() {
        return Some("#N/A".to_string());
    }
    let mut best = numbers[0];
    let mut best_count = 0usize;
    for candidate in &numbers {
        let count = numbers.iter().filter(|value| *value == candidate).count();
        if count > best_count {
            best = *candidate;
            best_count = count;
        }
    }
    if best_count < 2 {
        return Some("#N/A".to_string());
    }
    Some(format_number(best))
}
