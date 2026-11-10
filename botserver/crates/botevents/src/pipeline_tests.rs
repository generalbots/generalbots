//! Pipeline behaviour beyond the happy path: resumption, custom frames and the
//! model-facing error payload.

use std::sync::Arc;

use serde_json::json;

use super::{EventPipeline, PipelineError};
use crate::envelope::{EventSource, PendingEvent};
use crate::events::EventType;
use crate::redact::DefaultRedactor;
use crate::validate::Validator;

fn pipeline() -> EventPipeline {
    EventPipeline::new(Arc::new(DefaultRedactor::new()))
}

fn run_started(thread: &str) -> PendingEvent {
    PendingEvent::new(thread, "r1", EventSource::Runtime, EventType::RunStarted, json!({}))
}

#[test]
fn observe_seq_resumes_from_the_persisted_log() {
    let pipeline = pipeline();
    pipeline.observe_seq("t1", 41);
    let next = pipeline.prepare(run_started("t1")).expect("valid");
    assert_eq!(next.seq, 42);
    // A lower observation never rewinds the counter.
    pipeline.observe_seq("t1", 10);
    let next = pipeline.prepare(run_started("t1")).expect("valid");
    assert_eq!(next.seq, 43);
}

#[test]
fn prepare_custom_uses_the_custom_family() {
    let pipeline = pipeline();
    let envelope = pipeline
        .prepare_custom("t1", "r1", EventSource::App, "gb.profile", json!({ "id": "p1" }))
        .expect("valid");
    assert_eq!(envelope.event_type, EventType::Custom);
    assert_eq!(envelope.custom_name(), Some("gb.profile"));
    assert_eq!(envelope.data["payload"]["id"], "p1");
}

#[test]
fn prepare_custom_rejects_a_blank_name() {
    let pipeline = pipeline();
    let error = pipeline
        .prepare_custom("t1", "r1", EventSource::App, "", json!({}))
        .expect_err("must reject");
    let issues = error.issues().expect("validation issues");
    assert_eq!(issues.paths(), vec!["data.name"]);
}

#[test]
fn the_event_id_is_content_addressed() {
    let first = pipeline().prepare(run_started("t1")).expect("valid");
    let second = pipeline().prepare(run_started("t1")).expect("valid");
    assert_eq!(first.seq, second.seq);
    assert_eq!(first.event_id, second.event_id);
    assert_eq!(first.event_id.len(), 64);
    assert!(first.event_id.chars().all(|c| c.is_ascii_hexdigit()));

    let other_thread = pipeline().prepare(run_started("t2")).expect("valid");
    assert_ne!(first.event_id, other_thread.event_id);

    let other_family = pipeline()
        .prepare(PendingEvent::new(
            "t1",
            "r1",
            EventSource::Runtime,
            EventType::Custom,
            json!({ "name": "gb.ping" }),
        ))
        .expect("valid");
    assert_eq!(other_family.seq, 1);
    assert_ne!(first.event_id, other_family.event_id);
}

#[test]
fn a_custom_validator_replaces_the_default_limits() {
    let pipeline = pipeline().with_validator(Validator::new().with_max_payload_bytes(1024));
    let error = pipeline
        .prepare(PendingEvent::new(
            "t1",
            "r1",
            EventSource::Runtime,
            EventType::TextMessageContent,
            json!({ "message_id": "m1", "delta": "x".repeat(4096) }),
        ))
        .expect_err("must reject");
    assert!(error.issues().is_some());
}

#[test]
fn the_registry_is_shared_and_inspectable() {
    let registry = Arc::new(crate::sequencer::SequencerRegistry::new());
    let pipeline = pipeline().with_sequencers(Arc::clone(&registry));
    assert!(Arc::ptr_eq(pipeline.sequencers(), &registry));
    pipeline.prepare(run_started("t1")).expect("valid");
    assert_eq!(registry.sequencer("t1", 1).last_seq(), 1);
}

#[test]
fn a_validation_error_renders_as_a_model_retry_payload() {
    let pipeline = pipeline();
    let error = pipeline
        .prepare(PendingEvent::new(
            "t1",
            "r1",
            EventSource::Runtime,
            EventType::StepStarted,
            json!({ "name": "no id" }),
        ))
        .expect_err("must reject");
    assert!(error.issues().is_some());
    assert_eq!(
        error.to_json(),
        json!({
            "error": "invalid_event",
            "issues": [{ "path": "data.step_id", "reason": "required field is missing" }],
        })
    );
    assert!(error.to_string().starts_with("invalid event: data.step_id"));
    let as_error: &dyn std::error::Error = &error;
    assert!(as_error.to_string().contains("data.step_id"));
}

#[test]
fn an_internal_error_has_no_issues_and_uses_its_own_payload() {
    let error = PipelineError::Internal("hasher unavailable".to_string());
    assert!(error.issues().is_none());
    assert_eq!(
        error.to_json(),
        json!({ "error": "event_pipeline", "message": "hasher unavailable" })
    );
    assert_eq!(
        error.to_string(),
        "event pipeline error: hasher unavailable"
    );
}
