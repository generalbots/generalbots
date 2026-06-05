//! Banking reconciliation and delivery-platform integration.
//!
//! The crate models:
//! - Delivery platform connectors (iFood, Rappi, Uber Eats, etc.)
//! - Bank transactions and matching against platform settlements
//! - Company-wide billing dashboard aggregation

#![deny(missing_docs)]
#![deny(unsafe_code)]

pub mod platform;
pub mod transaction;
pub mod reconcile;
pub mod billing;
pub mod dashboard;

pub use platform::{DeliveryPlatform, PlatformConnector, ConnectorKind, ConnectorError};
pub use transaction::{BankTransaction, TransactionKind, TransactionSide, ReconcileStatus};
pub use reconcile::{
    Reconciliation, ReconciliationMatch, MatchConfidence, ReconcileError,
    match_transactions,
};
pub use billing::{BillingEntry, BillingKind, BillingPeriod};
pub use dashboard::{BillingDashboard, DashboardSummary, DashboardKpi};
