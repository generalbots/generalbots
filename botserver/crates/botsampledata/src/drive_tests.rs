//! Offline tests for the cash-flow CSV generator (issue #2075). The file is
//! written to Drive for the banking-import scenario, so its header and row
//! shapes are part of the contract.

use super::build_cashflow_csv;

fn lines(csv: &str) -> Vec<&str> {
    csv.lines().collect()
}

#[test]
fn the_header_uses_the_brazilian_column_names() {
    let csv = build_cashflow_csv(2026, 1);
    assert_eq!(lines(&csv)[0], "data,historico,valor,tipo");
}

#[test]
fn every_month_gets_six_rows_after_the_header() {
    for month in 1..=12 {
        let csv = build_cashflow_csv(2026, month);
        assert_eq!(lines(&csv).len(), 7, "month {month}");
    }
}

#[test]
fn the_dates_are_iso_formatted_with_a_zero_padded_month() {
    let csv = build_cashflow_csv(2026, 3);
    let rows = lines(&csv);
    let dates: Vec<&str> = rows
        .iter()
        .skip(1)
        .map(|line| line.split(',').next().expect("date column"))
        .collect();

    assert_eq!(
        dates,
        vec![
            "2026-03-02",
            "2026-03-05",
            "2026-03-08",
            "2026-03-12",
            "2026-03-15",
            "2026-03-20"
        ]
    );
}

#[test]
fn the_rows_carry_two_income_and_four_expense_entries() {
    let csv = build_cashflow_csv(2026, 1);
    let rows: Vec<&str> = lines(&csv).into_iter().skip(1).collect();

    let income = rows.iter().filter(|r| r.ends_with("receita")).count();
    let expense = rows.iter().filter(|r| r.ends_with("despesa")).count();

    assert_eq!(income, 2);
    assert_eq!(expense, 4);
    assert!(rows[0].contains("\"Client contract\",5000.00"));
    assert!(rows[1].contains("\"Consulting hours\",2500.00"));
}

#[test]
fn february_is_the_only_month_with_a_twenty_eight_day_baseline() {
    assert!(build_cashflow_csv(2026, 2).contains("\"Telecom\",-28.00,despesa"));
    assert!(build_cashflow_csv(2026, 4).contains("\"Telecom\",-30.00,despesa"));
    assert!(build_cashflow_csv(2026, 12).contains("\"Telecom\",-30.00,despesa"));
}

#[test]
fn a_single_digit_month_keeps_two_date_digits() {
    let csv = build_cashflow_csv(2026, 9);
    assert!(csv.contains("2026-09-02"));
    assert!(!csv.contains("2026-9-2"));
}
