//! Deterministic Vibe benchmark suite (Issue #800): more than 200 tasks
//! covering the three Vibe use cases in English and Brazilian Portuguese.
//!
//! Entries are produced from authored [`TaskSpec`]s with stable sequential
//! UUIDs so the committed `datasets/vibe-200.json` artifact can be
//! reproduced byte-for-byte via `boteval-run --dump-dataset`.

use crate::dataset::{Contract, Dataset, DatasetEntry};
use crate::vibe_tasks::{general_tasks, use_case_tasks, TaskSpec};
use uuid::Uuid;

pub const BENCHMARK_NAME: &str = "vibe-200";
pub const USE_CASES: [&str; 3] = ["software_development", "customer_support", "financial_analysis"];
pub const LANGS: [&str; 2] = ["en", "pt-BR"];

/// Builds the full benchmark dataset. The number of entries is deterministic
/// and exceeds 200.
pub fn vibe_benchmark() -> Dataset {
    let mut dataset = Dataset::new(BENCHMARK_NAME);
    dataset.description = Some(
        "Regression suite for the Vibe agent loop across use cases and languages".into(),
    );
    let mut counter: u128 = 1;

    for use_case in USE_CASES {
        let harness_idx: &[usize] = match use_case {
            "software_development" => &[0, 3],
            "customer_support" => &[0, 1],
            "financial_analysis" => &[0, 2],
            _ => &[],
        };
        for (index, spec) in use_case_tasks(use_case).iter().enumerate() {
            for lang in LANGS {
                push_entry(
                    &mut dataset,
                    &mut counter,
                    use_case,
                    lang,
                    spec,
                    harness_idx.contains(&index),
                );
            }
        }
    }
    for (index, spec) in general_tasks().iter().enumerate() {
        for lang in LANGS {
            push_entry(
                &mut dataset,
                &mut counter,
                "general",
                lang,
                spec,
                index < 1,
            );
        }
    }
    dataset
}

fn push_entry(
    dataset: &mut Dataset,
    counter: &mut u128,
    use_case: &str,
    lang: &str,
    spec: &TaskSpec,
    harness: bool,
) {
    let (prompt, contains, lang_code) = if lang == "en" {
        (spec.prompt_en, spec.contains_en, "en")
    } else {
        (spec.prompt_pt, spec.contains_pt, "pt-BR")
    };
    let contract = Contract {
        must_contain: contains.iter().map(|p| p.to_string()).collect(),
        must_not_contain: spec.forbid.iter().map(|p| p.to_string()).collect(),
        json_schema: None,
        max_tokens: spec.max_tokens,
        min_tokens: spec.min_tokens,
        language: Some(lang_code.into()),
    };
    let mut tags: Vec<String> = vec![
        use_case.to_string(),
        format!("lang:{lang}"),
        "vibe".to_string(),
    ];
    if harness {
        tags.push("harness".to_string());
    }
    dataset.push(DatasetEntry {
        id: Uuid::from_u128(*counter),
        prompt: prompt.to_string(),
        system_prompt: None,
        context: None,
        tags,
        contract,
    });
    *counter += 1;
}

/// Serializes the benchmark as pretty JSON, ready to be written to
/// `datasets/vibe-200.json`.
pub fn vibe_benchmark_json() -> Result<String, String> {
    vibe_benchmark().to_json()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn benchmark_has_more_than_200_entries() {
        let dataset = vibe_benchmark();
        assert!(
            dataset.len() >= 200,
            "expected at least 200 tasks, got {}",
            dataset.len()
        );
    }

    #[test]
    fn every_use_case_and_language_is_covered() {
        let dataset = vibe_benchmark();
        for use_case in USE_CASES {
            let count = dataset
                .entries
                .iter()
                .filter(|e| e.tags.iter().any(|t| t == use_case))
                .count();
            assert!(count > 0, "no entries for {use_case}");
        }
        for lang in LANGS {
            let count = dataset
                .entries
                .iter()
                .filter(|e| e.tags.iter().any(|t| t == &format!("lang:{lang}")))
                .count();
            assert!(count > 0, "no entries for {lang}");
        }
    }

    #[test]
    fn benchmark_ids_are_deterministic() {
        let first = vibe_benchmark();
        let second = vibe_benchmark();
        assert_eq!(first.entries.len(), second.entries.len());
        for (a, b) in first.entries.iter().zip(second.entries.iter()) {
            assert_eq!(a.id, b.id, "entry ids must be stable across builds");
            assert_eq!(a.prompt, b.prompt);
            assert_eq!(a.tags, b.tags);
        }
    }

    #[test]
    fn every_contract_has_at_least_one_check() {
        let dataset = vibe_benchmark();
        for entry in &dataset.entries {
            let checked = !entry.contract.must_contain.is_empty()
                || !entry.contract.must_not_contain.is_empty()
                || entry.contract.json_schema.is_some();
            assert!(checked, "entry {} has an empty contract", entry.id);
        }
    }

    #[test]
    fn benchmark_has_harness_tagged_tasks() {
        let dataset = vibe_benchmark();
        let harness_count = dataset
            .entries
            .iter()
            .filter(|e| e.tags.iter().any(|t| t == "harness"))
            .count();
        assert!(
            harness_count >= 8,
            "expected at least 8 harness tasks, got {harness_count}"
        );
    }

    #[test]
    fn benchmark_serializes_to_json_and_back() {
        let dataset = vibe_benchmark();
        let json = dataset.to_json().expect("serialize");
        let parsed = Dataset::from_json(&json).expect("parse");
        assert_eq!(parsed.len(), dataset.len());
        assert_eq!(parsed.entries[0].id, dataset.entries[0].id);
    }
}