//! Schema registry for Office events (#2503).
//!
//! The registry is generated from the taxonomy so every published event name
//! has exactly one entry, with a version, description, required fields and a
//! complete example payload. Bot authors can read the JSON form through the
//! admin API without a second, hand-maintained document drifting away.

use serde_json::{json, Value};

use super::taxonomy::{OfficeEventType, PAYLOAD_SCHEMA_VERSION};

/// Common required fields for every Office event payload.
pub const COMMON_REQUIRED_FIELDS: [&str; 7] = [
    "event_id",
    "type",
    "schema_version",
    "app",
    "workspace_id",
    "doc_id",
    "actor_id",
];

#[derive(Debug, Clone, PartialEq)]
pub struct SchemaEntry {
    pub name: &'static str,
    pub version: u32,
    pub description: &'static str,
    pub app: &'static str,
    pub required_fields: Vec<&'static str>,
    pub example: Value,
}

/// Returns one entry per event type in the taxonomy.
#[must_use]
pub fn registry() -> Vec<SchemaEntry> {
    OfficeEventType::ALL
        .iter()
        .map(|event_type| SchemaEntry {
            name: event_type.as_str(),
            version: PAYLOAD_SCHEMA_VERSION,
            description: event_type.description(),
            app: event_type.app(),
            required_fields: required_fields(*event_type),
            example: example_for(*event_type),
        })
        .collect()
}

/// JSON form served by the admin API and dumped next to the crate.
#[must_use]
pub fn registry_json() -> Value {
    let events: Vec<Value> = registry()
        .into_iter()
        .map(|entry| {
            json!({
                "name": entry.name,
                "version": entry.version,
                "description": entry.description,
                "app": entry.app,
                "required_fields": entry.required_fields,
                "example": entry.example,
            })
        })
        .collect();
    json!({
        "schema": "https://generalbots.org/events/office-registry.v1.json",
        "version": PAYLOAD_SCHEMA_VERSION,
        "privacy": "Payloads contain IDs and metadata only. Document body requires subscription opt-in and an ACL grant.",
        "delivery": {
            "signature_header": super::signing::SIGNATURE_HEADER,
            "signature_format": "t=<unix_seconds>,v1=<hex hmac-sha256 of '<t>.<raw_body>'>",
            "idempotency_header": super::signing::DELIVERY_HEADER,
            "event_header": super::signing::EVENT_HEADER,
            "tolerance_seconds": super::signing::DEFAULT_TOLERANCE_SECS,
            "retry": "At-least-once: 5xx, 429 and network errors are retried with capped exponential backoff.",
            "dead_letter": "Exhausted deliveries are retained for replay by idempotency key."
        },
        "count": events.len(),
        "events": events,
    })
}

#[must_use]
pub fn find(name: &str) -> Option<SchemaEntry> {
    OfficeEventType::from_name(name).map(|event_type| SchemaEntry {
        name: event_type.as_str(),
        version: PAYLOAD_SCHEMA_VERSION,
        description: event_type.description(),
        app: event_type.app(),
        required_fields: required_fields(event_type),
        example: example_for(event_type),
    })
}

fn required_fields(event_type: OfficeEventType) -> Vec<&'static str> {
    let mut fields = COMMON_REQUIRED_FIELDS.to_vec();
    fields.push("occurred_at");
    match event_type {
        OfficeEventType::DocCommentAdded
        | OfficeEventType::DocCommentUpdated
        | OfficeEventType::DocCommentDeleted
        | OfficeEventType::DocCommentMentionAdded => fields.push("comment_id"),
        _ => {}
    }
    fields
}

/// Example is metadata-only: it demonstrates the privacy default.
fn example_for(event_type: OfficeEventType) -> Value {
    let mut metadata = serde_json::Map::new();
    metadata.insert(
        "title".to_string(),
        Value::String("Quarterly report".to_string()),
    );
    if matches!(
        event_type,
        OfficeEventType::DocCommentAdded
            | OfficeEventType::DocCommentUpdated
            | OfficeEventType::DocCommentDeleted
            | OfficeEventType::DocCommentMentionAdded
    ) {
        metadata.insert(
            "comment_id".to_string(),
            Value::String("cmt-1".to_string()),
        );
    }
    json!({
        "event_id": "evt_01J8Z3KQ2M",
        "type": event_type.as_str(),
        "schema_version": PAYLOAD_SCHEMA_VERSION,
        "app": event_type.app(),
        "workspace_id": "ws_123",
        "doc_id": "doc_456",
        "actor_id": "usr_789",
        "occurred_at": "2026-11-22T12:00:00Z",
        "tags": ["finance"],
        "metadata": metadata,
    })
}

#[cfg(test)]
mod tests {
    use super::{find, registry, registry_json};
    use crate::office::taxonomy::OfficeEventType;

    #[test]
    fn registry_lists_every_event_with_an_example() {
        let entries = registry();
        assert_eq!(entries.len(), OfficeEventType::ALL.len());
        for entry in entries {
            assert!(!entry.description.is_empty());
            assert_eq!(entry.version, 1);
            assert_eq!(entry.example["type"], entry.name);
            assert_eq!(entry.example["schema_version"], 1);
            assert!(entry.example.get("event_id").is_some());
            assert!(entry.example.get("doc_id").is_some());
            assert!(entry.example.get("content").is_none());
        }
    }

    #[test]
    fn registry_contains_the_acceptance_criteria_events() {
        assert!(find("doc.comment.added").is_some());
        assert!(find("sheets.exported").is_some());
        assert!(find("unknown.event").is_none());
    }

    #[test]
    fn json_registry_has_a_count_and_all_entries() {
        let value = registry_json();
        assert_eq!(value["count"], OfficeEventType::ALL.len());
        assert_eq!(value["version"], 1);
        let events = value["events"].as_array().map(|items| items.len()).unwrap_or(0);
        assert_eq!(events, OfficeEventType::ALL.len());
        assert_eq!(value["delivery"]["signature_header"], "X-GB-Signature");
        assert_eq!(value["delivery"]["idempotency_header"], "X-GB-Delivery");
    }
}
