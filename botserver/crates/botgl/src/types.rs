use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlAccount {
    pub id: Uuid,
    pub branch_id: Uuid,
    pub code: String,
    pub name: String,
    pub account_type: String,
    pub parent_id: Option<Uuid>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlJournalEntry {
    pub id: Uuid,
    pub branch_id: Uuid,
    pub entry_date: NaiveDate,
    pub description: String,
    pub reference_type: Option<String>,
    pub reference_id: Option<Uuid>,
    pub status: String,
    pub created_by: Option<Uuid>,
    pub posted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlJournalLine {
    pub id: Uuid,
    pub entry_id: Uuid,
    pub account_id: Uuid,
    pub debit: rust_decimal::Decimal,
    pub credit: rust_decimal::Decimal,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialBalanceRow {
    pub account_code: String,
    pub account_name: String,
    pub account_type: String,
    pub debit_total: rust_decimal::Decimal,
    pub credit_total: rust_decimal::Decimal,
    pub balance: rust_decimal::Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomeStatementRow {
    pub account_code: String,
    pub account_name: String,
    pub amount: rust_decimal::Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceSheetRow {
    pub account_code: String,
    pub account_name: String,
    pub account_type: String,
    pub balance: rust_decimal::Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAccountRequest {
    pub code: String,
    pub name: String,
    pub account_type: String,
    pub parent_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateJournalEntryRequest {
    pub entry_date: NaiveDate,
    pub description: String,
    pub reference_type: Option<String>,
    pub reference_id: Option<Uuid>,
    pub lines: Vec<JournalLineInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalLineInput {
    pub account_id: Uuid,
    pub debit: rust_decimal::Decimal,
    pub credit: rust_decimal::Decimal,
    pub description: Option<String>,
}
