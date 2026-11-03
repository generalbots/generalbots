//! AG-UI interoperability projection.
//!
//! The stored envelope is snake_case with an explicit `v`; the AG-UI wire
//! shape is camelCase with the same event names. Exporting keeps a standard
//! client able to consume the stream while the internal log stays stable.

use serde_json::Value;

use super::envelope::Envelope;

#[must_use]
pub fn to_agui(envelope: &Envelope) -> Value {
    let mut value = serde_json::json!({
        "type": envelope.event_type.as_str(),
        "threadId": envelope.thread_id,
        "runId": envelope.run_id,
        "seq": envelope.seq,
        "eventId": envelope.event_id,
        "ts": envelope.ts.to_rfc3339(),
        "source": envelope.source.as_str(),
        "data": envelope.data,
    });
    if let (Some(parent), Some(map)) = (&envelope.parent_run_id, value.as_object_mut()) {
        map.insert("parentRunId".to_string(), Value::String(parent.clone()));
    }
    value
}

/// A product profile event (`gb.*`) is still `CUSTOM` on the wire; this helper
/// names the profile for consumers that route by custom name.
#[must_use]
pub fn custom_profile(envelope: &Envelope) -> Option<&str> {
    envelope.custom_name()
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::to_agui;
    use crate::envelope::{Envelope, EventSource, ENVELOPE_VERSION};
    use crate::events::EventType;

    #[test]
    fn exports_camel_case() {
        let envelope = Envelope {
            v: ENVELOPE_VERSION,
            event_type: EventType::RunStarted,
            thread_id: "t".to_string(),
            run_id: "r".to_string(),
            parent_run_id: Some("p".to_string()),
            seq: 1,
            event_id: "a".repeat(64),
            ts: Utc::now(),
            source: EventSource::Runtime,
            data: serde_json::json!({}),
        };
        let projected = to_agui(&envelope);
        assert_eq!(projected["type"], "RUN_STARTED");
        assert_eq!(projected["threadId"], "t");
        assert_eq!(projected["parentRunId"], "p");
    }
}
