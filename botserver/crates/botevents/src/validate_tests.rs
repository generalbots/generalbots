//! Validation contract: which envelopes are accepted and which paths a
//! rejected frame reports back to the model.

use chrono::Utc;
use serde_json::json;

use super::{ValidationIssue, ValidationIssues, Validator};
use crate::envelope::{Envelope, EventSource, ENVELOPE_VERSION};
use crate::events::EventType;

fn envelope(event_type: EventType, data: serde_json::Value) -> Envelope {
    Envelope {
        v: ENVELOPE_VERSION,
        event_type,
        thread_id: "thread-1".to_string(),
        run_id: "run-1".to_string(),
        parent_run_id: None,
        seq: 1,
        event_id: "a".repeat(64),
        ts: Utc::now(),
        source: EventSource::Runtime,
        data,
    }
}

fn issues(event_type: EventType, data: serde_json::Value) -> ValidationIssues {
    Validator::new()
        .validate(&envelope(event_type, data))
        .expect_err("must be rejected")
}

#[test]
fn issues_collect_paths_reasons_and_render_them() {
    let mut issues = ValidationIssues::new();
    assert!(issues.is_empty());
    assert_eq!(issues.len(), 0);
    issues.push("data.a", "required field is missing");
    issues.push("data.b", "required field is empty");
    assert_eq!(issues.len(), 2);
    assert!(!issues.is_empty());
    assert_eq!(issues.paths(), vec!["data.a", "data.b"]);
    assert_eq!(
        issues.to_json(),
        json!([
            { "path": "data.a", "reason": "required field is missing" },
            { "path": "data.b", "reason": "required field is empty" },
        ])
    );
    assert_eq!(
        issues.to_string(),
        "data.a: required field is missing; data.b: required field is empty"
    );
    let single = ValidationIssues {
        issues: vec![ValidationIssue::new("v", "bad")],
    };
    assert_eq!(single.to_string(), "v: bad");
    let as_error: &dyn std::error::Error = &issues;
    assert!(as_error.to_string().contains("data.a"));
}

#[test]
fn header_fields_are_checked_individually() {
    let mut value = envelope(EventType::RunStarted, json!({}));
    value.v = 99;
    value.thread_id = "  ".to_string();
    value.run_id = "r".repeat(201);
    value.seq = 0;
    value.event_id = String::new();
    let reported = Validator::new().validate(&value).expect_err("must reject");
    assert_eq!(
        reported.paths(),
        vec!["v", "thread_id", "run_id", "seq", "event_id"]
    );
    assert!(reported.issues[0].reason.contains("99"));
    assert_eq!(reported.issues[2].reason, "identifier exceeds 200 characters");
    assert_eq!(reported.issues[3].reason, "sequence numbers start at 1");
    assert_eq!(reported.issues[4].reason, "event id is required");
}

#[test]
fn identifiers_reject_control_characters() {
    let mut value = envelope(EventType::RunStarted, json!({}));
    value.thread_id = "thread\n1".to_string();
    let reported = Validator::new().validate(&value).expect_err("must reject");
    assert_eq!(reported.paths(), vec!["thread_id"]);
    assert_eq!(
        reported.issues[0].reason,
        "identifier contains control characters"
    );
}

#[test]
fn a_run_cannot_parent_itself() {
    let mut value = envelope(EventType::RunStarted, json!({}));
    value.parent_run_id = Some("run-1".to_string());
    let reported = Validator::new().validate(&value).expect_err("must reject");
    assert_eq!(reported.paths(), vec!["parent_run_id"]);
    assert_eq!(reported.issues[0].reason, "a run cannot parent itself");

    let mut nested = envelope(EventType::RunStarted, json!({}));
    nested.parent_run_id = Some("run-0".to_string());
    assert!(Validator::new().validate(&nested).is_ok());
}

#[test]
fn a_non_object_payload_stops_validation_early() {
    for data in [json!([]), json!("text"), json!(3), json!(null)] {
        let reported = issues(EventType::RunStarted, data);
        assert_eq!(reported.paths(), vec!["data"], "payload {reported}");
        assert_eq!(reported.issues[0].reason, "payload must be a JSON object");
    }
}

#[test]
fn a_required_field_may_be_missing_null_or_blank() {
    let reported = issues(EventType::StepStarted, json!({ "step_id": "s1" }));
    assert_eq!(reported.paths(), vec!["data.name"]);
    assert_eq!(reported.issues[0].reason, "required field is missing");

    let reported = issues(EventType::StepStarted, json!({ "step_id": "s1", "name": null }));
    assert_eq!(reported.issues[0].reason, "required field is missing");

    let reported = issues(EventType::StepStarted, json!({ "step_id": "s1", "name": "   " }));
    assert_eq!(reported.issues[0].reason, "required field is empty");
}

#[test]
fn run_finished_must_carry_a_terminal_state() {
    for state in ["succeeded", "failed", "cancelled"] {
        assert!(Validator::new()
            .validate(&envelope(EventType::RunFinished, json!({ "state": state })))
            .is_ok());
    }
    let reported = issues(EventType::RunFinished, json!({ "state": "paused" }));
    assert_eq!(reported.paths(), vec!["data.state"]);
    assert_eq!(
        reported.issues[0].reason,
        "RUN_FINISHED cannot carry the non-terminal state 'paused'"
    );
    let reported = issues(EventType::RunFinished, json!({ "state": "done" }));
    assert_eq!(reported.issues[0].reason, "unknown run state");
}

#[test]
fn step_finished_status_is_an_enumeration() {
    for status in ["ok", "error", "skipped", "cancelled"] {
        assert!(Validator::new()
            .validate(&envelope(
                EventType::StepFinished,
                json!({ "step_id": "s1", "status": status })
            ))
            .is_ok());
    }
    let reported = issues(
        EventType::StepFinished,
        json!({ "step_id": "s1", "status": "done" }),
    );
    assert_eq!(reported.paths(), vec!["data.status"]);
    assert_eq!(
        reported.issues[0].reason,
        "'done' is not one of ok, error, skipped, cancelled"
    );
    let reported = issues(
        EventType::StepFinished,
        json!({ "step_id": "s1", "status": 1 }),
    );
    assert_eq!(reported.issues[0].reason, "expected a string");
}

#[test]
fn tool_call_result_accepts_the_wider_status_set() {
    for status in ["ok", "error", "denied", "cancelled", "timeout"] {
        assert!(Validator::new()
            .validate(&envelope(
                EventType::ToolCallResult,
                json!({ "call_id": "c1", "status": status })
            ))
            .is_ok());
    }
    let reported = issues(
        EventType::ToolCallResult,
        json!({ "call_id": "c1", "status": "skipped" }),
    );
    assert_eq!(reported.paths(), vec!["data.status"]);
}

#[test]
fn a_missing_required_status_is_reported_once() {
    // `status` is a required field for the family, so the missing key is
    // reported once, by the required-field pass, and not again by the family
    // rule.
    let reported = issues(EventType::ToolCallResult, json!({ "call_id": "c1" }));
    assert_eq!(reported.paths(), vec!["data.status"]);
    assert_eq!(reported.issues[0].reason, "required field is missing");
}

#[test]
fn custom_names_are_bounded() {
    let long = "x".repeat(129);
    let reported = issues(EventType::Custom, json!({ "name": long }));
    assert_eq!(reported.paths(), vec!["data.name"]);
    assert_eq!(
        reported.issues[0].reason,
        "custom event name exceeds 128 characters"
    );
    let boundary = "x".repeat(128);
    assert!(Validator::new()
        .validate(&envelope(EventType::Custom, json!({ "name": boundary })))
        .is_ok());
}

#[test]
fn delta_patches_must_be_arrays() {
    for event_type in [EventType::StateDelta, EventType::ActivityDelta] {
        let mut data = json!({ "patch": "not an array" });
        if event_type == EventType::ActivityDelta {
            data["activity_id"] = json!("a1");
        } else {
            data["patch"] = json!({ "op": "add" });
        }
        let reported = issues(event_type, data);
        assert_eq!(reported.paths(), vec!["data.patch"], "{event_type}");
        assert_eq!(
            reported.issues[0].reason,
            "JSON Patch must be an array of operations"
        );
    }
    assert!(Validator::new()
        .validate(&envelope(
            EventType::StateDelta,
            json!({ "patch": [{ "op": "add", "path": "/a", "value": 1 }] })
        ))
        .is_ok());
}

#[test]
fn messages_snapshot_requires_an_array() {
    let reported = issues(EventType::MessagesSnapshot, json!({ "messages": {} }));
    assert_eq!(reported.paths(), vec!["data.messages"]);
    assert_eq!(reported.issues[0].reason, "messages must be an array");
}

#[test]
fn subagent_finished_requires_a_terminal_state() {
    let reported = issues(
        EventType::SubagentFinished,
        json!({ "child_run_id": "c1", "state": "running" }),
    );
    assert_eq!(reported.paths(), vec!["data.state"]);
    assert_eq!(
        reported.issues[0].reason,
        "SUBAGENT_FINISHED requires a terminal state, got 'running'"
    );
    assert!(Validator::new()
        .validate(&envelope(
            EventType::SubagentFinished,
            json!({ "child_run_id": "c1", "state": "failed" })
        ))
        .is_ok());
}

#[test]
fn a_subagent_run_must_differ_from_its_parent() {
    let reported = issues(
        EventType::SubagentStarted,
        json!({ "child_run_id": "run-1", "agent_name": "planner" }),
    );
    assert_eq!(reported.paths(), vec!["data.child_run_id"]);
    assert_eq!(
        reported.issues[0].reason,
        "a sub-agent run must differ from its parent run"
    );
    assert!(Validator::new()
        .validate(&envelope(
            EventType::SubagentStarted,
            json!({ "child_run_id": "run-2", "agent_name": "planner" })
        ))
        .is_ok());
}

#[test]
fn the_payload_limit_has_a_floor_of_one_kibibyte() {
    let validator = Validator::default().with_max_payload_bytes(0);
    let payload = "x".repeat(2048);
    let reported = validator
        .validate(&envelope(
            EventType::TextMessageContent,
            json!({ "message_id": "m1", "delta": payload }),
        ))
        .expect_err("must reject");
    assert_eq!(reported.paths(), vec!["data"]);
    assert!(reported.issues[0].reason.contains("the limit is 1024 bytes"));
}
