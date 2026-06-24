//! Scoring utilities: pass-rate, weighted score, per-tag breakdown.

use crate::contracts::{CheckOutcome, CheckResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Score {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub rate: f64,
}

impl Score {
    pub fn from_results(results: &[CheckResult]) -> Self {
        let mut total = 0;
        let mut passed = 0;
        let mut failed = 0;
        let mut skipped = 0;
        for result in results {
            total += 1;
            match result.outcome {
                CheckOutcome::Pass => passed += 1,
                CheckOutcome::Fail => failed += 1,
                CheckOutcome::Skipped => skipped += 1,
            }
        }
        let rate = if total == 0 {
            0.0
        } else {
            passed as f64 / total as f64
        };
        Self {
            total,
            passed,
            failed,
            skipped,
            rate,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    pub tag: String,
    pub score: Score,
}

pub fn score_breakdown(tag: &str, results: &[CheckResult]) -> ScoreBreakdown {
    ScoreBreakdown {
        tag: tag.into(),
        score: Score::from_results(results),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::CheckResult;

    #[test]
    fn score_basic() {
        let results = vec![
            CheckResult::pass("a"),
            CheckResult::pass("b"),
            CheckResult::fail("c", "bad"),
        ];
        let s = Score::from_results(&results);
        assert_eq!(s.total, 3);
        assert_eq!(s.passed, 2);
        assert_eq!(s.failed, 1);
        assert!((s.rate - 0.6666).abs() < 0.001);
    }
}
