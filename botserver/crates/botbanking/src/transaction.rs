//! Bank transaction and reconciliation-status types.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::platform::DeliveryPlatform;

/// Kind of bank transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionKind {
    /// Debit (money out).
    Debit,
    /// Credit (money in).
    Credit,
    /// Bank fee.
    Fee,
    /// Interest.
    Interest,
    /// Tax (IOF, IR).
    Tax,
    /// Reversal of a prior transaction.
    Reversal,
}

/// Which side of a reconciliation the transaction came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionSide {
    /// Bank statement side.
    Bank,
    /// Platform settlement side.
    Platform,
}

/// Reconciliation status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReconcileStatus {
    /// Not yet matched.
    Unmatched,
    /// Auto-matched with high confidence.
    AutoMatched,
    /// Auto-matched but flagged for review.
    NeedsReview,
    /// Manually matched by an operator.
    ManuallyMatched,
    /// Rejected / written off.
    Rejected,
}

/// A single bank or platform transaction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BankTransaction {
    /// Server-assigned ID.
    pub id: Uuid,
    /// Tenant.
    pub tenant_id: String,
    /// Which side.
    pub side: TransactionSide,
    /// Platform (None for pure bank entries).
    pub platform: Option<DeliveryPlatform>,
    /// External ID from the source.
    pub external_id: String,
    /// Kind.
    pub kind: TransactionKind,
    /// Amount (signed: credits positive, debits negative).
    pub amount: Decimal,
    /// Currency (ISO 4217).
    pub currency: String,
    /// Booking date.
    pub booked_at: DateTime<Utc>,
    /// Counterparty.
    pub counterparty: Option<String>,
    /// Free-form memo.
    pub memo: Option<String>,
    /// Reconciliation status.
    pub status: ReconcileStatus,
    /// Matched transaction (if any).
    pub matched_with: Option<Uuid>,
}
