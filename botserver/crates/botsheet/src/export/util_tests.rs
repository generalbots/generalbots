//! Shared exporter-helper expectations.

use super::support_tests::{cell, typed_cell};
use super::util::*;
use botsheet_core::engine::value::CellValue;
use botsheet_core::types::CellData;

#[test]
fn column_letters_cover_every_decade_boundary() {
    let cases = [
        (0, "A"),
        (1, "B"),
        (25, "Z"),
        (26, "AA"),
        (27, "AB"),
        (701, "ZZ"),
        (702, "AAA"),
    ];
    for (col, expected) in cases {
        assert_eq!(column_to_letter(col), expected, "column {col}");
    }
}

#[test]
fn typed_number_exports_as_a_number() {
    let number = typed_cell("1,234.50", CellValue::Number(1234.5));
    assert_eq!(cell_number(&number), Some(1234.5));
}

#[test]
fn typed_text_never_exports_as_a_number() {
    let text = typed_cell("0123", CellValue::Text("0123".to_string()));
    assert_eq!(cell_number(&text), None);
}

#[test]
fn untyped_cells_parse_their_display_value() {
    assert_eq!(cell_number(&cell(" 42 ")), Some(42.0));
    assert_eq!(cell_number(&cell("-0.5")), Some(-0.5));
    assert_eq!(cell_number(&cell("abc")), None);
    assert_eq!(cell_number(&cell("")), None);
}

#[test]
fn a_cell_without_a_display_value_has_no_number() {
    let empty = CellData { value: None, ..cell("") };
    assert_eq!(cell_number(&empty), None);
}
