//! The emission pipeline.
//!
//! `PendingEvent -> redact -> validate -> sequence -> Envelope`
//!
//! Validation runs before the sequence number is minted so an invalid frame
//! cannot leave a gap in the thread log. The envelope is validated with a
//! placeholder sequence of 1 (the satisfied minimum) and the real number is
//! assigned only after the payload passes.

use std::fmt;
use std::sync::Arc;

use chrono::Utc;
use serde_json::Value;
use sha2::{Digest, Sha256};

use super::envelope::{Envelope, PendingEvent, ENVELOPE_VERSION};
use super::redact::FrameRedactor;
use super::sequencer::SequencerRegistry;
use super::validate::{ValidationIssues, Validator};

#[derive(Debug, Clone)]
pub enum PipelineError {
    Validation(ValidationIssues),
    Internal(String),
}

impl PipelineError {
    #[must_use]
    pub fn issues(&self) -> Option<&ValidationIssues> {
        match self {
            Self::Validation(issues) => Some(issues),
            Self::Internal(_) => None,
        }
    }

    /// Model-facing error payload: schema paths plus reasons.
    #[must_use]
    pub fn to_json(&self) -> Value {
        match self {
            Self::Validation(issues) => serde_json::json!({
                "error": "invalid_event",
                "issues": issues.to_json(),
            }),
            Self::Internal(message) => serde_json::json!({
                "error": "event_pipeline",
                "message": message,
            }),
        }
    }
}

impl fmt::Display for PipelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation(issues) => write!(f, "invalid event: {issues}"),
            Self::Internal(message) => write!(f, "event pipeline error: {message}"),
        }
    }
}

impl std::error::Error for PipelineError {}

pub struct EventPipeline {
    redactor: Arc<dyn FrameRedactor>,
    validator: Validator,
    sequencers: Arc<SequencerRegistry>,
}

impl EventPipeline {
    #[must_use]
    pub fn new(redactor: Arc<dyn FrameRedactor>) -> Self {
        Self {
            redactor,
            validator: Validator::new(),
            sequencers: Arc::new(SequencerRegistry::new()),
        }
    }

    #[must_use]
    pub fn with_validator(mut self, validator: Validator) -> Self {
        self.validator = validator;
        self
    }

    #[must_use]
    pub fn with_sequencers(mut self, sequencers: Arc<SequencerRegistry>) -> Self {
        self.sequencers = sequencers;
        self
    }

    #[must_use]
    pub fn sequencers(&self) -> &Arc<SequencerRegistry> {
        &self.sequencers
    }

    /// Seeds the thread counter from the last persisted `seq` after a restart.
    pub fn observe_seq(&self, thread_id: &str, seq: u64) {
        self.sequencers.sequencer(thread_id, 1).advance_to(seq);
    }

    pub fn prepare(&self, pending: PendingEvent) -> Result<Envelope, PipelineError> {
        let PendingEvent {
            thread_id,
            run_id,
            parent_run_id,
            source,
            event_type,
            mut data,
        } = pending;

        self.redactor.redact(&mut data);

        let mut envelope = Envelope {
            v: ENVELOPE_VERSION,
            event_type,
            thread_id: thread_id.clone(),
            run_id,
            parent_run_id,
            // Placeholders, replaced after validation so a rejected frame does
            // not consume a sequence number. The validator only requires a
            // non-empty id here; the content-derived id is assigned after the
            // sequence number.
            seq: 1,
            event_id: "pending-event-id".to_string(),
            ts: Utc::now(),
            source,
            data,
        };

        self.validator
            .validate(&envelope)
            .map_err(PipelineError::Validation)?;

        let sequencer = self.sequencers.sequencer(&thread_id, 1);
        envelope.seq = sequencer.next_seq();
        envelope.event_id = derive_event_id(&envelope)?;
        Ok(envelope)
    }

    pub fn prepare_custom(
        &self,
        thread_id: impl Into<String>,
        run_id: impl Into<String>,
        source: super::envelope::EventSource,
        name: impl Into<String>,
        payload: Value,
    ) -> Result<Envelope, PipelineError> {
        self.prepare(PendingEvent::custom(thread_id, run_id, source, name, payload))
    }
}

/// Stable content id: the same frame replayed from the log produces the same
/// id, which lets a subscriber drop duplicates after a reconnect.
fn derive_event_id(envelope: &Envelope) -> Result<String, PipelineError> {
    let payload = serde_json::to_vec(&envelope.data)
        .map_err(|error| PipelineError::Internal(format!("payload is not serializable: {error}")))?;
    let mut hasher = Sha256::new();
    hasher.update(envelope.thread_id.as_bytes());
    hasher.update(b"|");
    hasher.update(envelope.seq.to_be_bytes());
    hasher.update(b"|");
    hasher.update(envelope.event_type.as_str().as_bytes());
    hasher.update(b"|");
    hasher.update(&payload);
    let digest = hasher.finalize();
    Ok(digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use serde_json::json;

    use super::{EventPipeline, PipelineError};
    use crate::envelope::{EventSource, PendingEvent};
    use crate::events::EventType;
    use crate::redact::DefaultRedactor;

    fn pipeline() -> EventPipeline {
        EventPipeline::new(Arc::new(
            DefaultRedactor::new().with_secrets(["super-secret-canary"]),
        ))
    }

    #[test]
    fn sequences_dense_and_deterministic() {
        let pipeline = pipeline();
        let first = pipeline
            .prepare(PendingEvent::new(
                "t1",
                "r1",
                EventSource::Runtime,
                EventType::RunStarted,
                json!({}),
            ))
            .expect("valid");
        let second = pipeline
            .prepare(PendingEvent::new(
                "t1",
                "r1",
                EventSource::Runtime,
                EventType::StepStarted,
                json!({ "step_id": "s1", "name": "prepare" }),
            ))
            .expect("valid");
        assert_eq!(first.seq, 1);
        assert_eq!(second.seq, 2);
        assert_eq!(first.event_id.len(), 64);
        assert_ne!(first.event_id, second.event_id);
    }

    #[test]
    fn redacts_before_sequencing() {
        let pipeline = pipeline();
        let envelope = pipeline
            .prepare(PendingEvent::new(
                "t1",
                "r1",
                EventSource::Tool,
                EventType::ToolCallResult,
                json!({
                    "call_id": "c1",
                    "status": "ok",
                    "result": { "stdout": "token=super-secret-canary" }
                }),
            ))
            .expect("valid");
        assert!(!envelope.data.to_string().contains("super-secret-canary"));
        assert!(envelope.data.to_string().contains("[REDACTED]"));
    }

    #[test]
    fn invalid_frame_does_not_consume_a_sequence_number() {
        let pipeline = pipeline();
        let invalid = pipeline.prepare(PendingEvent::new(
            "t2",
            "r1",
            EventSource::Runtime,
            EventType::StepStarted,
            json!({ "name": "missing id" }),
        ));
        assert!(matches!(invalid, Err(PipelineError::Validation(_))));
        let valid = pipeline
            .prepare(PendingEvent::new(
                "t2",
                "r1",
                EventSource::Runtime,
                EventType::RunStarted,
                json!({}),
            ))
            .expect("valid");
        assert_eq!(valid.seq, 1);
    }
}
