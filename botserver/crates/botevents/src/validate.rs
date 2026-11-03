//! Structural validation of an envelope before it is sequenced or persisted.
//!
//! Validation is closed by default: an event family whose required payload
//! keys are missing, a payload that is not an object, an unknown version or an
//! oversized payload is rejected with a JSON path the caller can return to the
//! model as a schema error.

use std::fmt;

use serde_json::Value;

use super::envelope::{Envelope, ENVELOPE_VERSION};
use super::events::EventType;
use super::state::RunState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    pub path: String,
    pub reason: String,
}

impl ValidationIssue {
    #[must_use]
    pub fn new(path: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            reason: reason.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ValidationIssues {
    pub issues: Vec<ValidationIssue>,
}

impl ValidationIssues {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, path: impl Into<String>, reason: impl Into<String>) {
        self.issues.push(ValidationIssue::new(path, reason));
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.issues.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.issues.len()
    }

    /// `data.missing_field` style paths, suitable for a model-facing retry.
    #[must_use]
    pub fn paths(&self) -> Vec<String> {
        self.issues
            .iter()
            .map(|issue| issue.path.clone())
            .collect()
    }

    #[must_use]
    pub fn to_json(&self) -> Value {
        Value::Array(
            self.issues
                .iter()
                .map(|issue| {
                    serde_json::json!({
                        "path": issue.path,
                        "reason": issue.reason,
                    })
                })
                .collect(),
        )
    }
}

impl fmt::Display for ValidationIssues {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, issue) in self.issues.iter().enumerate() {
            if index > 0 {
                f.write_str("; ")?;
            }
            write!(f, "{}: {}", issue.path, issue.reason)?;
        }
        Ok(())
    }
}

impl std::error::Error for ValidationIssues {}

const MAX_PAYLOAD_BYTES: usize = 256 * 1024;
const MAX_ID_LEN: usize = 200;

#[derive(Debug, Clone)]
pub struct Validator {
    max_payload_bytes: usize,
}

impl Default for Validator {
    fn default() -> Self {
        Self::new()
    }
}

impl Validator {
    #[must_use]
    pub fn new() -> Self {
        Self {
            max_payload_bytes: MAX_PAYLOAD_BYTES,
        }
    }

    #[must_use]
    pub fn with_max_payload_bytes(mut self, bytes: usize) -> Self {
        self.max_payload_bytes = bytes.max(1024);
        self
    }

    pub fn validate(&self, envelope: &Envelope) -> Result<(), ValidationIssues> {
        let mut issues = ValidationIssues::new();

        if envelope.v != ENVELOPE_VERSION {
            issues.push("v", format!("unsupported contract version {}", envelope.v));
        }
        check_id(&mut issues, "thread_id", &envelope.thread_id);
        check_id(&mut issues, "run_id", &envelope.run_id);
        if let Some(parent) = &envelope.parent_run_id {
            check_id(&mut issues, "parent_run_id", parent);
            if parent == &envelope.run_id {
                issues.push("parent_run_id", "a run cannot parent itself");
            }
        }
        if envelope.seq == 0 {
            issues.push("seq", "sequence numbers start at 1");
        }
        if envelope.event_id.trim().is_empty() {
            issues.push("event_id", "event id is required");
        }
        if !envelope.data.is_object() {
            issues.push("data", "payload must be a JSON object");
            return Err(issues);
        }

        let size = serde_json::to_vec(&envelope.data)
            .map(|bytes| bytes.len())
            .unwrap_or(usize::MAX);
        if size > self.max_payload_bytes {
            issues.push(
                "data",
                format!(
                    "payload is {size} bytes; the limit is {} bytes",
                    self.max_payload_bytes
                ),
            );
        }

        for field in envelope.event_type.required_data_fields() {
            match envelope.data.get(*field) {
                None | Some(Value::Null) => {
                    issues.push(format!("data.{field}"), "required field is missing");
                }
                Some(Value::String(text)) if text.trim().is_empty() => {
                    issues.push(format!("data.{field}"), "required field is empty");
                }
                Some(_) => {}
            }
        }

        self.validate_family(envelope, &mut issues);

        if issues.is_empty() {
            Ok(())
        } else {
            Err(issues)
        }
    }

    fn validate_family(&self, envelope: &Envelope, issues: &mut ValidationIssues) {
        let data = &envelope.data;
        match envelope.event_type {
            EventType::RunFinished => {
                if let Some(state) = data.get("state") {
                    match serde_json::from_value::<RunState>(state.clone()) {
                        Ok(state) if state.is_valid_finish_state() => {}
                        Ok(state) => issues.push(
                            "data.state",
                            format!("RUN_FINISHED cannot carry the non-terminal state '{state}'"),
                        ),
                        Err(_) => issues.push("data.state", "unknown run state"),
                    }
                }
            }
            EventType::StepFinished => {
                require_one_of(issues, data, "status", &["ok", "error", "skipped", "cancelled"]);
            }
            EventType::ToolCallResult => {
                require_one_of(
                    issues,
                    data,
                    "status",
                    &["ok", "error", "denied", "cancelled", "timeout"],
                );
            }
            EventType::Custom => {
                if let Some(name) = data.get("name").and_then(Value::as_str) {
                    if name.len() > 128 {
                        issues.push("data.name", "custom event name exceeds 128 characters");
                    }
                }
            }
            EventType::StateDelta | EventType::ActivityDelta => {
                if !matches!(data.get("patch"), Some(Value::Array(_))) {
                    issues.push("data.patch", "JSON Patch must be an array of operations");
                }
            }
            EventType::MessagesSnapshot => {
                if !matches!(data.get("messages"), Some(Value::Array(_))) {
                    issues.push("data.messages", "messages must be an array");
                }
            }
            EventType::SubagentFinished => {
                if let Some(state) = data.get("state") {
                    match serde_json::from_value::<RunState>(state.clone()) {
                        Ok(state) if state.is_terminal() => {}
                        Ok(state) => issues.push(
                            "data.state",
                            format!("SUBAGENT_FINISHED requires a terminal state, got '{state}'"),
                        ),
                        Err(_) => issues.push("data.state", "unknown run state"),
                    }
                }
            }
            EventType::SubagentStarted => {
                if let Some(child) = data.get("child_run_id").and_then(Value::as_str) {
                    if child == envelope.run_id {
                        issues.push(
                            "data.child_run_id",
                            "a sub-agent run must differ from its parent run",
                        );
                    }
                }
            }
            _ => {}
        }
    }
}

fn check_id(issues: &mut ValidationIssues, path: &str, value: &str) {
    if value.trim().is_empty() {
        issues.push(path, "identifier is required");
        return;
    }
    if value.chars().count() > MAX_ID_LEN {
        issues.push(path, "identifier exceeds 200 characters");
    }
    if value.chars().any(char::is_control) {
        issues.push(path, "identifier contains control characters");
    }
}

fn require_one_of(
    issues: &mut ValidationIssues,
    data: &Value,
    field: &str,
    allowed: &[&str],
) {
    let Some(value) = data.get(field) else {
        return;
    };
    let Some(text) = value.as_str() else {
        issues.push(format!("data.{field}"), "expected a string");
        return;
    };
    if !allowed.contains(&text) {
        issues.push(
            format!("data.{field}"),
            format!("'{text}' is not one of {}", allowed.join(", ")),
        );
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use serde_json::json;

    use super::Validator;
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

    #[test]
    fn accepts_a_complete_step_start() {
        let value = envelope(
            EventType::StepStarted,
            json!({ "step_id": "s1", "name": "prepare workspace" }),
        );
        assert!(Validator::new().validate(&value).is_ok());
    }

    #[test]
    fn rejects_missing_required_field_with_path() {
        let value = envelope(EventType::ToolCallStart, json!({ "call_id": "c1" }));
        let issues = Validator::new().validate(&value).expect_err("must reject");
        assert_eq!(issues.paths(), vec!["data.name".to_string()]);
    }

    #[test]
    fn rejects_non_terminal_finish_state() {
        let value = envelope(
            EventType::RunFinished,
            json!({ "state": "running" }),
        );
        assert!(Validator::new().validate(&value).is_err());
    }

    #[test]
    fn rejects_oversized_payload() {
        let value = envelope(
            EventType::TextMessageContent,
            json!({ "message_id": "m1", "delta": "x".repeat(4096) }),
        );
        assert!(Validator::new()
            .with_max_payload_bytes(1024)
            .validate(&value)
            .is_err());
    }
}
