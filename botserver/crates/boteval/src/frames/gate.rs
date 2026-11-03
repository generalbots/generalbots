//! Frame quality gate. Runs alongside the text gate; CI fails when any
//! threshold regresses.

use serde::{Deserialize, Serialize};

use super::metrics::FrameMetrics;
use super::prompts::RestraintReport;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameGateConfig {
    pub min_validity_rate: f64,
    pub min_fallback_coverage: f64,
    pub max_ui_ratio_on_prose: f64,
    pub max_duplicate_questions: usize,
    pub require_replay_determinism: bool,
}

impl Default for FrameGateConfig {
    fn default() -> Self {
        Self {
            min_validity_rate: 0.99,
            min_fallback_coverage: 1.0,
            max_ui_ratio_on_prose: 0.05,
            max_duplicate_questions: 0,
            require_replay_determinism: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameGateVerdict {
    pub pass: bool,
    pub reasons: Vec<String>,
    pub exit_code: i32,
}

pub fn evaluate(
    config: &FrameGateConfig,
    metrics: &FrameMetrics,
    restraint: &RestraintReport,
) -> FrameGateVerdict {
    let mut reasons = Vec::new();
    if metrics.total_frames == 0 {
        reasons.push("no golden transcript frames evaluated".to_string());
    }
    if metrics.validity_rate() < config.min_validity_rate {
        reasons.push(format!(
            "frame validity {:.4} < required {:.4}",
            metrics.validity_rate(),
            config.min_validity_rate
        ));
    }
    if metrics.fallback_coverage() < config.min_fallback_coverage {
        reasons.push(format!(
            "fallback coverage {:.4} < required {:.4}",
            metrics.fallback_coverage(),
            config.min_fallback_coverage
        ));
    }
    if metrics.duplicate_questions > config.max_duplicate_questions {
        reasons.push(format!(
            "{} duplicate questions > max {}",
            metrics.duplicate_questions, config.max_duplicate_questions
        ));
    }
    if config.require_replay_determinism && metrics.replay_failures > 0 {
        reasons.push(format!(
            "{} transcripts did not replay deterministically",
            metrics.replay_failures
        ));
    }
    if restraint.prose_cases == 0 {
        reasons.push("restraint set has no prose cases".to_string());
    } else if restraint.ui_ratio_on_prose > config.max_ui_ratio_on_prose {
        reasons.push(format!(
            "UI frames on {:.4} of prose prompts > max {:.4}",
            restraint.ui_ratio_on_prose, config.max_ui_ratio_on_prose
        ));
    }

    if reasons.is_empty() {
        FrameGateVerdict {
            pass: true,
            reasons,
            exit_code: 0,
        }
    } else {
        FrameGateVerdict {
            pass: false,
            reasons,
            exit_code: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{evaluate, FrameGateConfig};
    use crate::frames::metrics::FrameMetrics;
    use crate::frames::prompts::RestraintReport;

    fn healthy() -> (FrameMetrics, RestraintReport) {
        let metrics = FrameMetrics {
            transcripts: 1,
            total_frames: 10,
            valid_frames: 10,
            ui_frames: 2,
            ui_frames_with_fallback: 2,
            ..FrameMetrics::default()
        };
        let restraint = RestraintReport {
            prose_cases: 100,
            prose_with_ui: 2,
            ui_cases: 20,
            ui_without_ui: 1,
            unknown_outcomes: 0,
            ui_ratio_on_prose: 0.02,
            ui_recall: 0.95,
        };
        (metrics, restraint)
    }

    #[test]
    fn healthy_corpus_passes() {
        let (metrics, restraint) = healthy();
        let verdict = evaluate(&FrameGateConfig::default(), &metrics, &restraint);
        assert!(verdict.pass, "{:?}", verdict.reasons);
    }

    #[test]
    fn missing_fallback_fails() {
        let (mut metrics, restraint) = healthy();
        metrics.ui_frames_with_fallback = 1;
        let verdict = evaluate(&FrameGateConfig::default(), &metrics, &restraint);
        assert!(!verdict.pass);
    }
}
