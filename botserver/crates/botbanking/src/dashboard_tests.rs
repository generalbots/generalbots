//! Dashboard KPIs and their period-over-period deltas.

use chrono::NaiveDate;
use rust_decimal_macros::dec;
use uuid::Uuid;

use super::{build_dashboard, BillingDashboard};
use crate::billing::{BillingEntry, BillingKind, BillingPeriod};

fn date() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 1, 1).expect("date")
}

fn entry(amount: rust_decimal::Decimal) -> BillingEntry {
    BillingEntry {
        id: Uuid::new_v4(),
        tenant_id: "t1".to_string(),
        date: date(),
        kind: BillingKind::Sale,
        amount,
        currency: "BRL".to_string(),
        platform: None,
        description: None,
        recorded_at: chrono::Utc::now(),
    }
}

fn period(entries: Vec<BillingEntry>) -> BillingPeriod {
    BillingPeriod {
        id: Uuid::new_v4(),
        tenant_id: "t1".to_string(),
        start: date(),
        end: date(),
        entries,
    }
}

#[test]
fn an_empty_period_reports_zero_kpis_and_no_delta() {
    let summary = build_dashboard(&period(vec![]), None);
    assert_eq!(summary.kpis.len(), 3);
    for kpi in &summary.kpis {
        assert_eq!(kpi.value, dec!(0), "{}", kpi.code);
        assert_eq!(kpi.pop_delta_pct, None, "{}", kpi.code);
        assert_eq!(kpi.unit, "BRL");
    }
    assert!(!summary.period_id.is_empty());
    assert!(summary.computed_at <= chrono::Utc::now());
}

#[test]
fn expenses_are_reported_as_absolute_values() {
    let summary = build_dashboard(&period(vec![entry(dec!(1000)), entry(dec!(-250))]), None);
    assert_eq!(summary.kpi("gross_income").expect("kpi").value, dec!(1000));
    assert_eq!(summary.kpi("total_expense").expect("kpi").value, dec!(250));
    assert_eq!(summary.kpi("net").expect("kpi").value, dec!(750));
}

#[test]
fn a_zero_previous_period_suppresses_the_delta() {
    let previous = period(vec![]);
    let summary = build_dashboard(&period(vec![entry(dec!(1000))]), Some(&previous));
    for code in ["gross_income", "total_expense", "net"] {
        assert_eq!(
            summary.kpi(code).expect("kpi").pop_delta_pct,
            None,
            "{code} must not divide by zero"
        );
    }
}

#[test]
fn a_doubled_income_is_a_hundred_percent_delta() {
    let previous = period(vec![entry(dec!(500))]);
    let summary = build_dashboard(&period(vec![entry(dec!(1000))]), Some(&previous));
    assert_eq!(
        summary.kpi("gross_income").expect("kpi").pop_delta_pct,
        Some(dec!(100))
    );
}

#[test]
fn the_net_delta_is_measured_against_the_previous_net() {
    let previous = period(vec![entry(dec!(-100))]);
    let summary = build_dashboard(&period(vec![entry(dec!(900))]), Some(&previous));
    // (900 - (-100)) / |(-100)| * 100 = 1000%
    assert_eq!(
        summary.kpi("net").expect("kpi").pop_delta_pct,
        Some(dec!(1000))
    );
}

#[test]
fn a_missing_kpi_code_returns_none() {
    let summary = build_dashboard(&period(vec![]), None);
    assert!(summary.kpi("net_margin").is_none());
    assert!(summary.kpi("").is_none());
}

#[test]
fn the_dashboard_alias_is_the_summary() {
    let summary: BillingDashboard = build_dashboard(&period(vec![]), None);
    assert_eq!(summary.kpis.len(), 3);
}

#[test]
fn the_period_totals_ignore_kind_and_respect_the_amount_sign() {
    let mut commission = entry(dec!(-30));
    commission.kind = BillingKind::Commission;
    let mut payroll = entry(dec!(-70));
    payroll.kind = BillingKind::Payroll;
    let billing = period(vec![entry(dec!(200)), commission, payroll]);
    assert_eq!(billing.total_income(), dec!(200));
    assert_eq!(billing.total_expense(), dec!(-100));
    assert_eq!(billing.net(), dec!(100));
}

#[test]
fn a_zero_amount_counts_as_income_and_cancels_out() {
    let billing = period(vec![entry(dec!(0))]);
    assert_eq!(billing.total_income(), dec!(0));
    assert_eq!(billing.total_expense(), dec!(0));
    assert_eq!(billing.net(), dec!(0));
}
