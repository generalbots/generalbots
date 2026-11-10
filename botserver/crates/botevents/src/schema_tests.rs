//! The checked-in JSON Schema must stay in step with `EventType`.

use serde_json::Value;

use super::{envelope_schema, write_schema, SCHEMA_ID};
use crate::events::EventType;

#[test]
fn the_schema_header_is_stable() {
    let schema = envelope_schema();
    assert_eq!(schema["$id"], SCHEMA_ID);
    assert_eq!(schema["$schema"], "https://json-schema.org/draft/2020-12/schema");
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(schema["properties"]["v"]["const"], 1);
    assert_eq!(schema["properties"]["seq"]["minimum"], 1);
    assert_eq!(schema["properties"]["thread_id"]["maxLength"], 200);
}

#[test]
fn the_source_enumeration_matches_event_source() {
    let schema = envelope_schema();
    let sources = schema["properties"]["source"]["enum"]
        .as_array()
        .expect("source enum");
    assert_eq!(sources.len(), 7);
    for source in ["runtime", "model", "tool", "app", "user", "system", "channel"] {
        assert!(sources.iter().any(|value| value == source), "{source}");
    }
}

#[test]
fn every_family_declares_its_required_fields() {
    let schema = envelope_schema();
    let specs = schema["x-event-specs"].as_object().expect("x-event-specs");
    assert_eq!(specs.len(), EventType::ALL.len());
    for event_type in EventType::ALL {
        let entry = specs
            .get(event_type.as_str())
            .unwrap_or_else(|| panic!("missing spec for {event_type}"));
        let required = entry["required"].as_array().expect("required array");
        assert_eq!(required.len(), event_type.required_data_fields().len());
    }
}

#[test]
fn only_families_with_required_fields_get_a_conditional() {
    let schema = envelope_schema();
    let conditional = schema["allOf"].as_array().expect("allOf");
    let expected = EventType::ALL
        .iter()
        .filter(|event_type| !event_type.required_data_fields().is_empty())
        .count();
    assert_eq!(conditional.len(), expected);
    for entry in conditional {
        assert_eq!(entry["if"]["required"][0], "type");
        assert!(entry["then"]["properties"]["data"]["required"].is_array());
    }
}

#[test]
fn write_schema_produces_a_parseable_artifact() {
    let path = std::env::temp_dir().join(format!(
        "botevents-schema-{}-{}.json",
        std::process::id(),
        SCHEMA_ID.len()
    ));
    write_schema(&path).expect("writes the schema");
    let text = std::fs::read_to_string(&path).expect("reads the schema");
    let parsed: Value = serde_json::from_str(&text).expect("valid JSON");
    assert_eq!(parsed, envelope_schema());
    assert!(text.ends_with('\n'));
    std::fs::remove_file(&path).expect("removes the artifact");
}
