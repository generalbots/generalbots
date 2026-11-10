//! Table-driven tests for the cash-flow sheet rules.
//!
//! The sheet is written by accountants: amounts arrive as `R$ 1.234,56` or
//! `-12,00`, dates as `dd/mm/yyyy`, and the expense/income direction may come
//! from either the sign or a Portuguese kind label. These are the published
//! expectations the importer is allowed to rely on.

use super::*;
use chrono::NaiveDate;

fn row(date: &str, amount: &str, kind: Option<&str>) -> CashflowRow {
    CashflowRow {
        date: date.to_string(),
        description: Some("  Historico  ".to_string()),
        amount: amount.to_string(),
        kind: kind.map(str::to_string),
        category: None,
    }
}

#[test]
fn brazilian_amounts_are_parsed_with_their_separators() {
    let cases = [
        ("R$ 1.234,56", Some(1234.56)),
        ("R$ 1234,56", Some(1234.56)),
        ("1.234,56", Some(1234.56)),
        ("1234,56", Some(1234.56)),
        ("-12,00", Some(-12.0)),
        ("-R$ 12,00", Some(-12.0)),
        (" 100 ", Some(100.0)),
        ("1,234.56", Some(1234.56)),
        ("1234.56", Some(1234.56)),
    ];
    for (input, expected) in cases {
        assert_eq!(parse_br_amount(input), expected, "amount {input:?}");
    }
}

#[test]
fn a_lone_dot_is_read_as_a_decimal_point() {
    // "1.234" is ambiguous; the importer follows the decimal reading. A
    // thousands separator is only inferred when a comma is also present.
    assert_eq!(parse_br_amount("1.234"), Some(1.234));
}

#[test]
fn unparseable_amounts_are_rejected_instead_of_becoming_zero() {
    for input in ["", "   ", "abc", "R$", "--", "1,2,3"] {
        assert_eq!(parse_br_amount(input), None, "amount {input:?}");
    }
}

#[test]
fn dates_are_accepted_in_the_three_published_formats() {
    let expected = NaiveDate::from_ymd_opt(2026, 8, 1).expect("valid date");
    for input in ["2026-08-01", "01/08/2026", "2026/08/01", "  01/08/2026  "] {
        assert_eq!(parse_br_date(input), Some(expected), "date {input:?}");
    }
}

#[test]
fn invalid_dates_are_rejected() {
    for input in ["", "31/02/2026", "2026-13-01", "01-08-2026", "yesterday"] {
        assert_eq!(parse_br_date(input), None, "date {input:?}");
    }
}

#[test]
fn the_expense_direction_comes_from_the_sign_or_the_kind() {
    let cases = [
        (-1.0, "", true),
        (-100.5, "receita", true),
        (5.0, "despesa", true),
        (5.0, "saida", true),
        (5.0, "debito", true),
        (5.0, "receita", false),
        (5.0, "", false),
        // Callers normalize the label first; the matcher is case-sensitive.
        (5.0, "DESPESA", false),
    ];
    for (amount, kind, expected) in cases {
        assert_eq!(is_expense(kind, amount), expected, "kind {kind:?} amount {amount}");
    }
}

#[test]
fn rounding_is_half_away_from_zero_at_two_decimals() {
    let cases = [
        (1.234, 1.23),
        (1.235, 1.24),
        (-1.234, -1.23),
        (10.999, 11.0),
        (0.0, 0.0),
    ];
    for (input, expected) in cases {
        assert_eq!(round2(input), expected, "value {input}");
    }
}

#[test]
fn an_expense_row_is_normalized_to_a_negative_amount() {
    let (date, description, amount, category, kind) =
        normalize_row(&row("01/08/2026", "R$ 1.234,56", Some("Despesa"))).expect("valid row");
    assert_eq!(date, NaiveDate::from_ymd_opt(2026, 8, 1).expect("valid date"));
    assert_eq!(description, "Historico");
    assert_eq!(amount, -1234.56);
    assert_eq!(category, "despesa");
    assert_eq!(kind, "despesa");
}

#[test]
fn an_income_row_is_normalized_to_a_positive_amount() {
    let (_, _, amount, category, kind) =
        normalize_row(&row("2026-08-01", "100,00", None)).expect("valid row");
    assert_eq!(amount, 100.0);
    assert_eq!(category, "receita");
    assert_eq!(kind, "");
}

#[test]
fn a_positive_amount_with_an_expense_kind_still_imports_as_an_expense() {
    let (_, _, amount, category, _) =
        normalize_row(&row("01/08/2026", "50,00", Some("SAIDA"))).expect("valid row");
    assert_eq!(amount, -50.0);
    assert_eq!(category, "despesa");
}

#[test]
fn an_invalid_row_is_reported_with_its_reason() {
    let bad_date = normalize_row(&row("31/02/2026", "10,00", None)).expect_err("invalid date");
    assert!(
        bad_date["reason"]
            .as_str()
            .unwrap_or_default()
            .contains("invalid date"),
        "{bad_date}"
    );

    let bad_amount = normalize_row(&row("01/08/2026", "abc", None)).expect_err("invalid amount");
    assert!(
        bad_amount["reason"]
            .as_str()
            .unwrap_or_default()
            .contains("invalid amount"),
        "{bad_amount}"
    );
}

#[test]
fn csv_rows_are_read_through_the_english_headers() {
    let csv = "date,description,amount,type\n\
               01/08/2026,PIX recebido,\"R$ 1.000,00\",receita\n";
    let rows = parse_csv_rows(csv);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].date, "01/08/2026");
    assert_eq!(rows[0].description.as_deref(), Some("PIX recebido"));
    assert_eq!(rows[0].amount, "R$ 1.000,00");
    assert_eq!(rows[0].kind.as_deref(), Some("receita"));
}

#[test]
fn csv_rows_are_read_through_the_portuguese_headers() {
    let csv = "Data,Historico,Valor,Tipo\n15/07/2026,TARIFA,\"-12,00\",despesa\n";
    let rows = parse_csv_rows(csv);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].date, "15/07/2026");
    assert_eq!(rows[0].description.as_deref(), Some("TARIFA"));
    assert_eq!(rows[0].amount, "-12,00");
    assert_eq!(rows[0].kind.as_deref(), Some("despesa"));
}

#[test]
fn the_vencimento_alias_is_accepted_as_the_date_column() {
    let csv = "vencimento,descricao,valor\n01/08/2026,ALUGUEL,\"2.000,00\"\n";
    let rows = parse_csv_rows(csv);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].date, "01/08/2026");
}

#[test]
fn rows_without_a_date_or_an_amount_are_skipped() {
    let csv = "date,description,amount\n\
               ,missing date,10,00\n\
               01/08/2026,missing amount,\n\
               01/08/2026,kept,10,00\n";
    let rows = parse_csv_rows(csv);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].description.as_deref(), Some("kept"));
}

#[test]
fn blank_lines_and_missing_columns_are_tolerated() {
    assert!(parse_csv_rows("").is_empty());
    assert!(parse_csv_rows("date,description,amount").is_empty());
    assert!(parse_csv_rows("date,description,amount\n\n\n").is_empty());

    let rows = parse_csv_rows("date,amount\n01/08/2026,\"10,00\"\n");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].description.as_deref(), Some(""));
    assert_eq!(rows[0].kind.as_deref(), Some(""));
}
