//! Prompt sets and the restraint measurement.
//!
//! The "should be prose" set exists to catch a model that renders a card for
//! every request. It is scored against recorded outcomes: for each prompt the
//! harness reports whether the turn produced a UI frame.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptCase {
    pub id: String,
    pub prompt: String,
    pub tag: String,
    pub expects_ui: bool,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptOutcome {
    pub id: String,
    pub produced_ui: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RestraintReport {
    pub prose_cases: usize,
    pub prose_with_ui: usize,
    pub ui_cases: usize,
    pub ui_without_ui: usize,
    pub unknown_outcomes: usize,
    pub ui_ratio_on_prose: f64,
    pub ui_recall: f64,
}

const PROMPT_CASES: &str = include_str!("../../fixtures/run_events/prompts.json");

pub fn load_prompt_cases() -> Result<Vec<PromptCase>, String> {
    serde_json::from_str(PROMPT_CASES).map_err(|error| format!("invalid prompt cases: {error}"))
}

/// Scores recorded outcomes against the expectations.
#[must_use]
pub fn evaluate_restraint(cases: &[PromptCase], outcomes: &[PromptOutcome]) -> RestraintReport {
    let by_id: BTreeMap<&str, bool> = outcomes
        .iter()
        .map(|outcome| (outcome.id.as_str(), outcome.produced_ui))
        .collect();
    let mut report = RestraintReport::default();
    for case in cases {
        let Some(produced_ui) = by_id.get(case.id.as_str()) else {
            report.unknown_outcomes += 1;
            continue;
        };
        if case.expects_ui {
            report.ui_cases += 1;
            if !produced_ui {
                report.ui_without_ui += 1;
            }
        } else {
            report.prose_cases += 1;
            if *produced_ui {
                report.prose_with_ui += 1;
            }
        }
    }
    if report.prose_cases > 0 {
        report.ui_ratio_on_prose = report.prose_with_ui as f64 / report.prose_cases as f64;
    }
    if report.ui_cases > 0 {
        report.ui_recall = (report.ui_cases - report.ui_without_ui) as f64
            / report.ui_cases as f64;
    }
    report
}

#[cfg(test)]
mod tests {
    use super::{evaluate_restraint, PromptCase, PromptOutcome};

    fn case(id: &str, expects_ui: bool) -> PromptCase {
        PromptCase {
            id: id.to_string(),
            prompt: "p".to_string(),
            tag: "t".to_string(),
            expects_ui,
            notes: None,
        }
    }

    #[test]
    fn counts_ui_on_prose() {
        let cases = vec![case("a", false), case("b", false), case("c", true)];
        let outcomes = vec![
            PromptOutcome {
                id: "a".to_string(),
                produced_ui: true,
            },
            PromptOutcome {
                id: "b".to_string(),
                produced_ui: false,
            },
            PromptOutcome {
                id: "c".to_string(),
                produced_ui: true,
            },
        ];
        let report = evaluate_restraint(&cases, &outcomes);
        assert_eq!(report.prose_with_ui, 1);
        assert!((report.ui_ratio_on_prose - 0.5).abs() < f64::EPSILON);
        assert!((report.ui_recall - 1.0).abs() < f64::EPSILON);
    }
}
