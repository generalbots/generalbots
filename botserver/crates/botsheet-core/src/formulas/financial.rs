//! Core financial functions (#2434 parity, #2572 differential corpus).
//!
//! Sign conventions and iterative tolerances follow the local preview exactly,
//! so a workbook opens with the same numbers whether it was calculated by the
//! browser or by the server. Every result is finite-checked and every invalid
//! domain returns a canonical Excel error literal.

use crate::engine::format_number;
use crate::types::Worksheet;

use super::numeric_args::{
    at, error_literal, finite, log_positive, scalar, split_call_args, values,
};

const TOLERANCE: f64 = 1e-11;

pub(crate) fn raw_args<'a>(expr: &'a str, name: &str) -> Option<Vec<&'a str>> {
    let inner = expr.strip_prefix(name)?.strip_prefix('(')?.strip_suffix(')')?;
    Some(split_call_args(inner))
}

pub(crate) fn number_args(
    expr: &str,
    name: &str,
    worksheet: &Worksheet,
) -> Option<Result<Vec<f64>, String>> {
    let raw = raw_args(expr, name)?;
    let mut arguments = Vec::with_capacity(raw.len());
    for argument in raw {
        match scalar(argument, worksheet) {
            Ok(number) => arguments.push(number),
            Err(error) => return Some(Err(error)),
        }
    }
    Some(Ok(arguments))
}

pub(crate) fn guard(expr: &str, name: &str) -> Option<()> {
    expr.strip_prefix(name)?.strip_prefix('(')?;
    expr.strip_suffix(')')?;
    Some(())
}

fn growth(rate: f64, periods: f64) -> Result<f64, String> {
    let factor = (1.0 + rate).powf(periods);
    if factor.is_finite() && factor > 0.0 {
        Ok(factor)
    } else {
        Err("#NUM!".to_string())
    }
}

fn payment(rate: f64, periods: f64, present: f64, future: f64, kind: f64) -> Result<f64, String> {
    if periods == 0.0 {
        return Err("#NUM!".to_string());
    }
    if rate == 0.0 {
        return finite(-(present + future) / periods);
    }
    let factor = growth(rate, periods)?;
    let result = -(present * rate * factor + future * rate) / (factor - 1.0);
    finite(if kind != 0.0 { result / (1.0 + rate) } else { result })
}

fn future_value(rate: f64, periods: f64, installment: f64, present: f64, kind: f64) -> Result<f64, String> {
    if rate == 0.0 {
        return finite(-(present + installment * periods));
    }
    let factor = growth(rate, periods)?;
    finite(-(present * factor + installment * (1.0 + rate * kind) * (factor - 1.0) / rate))
}

fn present_value(rate: f64, periods: f64, installment: f64, future: f64, kind: f64) -> Result<f64, String> {
    if rate == 0.0 {
        return finite(-(future + installment * periods));
    }
    let factor = growth(rate, periods)?;
    let numerator = -(future + installment * (1.0 + rate * kind) * (factor - 1.0) / rate);
    finite(numerator / factor)
}

fn interest_payment(rate: f64, period: f64, periods: f64, present: f64, future: f64, kind: f64) -> Result<f64, String> {
    if period < 1.0 || period > periods {
        return Err("#NUM!".to_string());
    }
    if rate == 0.0 {
        return Ok(0.0);
    }
    let installment = payment(rate, periods, present, future, kind)?;
    let balance = future_value(rate, period - 1.0, installment, present, kind)?;
    let interest = balance * rate;
    finite(if kind != 0.0 { interest / (1.0 + rate) } else { interest })
}

/// Bracketed roots with a bisection fallback, matching the preview's seeds.
pub(crate) fn root_of<F>(evaluate: F, guess: f64) -> Result<f64, String>
where
    F: Fn(f64) -> Option<f64>,
{
    let mut seeds: Vec<f64> = vec![
        -0.999999, -0.9, -0.5, -0.1, 0.0, 0.001, 0.01, 0.05, 0.1, 0.2, 0.5, 1.0, 2.0, 10.0,
    ];
    if guess.is_finite() {
        seeds.insert(0, guess);
    }
    seeds.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    let mut previous: Option<(f64, f64)> = None;

    for x in seeds {
        let Some(y) = evaluate(x) else {
            continue;
        };
        if y.abs() < TOLERANCE {
            return Ok(x);
        }
        if let Some((left_x, left_y)) = previous {
            if left_y * y < 0.0 {
                let (mut low, mut high) = (left_x, x);
                let mut low_y = left_y;
                for _ in 0..200 {
                    let middle = (low + high) / 2.0;
                    let Some(middle_y) = evaluate(middle) else {
                        break;
                    };
                    if middle_y.abs() < TOLERANCE || (high - low) / 2.0 < TOLERANCE {
                        return Ok(middle);
                    }
                    if low_y * middle_y < 0.0 {
                        high = middle;
                    } else {
                        low = middle;
                        low_y = middle_y;
                    }
                }
            }
        }
        previous = Some((x, y));
    }
    Err("#NUM!".to_string())
}

/// NPV at a rate, with `offset = 1` for Excel's NPV convention.
pub(crate) fn npv_at(rate: f64, flows: &[f64], offset: usize) -> Result<f64, String> {
    if rate <= -1.0 {
        return Err("#NUM!".to_string());
    }
    let mut total = 0.0;
    for (index, flow) in flows.iter().enumerate() {
        total += flow / (1.0 + rate).powi(i32::try_from(index + offset).unwrap_or(i32::MAX));
    }
    finite(total)
}

/// Expands every cash-flow argument (arrays, ranges and scalars) into a list.
pub(crate) fn cash_flows(raw: &[&str], worksheet: &Worksheet) -> Result<Vec<f64>, String> {
    let mut flows = Vec::new();
    for argument in raw {
        flows.extend(values(argument, worksheet)?);
    }
    Ok(flows)
}

pub(crate) fn run(
    expr: &str,
    name: &str,
    worksheet: &Worksheet,
    operation: fn(Vec<f64>) -> Result<f64, String>,
) -> Option<String> {
    guard(expr, name)?;
    Some(match number_args(expr, name, worksheet)? {
        Ok(arguments) => match operation(arguments) {
            Ok(value) => format_number(value),
            Err(error) => error,
        },
        Err(error) => error,
    })
}

pub fn evaluate_pmt(expr: &str, worksheet: &Worksheet) -> Option<String> {
    run(expr, "PMT", worksheet, |arguments| {
        payment(at(&arguments, 0), at(&arguments, 1), at(&arguments, 2), at(&arguments, 3), at(&arguments, 4))
    })
}

pub fn evaluate_fv(expr: &str, worksheet: &Worksheet) -> Option<String> {
    run(expr, "FV", worksheet, |arguments| {
        future_value(at(&arguments, 0), at(&arguments, 1), at(&arguments, 2), at(&arguments, 3), at(&arguments, 4))
    })
}

pub fn evaluate_pv(expr: &str, worksheet: &Worksheet) -> Option<String> {
    run(expr, "PV", worksheet, |arguments| {
        present_value(at(&arguments, 0), at(&arguments, 1), at(&arguments, 2), at(&arguments, 3), at(&arguments, 4))
    })
}

pub fn evaluate_ipmt(expr: &str, worksheet: &Worksheet) -> Option<String> {
    run(expr, "IPMT", worksheet, |arguments| {
        interest_payment(
            at(&arguments, 0),
            at(&arguments, 1),
            at(&arguments, 2),
            at(&arguments, 3),
            at(&arguments, 4),
            at(&arguments, 5),
        )
    })
}

pub fn evaluate_ppmt(expr: &str, worksheet: &Worksheet) -> Option<String> {
    run(expr, "PPMT", worksheet, |arguments| {
        let rate = at(&arguments, 0);
        let periods = at(&arguments, 2);
        let installment = payment(rate, periods, at(&arguments, 3), at(&arguments, 4), at(&arguments, 5))?;
        let interest = interest_payment(rate, at(&arguments, 1), periods, at(&arguments, 3), at(&arguments, 4), at(&arguments, 5))?;
        finite(installment - interest)
    })
}

pub fn evaluate_nper(expr: &str, worksheet: &Worksheet) -> Option<String> {
    run(expr, "NPER", worksheet, |arguments| {
        let (rate, installment, present) = (at(&arguments, 0), at(&arguments, 1), at(&arguments, 2));
        let (future, kind) = (at(&arguments, 3), at(&arguments, 4));
        if rate == 0.0 {
            if installment == 0.0 {
                return Err("#NUM!".to_string());
            }
            return finite(-(present + future) / installment);
        }
        let weighted = installment * (1.0 + rate * kind);
        let ratio = (weighted - future * rate) / (present * rate + weighted);
        finite(log_positive(ratio)? / (1.0 + rate).ln())
    })
}

pub fn evaluate_rate(expr: &str, worksheet: &Worksheet) -> Option<String> {
    guard(expr, "RATE")?;
    let arguments = match number_args(expr, "RATE", worksheet)? {
        Ok(arguments) => arguments,
        Err(error) => return Some(error),
    };
    let (periods, installment, present) = (at(&arguments, 0), at(&arguments, 1), at(&arguments, 2));
    let (future, kind, guess) = (at(&arguments, 3), at(&arguments, 4), arguments.get(5).copied().unwrap_or(0.1));
    if installment == 0.0 {
        return Some("#NUM!".to_string());
    }
    let result = root_of(
        |rate| payment(rate, periods, present, future, kind).ok().map(|value| value - installment),
        guess,
    );
    Some(match result {
        Ok(value) => format_number(value),
        Err(error) => error,
    })
}

pub fn evaluate_npv(expr: &str, worksheet: &Worksheet) -> Option<String> {
    guard(expr, "NPV")?;
    let raw = raw_args(expr, "NPV")?;
    let rate = match scalar(raw.first().copied().unwrap_or(""), worksheet) {
        Ok(rate) => rate,
        Err(error) => return Some(error),
    };
    let flows = cash_flows(&raw[1..], worksheet);
    Some(match flows {
        Ok(flows) => match npv_at(rate, &flows, 1) {
            Ok(value) => format_number(value),
            Err(error) => error,
        },
        Err(error) => error,
    })
}

pub fn evaluate_irr(expr: &str, worksheet: &Worksheet) -> Option<String> {
    guard(expr, "IRR")?;
    let raw = raw_args(expr, "IRR")?;
    let first = raw.first().copied().unwrap_or("");
    if let Some(error) = error_literal(first) {
        return Some(error);
    }
    let flows = values(first, worksheet);
    let flows = match flows {
        Ok(flows) if flows.len() >= 2 => flows,
        Ok(_) => return Some("#NUM!".to_string()),
        Err(error) => return Some(error),
    };
    let guess = if raw.len() > 1 {
        match scalar(raw[1], worksheet) {
            Ok(value) if value.is_finite() => value,
            Ok(_) => 0.1,
            Err(error) => return Some(error),
        }
    } else {
        0.1
    };
    let result = root_of(|rate| npv_at(rate, &flows, 0).ok(), guess);
    Some(match result {
        Ok(value) => format_number(value),
        Err(error) => error,
    })
}

pub fn evaluate_mirr(expr: &str, worksheet: &Worksheet) -> Option<String> {
    guard(expr, "MIRR")?;
    let raw = raw_args(expr, "MIRR")?;
    let flows = match values(raw.first().copied().unwrap_or(""), worksheet) {
        Ok(flows) => flows,
        Err(error) => return Some(error),
    };
    let finance = match raw.get(1).map(|value| scalar(value, worksheet)).unwrap_or(Ok(0.0)) {
        Ok(value) => value,
        Err(error) => return Some(error),
    };
    let reinvest = match raw.get(2).map(|value| scalar(value, worksheet)).unwrap_or(Ok(0.0)) {
        Ok(value) => value,
        Err(error) => return Some(error),
    };
    let count = flows.len();
    if count < 2 || finance <= -1.0 || reinvest <= -1.0 {
        return Some("#NUM!".to_string());
    }
    let mut negative_present = 0.0;
    let mut positive_future = 0.0;
    for (index, flow) in flows.iter().enumerate() {
        if *flow < 0.0 {
            negative_present += flow / (1.0 + finance).powi(i32::try_from(index).unwrap_or(i32::MAX));
        } else {
            let periods = i32::try_from(count - 1 - index).unwrap_or(i32::MAX);
            positive_future += flow * (1.0 + reinvest).powi(periods);
        }
    }
    if negative_present >= 0.0 {
        return Some("#NUM!".to_string());
    }
    let exponent = 1.0 / (count as f64 - 1.0);
    let ratio = -positive_future / (negative_present * (1.0 + finance));
    let base = ratio.powf(exponent);
    if !base.is_finite() || base < 0.0 {
        return Some("#NUM!".to_string());
    }
    Some(match finite(base - 1.0) {
        Ok(value) => format_number(value),
        Err(error) => error,
    })
}
