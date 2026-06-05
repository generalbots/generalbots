//! Evaluation runner. Iterates over a dataset, calls a target function to
//! generate a response, validates it, and produces a final report.

use crate::contracts::{validate_response, CheckResult};
use crate::dataset::Dataset;
use crate::scoring::Score;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[async_trait]
pub trait LlmTarget: Send + Sync {
    async fn complete(&self, system_prompt: Option<&str>, user_prompt: &str) -> Result<String, String>;
}

pub struct StaticTarget {
    pub response: String,
}

#[async_trait]
impl LlmTarget for StaticTarget {
    async fn complete(&self, _system_prompt: Option<&str>, _user_prompt: &str) -> Result<String, String> {
        Ok(self.response.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryReport {
    pub entry_id: Uuid,
    pub tag: String,
    pub score: Score,
    pub results: Vec<CheckResult>,
    pub response: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalReport {
    pub dataset_name: String,
    pub score: Score,
    pub per_entry: Vec<EntryReport>,
    pub per_tag: HashMap<String, Score>,
}

pub async fn run_evaluation<T: LlmTarget + ?Sized>(
    dataset: &Dataset,
    target: &T,
) -> EvalReport {
    let mut per_entry = Vec::new();
    let mut all_results = Vec::new();
    let mut per_tag: HashMap<String, Vec<CheckResult>> = HashMap::new();
    for entry in &dataset.entries {
        let response = target
            .complete(entry.system_prompt.as_deref(), &entry.prompt)
            .unwrap_or_default();
        let results = validate_response(&entry.contract, &response);
        let score = Score::from_results(&results);
        for tag in &entry.tags {
            per_tag
                .entry(tag.clone())
                .or_default()
                .extend(results.clone());
        }
        all_results.extend(results.clone());
        per_entry.push(EntryReport {
            entry_id: entry.id,
            tag: entry.tags.join(","),
            score,
            results,
            response,
        });
    }
    let per_tag_score: HashMap<String, Score> = per_tag
        .into_iter()
        .map(|(tag, results)| (tag, Score::from_results(&results)))
        .collect();
    EvalReport {
        dataset_name: dataset.name.clone(),
        score: Score::from_results(&all_results),
        per_entry,
        per_tag: per_tag_score,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataset::{Contract, DatasetEntry};

    #[tokio::test]
    async fn runs_static_evaluation() {
        let mut ds = Dataset::new("demo");
        ds.push(DatasetEntry {
            id: Uuid::new_v4(),
            prompt: "say hi".into(),
            system_prompt: None,
            context: None,
            tags: vec!["greeting".into()],
            contract: Contract::must_contain_only(["olá"]),
        });
        let target = StaticTarget {
            response: "olá!".into(),
        };
        let report = run_evaluation(&ds, &target).await;
        assert_eq!(report.score.passed, 1);
    }
}
