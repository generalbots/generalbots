//! Offline tests for the lead CSV surface (issue #2075, CRM wave): cell
//! quoting, header mapping and amount parsing. No database, no filesystem —
//! the parsers work on an in-memory `&str`.

use super::*;

#[test]
fn plain_cells_are_emitted_without_quotes() {
    let cases = [
        ("", ""),
        ("Acme", "Acme"),
        ("São Paulo", "São Paulo"),
        ("a;b", "a;b"),
    ];

    for (input, expected) in cases {
        assert_eq!(csv_cell(input), expected, "input {input:?}");
    }
}

#[test]
fn cells_with_separators_or_quotes_are_quoted_and_escaped() {
    let cases = [
        ("a,b", "\"a,b\""),
        ("say \"hi\"", "\"say \"\"hi\"\"\""),
        ("line\nbreak", "\"line\nbreak\""),
    ];

    for (input, expected) in cases {
        assert_eq!(csv_cell(input), expected, "input {input:?}");
    }
}

#[test]
fn absent_optionals_export_as_empty_cells() {
    assert_eq!(csv_opt(&None), "");
    assert_eq!(csv_opt(&Some("value".to_string())), "value");
    assert_eq!(csv_opt(&Some("a,b".to_string())), "\"a,b\"");
}

#[test]
fn numbers_export_without_trailing_noise() {
    assert_eq!(csv_number(None), "");
    assert_eq!(csv_number(Some(1.5)), "1.5");
    assert_eq!(csv_number(Some(1_000.0)), "1000");
    assert_eq!(csv_number(Some(0.0)), "0");
}

#[test]
fn header_lookup_is_case_insensitive_and_trims_values() {
    let csv = "Title,First_Name,EMAIL,Value,Stage\n\
               Renewal,  Ana  ,ana@example.test,1000,proposal\n";

    let rows = parse_leads_csv(csv).expect("csv parses");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].title.as_deref(), Some("Renewal"));
    assert_eq!(rows[0].first_name.as_deref(), Some("Ana"));
    assert_eq!(rows[0].email.as_deref(), Some("ana@example.test"));
    assert_eq!(rows[0].value, Some(1_000.0));
    assert_eq!(rows[0].stage.as_deref(), Some("proposal"));
}

#[test]
fn blank_cells_become_none_instead_of_empty_strings() {
    let csv = "title,first_name,email,value\nDeal,   ,,\n";

    let rows = parse_leads_csv(csv).expect("csv parses");
    assert_eq!(rows[0].title.as_deref(), Some("Deal"));
    assert!(rows[0].first_name.is_none());
    assert!(rows[0].email.is_none());
    assert!(rows[0].value.is_none());
}

#[test]
fn unknown_columns_are_ignored() {
    let csv = "title,unexpected_column\nDeal,whatever\n";

    let rows = parse_leads_csv(csv).expect("csv parses");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].title.as_deref(), Some("Deal"));
    assert!(rows[0].email.is_none());
}

#[test]
fn an_empty_body_imports_nothing_instead_of_panicking() {
    let rows = parse_leads_csv("").expect("an empty body is not an error");
    assert!(rows.is_empty());
}

#[test]
fn a_header_only_body_imports_no_rows() {
    let rows = parse_leads_csv("title,email\n").expect("csv parses");
    assert!(rows.is_empty());
}

#[test]
fn published_amount_formats_are_parsed() {
    let cases = [
        ("1000", Some(1_000.0)),
        ("1000.50", Some(1_000.5)),
        ("1.234,56", Some(1_234.56)),
        ("1,234.56", Some(1_234.56)),
        ("1 234,56", Some(1_234.56)),
        ("R$ 2.500,00", Some(2_500.0)),
        ("0", Some(0.0)),
        ("", None),
        ("not a number", None),
    ];

    for (raw, expected) in cases {
        assert_eq!(parse_amount(raw), expected, "raw {raw:?}");
    }
}

#[test]
fn a_lone_comma_is_read_as_a_decimal_separator() {
    // pt-BR sheets write `1,5` for one and a half.
    assert_eq!(parse_amount("1,5"), Some(1.5));
    assert_eq!(parse_amount("1,234"), Some(1.234));
}

#[test]
fn thousands_separated_amounts_are_imported_instead_of_dropped() {
    let csv = "title,value\nBig deal,\"1,234.56\"\n";

    let rows = parse_leads_csv(csv).expect("csv parses");
    assert_eq!(
        rows[0].value,
        Some(1_234.56),
        "a thousands separator must not silently discard the deal value"
    );
}

#[test]
fn the_value_column_is_parsed_through_the_amount_rules() {
    let csv = "title,value\npt-BR,\"2.500,00\"\nUS,\"2,500.00\"\n";

    let rows = parse_leads_csv(csv).expect("csv parses");
    assert_eq!(rows[0].value, Some(2_500.0));
    assert_eq!(rows[1].value, Some(2_500.0));
}

#[test]
fn email_dedupe_keys_are_trimmed_and_case_folded() {
    assert_eq!(
        normalise_email(Some("  Ana@Example.TEST ")).as_deref(),
        Some("ana@example.test")
    );
    assert_eq!(normalise_email(Some("   ")), None);
    assert_eq!(normalise_email(None), None);
}

#[test]
fn a_repeated_row_inside_one_file_shares_the_dedupe_key() {
    let mut seen = std::collections::HashSet::new();
    let inserted: Vec<bool> = ["ana@example.test", " Ana@Example.test", ""]
        .iter()
        .map(|raw| match normalise_email(Some(*raw)) {
            Some(key) => seen.insert(key),
            None => false,
        })
        .collect();

    assert_eq!(inserted, vec![true, false, false]);
}
