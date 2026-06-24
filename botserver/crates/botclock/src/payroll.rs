//! Payroll integration: derive overtime / earnings from punches.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::rules::WorkdayRules;

/// One payroll period (typically a month or a fortnight).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PayrollPeriod {
    /// Server-assigned period ID.
    pub id: Uuid,
    /// Tenant.
    pub tenant_id: String,
    /// First day of the period (inclusive).
    pub start: NaiveDate,
    /// Last day of the period (inclusive).
    pub end: NaiveDate,
}

/// Breakdown of overtime within a payroll period.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OvertimeBreakdown {
    /// Minutes at the "first rate" (e.g. 50% in Brazil).
    pub first_rate_min: u32,
    /// Minutes at the "rest rate" (e.g. 100% in Brazil).
    pub rest_rate_min: u32,
}

/// A computed payroll summary for one employee in one period.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PayrollSummary {
    /// Employee user ID.
    pub employee_id: String,
    /// Period ID.
    pub period_id: Uuid,
    /// Total minutes worked (regular).
    pub regular_min: u32,
    /// Overtime breakdown.
    pub overtime: OvertimeBreakdown,
    /// Minutes in the time bank (optional).
    pub time_bank_min: i32,
    /// Computed at.
    pub computed_at: DateTime<Utc>,
}

impl PayrollSummary {
    /// Total worked minutes (regular + overtime).
    pub fn total_min(&self) -> u32 {
        self.regular_min
            .saturating_add(self.overtime.first_rate_min)
            .saturating_add(self.overtime.rest_rate_min)
    }
}

/// Errors raised by the payroll engine.
#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum PayrollError {
    /// End date precedes start date.
    #[error("payroll period end {end} precedes start {start}")]
    InvalidRange {
        /// Start.
        start: NaiveDate,
        /// End.
        end: NaiveDate,
    },
}

/// Compute a payroll summary for the given total minutes using the rules.
pub fn compute_summary(
    employee_id: impl Into<String>,
    period_id: Uuid,
    total_min: u32,
    rules: &WorkdayRules,
) -> PayrollSummary {
    let overtime = rules.overtime_minutes(total_min);
    let first = overtime.min(120);
    let rest = overtime.saturating_sub(120);
    let regular = total_min - overtime;
    PayrollSummary {
        employee_id: employee_id.into(),
        period_id,
        regular_min: regular,
        overtime: OvertimeBreakdown {
            first_rate_min: first,
            rest_rate_min: rest,
        },
        time_bank_min: 0,
        computed_at: Utc::now(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_summary_splits_overtime() {
        let rules = WorkdayRules::brazil_clt();
        let s = compute_summary("emp-1", Uuid::new_v4(), 600, &rules);
        assert_eq!(s.regular_min, 480);
        assert_eq!(s.overtime.first_rate_min, 120);
        assert_eq!(s.overtime.rest_rate_min, 0);
        assert_eq!(s.total_min(), 600);
    }
}
