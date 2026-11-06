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
}

pub(super) fn payload_str<'a>(data: &'a Value, key: &str) -> Result<&'a str, ReplayError> {
    data.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| ReplayError::InvalidPayload(format!("missing string {key}")))
}

pub(super) fn payload_state(data: &Value, key: &str) -> Result<RunState, ReplayError> {
    let value = data
        .get(key)
        .cloned()
        .ok_or_else(|| ReplayError::InvalidPayload(format!("missing state {key}")))?;
    serde_json::from_value(value)
        .map_err(|error| ReplayError::InvalidPayload(format!("bad state {key}: {error}")))
}
