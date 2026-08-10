//! Reconciliation engine.

use chrono::{DateTime, Duration, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::transaction::{BankTransaction, TransactionSide};

/// Confidence level of a match.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchConfidence {
    /// Exact amount + reference + date.
    Exact,
    /// Exact amount + same day.
    High,
    /// Within settlement window.
    Medium,
    /// Same amount, far apart — manual review.
    Low,
}

/// A single bank↔platform match.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReconciliationMatch {
    /// Bank transaction ID.
    pub bank_id: Uuid,
    /// Platform transaction ID.
    pub platform_id: Uuid,
    /// Confidence.
    pub confidence: MatchConfidence,
    /// Amount difference (always non-negative).
    pub delta: Decimal,
}

/// A reconciliation run for one tenant / period.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reconciliation {
    /// Server-assigned run ID.
    pub id: Uuid,
    /// Tenant.
    pub tenant_id: String,
    /// Start of the reconciliation window.
    pub window_start: DateTime<Utc>,
    /// End of the reconciliation window.
    pub window_end: DateTime<Utc>,
    /// All matches produced by the run.
    pub matches: Vec<ReconciliationMatch>,
    /// Transactions that could not be matched.
    pub unmatched: Vec<Uuid>,
    /// When the run completed.
    pub completed_at: DateTime<Utc>,
}

/// Errors raised by the engine.
#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum ReconcileError {
    /// Window end is before window start.
    #[error("window end {end} precedes start {start}")]
    InvalidWindow {
        /// Start.
        start: DateTime<Utc>,
        /// End.
        end: DateTime<Utc>,
    },
}

/// Match bank transactions against platform transactions, returning a
/// list of [`ReconciliationMatch`] and the IDs of unmatched bank txs.
pub fn match_transactions(
    bank: &[BankTransaction],
    platform: &[BankTransaction],
    window: Duration,
) -> Vec<ReconciliationMatch> {
    let mut matches = Vec::new();
    let mut used_platform: std::collections::HashSet<Uuid> = std::collections::HashSet::new();
    for b in bank {
        if b.side != TransactionSide::Bank {
            continue;
        }
        let mut best: Option<(Uuid, MatchConfidence, Decimal)> = None;
        for p in platform {
            if p.side != TransactionSide::Platform {
                continue;
            }
            if used_platform.contains(&p.id) {
                continue;
            }
            if p.platform != b.platform {
                continue;
            }
            let gap = (b.booked_at - p.booked_at).abs();
            if gap > window {
                continue;
            }
            let delta = (b.amount - p.amount).abs();
            let confidence = if delta.is_zero() && gap < Duration::hours(24) {
                MatchConfidence::Exact
            } else if delta.is_zero() && gap <= window {
                MatchConfidence::High
            } else if delta < Decimal::new(5, 2) {
                MatchConfidence::Medium
            } else {
                MatchConfidence::Low
            };
            match &best {
                Some((_, _, best_delta)) if *best_delta <= delta => {}
                _ => best = Some((p.id, confidence, delta)),
            }
        }
        if let Some((p_id, conf, delta)) = best {
            used_platform.insert(p_id);
            matches.push(ReconciliationMatch {
                bank_id: b.id,
                platform_id: p_id,
                confidence: conf,
                delta,
            });
        }
    }
    matches
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::platform::DeliveryPlatform;
    use rust_decimal_macros::dec;

    fn bank(id: Uuid, amount: Decimal, when: DateTime<Utc>, plat: Option<DeliveryPlatform>) -> BankTransaction {
        BankTransaction {
            id,
            tenant_id: "t1".to_string(),
            side: TransactionSide::Bank,
            platform: plat,
            external_id: format!("b-{id}"),
            kind: super::super::transaction::TransactionKind::Credit,
            amount,
            currency: "BRL".to_string(),
            booked_at: when,
            counterparty: None,
            memo: None,
            status: super::super::transaction::ReconcileStatus::Unmatched,
            matched_with: None,
        }
    }

    fn plat(id: Uuid, amount: Decimal, when: DateTime<Utc>, plat: DeliveryPlatform) -> BankTransaction {
        let mut t = bank(id, amount, when, Some(plat));
        t.side = TransactionSide::Platform;
        t.external_id = format!("p-{id}");
        t
    }

    #[test]
    fn exact_match() {
        let now = Utc::now();
        let b = vec![bank(Uuid::new_v4(), dec!(100.00), now, Some(DeliveryPlatform::IFood))];
        let p = vec![plat(Uuid::new_v4(), dec!(100.00), now, DeliveryPlatform::IFood)];
        let m = match_transactions(&b, &p, Duration::days(14));
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].confidence, MatchConfidence::Exact);
    }
}
