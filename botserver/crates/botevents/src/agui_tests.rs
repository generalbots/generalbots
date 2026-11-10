//! AG-UI projection: the camelCase shape a standard client consumes.

use chrono::{TimeZone, Utc};

use super::{custom_profile, to_agui};
use crate::envelope::{Envelope, EventSource, ENVELOPE_VERSION};
use crate::events::EventType;

fn envelope(event_type: EventType, data: serde_json::Value) -> Envelope {
    Envelope {
        v: ENVELOPE_VERSION,
        event_type,
        thread_id: "thread-1".to_string(),
        run_id: "run-1".to_string(),
        parent_run_id: None,
        seq: 3,
        event_id: "b".repeat(64),
        ts: Utc.with_ymd_and_hms(2026, 5, 6, 7, 8, 9).single().expect("valid time"),
        source: EventSource::Tool,
        data,
    }
}

#[test]
fn every_header_field_is_projected() {
    let projected = to_agui(&envelope(
        EventType::ToolCallStart,
        serde_json::json!({ "call_id": "c1", "name": "search" }),
    ));
    assert_eq!(projected["type"], "TOOL_CALL_START");
    assert_eq!(projected["threadId"], "thread-1");
    assert_eq!(projected["runId"], "run-1");
    assert_eq!(projected["seq"], 3);
    assert_eq!(projected["eventId"], "b".repeat(64));
    assert_eq!(projected["source"], "tool");
    assert_eq!(projected["data"]["name"], "search");
    assert_eq!(projected["ts"], "2026-05-06T07:08:09+00:00");
}

#[test]
fn a_root_run_has_no_parent_key() {
    let projected = to_agui(&envelope(EventType::RunStarted, serde_json::json!({})));
    assert!(projected.get("parentRunId").is_none());
    assert!(!projected["type"].as_str().unwrap_or_default().is_empty());
}

#[test]
fn custom_profile_names_only_custom_events() {
    let custom = envelope(
        EventType::Custom,
        serde_json::json!({ "name": "gb.question", "payload": {} }),
    );
    assert_eq!(custom_profile(&custom), Some("gb.question"));
    assert_eq!(
        custom_profile(&envelope(EventType::RunStarted, serde_json::json!({}))),
        None
    );
    assert_eq!(
        custom_profile(&envelope(EventType::Custom, serde_json::json!({}))),
        None
    );
}
