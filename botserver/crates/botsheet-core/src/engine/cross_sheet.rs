//! Cross-sheet reference support for the typed evaluator (#783).
//!
//! The legacy 170-function library reads cells through a single worksheet, so
//! `Sheet2!A1` arguments would otherwise be parsed as text tokens. This module
//! walks a raw argument list and substitutes every `Sheet!ref` / `Sheet!A1:B3`
//! occurrence (outside string literals) with its resolved literal text.
//! Aggregate functions collapse ranges to one pre-computed number so
//! `SUM(Sheet2!A1:A3)` keeps its meaning; anything that cannot be resolved is
//! left verbatim for the legacy dispatcher to handle as it sees fit.

use crate::formulas::{parse_cell_ref, parse_range, MAX_RANGE_CELLS};
use crate::types::Worksheet;

/// Legacy functions whose implementations consume the whole argument list as
/// a single numeric token via `get_range_values`; a cross-sheet range becomes
/// one pre-aggregated number for these, preserving their semantics.
const RANGE_AGGREGATE_FUNCS: &[&str] = &[
    "SUM", "AVERAGE", "COUNT", "COUNTA", "COUNTBLANK", "MAX", "MIN", "PRODUCT", "MEDIAN",
    "STDEV", "STDEVP",
];

/// Resolves cross-sheet references inside a raw function-call argument list,
/// returning the substituted text ready for the legacy dispatcher.
pub fn resolve_sheet_args(
    raw: &str,
    func_name: &str,
    worksheets: &[Worksheet],
) -> String {
    let bytes = raw.as_bytes();
    let mut out = String::with_capacity(raw.len() + 16);
    let mut i = 0usize;
    let mut in_quotes = false;
    while i < bytes.len() {
        let ch = bytes[i];
        if ch == b'"' {
            in_quotes = !in_quotes;
            out.push('"');
            i += 1;
            continue;
        }
        if ch != b'!' || in_quotes {
            out.push(ch as char);
            i += 1;
            continue;
        }
        // Outside quotes: scan back over the sheet-name characters that
        // precede the `!`, then split the cell token that follows it.
        let mut start = i;
        while start > 0 && is_sheet_name_char(bytes[start - 1]) {
            start -= 1;
        }
        let sheet_name = &raw[start..i];
        let (cell_tok, _) = split_ref_token(&raw[i + 1..]);
        let advanced = 1 + cell_tok.len();
        if !sheet_name.is_empty() && !cell_tok.is_empty() {
            match resolve_sheet_token(sheet_name, cell_tok, func_name, worksheets) {
                Some(text) => {
                    out.push_str(&text);
                    i += advanced;
                    continue;
                }
                None => {}
            }
        }
        out.push_str(&raw[start..i + 1]);
        out.push_str(cell_tok);
        i += advanced;
    }
    out
}

/// Resolves one `Sheet!token` occurrence to its literal value text.
fn resolve_sheet_token(
    sheet_name: &str,
    cell_tok: &str,
    func_name: &str,
    worksheets: &[Worksheet],
) -> Option<String> {
    let target = worksheets
        .iter()
        .find(|w| w.name.eq_ignore_ascii_case(sheet_name))?;
    if let Some((start, end)) = parse_range(cell_tok) {
        let cols = u64::from(end.1 - start.1) + 1;
        let rows = u64::from(end.0 - start.0) + 1;
        if cols * rows > MAX_RANGE_CELLS {
            return None;
        }
        if RANGE_AGGREGATE_FUNCS.contains(&func_name) {
            return aggregate_range(func_name, target, start, end);
        }
        return Some(range_text_values(target, start, end));
    }
    let (row, col) = parse_cell_ref(cell_tok)?;
    let key = format!("{row},{col}");
    Some(
        target
            .data
            .get(&key)
            .and_then(|c| c.value.clone())
            .unwrap_or_default(),
    )
}

/// Pre-aggregates a rectangular range on the target worksheet to a single
/// number, matching the legacy semantics of `func_name`.
fn aggregate_range(
    func_name: &str,
    target: &Worksheet,
    start: (u32, u32),
    end: (u32, u32),
) -> Option<String> {
    let mut nums: Vec<f64> = Vec::new();
    let mut non_empty = 0u64;
    let mut total = 0u64;
    let mut key = String::with_capacity(32);
    for row in start.0..=end.0 {
        for col in start.1..=end.1 {
            total += 1;
            key.clear();
            use std::fmt::Write;
            let _ = write!(&mut key, "{row},{col}");
            if let Some(cell) = target.data.get(&key) {
                if let Some(ref v) = cell.value {
                    if !v.trim().is_empty() {
                        non_empty += 1;
                        if let Ok(num) = v.trim().parse::<f64>() {
                            nums.push(num);
                        }
                    }
                }
            }
        }
    }
    match func_name {
        "SUM" => Some(sum(&nums)),
        "PRODUCT" => {
            if nums.is_empty() {
                None
            } else {
                Some(nums.iter().product::<f64>().to_string())
            }
        }
        "AVERAGE" => {
            if nums.is_empty() {
                None
            } else {
                Some((nums.iter().sum::<f64>() / nums.len() as f64).to_string())
            }
        }
        "COUNT" => Some(nums.len().to_string()),
        "COUNTA" => Some(non_empty.to_string()),
        "COUNTBLANK" => Some((total - non_empty).to_string()),
        "MAX" => nums.iter().cloned().reduce(f64::max).map(|v| v.to_string()),
        "MIN" => nums.iter().cloned().reduce(f64::min).map(|v| v.to_string()),
        "MEDIAN" => {
            if nums.is_empty() {
                None
            } else {
                nums.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let mid = nums.len() / 2;
                let median = if nums.len() % 2 == 0 {
                    (nums[mid - 1] + nums[mid]) / 2.0
                } else {
                    nums[mid]
                };
                Some(median.to_string())
            }
        }
        "STDEV" => {
            let n = nums.len();
            if n < 2 {
                return None;
            }
            let mean = nums.iter().sum::<f64>() / n as f64;
            let variance = nums.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (n as f64 - 1.0);
            Some(variance.sqrt().to_string())
        }
        "STDEVP" => {
            let n = nums.len();
            if n < 2 {
                return None;
            }
            let mean = nums.iter().sum::<f64>() / n as f64;
            let variance = nums.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n as f64;
            Some(variance.sqrt().to_string())
        }
        _ => None,
    }
}

fn sum(nums: &[f64]) -> String {
    nums.iter().sum::<f64>().to_string()
}

/// Comma-joins the raw values of a rectangular range (used by non-aggregate
/// functions such as `CONCATENATE(Sheet2!A1:A2)`).
fn range_text_values(target: &Worksheet, start: (u32, u32), end: (u32, u32)) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut key = String::with_capacity(32);
    for row in start.0..=end.0 {
        for col in start.1..=end.1 {
            key.clear();
            use std::fmt::Write;
            let _ = write!(&mut key, "{row},{col}");
            if let Some(v) = target.data.get(&key).and_then(|c| c.value.clone()) {
                parts.push(v);
            }
        }
    }
    parts.join(",")
}

fn is_sheet_name_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'.'
}

/// Splits `A1` or `A1:B3` off the front of a string, returning the token and
/// the rest. Used to resolve the cell part that follows a `Sheet!` qualifier.
fn split_ref_token(s: &str) -> (&str, &str) {
    let bytes = s.as_bytes();
    let mut i = 0usize;
    let n = bytes.len();
    if i < n && bytes[i] == b'$' {
        i += 1;
    }
    while i < n && bytes[i].is_ascii_alphabetic() {
        i += 1;
    }
    if i < n && bytes[i] == b'$' {
        i += 1;
    }
    while i < n && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i < n && bytes[i] == b':' {
        let range_rest = &s[i + 1..];
        let (_, after_end) = split_ref_token(range_rest);
        let end_len = i + 1 + (range_rest.len() - after_end.len());
        (&s[..end_len], after_end)
    } else {
        (&s[..i], &s[i..])
    }
}
#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::engine::ast::parse;
    use crate::engine::eval::eval_expr_in;
    use crate::types::{CellData, Worksheet};

    fn cell(text: &str) -> CellData {
        CellData {
            value: Some(text.to_string()),
            typed: Some(crate::engine::CellValue::parse(text)),
            formula: None,
            style: None,
            format: None,
            note: None,
            locked: None,
            has_comment: None,
            array_formula_id: None,
        }
    }

    fn sheet_pair() -> Vec<Worksheet> {
        let mut ws1 = Worksheet::default();
        ws1.name = "Sheet1".to_string();
        ws1.data = HashMap::from([("1,1".to_string(), cell("10"))]);
        let mut ws2 = Worksheet::default();
        ws2.name = "Sheet2".to_string();
        ws2.data = HashMap::from([
            ("1,1".to_string(), cell("20")),
            ("2,1".to_string(), cell("30")),
            ("3,1".to_string(), cell("40")),
        ]);
        vec![ws1, ws2]
    }

    #[test]
    fn single_cross_sheet_reference_resolves() {
        let worksheets = sheet_pair();
        let expr = parse("Sheet2!A1").unwrap();
        assert_eq!(eval_expr_in(&expr, &worksheets, 0).display(), "20");
    }

    #[test]
    fn case_insensitive_sheet_lookup() {
        let worksheets = sheet_pair();
        let expr = parse("sheet2!a1").unwrap();
        assert_eq!(eval_expr_in(&expr, &worksheets, 0).display(), "20");
    }

    #[test]
    fn unknown_sheet_yields_ref_error() {
        let worksheets = sheet_pair();
        let expr = parse("Other!A1").unwrap();
        assert!(eval_expr_in(&expr, &worksheets, 0).display().contains("REF"));
    }

    #[test]
    fn quoted_sheet_token_passes_through() {
        let raw = "SUM(\"Sheet2!A1\")".to_string();
        assert_eq!(resolve_sheet_args(&raw, "SUM", &sheet_pair()), raw);
    }

    #[test]
    fn range_aggregate_collapses_cross_sheet_range() {
        let worksheets = sheet_pair();
        let expr = parse("SUM(Sheet2!A1:A3)").unwrap();
        assert_eq!(eval_expr_in(&expr, &worksheets, 0).display(), "90");
    }

    #[test]
    fn unknown_sheet_argument_passes_through_verbatim() {
        let raw = "SUM(Other!A1:A3)".to_string();
        assert_eq!(resolve_sheet_args(&raw, "SUM", &sheet_pair()), raw);
    }
}
