//! Core formula evaluator tests (#2067).
//!
//! The math/logic/lookup dispatchers had no tests; these pin the entry point
//! `evaluate_formula` (typed engine first, legacy dispatcher as fallback) for
//! the functions a bot writes most often.

use super::evaluate_formula;
use crate::types::{CellData, Worksheet};

fn sheet() -> Worksheet {
    Worksheet {
        name: "Sheet1".to_string(),
        ..Default::default()
    }
}

fn value(formula: &str) -> String {
    let result = evaluate_formula(formula, &sheet());
    assert!(result.error.is_none(), "{formula} -> {:?}", result.error);
    result.value
}

fn cell(raw: &str) -> CellData {
    CellData {
        value: Some(raw.to_string()),
        typed: raw
            .parse::<f64>()
            .ok()
            .map(crate::engine::value::CellValue::Number),
        formula: None,
        style: None,
        format: None,
        note: None,
        locked: None,
        has_comment: None,
        array_formula_id: None,
    }
}

fn with_cells(cells: &[(&str, &str)]) -> Worksheet {
    let mut worksheet = sheet();
    for (reference, raw) in cells {
        worksheet.data.insert((*reference).to_string(), cell(raw));
    }
    worksheet
}

#[test]
fn aggregates_over_literals() {
    assert_eq!(value("=SUM(1,2,3)"), "6");
    assert_eq!(value("=AVERAGE(2,4,6)"), "4");
    assert_eq!(value("=COUNT(1,2,3,4)"), "4");
    assert_eq!(value("=MAX(1,9,3)"), "9");
    assert_eq!(value("=MIN(4,2,8)"), "2");
}

#[test]
fn aggregates_over_a_cell_range() {
    let worksheet = with_cells(&[("0,0", "1"), ("1,0", "2"), ("2,0", "3")]);
    let sum = evaluate_formula("=SUM(A1:A3)", &worksheet);
    assert_eq!(sum.value, "6");
    assert!(sum.error.is_none());
    let average = evaluate_formula("=AVERAGE(A1:A3)", &worksheet);
    assert_eq!(average.value, "2");
}

#[test]
fn rounding_family() {
    assert_eq!(value("=ROUND(2.567,2)"), "2.57");
    assert_eq!(value("=ROUNDUP(2.001,0)"), "3");
    assert_eq!(value("=ROUNDDOWN(2.999,0)"), "2");
}

#[test]
fn arithmetic_and_roots() {
    assert_eq!(value("=ABS(-7)"), "7");
    assert_eq!(value("=SQRT(81)"), "9");
    assert_eq!(value("=POWER(2,10)"), "1024");
    assert_eq!(value("=MOD(10,3)"), "1");
}

#[test]
fn cell_references_and_operators() {
    // A1 is row 0 col 0; A2 is row 1 col 0.
    let worksheet = with_cells(&[("0,0", "5"), ("1,0", "3")]);
    assert_eq!(evaluate_formula("=A1+A2", &worksheet).value, "8");
    assert_eq!(evaluate_formula("=A1*A2", &worksheet).value, "15");
    assert_eq!(evaluate_formula("=A1-A2", &worksheet).value, "2");
}

#[test]
fn logic_functions_return_the_expected_branch() {
    // The legacy dispatcher upper-cases the expression, so text branches and
    // boolean literals come back upper-case.
    assert_eq!(value("=IF(1>0,\"yes\",\"no\")"), "YES");
    assert_eq!(value("=IF(1<0,\"yes\",\"no\")"), "NO");
    assert_eq!(value("=AND(1,1,0)"), "FALSE");
    assert_eq!(value("=OR(0,0,1)"), "TRUE");
    assert_eq!(value("=NOT(0)"), "TRUE");
}

#[test]
fn a_plain_value_is_returned_verbatim() {
    let result = evaluate_formula("hello", &sheet());
    assert_eq!(result.value, "hello");
    assert!(result.error.is_none());
}

#[test]
fn an_unknown_function_reports_an_error_value() {
    let result = evaluate_formula("=NOT_A_FUNCTION(1)", &sheet());
    assert!(
        result.value.starts_with('#'),
        "expected an error value, got {:?}",
        result.value
    );
}

#[test]
fn pt_br_aliases_evaluate_like_their_english_names() {
    let worksheet = with_cells(&[("0,0", "1"), ("1,0", "2"), ("2,0", "3")]);
    // UAT SHEET-04 (#2258): localised names must reach the same evaluator, not
    // fall through to #ERROR! because the function guard wants the English
    // prefix.
    assert_eq!(evaluate_formula("=SOMA(A1:A3)", &worksheet).value, "6");
    assert_eq!(evaluate_formula("=MÉDIA(A1:A3)", &worksheet).value, "2");
    assert_eq!(evaluate_formula("=MEDIA(A1:A3)", &worksheet).value, "2");
    assert_eq!(evaluate_formula("=MÁXIMO(A1:A3)", &worksheet).value, "3");
    assert_eq!(evaluate_formula("=MÍNIMO(A1:A3)", &worksheet).value, "1");
}
