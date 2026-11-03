//! The checked-in JSON Schema for the envelope.
//!
//! `EVENT_SPECS` in `events.rs` is the single source of truth for required
//! payload keys; this module turns it into draft 2020-12 JSON Schema so the
//! server validator, the model tool schemas and the eval fixtures all agree.
//! `examples/dump_schema.rs` writes the artifact consumed by the frontend and
//! by `boteval`.

use serde_json::{json, Map, Value};

use super::envelope::ENVELOPE_VERSION;
use super::events::EventType;

pub const SCHEMA_ID: &str = "https://general bots/events/run-event.v1.json";

#[must_use]
pub fn envelope_schema() -> Value {
    let types: Vec<Value> = EventType::ALL
        .iter()
        .map(|event_type| Value::String(event_type.as_str().to_string()))
        .collect();

    let mut all_of = Vec::new();
    let mut specs = Map::new();
    for event_type in EventType::ALL {
        let required = event_type.required_data_fields();
        specs.insert(
            event_type.as_str().to_string(),
            json!({ "required": required }),
        );
        if required.is_empty() {
            continue;
        }
        all_of.push(json!({
            "if": {
                "properties": { "type": { "const": event_type.as_str() } },
                "required": ["type"]
            },
            "then": {
                "properties": {
                    "data": {
                        "type": "object",
                        "required": required
                    }
                }
            }
        }));
    }

    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": SCHEMA_ID,
        "title": "General Bots run event",
        "description": "One AG-UI compatible frame in a conversation thread. Sequence numbers are dense and gapless per thread.",
        "type": "object",
        "additionalProperties": false,
        "required": [
            "v", "type", "thread_id", "run_id", "seq", "event_id", "ts", "source", "data"
        ],
        "properties": {
            "v": { "const": ENVELOPE_VERSION },
            "type": { "enum": types },
            "thread_id": { "type": "string", "minLength": 1, "maxLength": 200 },
            "run_id": { "type": "string", "minLength": 1, "maxLength": 200 },
            "parent_run_id": { "type": "string", "minLength": 1, "maxLength": 200 },
            "seq": { "type": "integer", "minimum": 1 },
            "event_id": { "type": "string", "minLength": 16, "maxLength": 128 },
            "ts": { "type": "string", "format": "date-time" },
            "source": {
                "enum": ["runtime", "model", "tool", "app", "user", "system", "channel"]
            },
            "data": { "type": "object" }
        },
        "allOf": all_of,
        "x-event-specs": specs
    })
}

/// Writes the schema next to the crate so the frontend and the eval fixtures
/// can load the exact same file.
pub fn write_schema(path: &std::path::Path) -> std::io::Result<()> {
    let schema = envelope_schema();
    let text = serde_json::to_string_pretty(&schema)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, format!("{text}\n"))
}

#[cfg(test)]
mod tests {
    use super::envelope_schema;
    use crate::events::EventType;

    #[test]
    fn schema_lists_every_family() {
        let schema = envelope_schema();
        let enum_values = schema["properties"]["type"]["enum"]
            .as_array()
            .expect("type enum");
        assert_eq!(enum_values.len(), EventType::ALL.len());
        assert!(enum_values.iter().any(|value| value == "CUSTOM"));
    }

    #[test]
    fn schema_requires_the_envelope_header() {
        let schema = envelope_schema();
        let required = schema["required"].as_array().expect("required");
        for field in ["v", "type", "thread_id", "run_id", "seq", "event_id", "ts", "source", "data"] {
            assert!(required.iter().any(|value| value == field), "missing {field}");
        }
    }
}
