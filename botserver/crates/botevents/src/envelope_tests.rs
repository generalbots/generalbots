//! Envelope and pending-event behaviour that the transport relies on.

use chrono::{TimeZone, Utc};
use serde_json::json;

use super::{Envelope, EventSource, PendingEvent, ENVELOPE_VERSION};
use crate::events::{EventType, TextMessageContentData};

fn envelope(event_type: EventType, data: serde_json::Value) -> Envelope {
    Envelope {
        v: ENVELOPE_VERSION,
        event_type,
        thread_id: "thread-1".to_string(),
        run_id: "run-1".to_string(),
        parent_run_id: None,
        seq: 7,
        event_id: "f".repeat(64),
        ts: Utc.with_ymd_and_hms(2026, 1, 2, 3, 4, 5).single().expect("valid time"),
        source: EventSource::Runtime,
        data,
    }
}

#[test]
fn every_source_has_a_stable_wire_name() {
    let expected = [
        (EventSource::Runtime, "runtime"),
        (EventSource::Model, "model"),
        (EventSource::Tool, "tool"),
        (EventSource::App, "app"),
        (EventSource::User, "user"),
        (EventSource::System, "system"),
        (EventSource::Channel, "channel"),
    ];
    for (source, wire) in expected {
        assert_eq!(source.as_str(), wire);
        assert_eq!(source.to_string(), wire);
        assert_eq!(serde_json::to_value(source).expect("serializes"), json!(wire));
        let back: EventSource = serde_json::from_str(&format!("\"{wire}\"")).expect("parses");
        assert_eq!(back, source);
    }
    assert!(serde_json::from_str::<EventSource>("\"console\"").is_err());
}

#[test]
fn pending_event_defaults_to_a_root_run() {
    let event = PendingEvent::new("t", "r", EventSource::Runtime, EventType::RunStarted, json!({}));
    assert_eq!(event.parent_run_id, None);
    assert_eq!(event.thread_id, "t");
    assert_eq!(event.run_id, "r");
    assert_eq!(event.source, EventSource::Runtime);
    assert_eq!(event.event_type, EventType::RunStarted);
}

#[test]
fn with_parent_records_the_nested_run() {
    let parent = PendingEvent::new("t", "child", EventSource::Tool, EventType::RunStarted, json!({}))
        .with_parent(Some("parent".to_string()));
    assert_eq!(parent.parent_run_id.as_deref(), Some("parent"));
    let cleared = parent.with_parent(None);
    assert_eq!(cleared.parent_run_id, None);
}

#[test]
fn custom_pending_event_defaults_an_absent_payload() {
    let event = PendingEvent::custom("t", "r", EventSource::App, "gb.profile", json!({ "a": 1 }));
    assert_eq!(event.data["name"], "gb.profile");
    assert_eq!(event.data["payload"]["a"], 1);
}

#[test]
fn typed_data_parses_the_family_payload() {
    let value = envelope(
        EventType::TextMessageContent,
        json!({ "message_id": "m1", "delta": "hello" }),
    );
    let typed: TextMessageContentData = value.typed_data().expect("parses");
    assert_eq!(typed.message_id, "m1");
    assert_eq!(typed.delta, "hello");
    assert!(value.typed_data::<TextMessageContentData>().is_ok());
    let mismatch = envelope(EventType::TextMessageContent, json!({ "message_id": "m1" }));
    assert!(mismatch.typed_data::<TextMessageContentData>().is_err());
}

#[test]
fn only_run_terminators_are_terminal() {
    assert!(envelope(EventType::RunFinished, json!({})).is_terminal());
    assert!(envelope(EventType::RunError, json!({})).is_terminal());
    for event_type in [EventType::RunStarted, EventType::StepFinished, EventType::Custom] {
        assert!(!envelope(event_type, json!({})).is_terminal());
    }
}

#[test]
fn custom_name_is_only_read_from_custom_events() {
    let custom = envelope(EventType::Custom, json!({ "name": "gb.question" }));
    assert_eq!(custom.custom_name(), Some("gb.question"));
    let anonymous = envelope(EventType::Custom, json!({ "name": 3 }));
    assert_eq!(anonymous.custom_name(), None);
    let nameless = envelope(EventType::Custom, json!({}));
    assert_eq!(nameless.custom_name(), None);
    let other = envelope(EventType::RunStarted, json!({ "name": "gb.question" }));
    assert_eq!(other.custom_name(), None);
}

#[test]
fn to_json_round_trips_and_omits_an_absent_parent() {
    let root = envelope(EventType::StepStarted, json!({ "step_id": "s1", "name": "go" }));
    let value = root.to_json().expect("serializes");
    assert_eq!(value["v"], ENVELOPE_VERSION);
    assert_eq!(value["type"], "STEP_STARTED");
    assert!(value.get("parent_run_id").is_none());
    assert_eq!(value["seq"], 7);
    let back: Envelope = serde_json::from_value(value).expect("deserializes");
    assert_eq!(back, root);

    let mut nested = root.clone();
    nested.parent_run_id = Some("run-0".to_string());
    let value = nested.to_json().expect("serializes");
    assert_eq!(value["parent_run_id"], "run-0");
    let back: Envelope = serde_json::from_value(value).expect("deserializes");
    assert_eq!(back, nested);
}
