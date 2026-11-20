//! Shared argument parsing for the parity function families (#2572).
//!
//! The legacy engine's `split_args` only understands parentheses, so a preview
//! literal such as `IRR([-1000,100,200],0.1)` is split in the middle of the
//! array. These helpers add bracket- and quote-aware splitting, canonical error
//! propagation and one finite-number formatter shared by the new families.

use crate::engine::canonical_error_literal;
use crate::types::Worksheet;

use super::helpers::resolve_cell_value;
use super::refs::{get_range_values, resolve_cell_references};

/// Splits a call's arguments, honouring nested calls, `[...]` arrays and
/// quoted strings. Commas inside any of those are data, not separators.
pub fn split_call_args(input: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut in_string = false;
    let mut start = 0usize;
    let chars: Vec<(usize, char)> = input.char_indices().collect();
    let mut index = 0usize;

    while index < chars.len() {
        let (byte, ch) = chars[index];
        if in_string {
            if ch == '"' {
                if chars.get(index + 1).is_some_and(|(_, next)| *next == '"') {
                    index += 2;
                    continue;
                }
                in_string = false;
            }
            index += 1;
            continue;
        }
        match ch {
            '"' => in_string = true,
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            ',' if paren_depth == 0 && bracket_depth == 0 => {
                parts.push(&input[start..byte]);
                start = byte + 1;
            }
            _ => {}
        }
        index += 1;
    }
    if start < input.len() {
        parts.push(&input[start..]);
    }
    parts
}

/// Returns the canonical spelling of an error literal used as an argument.
pub fn error_literal(raw: &str) -> Option<String> {
    canonical_error_literal(raw.trim()).map(str::to_string)
}

/// Evaluates one scalar argument: a number, a cell reference or quoted number.
pub fn scalar(raw: &str, worksheet: &Worksheet) -> Result<f64, String> {
    let text = raw.trim();
    if let Some(error) = error_literal(text) {
        return Err(error);
    }
    let unquoted = text
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(text);
    if let Some(number) = arithmetic(unquoted, worksheet) {
        return Ok(number);
    }
    let resolved = resolve_cell_value(unquoted, worksheet);
    resolved
        .trim()
        .parse::<f64>()
        .map_err(|_| "#VALUE!".to_string())
}

/// Full-precision arithmetic for arguments such as `0.05/12`. The legacy
/// helper formats to six decimals, which would shift every financial vector.
fn arithmetic(raw: &str, worksheet: &Worksheet) -> Option<f64> {
    let resolved = resolve_cell_references(raw, worksheet);
    evaluate(&resolved)
}

fn evaluate(expression: &str) -> Option<f64> {
    let expression = expression.replace(' ', "");
    if let Ok(number) = expression.parse::<f64>() {
        return Some(number);
    }
    for operator in ['+', '-', '*', '/'] {
        if let Some(position) = expression.rfind(operator) {
            if matches!(operator, '+' | '-') && position == 0 {
                continue;
            }
            let left = evaluate(&expression[..position])?;
            let right = evaluate(&expression[position + 1..])?;
            return match operator {
                '+' => Some(left + right),
                '-' => Some(left - right),
                '*' => Some(left * right),
                '/' if right != 0.0 => Some(left / right),
                _ => None,
            };
        }
    }
    None
}

/// Expands one argument into numbers: `[1,2,3]`, `A1:A3`, or a single value.
pub fn values(raw: &str, worksheet: &Worksheet) -> Result<Vec<f64>, String> {
    let text = raw.trim();
    if let Some(error) = error_literal(text) {
        return Err(error);
    }
    if let Some(inner) = text.strip_prefix('[').and_then(|value| value.strip_suffix(']')) {
        let mut result = Vec::new();
        for part in split_call_args(inner) {
            result.push(scalar(part, worksheet)?);
        }
        return Ok(result);
    }
    if text.contains(':') {
        let numbers = get_range_values(text, worksheet);
        if numbers.is_empty() {
            return Err("#VALUE!".to_string());
        }
        return Ok(numbers);
    }
    Ok(vec![scalar(text, worksheet)?])
}

/// Optional argument with the Excel default of zero.
pub fn at(arguments: &[f64], index: usize) -> f64 {
    arguments.get(index).copied().unwrap_or(0.0)
}

/// Validates a finite result, mapping overflow and NaN to Excel's `#NUM!`.
/// Formatting is deferred to the caller so chained calculations keep precision.
pub fn finite(value: f64) -> Result<f64, String> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err("#NUM!".to_string())
    }
}

/// `ln(x)` with Excel's domain error rather than a NaN leaking into a result.
pub fn log_positive(value: f64) -> Result<f64, String> {
    if value > 0.0 {
        Ok(value.ln())
    } else {
        Err("#NUM!".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Worksheet;

    #[test]
    fn split_keeps_arrays_and_strings_together() {
        let parts = split_call_args("[-1000,100,200],0.1,\"a,b\",IF(1,2,3)");
        assert_eq!(parts.len(), 4);
        assert_eq!(parts[0], "[-1000,100,200]");
        assert_eq!(parts[2], "\"a,b\"");
        assert_eq!(parts[3], "IF(1,2,3)");
    }

    #[test]
    fn arrays_and_error_literals_are_typed() {
        let worksheet = Worksheet::default();
        let numbers = values("[-1000,100,200]", &worksheet);
        assert_eq!(numbers, Ok(vec![-1000.0, 100.0, 200.0]));
        assert_eq!(values("#N/A", &worksheet), Err("#N/A".to_string()));
        assert_eq!(values("nope", &worksheet), Err("#VALUE!".to_string()));
        assert_eq!(finite(f64::INFINITY), Err("#NUM!".to_string()));
    }
}
