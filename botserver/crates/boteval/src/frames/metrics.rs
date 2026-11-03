//! Frame metrics computed from golden transcripts.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use botevents::Validator;

use super::fixtures::GoldenTranscript;
use super::replay::replay;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FrameMetrics {
    pub transcripts: usize,
    pub total_frames: usize,
    pub valid_frames: usize,
    pub invalid_frames: usize,
    pub ui_frames: usize,
    pub ui_frames_with_fallback: usize,
    pub duplicate_questions: usize,
    pub replay_failures: usize,
    pub custom_counts: BTreeMap<String, usize>,
}

impl FrameMetrics {
    #[must_use]
    pub fn validity_rate(&self) -> f64 {
        if self.total_frames == 0 {
            return 1.0;
        }
        self.valid_frames as f64 / self.total_frames as f64
    }

    #[must_use]
    pub fn fallback_coverage(&self) -> f64 {
        if self.ui_frames == 0 {
            return 1.0;
        }
        self.ui_frames_with_fallback as f64 / self.ui_frames as f64
    }
}

/// Measures validity, fallback coverage, duplicate questions and replay
/// determinism over every recorded transcript.
#[must_use]
pub fn measure(transcripts: &[GoldenTranscript]) -> FrameMetrics {
    let validator = Validator::new();
    let mut metrics = FrameMetrics {
        transcripts: transcripts.len(),
        ..FrameMetrics::default()
    };

    for transcript in transcripts {
        let mut question_ids: BTreeMap<String, usize> = BTreeMap::new();
        for frame in &transcript.frames {
            metrics.total_frames += 1;
            if validator.validate(frame).is_ok() {
                metrics.valid_frames += 1;
            } else {
                metrics.invalid_frames += 1;
            }
            if let Some(name) = frame.custom_name() {
                *metrics.custom_counts.entry(name.to_string()).or_insert(0) += 1;
                if is_ui_frame(name) {
                    metrics.ui_frames += 1;
                    if has_fallback(&frame.data) {
                        metrics.ui_frames_with_fallback += 1;
                    }
                }
                if name == "gb.question" {
                    if let Some(id) = question_id(&frame.data) {
                        let count = question_ids.entry(id.to_string()).or_insert(0);
                        *count += 1;
                        if *count > 1 {
                            metrics.duplicate_questions += 1;
                        }
                    }
                }
            }
        }

        match replay(&transcript.frames) {
            Ok(state) => {
                if let Ok(again) = replay(&transcript.frames) {
                    if state != again {
                        metrics.replay_failures += 1;
                    }
                }
            }
            Err(_) => metrics.replay_failures += 1,
        }
    }

    metrics
}

/// Names carried as `CUSTOM` that count as a UI frame.
#[must_use]
pub fn is_ui_frame(name: &str) -> bool {
    matches!(
        name,
        "gb.surface"
            | "gb.question"
            | "gb.plan"
            | "gb.artifact"
            | "gb.app"
            | "gb.metrics"
            | "gb.flow"
            | "gb.timeline"
            | "gb.chart"
    )
}

fn has_fallback(data: &Value) -> bool {
    data.get("payload")
        .and_then(|payload| payload.get("fallback"))
        .and_then(Value::as_str)
        .map(|fallback| !fallback.trim().is_empty())
        .unwrap_or(false)
}

fn question_id(data: &Value) -> Option<&str> {
    let payload = data.get("payload")?;
    payload
        .get("question_id")
        .and_then(Value::as_str)
        .or_else(|| payload.get("id").and_then(Value::as_str))
}

#[cfg(test)]
mod tests {
    use super::{is_ui_frame, FrameMetrics};

    #[test]
    fn ui_frame_names_are_explicit() {
        assert!(is_ui_frame("gb.surface"));
        assert!(is_ui_frame("gb.question"));
        assert!(!is_ui_frame("gb.status"));
    }

    #[test]
    fn empty_metrics_are_perfect() {
        let metrics = FrameMetrics::default();
        assert!((metrics.validity_rate() - 1.0).abs() < f64::EPSILON);
        assert!((metrics.fallback_coverage() - 1.0).abs() < f64::EPSILON);
    }
}
