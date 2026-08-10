//! CI gate: decides pass/fail based on a threshold. The CI workflow can call
//! \`CiGate::evaluate\` and use the returned exit code.

use crate::runner::EvalReport;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CiGateConfig {
    pub min_pass_rate: f64,
    pub max_failures: usize,
    pub required_tags: Vec<String>,
    pub tag_pass_rates: HashMap<String, f64>,
    pub max_cost_per_task: f64,
    pub harness_min_tool_calls: u32,
}

impl Default for CiGateConfig {
    fn default() -> Self {
        Self {
            min_pass_rate: 0.95,
            max_failures: 0,
            required_tags: Vec::new(),
            tag_pass_rates: HashMap::new(),
            max_cost_per_task: 0.0,
            harness_min_tool_calls: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CiGateVerdict {
    pub pass: bool,
    pub reasons: Vec<String>,
    pub exit_code: i32,
}

pub struct CiGate {
    pub config: CiGateConfig,
}

impl CiGate {
    pub fn new(config: CiGateConfig) -> Self {
        Self { config }
    }

    pub fn evaluate(&self, report: &EvalReport) -> CiGateVerdict {
        let mut reasons = Vec::new();
        if report.score.rate < self.config.min_pass_rate {
            reasons.push(format!(
                "pass rate {:.4} < required {:.4}",
                report.score.rate, self.config.min_pass_rate
            ));
        }
        if report.score.failed > self.config.max_failures {
            reasons.push(format!(
                "failures {} > max {}",
                report.score.failed, self.config.max_failures
            ));
        }
        for tag in &self.config.required_tags {
            match report.per_tag.get(tag) {
                Some(score) if score.rate >= self.config.min_pass_rate => {}
                Some(score) => reasons.push(format!(
                    "tag {tag} rate {:.4} below threshold",
                    score.rate
                )),
                None => reasons.push(format!("required tag {tag} not present in report")),
            }
        }
        for (tag, required) in &self.config.tag_pass_rates {
            if let Some(score) = report.per_tag.get(tag) {
                if score.rate < *required {
                    reasons.push(format!(
                        "tag {tag} rate {:.4} below tag-specific {:.4}",
                        score.rate, required
                    ));
                }
            }
        }
        let task_count = report.per_entry.len();
        if self.config.max_cost_per_task > 0.0 && task_count > 0 {
            let per_task_cost = report.total_cost / task_count as f64;
            if per_task_cost > self.config.max_cost_per_task {
                reasons.push(format!(
                    "average cost per task {:.4} > cap {:.4}",
                    per_task_cost, self.config.max_cost_per_task
                ));
            }
        }
        if self.config.harness_min_tool_calls > 0 {
            let harness_entries: Vec<&crate::runner::EntryReport> = report
                .per_entry
                .iter()
                .filter(|e| e.tag.split(',').any(|t| t == "harness"))
                .collect();
            if harness_entries.is_empty() {
                reasons.push("no harness-tagged tasks evaluated".to_string());
            } else {
                let offenders = harness_entries
                    .iter()
                    .filter(|e| e.tool_calls < self.config.harness_min_tool_calls)
                    .count();
                if offenders > 0 {
                    reasons.push(format!(
                        "{offenders} harness tasks used fewer than {} tool calls",
                        self.config.harness_min_tool_calls
                    ));
                }
            }
        }
        if reasons.is_empty() {
            CiGateVerdict {
                pass: true,
                reasons,
                exit_code: 0,
            }
        } else {
            CiGateVerdict {
                pass: false,
                reasons,
                exit_code: 1,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::EvalReport;
    use std::collections::HashMap;

    fn empty_report() -> EvalReport {
        EvalReport {
            dataset_name: "x".into(),
            score: crate::scoring::Score {
                total: 0,
                passed: 0,
                failed: 0,
                skipped: 0,
                rate: 1.0,
            },
            per_entry: vec![],
            per_tag: HashMap::new(),
            total_cost: 0.0,
            total_tool_calls: 0,
        }
    }

    #[test]
    fn gate_passes_on_perfect_score() {
        let gate = CiGate::new(CiGateConfig::default());
        let verdict = gate.evaluate(&empty_report());
        assert!(verdict.pass);
        assert_eq!(verdict.exit_code, 0);
    }

    #[test]
    fn gate_fails_below_threshold() {
        let gate = CiGate::new(CiGateConfig {
            min_pass_rate: 0.95,
            ..CiGateConfig::default()
        });
        let mut report = empty_report();
        report.score.rate = 0.5;
        let verdict = gate.evaluate(&report);
        assert!(!verdict.pass);
        assert_eq!(verdict.exit_code, 1);
    }

    #[test]
    fn gate_fails_when_cost_cap_violated() {
        let gate = CiGate::new(CiGateConfig {
            max_cost_per_task: 0.10,
            ..CiGateConfig::default()
        });
        let mut report = empty_report();
        report.total_cost = 1.0;
        report.score.total = 5;
        let verdict = gate.evaluate(&report);
        assert!(!verdict.pass);
        assert!(
            verdict
                .reasons
                .iter()
                .any(|r| r.contains("cost per task"))
        );
    }

    #[test]
    fn gate_fails_when_harness_usage_below_floor() {
        let gate = CiGate::new(CiGateConfig {
            harness_min_tool_calls: 2,
            ..CiGateConfig::default()
        });
        let mut report = empty_report();
        report.total_cost = 0.0;
        report.per_entry.push(crate::runner::EntryReport {
            entry_id: uuid::Uuid::new_v4(),
            tag: "software_development,harness".into(),
            score: crate::scoring::Score {
                total: 1,
                passed: 1,
                failed: 0,
                skipped: 0,
                rate: 1.0,
            },
            results: vec![],
            response: "ok".into(),
            cost: 0.0,
            tool_calls: 0,
        });
        let verdict = gate.evaluate(&report);
        assert!(!verdict.pass);
        assert!(
            verdict
                .reasons
                .iter()
                .any(|r| r.contains("harness tasks used fewer"))
        );
    }
}
