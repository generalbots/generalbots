use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::reconciliation::{BankStatementEntry, Transaction};
use super::reconciliation_engine::ReconciliationEngine as Engine;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationJobRequest {
    pub tenant_id: Uuid,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub providers: Vec<String>,
    pub bank_account_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationJobResult {
    pub job_id: Uuid,
    pub status: JobStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub matched: u32,
    pub partial: u32,
    pub unmatched_transactions: u32,
    pub unmatched_bank_entries: u32,
    pub total_discrepancy_cents: i64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

pub struct ReconciliationService {
    engine: Engine,
}

impl ReconciliationService {
    pub fn new() -> Self {
        Self {
            engine: Engine::with_defaults(),
        }
    }

    pub fn with_engine(engine: Engine) -> Self {
        Self { engine }
    }

    pub fn reconcile(
        &self,
        job_id: Uuid,
        transactions: Vec<Transaction>,
        bank_entries: Vec<BankStatementEntry>,
    ) -> ReconciliationJobResult {
        let started_at = Utc::now();
        let duplicates = self.engine.detect_duplicates(&transactions);
        let duplicate_set: std::collections::HashSet<Uuid> = duplicates.into_iter().collect();
        let txns_for_match: Vec<Transaction> = transactions
            .into_iter()
            .filter(|t| !duplicate_set.contains(&t.id))
            .collect();

        let mut records = Vec::with_capacity(txns_for_match.len());
        let mut matched_bank_ids = std::collections::HashSet::new();
        let mut matched_count: u32 = 0;
        let mut partial_count: u32 = 0;
        let mut total_disc: i64 = 0;

        for txn in &txns_for_match {
            let candidates = self.engine.find_candidates(txn, &bank_entries);
            let best = candidates.first();
            let entry = best.and_then(|c| bank_entries.iter().find(|e| e.id == c.bank_entry_id));
            let record = self.engine.build_record(txn, entry, best);
            if record.bank_entry_id.is_some() {
                matched_bank_ids.insert(record.bank_entry_id.unwrap());
            }
            match record.status {
                super::reconciliation::ReconciliationStatus::Matched => matched_count += 1,
                super::reconciliation::ReconciliationStatus::PartialMatch => partial_count += 1,
                _ => {}
            }
            total_disc += record.amount_difference_cents;
            records.push(record);
        }

        let unmatched_txns = (txns_for_match.len() as u32).saturating_sub(records.len() as u32);
        let unmatched_entries =
            (bank_entries.len() as u32).saturating_sub(matched_bank_ids.len() as u32);

        ReconciliationJobResult {
            job_id,
            status: JobStatus::Completed,
            started_at,
            completed_at: Some(Utc::now()),
            matched: matched_count,
            partial: partial_count,
            unmatched_transactions: unmatched_txns,
            unmatched_bank_entries: unmatched_entries,
            total_discrepancy_cents: total_disc,
            error: None,
        }
    }
}

impl Default for ReconciliationService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn txn(amount: i64, hours_ago: i64) -> Transaction {
        Transaction {
            id: Uuid::new_v4(),
            external_id: format!("T-{amount}-{hours_ago}"),
            provider: crate::reconciliation::SourceProvider::IFood,
            amount_cents: amount,
            currency: "BRL".into(),
            occurred_at: Utc::now() - Duration::hours(hours_ago),
            description: "Pedido".into(),
            counterparty: Some("Restaurante X LTDA".into()),
            raw_payload: serde_json::json!({}),
            created_at: Utc::now(),
        }
    }

    fn entry(amount: i64, hours_ago: i64) -> BankStatementEntry {
        BankStatementEntry {
            id: Uuid::new_v4(),
            bank_txn_id: format!("B-{amount}-{hours_ago}"),
            amount_cents: amount,
            currency: "BRL".into(),
            posted_at: Utc::now() - Duration::hours(hours_ago),
            description: "PIX".into(),
            counterparty: Some("Restaurante X".into()),
            reference: None,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn full_reconcile_matches_most() {
        let svc = ReconciliationService::new();
        let t1 = txn(1000, 1);
        let t2 = txn(2500, 2);
        let e1 = entry(1000, 1);
        let e2 = entry(2500, 2);
        let result = svc.reconcile(Uuid::new_v4(), vec![t1, t2], vec![e1, e2]);
        assert_eq!(result.status, JobStatus::Completed);
        assert!(result.matched >= 1);
    }
}
