//! JSON export expectations.

use super::support_tests::{sheet, typed_cell, ws};
use super::*;
use botsheet_core::engine::value::CellValue;
use botsheet_core::types::Spreadsheet;

#[test]
fn export_is_pretty_printed_and_reparses_into_the_same_workbook() {
    let book = sheet("Budget", vec![ws("Q1", &[(0, 0, "alpha"), (1, 1, "42")])]);
    let json = export_to_json(&book);
    assert!(json.contains("\n  "), "expected pretty output: {json}");

    let reparsed: Spreadsheet = serde_json::from_str(&json).expect("export must round-trip");
    assert_eq!(reparsed.id, book.id);
    assert_eq!(reparsed.name, "Budget");
    assert_eq!(reparsed.worksheets.len(), 1);
    assert_eq!(
        reparsed.worksheets[0].data["1,1"].value.as_deref(),
        Some("42")
    );
}

#[test]
fn typed_values_survive_the_round_trip() {
    let mut worksheet = ws("S", &[]);
    worksheet
        .data
        .insert("0,0".to_string(), typed_cell("7", CellValue::Number(7.0)));
    let json = export_to_json(&sheet("Book", vec![worksheet]));
    let reparsed: Spreadsheet = serde_json::from_str(&json).expect("typed value round-trips");
    assert_eq!(
        reparsed.worksheets[0].data["0,0"].typed,
        Some(CellValue::Number(7.0))
    );
}

#[test]
fn empty_workbook_serializes_without_worksheets() {
    let json = export_to_json(&sheet("Empty", vec![]));
    let value: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert_eq!(value["worksheets"].as_array().map(Vec::len), Some(0));
    assert_eq!(value["name"], "Empty");
}
