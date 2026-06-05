//! Dashboard summary and KPIs for the billing overview.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::billing::BillingPeriod;

/// Single KPI shown on the dashboard.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardKpi {
    /// KPI code (e.g. `"gross_income"`, `"net_margin"`).
    pub code: String,
    /// Display label.
    pub label: String,
    /// Numeric value (decimal).
    pub value: Decimal,
    /// Unit (e.g. `"BRL"`, `"%"`).
    pub unit: String,
    /// Period-over-period delta percentage (signed).
    pub pop_delta_pct: Option<Decimal>,
}

/// Full dashboard summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardSummary {
    /// Period ID this summary is for.
    pub period_id: String,
    /// All KPIs.
    pub kpis: Vec<DashboardKpi>,
    /// Computed at.
    pub computed_at: DateTime<Utc>,
}

impl DashboardSummary {
    /// Find a KPI by code.
    pub fn kpi(&self, code: &str) -> Option<&DashboardKpi> {
        self.kpis.iter().find(|k| k.code == code)
    }
}

/// Build a summary from a billing period.
pub fn build_dashboard(
    period: &BillingPeriod,
    previous: Option<&BillingPeriod>,
) -> DashboardSummary {
    let income = period.total_income();
    let expense = period.total_expense();
    let net = period.net();
    let pop = |current: Decimal, prev: Option<Decimal>| -> Option<Decimal> {
        let prev = prev?;
        if prev.is_zero() {
            return None;
        }
        Some(((current - prev) / prev.abs()) * Decimal::new(100, 0))
    };
    let prev_income = previous.map(|p| p.total_income());
    let prev_expense = previous.map(|p| p.total_expense());
    let prev_net = previous.map(|p| p.net());
    DashboardSummary {
        period_id: period.id.to_string(),
        kpis: vec![
            DashboardKpi {
                code: "gross_income".to_string(),
                label: "Gross income".to_string(),
                value: income,
                unit: "BRL".to_string(),
                pop_delta_pct: pop(income, prev_income),
            },
            DashboardKpi {
                code: "total_expense".to_string(),
                label: "Total expenses".to_string(),
                value: expense.abs(),
                unit: "BRL".to_string(),
                pop_delta_pct: pop(expense.abs(), prev_expense.map(|v| v.abs())),
            },
            DashboardKpi {
                code: "net".to_string(),
                label: "Net".to_string(),
                value: net,
                unit: "BRL".to_string(),
                pop_delta_pct: pop(net, prev_net),
            },
        ],
        computed_at: Utc::now(),
    }
}

/// Alias kept for short import paths.
pub type BillingDashboard = DashboardSummary;

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;

    use super::super::billing::{BillingEntry, BillingKind};

    fn period(income: Decimal, expense: Decimal) -> BillingPeriod {
        let mut entries = Vec::new();
        if !income.is_zero() {
            entries.push(BillingEntry {
                id: uuid::Uuid::new_v4(),
                tenant_id: "t1".to_string(),
                date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                kind: BillingKind::Sale,
                amount: income,
                currency: "BRL".to_string(),
                platform: None,
                description: None,
                recorded_at: Utc::now(),
            });
        }
        if !expense.is_zero() {
            entries.push(BillingEntry {
                id: uuid::Uuid::new_v4(),
                tenant_id: "t1".to_string(),
                date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                kind: BillingKind::Commission,
                amount: -expense.abs(),
                currency: "BRL".to_string(),
                platform: None,
                description: None,
                recorded_at: Utc::now(),
            });
        }
        BillingPeriod {
            id: uuid::Uuid::new_v4(),
            tenant_id: "t1".to_string(),
            start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            entries,
        }
    }

    #[test]
    fn build_dashboard_aggregates() {
        let p = period(dec!(1000), dec!(100));
        let s = build_dashboard(&p, None);
        assert!(s.kpi("gross_income").is_some());
        assert!(s.kpi("net").is_some());
        assert_eq!(s.kpi("net").unwrap().value, dec!(900));
    }
}
