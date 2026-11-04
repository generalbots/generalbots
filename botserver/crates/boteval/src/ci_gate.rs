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
            // A required tag may have its own threshold in `tag_pass_rates`
            // (the benchmark uses 0.90 there). Checking every required tag
            // against the overall 0.95 made the tag-specific table dead code
            // and failed categories that met their documented target.
            let threshold = self
                .config
                .tag_pass_rates
                .get(tag)
                .copied()
                .unwrap_or(self.config.min_pass_rate);
            match report.per_tag.get(tag) {
                Some(score) if score.rate >= threshold => {}
                Some(score) => reasons.push(format!(
                    "tag {tag} rate {:.4} below threshold {:.4}",
                    score.rate, threshold
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
        // Task count falls back to score.total so hand-built reports (and
        // runners without per-entry rows) still get a cost-per-task verdict.
        let task_count = report.per_entry.len().max(report.score.total);
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
    fn required_tag_uses_its_specific_threshold() {
        // 93% overall meets neither 95% nor the 90% tag target alone; the
        // point is that a required tag is judged by tag_pass_rates, not by
        // the overall min_pass_rate.
        let gate = CiGate::new(CiGateConfig {
            min_pass_rate: 0.90,
            max_failures: 100,
            required_tags: vec!["general".to_string()],
            tag_pass_rates: HashMap::from([("general".to_string(), 0.90)]),
            max_cost_per_task: 0.0,
            harness_min_tool_calls: 0,
        });
        let mut report = empty_report();
        report.score.total = 100;
        report.score.passed = 93;
        report.score.failed = 7;
        report.score.rate = 0.93;
        report.per_tag.insert(
            "general".to_string(),
            crate::scoring::Score {
                total: 100,
                passed: 93,
                failed: 7,
                skipped: 0,
                rate: 0.93,
            },
        );
        let verdict = gate.evaluate(&report);
        assert!(verdict.pass, "reasons: {:?}", verdict.reasons);
    }

    #[test]
    fn gate_toml_is_valid_and_complete() {
        // Guards the shipped gate config against silent parse failures:
        // past edits (JSON-style `key: value`, multi-line inline tables)
        // broke `toml::from_str` and let the gate never actually run.
        let raw = include_str!("../ci/gate.toml");
        let cfg: CiGateConfig = toml::from_str(raw)
            .expect("ci/gate.toml must remain valid TOML deserializing to CiGateConfig");
        assert_eq!(cfg.min_pass_rate, 0.95);
        assert_eq!(cfg.max_failures, 28);
        assert_eq!(cfg.max_cost_per_task, 0.05);
        assert_eq!(cfg.harness_min_tool_calls, 2);
        assert_eq!(cfg.required_tags.len(), 5);
        // tag_pass_rates must contain ONLY tag keys (a section placed too
        // late would swallow top-level keys like max_cost_per_task).
        assert_eq!(cfg.tag_pass_rates.len(), 5);
        for tag in [
            "software_development",
            "customer_support",
            "financial_analysis",
            "general",
            "harness",
        ] {
            assert!(
                cfg.tag_pass_rates.contains_key(tag),
                "missing tag_pass_rates entry for {tag}"
            );
        }
    }

    #[test]
    fn only_one_gate_config_exists() {
        // #1345 — the gate config was duplicated (root gate.toml vs ci/gate.toml)
        // and the copies diverged: CI read the copy nobody edited. ci/gate.toml is
        // now the single source of truth and the root file must stay a comment-only
        // stub so editing it can never silently override or diverge from CI.
        let raw = include_str!("../gate.toml");
        let parsed: toml::Value =
            toml::from_str(raw).expect("../gate.toml must remain valid TOML");
        assert!(
            parsed.as_table().map(|t| t.is_empty()).unwrap_or(true),
            "botserver/crates/boteval/gate.toml must stay a comment-only stub; \
             the single gate config is ci/gate.toml"
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
