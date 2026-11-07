use serde_json::Value;

use botevents::{Envelope, EventType, RunState};

use super::apply_patch;
use super::state::{
    payload_state, payload_str, CustomRecord, MessageView, ReplayError, ReplayState, RunView,
    StepView, ToolView,
};

impl ReplayState {
    pub(super) fn apply_family(&mut self, envelope: &Envelope) -> Result<(), ReplayError> {
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
