//! Contract tests for the pure helpers in `utils::resolve_from_scope_row`
//! (#2064): the `FILTER` predicate builder and the content-type map.
//!
//! `parse_filter` is a SQL-injection guard — the column half of the predicate
//! is interpolated into the query, so anything that is not an identifier must
//! be rejected here, not at the database.

use super::{get_content_type, parse_filter};

#[test]
fn the_identity_filter_is_translated_to_true() {
    assert_eq!(parse_filter("1=1").expect("identity"), ("TRUE".to_string(), vec![]));
    assert_eq!(
        parse_filter("  1=1  ").expect("identity with spaces"),
        ("TRUE".to_string(), vec![])
    );
}

#[test]
fn a_simple_filter_becomes_a_positional_predicate() {
    let (predicate, binds) = parse_filter("name=Alice").expect("valid filter");
    assert_eq!(predicate, "name = $1");
    assert_eq!(binds, vec!["Alice".to_string()]);
}

#[test]
fn whitespace_around_the_operator_is_trimmed() {
    let (predicate, binds) = parse_filter("  status = open  ").expect("valid filter");
    assert_eq!(predicate, "status = $1");
    assert_eq!(binds, vec!["open".to_string()]);
}

#[test]
fn only_the_first_equals_separates_column_from_value() {
    let (predicate, binds) = parse_filter("payload=a=b").expect("valid filter");
    assert_eq!(predicate, "payload = $1");
    assert_eq!(binds, vec!["a=b".to_string()]);
}

#[test]
fn a_missing_operator_is_rejected() {
    assert!(parse_filter("name").is_err());
    assert!(parse_filter("").is_err());
}

#[test]
fn a_non_identifier_column_is_rejected() {
    // Every one of these would break out of the predicate if interpolated.
    for filter in [
        "name; DROP TABLE users=1",
        "name'=1",
        "na-me=1",
        "na me=1",
        "(select 1)=1",
    ] {
        assert!(parse_filter(filter).is_err(), "`{filter}` must be rejected");
    }
}

#[test]
fn an_empty_column_is_rejected() {
    // `"=value"` has a valid value half but no column: interpolating it would
    // produce `" = $1"`, which the database rejects at runtime.
    for filter in ["=value", "  =value"] {
        assert!(parse_filter(filter).is_err(), "`{filter}` must be rejected");
    }
}

#[test]
fn the_value_is_never_interpolated() {
    // The value travels as a bind parameter, so quotes and dashes are harmless.
    let (predicate, binds) = parse_filter("comment=it's a 'test'").expect("valid filter");
    assert_eq!(predicate, "comment = $1");
    assert_eq!(binds, vec!["it's a 'test'".to_string()]);
}

#[test]
fn known_extensions_map_to_their_content_types() {
    let cases = [
        ("a.pdf", "application/pdf"),
        ("a.png", "image/png"),
        ("a.jpg", "image/jpeg"),
        ("a.jpeg", "image/jpeg"),
        ("a.gif", "image/gif"),
        ("a.svg", "image/svg+xml"),
        ("a.html", "text/html"),
        ("a.htm", "text/html"),
        ("a.css", "text/css"),
        ("a.js", "application/javascript"),
        ("a.json", "application/json"),
        ("a.xml", "application/xml"),
        ("a.txt", "text/plain"),
        ("a.csv", "text/csv"),
        ("a.zip", "application/zip"),
    ];
    for (path, expected) in cases {
        assert_eq!(get_content_type(path), expected, "path `{path}`");
    }
}

#[test]
fn office_documents_use_their_full_mime_types() {
    assert_eq!(
        get_content_type("report.xlsx"),
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
    );
    assert_eq!(
        get_content_type("report.docx"),
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
    );
}

#[test]
fn unknown_or_absent_extensions_fall_back_to_octet_stream() {
    for path in ["a.exe", "a", "", "dir/file.unknownext", "a.tar.gz"] {
        assert_eq!(
            get_content_type(path),
            "application/octet-stream",
            "path `{path}`"
        );
    }
}
