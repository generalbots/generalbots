//! Contract tests for the CSV/TSV/JSON row mapping (#2064).
//!
//! `EXPORT` turns the `Dynamic` rows a query produced into a file. The escaping
//! rules decide whether a comma inside a value shifts the following columns, so
//! they are pinned against RFC 4180.

use super::*;
use botbasic_core::utils::json_value_to_dynamic;
use serde_json::json;

fn rows(value: Value) -> Dynamic {
    json_value_to_dynamic(&value)
}

fn temp_path(name: &str) -> String {
    std::env::temp_dir()
        .join(format!("gb_{name}_{}.csv", std::process::id()))
        .to_string_lossy()
        .into_owned()
}

#[test]
fn plain_values_are_written_verbatim() {
    assert_eq!(escape_csv_value("ana"), "ana");
    assert_eq!(escape_csv_value(""), "");
    assert_eq!(escape_csv_value("recife - pe"), "recife - pe");
}

#[test]
fn values_with_a_separator_are_quoted() {
    assert_eq!(escape_csv_value("Silva, Ana"), "\"Silva, Ana\"");
    assert_eq!(escape_csv_value("line1\nline2"), "\"line1\nline2\"");
    assert_eq!(escape_csv_value("carriage\rreturn"), "\"carriage\rreturn\"");
}

#[test]
fn embedded_quotes_are_doubled_not_dropped() {
    // RFC 4180: the surrounding quotes are kept and the inner ones doubled.
    assert_eq!(escape_csv_value("say \"hi\""), "\"say \"\"hi\"\"\"");
    assert_eq!(escape_csv_value("\""), "\"\"\"\"");
}

#[test]
fn headers_come_from_the_first_row_sorted() {
    let array = to_array(rows(json!([
        {"name": "ana", "amount": 10, "city": "recife"},
        {"name": "bruno"},
    ])));
    assert_eq!(get_headers_from_array(&array), ["amount", "city", "name"]);

    let empty: Array = Vec::new();
    assert!(get_headers_from_array(&empty).is_empty());
}

#[test]
fn export_csv_writes_a_header_and_escaped_rows() {
    let path = temp_path("csv_export");
    let data = rows(json!([
        {"name": "Silva, Ana", "amount": 10},
        {"name": "bruno", "amount": 5},
    ]));

    export_csv(&path, data).expect("export");
    let written = std::fs::read_to_string(&path).expect("read back");
    let _ = std::fs::remove_file(&path);

    assert_eq!(
        written,
        "amount,name\n10,\"Silva, Ana\"\n5,bruno\n"
    );
}

#[test]
fn export_csv_refuses_an_empty_dataset() {
    assert!(export_csv(&temp_path("csv_empty"), rows(json!([]))).is_err());
}

#[test]
fn export_tsv_does_not_quote_commas() {
    let path = temp_path("tsv_export");
    let data = rows(json!([{"name": "Silva, Ana", "amount": 10}]));

    export_tsv(&path, data).expect("export");
    let written = std::fs::read_to_string(&path).expect("read back");
    let _ = std::fs::remove_file(&path);

    // A tab-separated file has no comma ambiguity, so the value stays raw.
    assert_eq!(written, "amount\tname\n10\tSilva, Ana\n");
}

#[test]
fn export_json_is_a_pretty_printed_round_trip() {
    let path = temp_path("json_export");
    let data = rows(json!([{"name": "ana", "amount": 10}]));

    export_json(&path, data).expect("export");
    let written = std::fs::read_to_string(&path).expect("read back");
    let _ = std::fs::remove_file(&path);

    let parsed: Value = serde_json::from_str(&written).expect("valid JSON");
    assert_eq!(parsed, json!([{"amount": 10, "name": "ana"}]));
}

#[test]
fn a_scalar_dataset_is_a_single_row() {
    let scalar = Dynamic::from("only");
    assert_eq!(to_array(scalar).len(), 1);
}

#[test]
fn csv_lines_split_on_unquoted_separators_only() {
    assert_eq!(parse_csv_line("a,b,c"), ["a", "b", "c"]);
    assert_eq!(parse_csv_line("\"Silva, Ana\",10"), ["Silva, Ana", "10"]);
    // Surrounding spaces are trimmed from every field.
    assert_eq!(parse_csv_line(" a , b "), ["a", "b"]);
    assert_eq!(parse_csv_line(""), [""]);
}
