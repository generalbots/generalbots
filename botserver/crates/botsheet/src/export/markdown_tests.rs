//! Markdown table export expectations.

use super::support_tests::{sheet, ws};
use super::*;

#[test]
fn document_and_worksheet_headings_are_emitted() {
    let md = export_to_markdown(&sheet("Budget", vec![ws("Q1", &[(0, 0, "x")])]));
    assert!(md.starts_with("# Budget\n\n## Q1\n\n"));
}

#[test]
fn empty_worksheet_uses_the_marker() {
    let md = export_to_markdown(&sheet("Book", vec![ws("Empty", &[])]));
    assert!(md.contains("## Empty\n\n*Empty worksheet*\n\n"));
    assert!(!md.contains('|'));
}

#[test]
fn table_has_letter_header_and_separator_row() {
    let book = sheet("Book", vec![ws("S", &[(0, 0, "a"), (0, 1, "b")])]);
    let md = export_to_markdown(&book);
    assert!(md.contains("| A | B |\n| --- | --- |\n| a | b |\n"));
}

#[test]
fn pipes_in_values_are_escaped() {
    let book = sheet("Book", vec![ws("S", &[(0, 0, "a|b")])]);
    assert!(export_to_markdown(&book).contains("| a\\|b |"));
}

#[test]
fn every_worksheet_gets_its_own_table() {
    let book = sheet(
        "Book",
        vec![ws("One", &[(0, 0, "1")]), ws("Two", &[(0, 0, "2")])],
    );
    let md = export_to_markdown(&book);
    assert!(md.contains("## One"));
    assert!(md.contains("## Two"));
    assert_eq!(md.matches("| --- |").count(), 2);
}

#[test]
fn rows_are_padded_to_the_widest_row() {
    let book = sheet("Book", vec![ws("S", &[(0, 2, "c")])]);
    let md = export_to_markdown(&book);
    assert!(md.contains("| A | B | C |"));
    assert!(md.contains("|  |  | c |"));
}
