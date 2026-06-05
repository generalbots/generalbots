//! Company-wide billing entries.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::platform::DeliveryPlatform;

/// Kind of billing entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BillingKind {
    /// Sale income.
    Sale,
    /// Delivery commission (iFood etc.).
    Commission,
    /// Subscription.
    Subscription,
    /// Marketing spend.
    Marketing,
    /// Bank fees aggregated.
    BankFee,
    /// Payroll cost.
    Payroll,
    /// Tax.
    Tax,
    /// Other.
    Other,
}

/// A single billing entry that flows into the dashboard.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BillingEntry {
    /// Server-assigned ID.
    pub id: Uuid,
    /// Tenant.
    pub tenant_id: String,
    /// Date the entry applies to.
    pub date: NaiveDate,
    /// Kind.
    pub kind: BillingKind,
    /// Amount (signed: income positive, expense negative).
    pub amount: Decimal,
    /// Currency.
    pub currency: String,
    /// Platform (if applicable).
    pub platform: Option<DeliveryPlatform>,
    /// Free-form description.
    pub description: Option<String>,
    /// Recorded at.
    pub recorded_at: DateTime<Utc>,
}

/// A billing aggregation period (week / fortnight / month).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BillingPeriod {
    /// Server-assigned ID.
    pub id: Uuid,
    /// Tenant.
    pub tenant_id: String,
    /// Start (inclusive).
    pub start: NaiveDate,
    /// End (inclusive).
    pub end: NaiveDate,
    /// Entries inside the period.
    pub entries: Vec<BillingEntry>,
}

impl BillingPeriod {
    /// Sum income (positive entries).
    pub fn total_income(&self) -> Decimal {
        self.entries
            .iter()
            .filter(|e| e.amount.is_sign_positive())
            .map(|e| e.amount)
            .sum()
    }

    /// Sum expenses (negative entries).
    pub fn total_expense(&self) -> Decimal {
        self.entries
            .iter()
            .filter(|e| e.amount.is_sign_negative())
            .map(|e| e.amount)
            .sum()
    }

    /// Net = income - |expenses|.
    pub fn net(&self) -> Decimal {
        self.total_income() + self.total_expense()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn entry(amount: Decimal, kind: BillingKind) -> BillingEntry {
        BillingEntry {
            id: Uuid::new_v4(),
            tenant_id: "t1".to_string(),
            date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            kind,
            amount,
            currency: "BRL".to_string(),
            platform: None,
            description: None,
            recorded_at: Utc::now(),
        }
    }

    #[test]
    fn net_aggregates() {
        let period = BillingPeriod {
            id: Uuid::new_v4(),
            tenant_id: "t1".to_string(),
            start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            entries: vec![
                entry(dec!(1000.00), BillingKind::Sale),
                entry(dec!(-50.00), BillingKind::Commission),
            ],
        };
        assert_eq!(period.total_income(), dec!(1000.00));
        assert_eq!(period.total_expense(), dec!(-50.00));
        assert_eq!(period.net(), dec!(950.00));
    }
}
