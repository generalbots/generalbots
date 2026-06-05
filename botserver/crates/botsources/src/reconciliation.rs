use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SourceProvider {
    IFood,
    Rappi,
    UberEats,
    Bank,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReconciliationStatus {
    Matched,
    PartialMatch,
    Unmatched,
    Duplicate,
    Disputed,
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub external_id: String,
    pub provider: SourceProvider,
    pub amount_cents: i64,
    pub currency: String,
    pub occurred_at: DateTime<Utc>,
    pub description: String,
    pub counterparty: Option<String>,
    pub raw_payload: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankStatementEntry {
    pub id: Uuid,
    pub bank_txn_id: String,
    pub amount_cents: i64,
    pub currency: String,
    pub posted_at: DateTime<Utc>,
    pub description: String,
    pub counterparty: Option<String>,
    pub reference: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchCandidate {
    pub transaction_id: Uuid,
    pub bank_entry_id: Uuid,
    pub confidence: f64,
    pub matched_rules: Vec<MatchRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MatchRule {
    ExactAmount,
    NearAmount,
    ExactDate,
    NearDate,
    ExactCounterparty,
    FuzzyDescription,
    ReferenceMatch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationRecord {
    pub id: Uuid,
    pub transaction_id: Uuid,
    pub bank_entry_id: Option<Uuid>,
    pub status: ReconciliationStatus,
    pub confidence: f64,
    pub matched_rules: Vec<MatchRule>,
    pub amount_difference_cents: i64,
    pub date_difference_hours: i64,
    pub notes: Option<String>,
    pub reviewed_by: Option<String>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationRun {
    pub id: Uuid,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub transactions_processed: u32,
    pub matched: u32,
    pub partial: u32,
    pub unmatched: u32,
    pub duplicates: u32,
    pub total_discrepancy_cents: i64,
    pub status: RunStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RunStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationConfig {
    pub amount_tolerance_cents: i64,
    pub date_tolerance_hours: i64,
    pub min_confidence: f64,
    pub auto_match_threshold: f64,
    pub counterparty_aliases: HashMap<String, String>,
    pub description_stopwords: Vec<String>,
}

impl Default for ReconciliationConfig {
    fn default() -> Self {
        Self {
            amount_tolerance_cents: 0,
            date_tolerance_hours: 24,
            min_confidence: 0.5,
            auto_match_threshold: 0.9,
            counterparty_aliases: HashMap::new(),
            description_stopwords: vec![
                "ltda".into(),
                "sa".into(),
                "me".into(),
                "epp".into(),
                "eireli".into(),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationReport {
    pub run_id: Uuid,
    pub period: (DateTime<Utc>, DateTime<Utc>),
    pub summary: ReconciliationSummary,
    pub records: Vec<ReconciliationRecord>,
    pub unmatched_transactions: Vec<Transaction>,
    pub unmatched_bank_entries: Vec<BankStatementEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReconciliationSummary {
    pub total_transactions: u32,
    pub total_bank_entries: u32,
    pub matched: u32,
    pub partial: u32,
    pub unmatched_transactions: u32,
    pub unmatched_bank_entries: u32,
    pub total_amount_transactions_cents: i64,
    pub total_amount_bank_cents: i64,
    pub net_difference_cents: i64,
    pub average_confidence: f64,
}
