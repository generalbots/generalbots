//! Event family vocabulary and payload serialization.

use std::collections::HashMap;

use serde_json::json;

use super::{
    to_data, EventType, RunFinishedData, TextMessageStartData, ToolCallResultData, Usage,
};
use crate::state::RunState;

#[test]
fn every_family_has_a_screaming_snake_name() {
    let expected = [
        (EventType::RunStarted, "RUN_STARTED"),
        (EventType::RunFinished, "RUN_FINISHED"),
        (EventType::RunError, "RUN_ERROR"),
        (EventType::StepStarted, "STEP_STARTED"),
        (EventType::StepFinished, "STEP_FINISHED"),
        (EventType::ToolCallStart, "TOOL_CALL_START"),
        (EventType::ToolCallArgs, "TOOL_CALL_ARGS"),
        (EventType::ToolCallEnd, "TOOL_CALL_END"),
        (EventType::ToolCallResult, "TOOL_CALL_RESULT"),
        (EventType::TextMessageStart, "TEXT_MESSAGE_START"),
        (EventType::TextMessageContent, "TEXT_MESSAGE_CONTENT"),
        (EventType::TextMessageEnd, "TEXT_MESSAGE_END"),
        (EventType::StateSnapshot, "STATE_SNAPSHOT"),
        (EventType::StateDelta, "STATE_DELTA"),
        (EventType::MessagesSnapshot, "MESSAGES_SNAPSHOT"),
        (EventType::ActivitySnapshot, "ACTIVITY_SNAPSHOT"),
        (EventType::ActivityDelta, "ACTIVITY_DELTA"),
        (EventType::Custom, "CUSTOM"),
        (EventType::Raw, "RAW"),
        (EventType::SubagentStarted, "SUBAGENT_STARTED"),
        (EventType::SubagentFinished, "SUBAGENT_FINISHED"),
    ];
    assert_eq!(expected.len(), EventType::ALL.len());
    for (event_type, wire) in expected {
        assert_eq!(event_type.as_str(), wire);
        assert_eq!(event_type.to_string(), wire);
        assert_eq!(
            serde_json::to_value(event_type).expect("serializes"),
            json!(wire)
        );
    }
}

#[test]
fn every_family_in_all_is_serializable_and_back() {
    for event_type in EventType::ALL {
        let json = serde_json::to_string(&event_type).expect("serializes");
        let back: EventType = serde_json::from_str(&json).expect("deserializes");
        assert_eq!(back, event_type);
    }
}

#[test]
fn required_fields_are_declared_for_the_payload_carrying_families() {
    assert!(EventType::RunStarted.required_data_fields().is_empty());
    assert!(EventType::RunFinished.required_data_fields().is_empty());
    assert_eq!(EventType::RunError.required_data_fields(), &["message"][..]);
    assert_eq!(
        EventType::ToolCallResult.required_data_fields(),
        &["call_id", "status"][..]
    );
    assert_eq!(EventType::Custom.required_data_fields(), &["name"][..]);
    for event_type in EventType::ALL {
        for field in event_type.required_data_fields() {
            assert!(!field.is_empty(), "{event_type} declares an empty field");
        }
    }
}

#[test]
fn to_data_keeps_an_object_payload_flat() {
    let data = to_data(&TextMessageStartData {
        message_id: "m1".to_string(),
        role: "assistant".to_string(),
    });
    assert_eq!(data, json!({ "message_id": "m1", "role": "assistant" }));
}

#[test]
fn to_data_wraps_a_scalar_payload_under_value() {
    assert_eq!(to_data(&"plain text"), json!({ "value": "plain text" }));
    assert_eq!(to_data(&7u8), json!({ "value": 7 }));
}

#[test]
fn to_data_fails_closed_on_an_unserializable_payload() {
    let mut map: HashMap<(u8, u8), u8> = HashMap::new();
    map.insert((1, 2), 3);
    assert_eq!(to_data(&map), json!({}));
}

#[test]
fn optional_fields_are_omitted_instead_of_serialized_as_null() {
    let finished = to_data(&RunFinishedData {
        state: RunState::Succeeded,
        duration_ms: None,
        exit_code: None,
        usage: None,
    });
    assert_eq!(finished, json!({ "state": "succeeded" }));

    let usage = to_data(&Usage::default());
    assert_eq!(usage, json!({}));

    let result = to_data(&ToolCallResultData {
        call_id: "c1".to_string(),
        status: "ok".to_string(),
        result: serde_json::Value::Null,
        duration_ms: Some(12),
    });
    assert_eq!(result["duration_ms"], 12);
    assert!(result.get("result").is_some());
}

#[test]
fn text_message_start_defaults_the_role_to_assistant() {
    let typed: TextMessageStartData =
        serde_json::from_value(json!({ "message_id": "m1" })).expect("parses");
    assert_eq!(typed.role, "assistant");
    let typed: TextMessageStartData =
        serde_json::from_value(json!({ "message_id": "m1", "role": "user" })).expect("parses");
    assert_eq!(typed.role, "user");
}
