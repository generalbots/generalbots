//! Agent/LLM-driven checkpoint selection for goal runs (#1706).
//!
//! The fixed-increment runner advances by `step_value` on every checkpoint.
//! When a run carries an instruction, the checkpoint step is chosen by the
//! configured LLM instead: the model sees the instruction and the remaining
//! distance and returns a bounded increment. Any failure (no provider, HTTP
//! error, malformed reply) keeps the fixed step, so a goal run never stalls
//! because the model is unavailable.
//!
//! Parsing is dependency-free and covered by executed tests; the network call
//! lives behind `choose_step` and is compiled only with the `llm` feature.

/// Largest fraction of the remaining distance a model-chosen step may take.
/// Keeps a model from jumping straight to the target and ending a long run.
const MAX_FRACTION: f64 = 0.2;

/// Extracts and clamps the `step` from a model reply such as
/// `{"step": 3}` (prose around the JSON is tolerated). `None` when the reply
/// carries no usable number.
#[must_use]
pub fn parse_step(reply: &str, remaining: f64) -> Option<f64> {
    if !remaining.is_finite() || remaining <= 0.0 {
        return None;
    }
    let start = reply.find('{')?;
    let end = reply.rfind('}')?;
    if end <= start {
        return None;
    }
    let value: serde_json::Value = serde_json::from_str(&reply[start..=end]).ok()?;
    let step = value.get("step")?.as_f64()?;
    if !step.is_finite() || step <= 0.0 {
        return None;
    }
    let ceiling = (remaining * MAX_FRACTION).max(0.01);
    Some(step.min(ceiling).min(remaining))
}

#[cfg(feature = "llm")]
pub async fn choose_step(
    provider: &std::sync::Arc<dyn botlib::traits::LLMProvider>,
    instruction: &str,
    current: f64,
    target: f64,
) -> Option<f64> {
    let remaining = target - current;
    if !remaining.is_finite() || remaining <= 0.0 {
        return None;
    }
    let ceiling = (remaining * MAX_FRACTION).max(0.01);
    let prompt = format!(
        "You drive the next checkpoint of a long-horizon goal.\n\
         Instruction: {}\n\
         Current value: {current:.2}. Target: {target:.2}. Remaining: {remaining:.2}.\n\
         Choose the increment for this checkpoint. Reply with JSON only, no prose:\n\
         {{\"step\": <number>}}\n\
         The step must be greater than 0 and at most {ceiling:.2}.",
        instruction.trim()
    );
    let reply = provider.generate_simple(&prompt).await.ok()?;
    parse_step(&reply, remaining)
}

#[cfg(not(feature = "llm"))]
pub async fn choose_step(
    _provider: &std::sync::Arc<dyn botlib::traits::LLMProvider>,
    _instruction: &str,
    _current: f64,
    _target: f64,
) -> Option<f64> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_plain_and_fenced_json() {
        assert_eq!(parse_step(r#"{"step": 3}"#, 100.0), Some(3.0));
        assert_eq!(
            parse_step("Sure:\n```json\n{\"step\": 2.5}\n```", 100.0),
            Some(2.5)
        );
    }

    #[test]
    fn clamps_to_the_remaining_distance_and_fraction() {
        assert_eq!(parse_step(r#"{"step": 50}"#, 100.0), Some(20.0));
        assert_eq!(parse_step(r#"{"step": 50}"#, 5.0), Some(1.0));
        assert_eq!(parse_step(r#"{"step": 0.0001}"#, 100.0), Some(0.0001));
    }

    #[test]
    fn rejects_unusable_replies() {
        assert_eq!(parse_step("no json here", 10.0), None);
        assert_eq!(parse_step(r#"{"step": 0}"#, 10.0), None);
        assert_eq!(parse_step(r#"{"step": -1}"#, 10.0), None);
        assert_eq!(parse_step(r#"{"other": 1}"#, 10.0), None);
        assert_eq!(parse_step(r#"{"step": 1}"#, 0.0), None);
    }
}
