//! The wire envelope every run event is wrapped in.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::events::{CustomData, EventType};

/// Contract version. A reducer refuses an envelope whose `v` it does not know
/// instead of guessing at the payload shape.
pub const ENVELOPE_VERSION: u8 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventSource {
    Runtime,
    Model,
    Tool,
    App,
    User,
    System,
    Channel,
}

impl EventSource {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Runtime => "runtime",
            Self::Model => "model",
            Self::Tool => "tool",
            Self::App => "app",
            Self::User => "user",
            Self::System => "system",
            Self::Channel => "channel",
        }
    }
}

impl std::fmt::Display for EventSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// An event before it has been redacted, validated and sequenced.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingEvent {
    pub thread_id: String,
    pub run_id: String,
    pub parent_run_id: Option<String>,
    pub source: EventSource,
    pub event_type: EventType,
    pub data: Value,
}

impl PendingEvent {
    #[must_use]
    pub fn new(
        thread_id: impl Into<String>,
        run_id: impl Into<String>,
        source: EventSource,
        event_type: EventType,
        data: Value,
    ) -> Self {
        Self {
            thread_id: thread_id.into(),
            run_id: run_id.into(),
            parent_run_id: None,
            source,
            event_type,
            data,
        }
    }

    #[must_use]
    pub fn with_parent(mut self, parent_run_id: Option<String>) -> Self {
        self.parent_run_id = parent_run_id;
        self
    }

    #[must_use]
    pub fn custom(
        thread_id: impl Into<String>,
        run_id: impl Into<String>,
        source: EventSource,
        name: impl Into<String>,
        payload: Value,
    ) -> Self {
        let data = CustomData {
            name: name.into(),
            payload,
        };
        Self::new(
            thread_id,
            run_id,
            source,
            EventType::Custom,
            serde_json::to_value(data).unwrap_or(Value::Null),
        )
    }
}

/// A sequenced, redacted event ready for the log and the transport.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Envelope {
    /// Contract version.
    pub v: u8,
    /// AG-UI event family.
    #[serde(rename = "type")]
    pub event_type: EventType,
    /// Conversation thread. `seq` is dense and gapless inside one thread.
    pub thread_id: String,
    /// The run that produced this event.
    pub run_id: String,
    /// Parent run, when this event belongs to a nested (sub-agent) run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_run_id: Option<String>,
    /// Monotonic position inside `thread_id`, starting at 1.
    pub seq: u64,
    /// Stable id derived from thread, seq, type and payload; supports an
    /// idempotent reducer and deduplicates transport retries.
    pub event_id: String,
    /// Server clock.
    pub ts: DateTime<Utc>,
    /// Who authored the event.
    pub source: EventSource,
    /// Family-specific payload.
    pub data: Value,
}

impl Envelope {
    /// Deserializes the payload into its typed representation.
    pub fn typed_data<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_value(self.data.clone())
    }

    #[must_use]
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.event_type,
            EventType::RunFinished | EventType::RunError
        )
    }

    #[must_use]
    pub fn custom_name(&self) -> Option<&str> {
        if self.event_type != EventType::Custom {
            return None;
        }
        self.data.get("name").and_then(Value::as_str)
    }

    pub fn to_json(&self) -> Result<Value, serde_json::Error> {
        serde_json::to_value(self)
    }
}

#[cfg(test)]
mod tests {
    use super::{EventSource, PendingEvent};
    use crate::events::EventType;
    use serde_json::json;

    #[test]
    fn custom_pending_event_carries_gb_name() {
        let event = PendingEvent::custom(
            "thread-1",
            "run-1",
            EventSource::Model,
            "gb.question",
            json!({ "id": "q1" }),
        );
        assert_eq!(event.event_type, EventType::Custom);
        assert_eq!(
            event.data.get("name").and_then(|value| value.as_str()),
            Some("gb.question")
        );
    }
}
