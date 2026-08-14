//! Evaluation runner. Iterates over a dataset, calls a target function to
//! generate a response, validates it, and produces a final report.

use crate::contracts::{validate_response, validate_tool_floor, CheckResult};
use crate::dataset::Dataset;
use crate::scoring::Score;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[async_trait]
pub trait LlmTarget: Send + Sync {
    async fn complete(&self, system_prompt: Option<&str>, user_prompt: &str) -> Result<String, String>;

    async fn complete_with_usage(
        &self,
        system_prompt: Option<&str>,
        user_prompt: &str,
    ) -> TaskOutcome {
        match self.complete(system_prompt, user_prompt).await {
            Ok(response) => TaskOutcome {
                response,
                cost: 0.0,
                tool_calls: 0,
            },
            Err(_) => TaskOutcome::default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TaskOutcome {
    pub response: String,
    pub cost: f64,
    pub tool_calls: u32,
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

#[async_trait]
pub trait HarnessTarget: Send + Sync {
    async fn run(&self, prompt: &str) -> HarnessOutcome;
}

#[derive(Debug, Clone, Default)]
pub struct HarnessOutcome {
    pub passed: bool,
    pub tool_calls: u32,
    pub cost: f64,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryReport {
    pub entry_id: Uuid,
    pub tag: String,
    pub score: Score,
    pub results: Vec<CheckResult>,
    pub response: String,
    pub cost: f64,
    pub tool_calls: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalReport {
    pub dataset_name: String,
    pub score: Score,
    pub per_entry: Vec<EntryReport>,
    pub per_tag: HashMap<String, Score>,
    pub total_cost: f64,
    pub total_tool_calls: u32,
}

pub async fn run_evaluation<T: LlmTarget + ?Sized>(
    dataset: &Dataset,
    target: &T,
) -> EvalReport {
    let mut per_entry = Vec::new();
    let mut all_results = Vec::new();
    let mut per_tag: HashMap<String, Vec<CheckResult>> = HashMap::new();
    let mut total_cost = 0.0;
    let mut total_tool_calls = 0;
    for entry in &dataset.entries {
        let outcome = target
            .complete_with_usage(entry.system_prompt.as_deref(), &entry.prompt)
            .await;
        let results = validate_response(&entry.contract, &outcome.response);
        let score = Score::from_results(&results);
        for tag in &entry.tags {
            per_tag
                .entry(tag.clone())
                .or_default()
                .extend(results.clone());
        }
        total_cost += outcome.cost;
        total_tool_calls += outcome.tool_calls;
        all_results.extend(results.clone());
        per_entry.push(EntryReport {
            entry_id: entry.id,
            tag: entry.tags.join(","),
            score,
            results,
            response: outcome.response,
            cost: outcome.cost,
            tool_calls: outcome.tool_calls,
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
        total_cost,
        total_tool_calls,
    }
}

pub async fn run_mixed_evaluation<T: LlmTarget + ?Sized, H: HarnessTarget + ?Sized>(
    dataset: &Dataset,
    llm: &T,
    harness: &H,
) -> EvalReport {
    let mut per_entry = Vec::new();
    let mut all_results = Vec::new();
    let mut per_tag: HashMap<String, Vec<CheckResult>> = HashMap::new();
    let mut total_cost = 0.0;
    let mut total_tool_calls = 0;
    for entry in &dataset.entries {
        let (response, results, entry_cost, entry_tool_calls) =
            if entry.tags.iter().any(|t| t == "harness") {
                let ran = harness.run(&entry.prompt).await;
                // #817 — a harness entry must satisfy BOTH its completion
                // state AND its declared contract (must_contain, schema, tool
                // floor, …). Previously only `ran.passed` was checked, so an
                // agent that "completed" while ignoring the contract passed.
                let mut results = vec![if ran.passed {
                    CheckResult::pass("harness_passed")
                } else {
                    CheckResult::fail("harness_passed", "agent run did not complete")
                }];
                results.extend(validate_response(&entry.contract, &ran.summary));
                if let Some(floor) = validate_tool_floor(&entry.contract, ran.tool_calls) {
                    results.push(floor);
                }
                (ran.summary, results, ran.cost, ran.tool_calls)
            } else {
                let completed = llm
                    .complete_with_usage(entry.system_prompt.as_deref(), &entry.prompt)
                    .await;
                let response = completed.response;
                (
                    response.clone(),
                    validate_response(&entry.contract, &response),
                    completed.cost,
                    completed.tool_calls,
                )
            };
        total_cost += entry_cost;
        total_tool_calls += entry_tool_calls;
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
            cost: entry_cost,
            tool_calls: entry_tool_calls,
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
        total_cost,
        total_tool_calls,
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
