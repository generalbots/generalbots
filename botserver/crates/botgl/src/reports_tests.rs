//! Offline unit tests for the general-ledger report helpers (#2068).
//!
//! `format_currency` is what every GL statement uses to print money, so it must
//! always render exactly two decimals and never fall back to a locale-dependent
//! form. The chart of accounts is static data, but its *shape* (unique,
//! hierarchical codes inside the five known types) is what the trial-balance
//! rollups rely on.

use super::{format_currency, generate_chart_of_accounts};
use rust_decimal::Decimal;
use std::str::FromStr;

#[test]
fn currency_always_has_two_decimals() {
    assert_eq!(format_currency(&Decimal::from(0)), "0.00");
    assert_eq!(format_currency(&Decimal::from(7)), "7.00");
    assert_eq!(format_currency(&Decimal::from_str("1234.5").expect("literal")), "1234.50");
    assert_eq!(format_currency(&Decimal::from_str("0.1").expect("literal")), "0.10");
    // NOTE: a third decimal is truncated by `{:.2}`, not rounded
    // (1234.567 renders as "1234.56"). Money rounding is a separate decision and
    // is deliberately not pinned here — see the report's "known gaps" note.
}

#[test]
fn currency_keeps_the_sign_and_thousands_as_plain_digits() {
    assert_eq!(
        format_currency(&Decimal::from_str("-1234.56").expect("literal")),
        "-1234.56"
    );
    // No grouping separators: the value is consumed by CSV/JSON exports too.
    assert_eq!(
        format_currency(&Decimal::from_str("1234567.89").expect("literal")),
        "1234567.89"
    );
}

#[test]
fn chart_of_accounts_has_the_five_statement_groups() {
    let chart = generate_chart_of_accounts();
    assert_eq!(chart.len(), 24);

    let roots: Vec<&str> = chart
        .iter()
        .filter(|(code, _, _)| !code.contains('.'))
        .map(|(code, _, _)| *code)
        .collect();
    assert_eq!(roots, ["1", "2", "3", "4", "5"]);

    let types: std::collections::BTreeSet<&str> =
        chart.iter().map(|(_, _, kind)| *kind).collect();
    assert_eq!(
        types.into_iter().collect::<Vec<_>>(),
        ["asset", "equity", "expense", "liability", "revenue"]
    );
}

#[test]
fn chart_codes_are_unique_and_hierarchical() {
    let chart = generate_chart_of_accounts();

    let mut codes: Vec<&str> = chart.iter().map(|(code, _, _)| *code).collect();
    let total = codes.len();
    codes.sort_unstable();
    codes.dedup();
    assert_eq!(codes.len(), total, "account codes must be unique");

    let known: std::collections::HashSet<&str> = codes.iter().copied().collect();
    for (code, name, _) in &chart {
        assert!(!name.is_empty(), "account {code} has no name");
        if let Some((parent, _)) = code.rsplit_once('.') {
            assert!(
                known.contains(parent),
                "account {code} has no parent {parent} in the chart"
            );
        }
    }
}

#[test]
fn chart_groups_keep_their_account_type() {
    let chart = generate_chart_of_accounts();
    for (code, _, kind) in &chart {
        let root = code.split('.').next().unwrap_or(code);
        let expected = match root {
            "1" => "asset",
            "2" => "liability",
            "3" => "equity",
            "4" => "revenue",
            "5" => "expense",
            _ => "unknown-root",
        };
        assert_eq!(*kind, expected, "account {code} has the wrong type");
    }
}
