//! Row-visibility computation for worksheet filters (#790).
//!
//! The frontend stores filter configurations (checkbox lists and/or value
//! conditions per column) but the server never translated them into the
//! `hidden_rows` the grid consumes. This module evaluates every active filter
//! against the worksheet's data and produces the sorted list of rows the
//! frontend must hide, mirroring spreadsheet-expectation semantics: a row is
//! hidden when at least one filter column excludes it.

use std::collections::BTreeSet;

use crate::engine::CellValue;
use crate::types::{FilterConfig, Worksheet};

/// Returns a sorted list of rows to hide, or an empty list when no filter is
/// active or every row passes. Empty data → empty result.
pub fn compute_hidden_rows(worksheet: &Worksheet) -> Vec<u32> {
    let Some(filters) = &worksheet.filters else {
        return Vec::new();
    };
    let active: Vec<(u32, &FilterConfig)> = filters
        .iter()
        .filter(|(_, config)| !is_inert(config))
        .map(|(col, config)| (*col, config))
        .collect();
    if active.is_empty() {
        return Vec::new();
    }

    let rows: BTreeSet<u32> = worksheet
        .data
        .keys()
        .filter_map(|k| k.split_once(',').and_then(|(r, _)| r.parse().ok()))
        .collect();

    let mut hidden: Vec<u32> = Vec::new();
    for row in rows {
        if active.iter().any(|(col, config)| {
            let cell = worksheet
                .data
                .get(&format!("{row},{col}"))
                .and_then(|c| c.typed.clone())
                .or_else(|| {
                    worksheet
                        .data
                        .get(&format!("{row},{col}"))
                        .and_then(|c| c.value.clone())
                        .as_deref()
                        .map(CellValue::parse)
                });
            !row_passes(config, cell)
        }) {
            hidden.push(row);
        }
    }
    hidden
}

/// Whether the filter configuration actually constrains anything.
fn is_inert(config: &FilterConfig) -> bool {
    let kind = config.filter_type.trim().to_ascii_lowercase();
    matches!(kind.as_str(), "" | "none" | "all" | "clearall")
        && config.values.is_empty()
        && config.condition.is_none()
        && config.value1.is_none()
}

/// Whether a row's cell passes every clause of one column filter.
fn row_passes(config: &FilterConfig, cell: Option<CellValue>) -> bool {
    let display = cell.as_ref().map(|c| c.display()).unwrap_or_default();

    if !config.values.is_empty() {
        let matched = config
            .values
            .iter()
            .any(|v| cell_values_equal(cell.as_ref(), v));
        if !matched {
            return false;
        }
    }

    let condition = config.condition.as_deref().map(|c| c.trim().to_ascii_lowercase());
    if condition.as_deref().is_none_or(|c| c.is_empty()) {
        return true;
    }
    if let Some(cond) = condition {
        let needle = config.value1.as_deref().unwrap_or("");
        let value = display.to_ascii_lowercase();
        let needle = needle.to_ascii_lowercase();
        let passes = match cond.as_str() {
            "contains" => value.contains(&needle),
            "notcontains" => !value.contains(&needle),
            "equals" => value == needle,
            "notequals" => value != needle,
            "beginswith" => value.starts_with(&needle),
            "endswith" => value.ends_with(&needle),
            "gt" => compare_by_value(cell.as_ref(), config.value1.as_deref(), NumericCompare::Gt),
            "lt" => compare_by_value(cell.as_ref(), config.value1.as_deref(), NumericCompare::Lt),
            "gte" => compare_by_value(cell.as_ref(), config.value1.as_deref(), NumericCompare::Gte),
            "lte" => compare_by_value(cell.as_ref(), config.value1.as_deref(), NumericCompare::Lte),
            "between" => {
                let lo = config.value1.as_deref();
                let hi = config.value2.as_deref();
                match (lo, hi) {
                    (Some(lo), Some(hi)) => {
                        compare_by_value(cell.as_ref(), Some(lo), NumericCompare::Gte)
                            && compare_by_value(cell.as_ref(), Some(hi), NumericCompare::Lte)
                    }
                    _ => true,
                }
            }
            // Unknown condition strings must never silently pass rows — a
            // typo in a filter definition would otherwise show everything.
            _ => false,
        };
        if !passes {
            return false;
        }
    }
    true
}

/// Equality for filter lists: numeric cells compare numerically, otherwise a
/// case-insensitive string comparison against the raw stored value.
fn cell_values_equal(cell: Option<&CellValue>, expected: &str) -> bool {
    match cell {
        Some(CellValue::Number(n)) => expected
            .trim()
            .parse::<f64>()
            .is_ok_and(|e| *n == e),
        Some(c) => c.display().eq_ignore_ascii_case(expected),
        None => expected.is_empty(),
    }
}

/// Comparison mode carried from a filter condition into [`compare_by_value`]
/// so the numeric and string fallback paths agree on direction.
#[derive(Clone, Copy)]
enum NumericCompare {
    Gt,
    Lt,
    Gte,
    Lte,
}

/// Numeric-aware comparison: when both sides parse as numbers the comparison
/// is numeric; otherwise it falls back to trimmed case-insensitive strings.
fn compare_by_value(
    cell: Option<&CellValue>,
    expected: Option<&str>,
    mode: NumericCompare,
) -> bool {
    let Some(expected) = expected else {
        return true;
    };
    let cell_num = match cell {
        Some(CellValue::Number(n)) => Some(*n),
        _ => cell
            .map(|c| c.display())
            .unwrap_or_default()
            .trim()
            .parse::<f64>()
            .ok(),
    };
    if let (Some(a), Some(b)) = (cell_num, expected.trim().parse::<f64>().ok()) {
        return match mode {
            NumericCompare::Gt => a > b,
            NumericCompare::Lt => a < b,
            NumericCompare::Gte => a >= b,
            NumericCompare::Lte => a <= b,
        };
    }
    let a = cell
        .map(|c| c.display())
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    let b = expected.trim().to_ascii_lowercase();
    match mode {
        NumericCompare::Gt => a > b,
        NumericCompare::Lt => a < b,
        NumericCompare::Gte => a >= b,
        NumericCompare::Lte => a <= b,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn ws_with(values: &[(&str, &str)], filters: Vec<(u32, FilterConfig)>) -> Worksheet {
        let mut data = HashMap::new();
        for (key, val) in values {
            data.insert(
                key.to_string(),
                crate::types::CellData {
                    value: Some(val.to_string()),
                    typed: Some(CellValue::parse(val)),
                    formula: None,
                    style: None,
                    format: None,
                    note: None,
                    locked: None,
                    has_comment: None,
                    array_formula_id: None,
                },
            );
        }
        Worksheet {
            data,
            filters: Some(filters.into_iter().collect()),
            ..Worksheet::default()
        }
    }

    fn list_filter(values: Vec<String>) -> FilterConfig {
        FilterConfig {
            filter_type: "values".to_string(),
            values,
            condition: None,
            value1: None,
            value2: None,
        }
    }

    fn condition_filter(condition: &str, value1: &str, value2: Option<&str>) -> FilterConfig {
        FilterConfig {
            filter_type: "condition".to_string(),
            values: Vec::new(),
            condition: Some(condition.to_string()),
            value1: Some(value1.to_string()),
            value2: value2.map(String::from),
        }
    }

    #[test]
    fn no_filters_hide_nothing() {
        let w = ws_with(&[("0,0", "10"), ("1,0", "20")], Vec::new());
        assert!(compute_hidden_rows(&w).is_empty());
    }

    #[test]
    fn value_list_hides_non_matching_rows() {
        let w = ws_with(
            &[("0,0", "a"), ("1,0", "b"), ("2,0", "c")],
            vec![(0, list_filter(vec!["a".to_string(), "c".to_string()]))],
        );
        assert_eq!(compute_hidden_rows(&w), vec![1]);
    }

    #[test]
    fn numeric_gte_condition() {
        let w = ws_with(
            &[("0,0", "10"), ("1,0", "20"), ("2,0", "30")],
            vec![(0, condition_filter("gte", "20", None))],
        );
        assert_eq!(compute_hidden_rows(&w), vec![0]);
    }

    #[test]
    fn contains_condition() {
        let w = ws_with(
            &[("0,0", "apple"), ("1,0", "banana"), ("2,0", "pineapple")],
            vec![(0, condition_filter("contains", "apple", None))],
        );
        assert_eq!(compute_hidden_rows(&w), vec![1]);
    }

    #[test]
    fn between_condition_hides_outside() {
        let w = ws_with(
            &[("0,0", "5"), ("1,0", "15"), ("2,0", "25")],
            vec![(0, condition_filter("between", "10", Some("20")))],
        );
        assert_eq!(compute_hidden_rows(&w), vec![0, 2]);
    }

    #[test]
    fn unknown_condition_never_silently_passes() {
        let w = ws_with(
            &[("0,0", "10"), ("1,0", "20")],
            vec![(0, condition_filter("bogusop", "10", None))],
        );
        assert_eq!(compute_hidden_rows(&w), vec![0, 1]);
    }
}