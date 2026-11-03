//! Loading and checking the checked-in frame fixtures.

use serde::{Deserialize, Serialize};

use botevents::{Envelope, Validator};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameFixture {
    pub name: String,
    pub envelope: serde_json::Value,
    pub valid: bool,
    #[serde(default)]
    pub expected_error_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoldenTranscript {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub frames: Vec<Envelope>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactionCase {
    pub name: String,
    pub secrets: Vec<String>,
    pub payload: serde_json::Value,
    pub must_not_contain: Vec<String>,
    pub must_contain: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FixtureReport {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub failures: Vec<String>,
}

impl FixtureReport {
    #[must_use]
    pub fn is_pass(&self) -> bool {
        self.failed == 0 && self.total > 0
    }
}

const FRAME_FIXTURES: &str = include_str!("../../fixtures/run_events/frames.json");
const GOLDEN_TRANSCRIPTS: &str = include_str!("../../fixtures/run_events/golden.json");
const REDACTION_CASES: &str = include_str!("../../fixtures/run_events/redaction.json");

/// Runs every fixture through parse + validation and checks the expected
/// verdict plus the expected schema paths.
pub fn load_frame_fixtures() -> Result<FixtureReport, String> {
    let fixtures: Vec<FrameFixture> = serde_json::from_str(FRAME_FIXTURES)
        .map_err(|error| format!("invalid frame fixtures: {error}"))?;
    let validator = Validator::new();
    let mut report = FixtureReport {
        total: fixtures.len(),
        ..FixtureReport::default()
    };
    for fixture in fixtures {
        match serde_json::from_value::<Envelope>(fixture.envelope) {
            Ok(envelope) => {
                let result = validator.validate(&envelope);
                match (fixture.valid, result) {
                    (true, Ok(())) => report.passed += 1,
                    (true, Err(issues)) => {
                        report.failed += 1;
                        report.failures.push(format!(
                            "{}: expected valid, got {issues}",
                            fixture.name
                        ));
                    }
                    (false, Ok(())) => {
                        report.failed += 1;
                        report
                            .failures
                            .push(format!("{}: expected invalid, was accepted", fixture.name));
                    }
                    (false, Err(issues)) => {
                        let paths = issues.paths();
                        let missing: Vec<&String> = fixture
                            .expected_error_paths
                            .iter()
                            .filter(|path| !paths.contains(path))
                            .collect();
                        if missing.is_empty() {
                            report.passed += 1;
                        } else {
                            report.failed += 1;
                            report.failures.push(format!(
                                "{}: missing expected paths {missing:?} (got {paths:?})",
                                fixture.name
                            ));
                        }
                    }
                }
            }
            Err(error) => {
                if fixture.valid {
                    report.failed += 1;
                    report
                        .failures
                        .push(format!("{}: does not parse: {error}", fixture.name));
                } else {
                    report.passed += 1;
                }
            }
        }
    }
    Ok(report)
}

pub fn load_golden() -> Result<Vec<GoldenTranscript>, String> {
    serde_json::from_str(GOLDEN_TRANSCRIPTS)
        .map_err(|error| format!("invalid golden transcripts: {error}"))
}

pub fn load_redaction_cases() -> Result<Vec<RedactionCase>, String> {
    serde_json::from_str(REDACTION_CASES)
        .map_err(|error| format!("invalid redaction cases: {error}"))
}

#[cfg(test)]
mod tests {
    use botevents::{DefaultRedactor, FrameRedactor};

    use super::{load_frame_fixtures, load_golden, load_redaction_cases};
    use crate::frames::gate::{evaluate, FrameGateConfig};
    use crate::frames::metrics::measure;
    use crate::frames::prompts::{evaluate_restraint, load_prompt_cases, PromptOutcome};
    use crate::frames::replay::replay;

    #[test]
    fn every_fixture_matches_its_expectation() {
        let report = load_frame_fixtures().expect("fixtures parse");
        assert!(report.is_pass(), "{:?}", report.failures);
        assert!(report.total >= 21, "one fixture per event family");
    }

    #[test]
    fn golden_transcripts_are_recorded() {
        let golden = load_golden().expect("golden transcripts parse");
        assert!(!golden.is_empty());
    }

    #[test]
    fn golden_transcripts_replay_twice_to_the_same_state() {
        for transcript in load_golden().expect("golden transcripts parse") {
            let first = replay(&transcript.frames).expect("first replay");
            let second = replay(&transcript.frames).expect("second replay");
            assert_eq!(first, second, "transcript {} is not deterministic", transcript.name);
        }
    }

    #[test]
    fn metrics_and_gate_pass_on_the_golden_corpus() {
        let golden = load_golden().expect("golden transcripts parse");
        let metrics = measure(&golden);
        assert_eq!(metrics.invalid_frames, 0);
        assert_eq!(metrics.duplicate_questions, 0);
        assert_eq!(metrics.replay_failures, 0);
        let cases = load_prompt_cases().expect("prompt cases parse");
        assert!(cases.len() >= 200, "prompt set must cover 200+ cases");
        let outcomes: Vec<PromptOutcome> = cases
            .iter()
            .map(|case| PromptOutcome {
                id: case.id.clone(),
                produced_ui: case.expects_ui,
            })
            .collect();
        let restraint = evaluate_restraint(&cases, &outcomes);
        let verdict = evaluate(&FrameGateConfig::default(), &metrics, &restraint);
        assert!(verdict.pass, "{:?}", verdict.reasons);
    }

    #[test]
    fn redaction_cases_are_recorded_and_masked() {
        let cases = load_redaction_cases().expect("redaction cases parse");
        assert!(!cases.is_empty());
        for case in cases {
            let redactor = DefaultRedactor::new().with_secrets(case.secrets);
            let mut payload = case.payload;
            redactor.redact(&mut payload);
            let text = payload.to_string();
            for forbidden in case.must_not_contain {
                assert!(!text.contains(&forbidden), "{} leaked {forbidden}", case.name);
            }
            for expected in case.must_contain {
                assert!(text.contains(&expected), "{} missed {expected}", case.name);
            }
        }
    }
}
