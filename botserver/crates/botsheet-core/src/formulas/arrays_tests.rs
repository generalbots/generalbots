use super::*;
use crate::types::{CellData, Worksheet};
use std::collections::HashMap;

fn ws_with(values: &[(&str, &str)]) -> Worksheet {
    let mut data = HashMap::new();
    for (key, val) in values {
        data.insert(
            key.to_string(),
            CellData {
                value: Some(val.to_string()),
                typed: None,
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
        ..Worksheet::default()
    }
}

fn ev(expr: &str, ws: &Worksheet) -> Option<String> {
    evaluate_filter(expr, ws)
}

#[test]
fn true_criterion_keeps_all() {
    let ws = ws_with(&[("0,0", "1"), ("1,0", "2"), ("2,0", "3")]);
    assert_eq!(ev("FILTER(A1:A3,TRUE)", &ws), Some("1,2,3".to_string()));
}

#[test]
fn comparison_criterion_filters_rows() {
    let ws = ws_with(&[("0,0", "1"), ("1,0", "10"), ("2,0", "3")]);
    assert_eq!(ev("FILTER(A1:A3,A1:A3>2)", &ws), Some("10,3".to_string()));
}

#[test]
fn criterion_range_uses_truthiness() {
    let ws = ws_with(&[("0,0", "a"), ("1,0", "b"), ("2,0", "c"), ("0,1", "TRUE"), ("1,1", "FALSE"), ("2,1", "TRUE")]);
    assert_eq!(ev("FILTER(A1:A3,B1:B3)", &ws), Some("a,c".to_string()));
}

#[test]
fn scalar_criterion_expands() {
    let ws = ws_with(&[("0,0", "1"), ("1,0", "2")]);
    assert_eq!(ev("FILTER(A1:A2,\"x\")", &ws), Some("1,2".to_string()));
}

#[test]
fn not_equals_operator() {
    let ws = ws_with(&[("0,0", "a"), ("1,0", "b"), ("2,0", "a")]);
    assert_eq!(ev("FILTER(A1:A3,A1:A3<>\"a\")", &ws), Some("b".to_string()));
}
