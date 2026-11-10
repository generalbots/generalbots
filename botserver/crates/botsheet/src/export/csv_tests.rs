//! CSV export expectations.

use super::support_tests::{sheet, ws};
use super::*;

#[test]
fn grid_is_padded_to_the_last_used_row_and_column() {
    let book = sheet("Book", vec![ws("S", &[(0, 0, "a"), (2, 2, "c")])]);
    assert_eq!(export_to_csv(&book), "a,,\n,,\n,,c\n");
}

#[test]
fn empty_workbook_and_empty_worksheet_export_nothing() {
    assert_eq!(export_to_csv(&sheet("Book", vec![])), "");
    // A worksheet with no cells is one empty row.
    assert_eq!(export_to_csv(&sheet("Book", vec![ws("S", &[])])), "\n");
}

#[test]
fn separators_quotes_and_newlines_are_escaped() {
    let cases = [
        ("plain", "plain"),
        ("a,b", "\"a,b\""),
        ("say \"hi\"", "\"say \"\"hi\"\"\""),
        ("line1\nline2", "\"line1\nline2\""),
        ("", ""),
    ];
    for (input, expected) in cases {
        let book = sheet("Book", vec![ws("S", &[(0, 0, input)])]);
        assert_eq!(export_to_csv(&book), format!("{expected}\n"), "input {input:?}");
    }
}

#[test]
fn only_the_first_worksheet_is_exported() {
    let book = sheet(
        "Book",
        vec![ws("First", &[(0, 0, "one")]), ws("Second", &[(0, 0, "two")])],
    );
    assert_eq!(export_to_csv(&book), "one\n");
}

#[test]
fn keys_that_are_not_row_col_pairs_are_ignored_for_sizing() {
    let mut worksheet = ws("S", &[(0, 0, "kept")]);
    worksheet.data.insert("not-a-key".to_string(), super::support_tests::cell("x"));
    let book = sheet("Book", vec![worksheet]);
    assert_eq!(export_to_csv(&book), "kept\n");
}
