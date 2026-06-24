pub mod platform;
pub mod transaction;
pub mod reconcile;
pub mod billing;
pub mod dashboard;
pub mod db;
pub mod storage;
pub mod handlers;
pub mod routes;

pub use routes::configure;
pub use platform::{DeliveryPlatform, PlatformConnector, ConnectorKind, ConnectorError};
pub use transaction::{BankTransaction, TransactionKind, TransactionSide, ReconcileStatus};
pub use reconcile::{
    Reconciliation, ReconciliationMatch, MatchConfidence, ReconcileError,
    match_transactions,
};
pub use billing::{BillingEntry, BillingKind, BillingPeriod};
pub use dashboard::{BillingDashboard, DashboardSummary, DashboardKpi};
