use super::reconciliation::{
    BankStatementEntry, MatchCandidate, MatchRule, ReconciliationConfig, ReconciliationRecord,
    ReconciliationStatus, ReconciliationSummary, SourceProvider, Transaction,
};
use chrono::Duration;
use std::collections::HashMap;
use uuid::Uuid;

pub struct ReconciliationEngine {
    config: ReconciliationConfig,
}

impl ReconciliationEngine {
    pub fn new(config: ReconciliationConfig) -> Self {
        Self { config }
    }

    pub fn with_defaults() -> Self {
        Self::new(ReconciliationConfig::default())
    }

    pub fn find_candidates(
        &self,
        txn: &Transaction,
        bank_entries: &[BankStatementEntry],
    ) -> Vec<MatchCandidate> {
        let mut candidates = Vec::new();
        for entry in bank_entries {
            if let Some(candidate) = self.score_match(txn, entry) {
                if candidate.confidence >= self.config.min_confidence {
                    candidates.push(candidate);
                }
            }
        }
        candidates.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
        candidates
    }

    fn score_match(
        &self,
        txn: &Transaction,
        entry: &BankStatementEntry,
    ) -> Option<MatchCandidate> {
        let mut rules = Vec::new();
        let mut score = 0.0_f64;

        let amount_diff = (txn.amount_cents - entry.amount_cents).abs();
        if amount_diff == 0 {
            rules.push(MatchRule::ExactAmount);
            score += 0.55;
        } else if amount_diff <= self.config.amount_tolerance_cents {
            rules.push(MatchRule::NearAmount);
            score += 0.30;
        } else if amount_diff <= (txn.amount_cents.abs() / 100).max(500) {
            rules.push(MatchRule::NearAmount);
            score += 0.10;
        }

        let date_diff = (txn.occurred_at - entry.posted_at).num_hours().abs();
        if date_diff == 0 {
            rules.push(MatchRule::ExactDate);
            score += 0.15;
        } else if date_diff <= self.config.date_tolerance_hours {
            rules.push(MatchRule::NearDate);
            score += 0.08;
        } else if date_diff > Duration::days(7).num_hours() {
            return None;
        }

        if let (Some(t_cp), Some(e_cp)) = (&txn.counterparty, &entry.counterparty) {
            let t_norm = self.normalize_counterparty(t_cp);
            let e_norm = self.normalize_counterparty(e_cp);
            if t_norm == e_norm && !t_norm.is_empty() {
                rules.push(MatchRule::ExactCounterparty);
                score += 0.20;
            } else if self.fuzzy_match(&t_norm, &e_norm) {
                rules.push(MatchRule::FuzzyDescription);
                score += 0.05;
            }
        }

        if let Some(reference) = &entry.reference {
            if txn.description.contains(reference) || reference.contains(&txn.description) {
                rules.push(MatchRule::ReferenceMatch);
                score += 0.10;
            }
        }

        if rules.is_empty() {
            return None;
        }

        Some(MatchCandidate {
            transaction_id: txn.id,
            bank_entry_id: entry.id,
            confidence: score.min(1.0),
            matched_rules: rules,
        })
    }

    fn normalize_counterparty(&self, value: &str) -> String {
        let lowered = value.to_lowercase();
        let mut result = String::new();
        for word in lowered.split_whitespace() {
            let cleaned: String = word
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect();
            if !self.config.description_stopwords.contains(&cleaned) && !cleaned.is_empty() {
                if !result.is_empty() {
                    result.push(' ');
                }
                result.push_str(&cleaned);
            }
        }
        if let Some(canonical) = self.config.counterparty_aliases.get(&result) {
            return canonical.clone();
        }
        result
    }

    fn fuzzy_match(&self, a: &str, b: &str) -> bool {
        if a.is_empty() || b.is_empty() {
            return false;
        }
        let a_tokens: Vec<&str> = a.split_whitespace().collect();
        let b_tokens: Vec<&str> = b.split_whitespace().collect();
        let common = a_tokens
            .iter()
            .filter(|t| b_tokens.contains(t))
            .count();
        common >= 1 && common * 2 >= a_tokens.len().min(b_tokens.len())
    }

    pub fn build_record(
        &self,
        txn: &Transaction,
        entry: Option<&BankStatementEntry>,
        candidate: Option<&MatchCandidate>,
    ) -> ReconciliationRecord {
        let now = chrono::Utc::now();
        match (entry, candidate) {
            (Some(e), Some(c)) => {
                let status = if c.confidence >= self.config.auto_match_threshold {
                    ReconciliationStatus::Matched
                } else {
                    ReconciliationStatus::PartialMatch
                };
                ReconciliationRecord {
                    id: Uuid::new_v4(),
                    transaction_id: txn.id,
                    bank_entry_id: Some(e.id),
                    status,
                    confidence: c.confidence,
                    matched_rules: c.matched_rules.clone(),
                    amount_difference_cents: txn.amount_cents - e.amount_cents,
                    date_difference_hours: (txn.occurred_at - e.posted_at).num_hours(),
                    notes: None,
                    reviewed_by: None,
                    reviewed_at: None,
                    created_at: now,
                }
            }
            _ => ReconciliationRecord {
                id: Uuid::new_v4(),
                transaction_id: txn.id,
                bank_entry_id: None,
                status: ReconciliationStatus::Unmatched,
                confidence: 0.0,
                matched_rules: Vec::new(),
                amount_difference_cents: 0,
                date_difference_hours: 0,
                notes: None,
                reviewed_by: None,
                reviewed_at: None,
                created_at: now,
            },
        }
    }

    pub fn detect_duplicates(&self, txns: &[Transaction]) -> Vec<Uuid> {
        let mut duplicates = Vec::new();
        let mut seen: HashMap<(i64, i64, String), Uuid> = HashMap::new();
        for txn in txns {
            let key = (
                txn.amount_cents,
                txn.occurred_at.timestamp(),
                txn.external_id.clone(),
            );
            if let Some(existing) = seen.get(&key) {
                if *existing != txn.id {
                    duplicates.push(txn.id);
                }
            } else {
                seen.insert(key, txn.id);
            }
        }
        duplicates
    }

    pub fn summarize(
        &self,
        txns: &[Transaction],
        entries: &[BankStatementEntry],
        records: &[ReconciliationRecord],
    ) -> ReconciliationSummary {
        let mut summary = ReconciliationSummary {
            total_transactions: txns.len() as u32,
            total_bank_entries: entries.len() as u32,
            total_amount_transactions_cents: txns.iter().map(|t| t.amount_cents).sum(),
            total_amount_bank_cents: entries.iter().map(|e| e.amount_cents).sum(),
            ..Default::default()
        };
        let mut total_conf = 0.0;
        let mut count = 0;
        let mut matched_ids = std::collections::HashSet::new();
        for r in records {
            match r.status {
                ReconciliationStatus::Matched => summary.matched += 1,
                ReconciliationStatus::PartialMatch => summary.partial += 1,
                ReconciliationStatus::Unmatched => {}
                ReconciliationStatus::Duplicate => summary.duplicates += 1,
                ReconciliationStatus::Disputed => {}
                ReconciliationStatus::Pending => {}
            }
            if r.confidence > 0.0 {
                total_conf += r.confidence;
                count += 1;
            }
            if let Some(bid) = r.bank_entry_id {
                matched_ids.insert(bid);
            }
        }
        summary.average_confidence = if count > 0 { total_conf / count as f64 } else { 0.0 };
        summary.unmatched_bank_entries =
            (entries.len() as u32).saturating_sub(matched_ids.len() as u32);
        summary.net_difference_cents =
            summary.total_amount_transactions_cents - summary.total_amount_bank_cents;
        summary.unmatched_transactions = (txns.len() as u32).saturating_sub(records.len() as u32);
        summary
    }

    pub fn provider_label(provider: &SourceProvider) -> &'static str {
        match provider {
            SourceProvider::IFood => "iFood",
            SourceProvider::Rappi => "Rappi",
            SourceProvider::UberEats => "Uber Eats",
            SourceProvider::Bank => "Banco",
            SourceProvider::Manual => "Manual",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn txn(amount: i64, hours_ago: i64, desc: &str, cp: Option<&str>) -> Transaction {
        Transaction {
            id: Uuid::new_v4(),
            external_id: format!("EXT-{amount}-{hours_ago}"),
            provider: SourceProvider::IFood,
            amount_cents: amount,
            currency: "BRL".into(),
            occurred_at: Utc::now() - Duration::hours(hours_ago),
            description: desc.into(),
            counterparty: cp.map(String::from),
            raw_payload: serde_json::json!({}),
            created_at: Utc::now(),
        }
    }

    fn entry(amount: i64, hours_ago: i64, desc: &str, cp: Option<&str>) -> BankStatementEntry {
        BankStatementEntry {
            id: Uuid::new_v4(),
            bank_txn_id: format!("BANK-{amount}-{hours_ago}"),
            amount_cents: amount,
            currency: "BRL".into(),
            posted_at: Utc::now() - Duration::hours(hours_ago),
            description: desc.into(),
            counterparty: cp.map(String::from),
            reference: None,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn exact_match_returns_high_confidence() {
        let engine = ReconciliationEngine::with_defaults();
        let t = txn(1500, 1, "Pedido #42", Some("Restaurante XYZ LTDA"));
        let e = entry(1500, 1, "PIX RECEBIDO", Some("Restaurante XYZ"));
        let cands = engine.find_candidates(&t, &[e]);
        assert!(!cands.is_empty());
        assert!(cands[0].confidence >= 0.5);
    }

    #[test]
    fn no_match_returns_empty() {
        let engine = ReconciliationEngine::with_defaults();
        let t = txn(1500, 1, "Pedido", Some("Foo"));
        let e = entry(9999, 100, "Other", Some("Bar"));
        let cands = engine.find_candidates(&t, &[e]);
        assert!(cands.is_empty());
    }

    #[test]
    fn duplicates_detected_by_key() {
        let engine = ReconciliationEngine::with_defaults();
        let t1 = txn(100, 0, "a", None);
        let t2 = txn(100, 0, "a", None);
        let dups = engine.detect_duplicates(&[t1, t2]);
        assert_eq!(dups.len(), 1);
    }
}
