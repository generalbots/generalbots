//! Event families carried inside the envelope.
//!
//! The `type` names follow the AG-UI vocabulary. Product-specific payloads
//! (questions, surfaces, artifacts, steering receipts) travel as `CUSTOM`
//! events whose data carries a `gb.*` name, so a standard AG-UI client can
//! ignore them without failing to parse the stream.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use super::state::RunState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventType {
    RunStarted,
    RunFinished,
    RunError,
    StepStarted,
    StepFinished,
    ToolCallStart,
    ToolCallArgs,
    ToolCallEnd,
    ToolCallResult,
    TextMessageStart,
    TextMessageContent,
    TextMessageEnd,
    StateSnapshot,
    StateDelta,
    MessagesSnapshot,
    ActivitySnapshot,
    ActivityDelta,
    Custom,
    Raw,
    SubagentStarted,
    SubagentFinished,
}

impl EventType {
    pub const ALL: [EventType; 21] = [
        Self::RunStarted,
        Self::RunFinished,
        Self::RunError,
        Self::StepStarted,
        Self::StepFinished,
        Self::ToolCallStart,
        Self::ToolCallArgs,
        Self::ToolCallEnd,
        Self::ToolCallResult,
        Self::TextMessageStart,
        Self::TextMessageContent,
        Self::TextMessageEnd,
        Self::StateSnapshot,
        Self::StateDelta,
        Self::MessagesSnapshot,
        Self::ActivitySnapshot,
        Self::ActivityDelta,
        Self::Custom,
        Self::Raw,
        Self::SubagentStarted,
        Self::SubagentFinished,
    ];

    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RunStarted => "RUN_STARTED",
            Self::RunFinished => "RUN_FINISHED",
            Self::RunError => "RUN_ERROR",
            Self::StepStarted => "STEP_STARTED",
            Self::StepFinished => "STEP_FINISHED",
            Self::ToolCallStart => "TOOL_CALL_START",
            Self::ToolCallArgs => "TOOL_CALL_ARGS",
            Self::ToolCallEnd => "TOOL_CALL_END",
            Self::ToolCallResult => "TOOL_CALL_RESULT",
            Self::TextMessageStart => "TEXT_MESSAGE_START",
            Self::TextMessageContent => "TEXT_MESSAGE_CONTENT",
            Self::TextMessageEnd => "TEXT_MESSAGE_END",
            Self::StateSnapshot => "STATE_SNAPSHOT",
            Self::StateDelta => "STATE_DELTA",
            Self::MessagesSnapshot => "MESSAGES_SNAPSHOT",
            Self::ActivitySnapshot => "ACTIVITY_SNAPSHOT",
            Self::ActivityDelta => "ACTIVITY_DELTA",
            Self::Custom => "CUSTOM",
            Self::Raw => "RAW",
            Self::SubagentStarted => "SUBAGENT_STARTED",
            Self::SubagentFinished => "SUBAGENT_FINISHED",
        }
    }

    /// Required `data` keys for this family. This table is the single source
    /// for both validation and the checked-in JSON Schema.
    #[must_use]
    pub fn required_data_fields(self) -> &'static [&'static str] {
        match self {
            Self::RunStarted | Self::RunFinished => &[],
            Self::RunError => &["message"],
            Self::StepStarted => &["step_id", "name"],
            Self::StepFinished => &["step_id", "status"],
            Self::ToolCallStart => &["call_id", "name"],
            Self::ToolCallArgs => &["call_id"],
            Self::ToolCallEnd => &["call_id"],
            Self::ToolCallResult => &["call_id", "status"],
            Self::TextMessageStart => &["message_id"],
            Self::TextMessageContent => &["message_id", "delta"],
            Self::TextMessageEnd => &["message_id"],
            Self::StateSnapshot => &["state"],
            Self::StateDelta => &["patch"],
            Self::MessagesSnapshot => &["messages"],
            Self::ActivitySnapshot => &["activity_id", "activity_type"],
            Self::ActivityDelta => &["activity_id", "patch"],
            Self::Custom => &["name"],
            Self::Raw => &["raw"],
            Self::SubagentStarted => &["child_run_id", "agent_name"],
            Self::SubagentFinished => &["child_run_id", "state"],
        }
    }
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Usage {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost_usd: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunStartedData {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunFinishedData {
    pub state: RunState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunErrorData {
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepStartedData {
    pub step_id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepFinishedData {
    pub step_id: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCallStartData {
    pub call_id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub risk: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCallArgsData {
    pub call_id: String,
    #[serde(default)]
    pub arguments: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCallEndData {
    pub call_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCallResultData {
    pub call_id: String,
    pub status: String,
    #[serde(default)]
    pub result: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextMessageStartData {
    pub message_id: String,
    #[serde(default = "assistant_role")]
    pub role: String,
}

fn assistant_role() -> String {
    "assistant".to_string()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextMessageContentData {
    pub message_id: String,
    pub delta: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextMessageEndData {
    pub message_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateSnapshotData {
    pub state: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateDeltaData {
    pub patch: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessagesSnapshotData {
    pub messages: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActivitySnapshotData {
    pub activity_id: String,
    pub activity_type: String,
    #[serde(default)]
    pub content: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActivityDeltaData {
    pub activity_id: String,
    pub patch: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomData {
    pub name: String,
    #[serde(default)]
    pub payload: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubagentStartedData {
    pub child_run_id: String,
    pub agent_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubagentFinishedData {
    pub child_run_id: String,
    pub state: RunState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

/// Serializes a typed payload into the untyped `data` object the envelope
/// carries. Fails closed: a struct that cannot serialize is a programming
/// error and becomes an empty object, which the validator then rejects for
/// families with required fields.
#[must_use]
pub fn to_data<T: Serialize>(payload: &T) -> Value {
    match serde_json::to_value(payload) {
        Ok(Value::Object(map)) => Value::Object(map),
        Ok(other) => {
            let mut map = Map::new();
            map.insert("value".to_string(), other);
            Value::Object(map)
        }
        Err(_) => Value::Object(Map::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::EventType;

    #[test]
    fn names_match_agui_vocabulary() {
        assert_eq!(EventType::RunStarted.as_str(), "RUN_STARTED");
        assert_eq!(EventType::ToolCallStart.as_str(), "TOOL_CALL_START");
        assert_eq!(EventType::Custom.as_str(), "CUSTOM");
    }

    #[test]
    fn every_family_is_in_all() {
        assert_eq!(EventType::ALL.len(), 21);
    }
}
