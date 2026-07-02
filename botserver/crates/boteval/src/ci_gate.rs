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
}

impl Default for CiGateConfig {
    fn default() -> Self {
        Self {
            min_pass_rate: 0.95,
            max_failures: 0,
            required_tags: Vec::new(),
            tag_pass_rates: HashMap::new(),
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
}
