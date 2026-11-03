//! Golden-transcript replay.
//!
//! The reducer is the reference client: it consumes frames in `seq` order,
//! refuses gaps, ignores replayed `event_id`s and enforces the run state
//! machine. A recorded transcript that replays to the same state twice proves
//! the stream is deterministic.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use botevents::{Envelope, EventType, RunState, Validator};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayError {
    Validation(String),
    SequenceGap { expected: u64, got: u64 },
    UnknownRun(String),
    TerminalRun(String),
    InvalidTransition { run_id: String, from: String, to: String },
    InvalidPayload(String),
}

impl std::fmt::Display for ReplayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Validation(message) => write!(f, "invalid frame: {message}"),
            Self::SequenceGap { expected, got } => {
                write!(f, "sequence gap: expected {expected}, got {got}")
            }
            Self::UnknownRun(run_id) => write!(f, "event for unknown run {run_id}"),
            Self::TerminalRun(run_id) => write!(f, "event after terminal state for run {run_id}"),
            Self::InvalidTransition { run_id, from, to } => {
                write!(f, "run {run_id} cannot move from {from} to {to}")
            }
            Self::InvalidPayload(message) => write!(f, "invalid payload: {message}"),
        }
    }
}

impl std::error::Error for ReplayError {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepView {
    pub name: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolView {
    pub name: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunView {
    pub state: RunState,
    #[serde(default)]
    pub parent_run_id: Option<String>,
    #[serde(default)]
    pub steps: BTreeMap<String, StepView>,
    #[serde(default)]
    pub tools: BTreeMap<String, ToolView>,
}

impl RunView {
    fn running(parent_run_id: Option<String>) -> Self {
        Self {
            state: RunState::Running,
            parent_run_id,
            steps: BTreeMap::new(),
            tools: BTreeMap::new(),
        }
    }

    fn is_terminal(&self) -> bool {
        self.state.is_terminal()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageView {
    pub id: String,
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomRecord {
    pub name: String,
    pub payload: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayState {
    pub last_seq: u64,
    pub runs: BTreeMap<String, RunView>,
    pub messages: BTreeMap<String, MessageView>,
    pub data: Value,
    pub activities: BTreeMap<String, Value>,
    pub custom: Vec<CustomRecord>,
    pub event_ids: BTreeSet<String>,
}

impl Default for ReplayState {
    fn default() -> Self {
        Self {
            last_seq: 0,
            runs: BTreeMap::new(),
            messages: BTreeMap::new(),
            data: Value::Null,
            activities: BTreeMap::new(),
            custom: Vec::new(),
            event_ids: BTreeSet::new(),
        }
    }
}

impl ReplayState {
    /// Applies one frame. Replayed frames (`event_id` already seen) are
    /// idempotent no-ops, which is what makes reconnect retries safe.
    pub fn apply(&mut self, envelope: &Envelope) -> Result<(), ReplayError> {
        if self.event_ids.contains(&envelope.event_id) {
            return Ok(());
        }
        let expected = self.last_seq.saturating_add(1);
        if envelope.seq != expected {
            return Err(ReplayError::SequenceGap {
                expected,
                got: envelope.seq,
            });
        }
        Validator::new()
            .validate(envelope)
            .map_err(|issues| ReplayError::Validation(issues.to_string()))?;

        let run_id = envelope.run_id.clone();
        if matches!(envelope.event_type, EventType::RunStarted) {
            self.runs
                .entry(run_id.clone())
                .or_insert_with(|| RunView::running(envelope.parent_run_id.clone()));
        } else if matches!(envelope.event_type, EventType::SubagentStarted) {
            let child = payload_str(&envelope.data, "child_run_id")?;
            let agent = payload_str(&envelope.data, "agent_name")?;
            let child_run = self
                .runs
                .entry(child.to_string())
                .or_insert_with(|| RunView::running(Some(run_id.clone())));
            child_run.parent_run_id = Some(run_id.clone());
            let _ = agent;
        }

        if !matches!(envelope.event_type, EventType::RunStarted) {
            if let Some(run) = self.runs.get(&run_id) {
                if run.is_terminal() {
                    return Err(ReplayError::TerminalRun(run_id));
                }
            } else if !matches!(
                envelope.event_type,
                EventType::SubagentStarted | EventType::Custom | EventType::Raw
            ) {
                return Err(ReplayError::UnknownRun(run_id));
            }
        }

        self.apply_family(envelope)?;
        self.last_seq = envelope.seq;
        self.event_ids.insert(envelope.event_id.clone());
        Ok(())
    }

    fn apply_family(&mut self, envelope: &Envelope) -> Result<(), ReplayError> {
        let run_id = envelope.run_id.clone();
        let data = &envelope.data;
        match envelope.event_type {
            EventType::RunStarted => {}
            EventType::RunFinished => {
                let next = payload_state(data, "state")?;
                if !next.is_terminal() {
                    return Err(ReplayError::InvalidPayload(
                        "RUN_FINISHED must carry a terminal state".to_string(),
                    ));
                }
                self.transition(&run_id, next)?;
            }
            EventType::RunError => {
                self.transition(&run_id, RunState::Failed)?;
            }
            EventType::StepStarted => {
                let step_id = payload_str(data, "step_id")?.to_string();
                let name = payload_str(data, "name")?.to_string();
                let run = self.run_mut(&run_id)?;
                run.steps.insert(
                    step_id,
                    StepView {
                        name,
                        status: "running".to_string(),
                    },
                );
            }
            EventType::StepFinished => {
                let step_id = payload_str(data, "step_id")?.to_string();
                let status = payload_str(data, "status")?.to_string();
                let run = self.run_mut(&run_id)?;
                match run.steps.get_mut(&step_id) {
                    Some(step) => step.status = status,
                    None => {
                        return Err(ReplayError::InvalidPayload(format!(
                            "STEP_FINISHED for unknown step {step_id}"
                        )))
                    }
                }
            }
            EventType::ToolCallStart => {
                let call_id = payload_str(data, "call_id")?.to_string();
                let name = payload_str(data, "name")?.to_string();
                let run = self.run_mut(&run_id)?;
                run.tools.insert(
                    call_id,
                    ToolView {
                        name,
                        status: "running".to_string(),
                    },
                );
            }
            EventType::ToolCallResult => {
                let call_id = payload_str(data, "call_id")?.to_string();
                let status = payload_str(data, "status")?.to_string();
                let run = self.run_mut(&run_id)?;
                match run.tools.get_mut(&call_id) {
                    Some(tool) => tool.status = status,
                    None => {
                        return Err(ReplayError::InvalidPayload(format!(
                            "TOOL_CALL_RESULT for unknown call {call_id}"
                        )))
                    }
                }
            }
            EventType::TextMessageStart => {
                let message_id = payload_str(data, "message_id")?.to_string();
                let role = data
                    .get("role")
                    .and_then(Value::as_str)
                    .unwrap_or("assistant")
                    .to_string();
                self.messages.insert(
                    message_id.clone(),
                    MessageView {
                        id: message_id,
                        role,
                        content: String::new(),
                    },
                );
            }
            EventType::TextMessageContent => {
                let message_id = payload_str(data, "message_id")?.to_string();
                let delta = payload_str(data, "delta")?;
                match self.messages.get_mut(&message_id) {
                    Some(message) => message.content.push_str(delta),
                    None => {
                        return Err(ReplayError::InvalidPayload(format!(
                            "TEXT_MESSAGE_CONTENT for unknown message {message_id}"
                        )))
                    }
                }
            }
            EventType::TextMessageEnd => {
                let message_id = payload_str(data, "message_id")?.to_string();
                let content = data
                    .get("content")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                match self.messages.get_mut(&message_id) {
                    Some(message) => {
                        if let Some(content) = content {
                            message.content = content;
                        }
                    }
                    None => {
                        return Err(ReplayError::InvalidPayload(format!(
                            "TEXT_MESSAGE_END for unknown message {message_id}"
                        )))
                    }
                }
            }
            EventType::StateSnapshot => {
                self.data = data.get("state").cloned().unwrap_or(Value::Null);
            }
            EventType::StateDelta => {
                let patch = data
                    .get("patch")
                    .and_then(Value::as_array)
                    .ok_or_else(|| {
                        ReplayError::InvalidPayload("patch is not an array".to_string())
                    })?;
                apply_patch(&mut self.data, patch)?;
            }
            EventType::MessagesSnapshot => {
                if let Some(messages) = data.get("messages").and_then(Value::as_array) {
                    self.messages.clear();
                    for message in messages {
                        let id = message
                            .get("id")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_string();
                        if id.is_empty() {
                            continue;
                        }
                        self.messages.insert(
                            id.clone(),
                            MessageView {
                                id,
                                role: message
                                    .get("role")
                                    .and_then(Value::as_str)
                                    .unwrap_or("assistant")
                                    .to_string(),
                                content: message
                                    .get("content")
                                    .and_then(Value::as_str)
                                    .unwrap_or_default()
                                    .to_string(),
                            },
                        );
                    }
                }
            }
            EventType::ActivitySnapshot => {
                let activity_id = payload_str(data, "activity_id")?.to_string();
                self.activities.insert(
                    activity_id,
                    data.get("content").cloned().unwrap_or(Value::Null),
                );
            }
            EventType::ActivityDelta => {
                let activity_id = payload_str(data, "activity_id")?.to_string();
                let patch = data
                    .get("patch")
                    .and_then(Value::as_array)
                    .ok_or_else(|| {
                        ReplayError::InvalidPayload("patch is not an array".to_string())
                    })?;
                let entry = self
                    .activities
                    .entry(activity_id)
                    .or_insert(Value::Object(serde_json::Map::new()));
                apply_patch(entry, patch)?;
            }
            EventType::SubagentStarted => {}
            EventType::SubagentFinished => {
                let child = payload_str(data, "child_run_id")?.to_string();
                let state = payload_state(data, "state")?;
                let child_run = self
                    .runs
                    .get_mut(&child)
                    .ok_or_else(|| ReplayError::UnknownRun(child.clone()))?;
                if !state.is_terminal() {
                    return Err(ReplayError::InvalidPayload(
                        "SUBAGENT_FINISHED requires a terminal state".to_string(),
                    ));
                }
                child_run.state = state;
            }
            EventType::Custom => {
                self.custom.push(CustomRecord {
                    name: data
                        .get("name")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                    payload: data.get("payload").cloned().unwrap_or(Value::Null),
                });
            }
            EventType::ToolCallArgs | EventType::ToolCallEnd | EventType::Raw => {}
        }
        Ok(())
    }

    fn run_mut(&mut self, run_id: &str) -> Result<&mut RunView, ReplayError> {
        self.runs
            .get_mut(run_id)
            .ok_or_else(|| ReplayError::UnknownRun(run_id.to_string()))
    }

    fn transition(&mut self, run_id: &str, next: RunState) -> Result<(), ReplayError> {
        let run = self.run_mut(run_id)?;
        if !run.state.can_transition_to(next) {
            return Err(ReplayError::InvalidTransition {
                run_id: run_id.to_string(),
                from: run.state.to_string(),
                to: next.to_string(),
            });
        }
        run.state = next;
        Ok(())
    }
}

/// Replays a complete transcript.
pub fn replay(frames: &[Envelope]) -> Result<ReplayState, ReplayError> {
    let mut state = ReplayState::default();
    for frame in frames {
        state.apply(frame)?;
    }
    Ok(state)
}

fn payload_str<'a>(data: &'a Value, key: &str) -> Result<&'a str, ReplayError> {
    data.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| ReplayError::InvalidPayload(format!("missing string {key}")))
}

fn payload_state(data: &Value, key: &str) -> Result<RunState, ReplayError> {
    let value = data
        .get(key)
        .cloned()
        .ok_or_else(|| ReplayError::InvalidPayload(format!("missing state {key}")))?;
    serde_json::from_value(value)
        .map_err(|error| ReplayError::InvalidPayload(format!("bad state {key}: {error}")))
}

/// Applies the JSON Patch subset the surface protocol uses (`add`, `remove`,
/// `replace`). Unknown operations are rejected so a malformed patch can never
/// silently produce a different client state.
pub fn apply_patch(document: &mut Value, patch: &[Value]) -> Result<(), ReplayError> {
    for operation in patch {
        let op = operation
            .get("op")
            .and_then(Value::as_str)
            .ok_or_else(|| ReplayError::InvalidPayload("patch op is missing".to_string()))?;
        let path = operation
            .get("path")
            .and_then(Value::as_str)
            .ok_or_else(|| ReplayError::InvalidPayload("patch path is missing".to_string()))?;
        if path.is_empty() {
            match op {
                "replace" | "add" => {
                    *document = operation.get("value").cloned().unwrap_or(Value::Null);
                }
                _ => {
                    return Err(ReplayError::InvalidPayload(
                        "cannot remove the document root".to_string(),
                    ))
                }
            }
            continue;
        }
        let (parent_pointer, last) = match path.rfind('/') {
            Some(index) => (&path[..index], &path[index + 1..]),
            None => ("", path.trim_start_matches('/')),
        };
        let parent = if parent_pointer.is_empty() {
            document
        } else {
            document.pointer_mut(parent_pointer).ok_or_else(|| {
                ReplayError::InvalidPayload(format!("patch path {parent_pointer} not found"))
            })?
        };
        let last = decode_token(last);
        match parent {
            Value::Object(map) => match op {
                "add" | "replace" => {
                    map.insert(last, operation.get("value").cloned().unwrap_or(Value::Null));
                }
                "remove" => {
                    map.remove(&last);
                }
                other => {
                    return Err(ReplayError::InvalidPayload(format!(
                        "unsupported patch op {other}"
                    )))
                }
            },
            Value::Array(items) => {
                if last == "-" {
                    if op != "add" {
                        return Err(ReplayError::InvalidPayload(
                            "'-' is only valid for add".to_string(),
                        ));
                    }
                    items.push(operation.get("value").cloned().unwrap_or(Value::Null));
                    continue;
                }
                let index: usize = last
                    .parse()
                    .map_err(|_| ReplayError::InvalidPayload(format!("bad index {last}")))?;
                match op {
                    "add" => {
                        if index > items.len() {
                            return Err(ReplayError::InvalidPayload(format!(
                                "index {index} out of bounds"
                            )));
                        }
                        items.insert(index, operation.get("value").cloned().unwrap_or(Value::Null));
                    }
                    "replace" => {
                        let slot = items.get_mut(index).ok_or_else(|| {
                            ReplayError::InvalidPayload(format!("index {index} out of bounds"))
                        })?;
                        *slot = operation.get("value").cloned().unwrap_or(Value::Null);
                    }
                    "remove" => {
                        if index >= items.len() {
                            return Err(ReplayError::InvalidPayload(format!(
                                "index {index} out of bounds"
                            )));
                        }
                        items.remove(index);
                    }
                    other => {
                        return Err(ReplayError::InvalidPayload(format!(
                            "unsupported patch op {other}"
                        )))
                    }
                }
            }
            _ => {
                return Err(ReplayError::InvalidPayload(
                    "patch parent is not a container".to_string(),
                ))
            }
        }
    }
    Ok(())
}

fn decode_token(token: &str) -> String {
    token.replace("~1", "/").replace("~0", "~")
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::apply_patch;

    #[test]
    fn patch_add_and_replace() {
        let mut document = json!({ "a": 1, "list": [1, 2] });
        apply_patch(
            &mut document,
            &[
                json!({ "op": "replace", "path": "/a", "value": 2 }),
                json!({ "op": "add", "path": "/list/-", "value": 3 }),
            ],
        )
        .expect("valid patch");
        assert_eq!(document, json!({ "a": 2, "list": [1, 2, 3] }));
    }

    #[test]
    fn patch_rejects_unknown_op() {
        let mut document = json!({});
        assert!(apply_patch(
            &mut document,
            &[json!({ "op": "move", "path": "/a" })]
        )
        .is_err());
    }
}
