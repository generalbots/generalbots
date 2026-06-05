use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HandoffStatus {
    Requested,
    Accepted,
    Rejected,
    Completed,
    Escalated,
}

impl std::fmt::Display for HandoffStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Requested => write!(f, "requested"),
            Self::Accepted => write!(f, "accepted"),
            Self::Rejected => write!(f, "rejected"),
            Self::Completed => write!(f, "completed"),
            Self::Escalated => write!(f, "escalated"),
        }
    }
}

impl std::str::FromStr for HandoffStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "requested" => Ok(Self::Requested),
            "accepted" => Ok(Self::Accepted),
            "rejected" => Ok(Self::Rejected),
            "completed" => Ok(Self::Completed),
            "escalated" => Ok(Self::Escalated),
            _ => Err(format!("Unknown handoff status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSnapshot {
    pub recent_messages: Vec<ArchivedMessage>,
    pub session_state: HashMap<String, String>,
    pub bot_memory: HashMap<String, String>,
    pub active_tools: Vec<String>,
    pub current_intent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchivedMessage {
    pub role: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub message_type: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffRequest {
    pub id: Uuid,
    pub session_id: String,
    pub bot_id: Uuid,
    pub user_id: Uuid,
    pub context_snapshot: ContextSnapshot,
    pub reason: String,
    pub assigned_agent: Option<Uuid>,
    pub status: HandoffStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum HandoffError {
    NotConnected(String),
    AgentUnavailable(Uuid),
    RequestNotFound(Uuid),
    InvalidTransition {
        from: HandoffStatus,
        to: HandoffStatus,
    },
    StorageError(String),
}

impl std::fmt::Display for HandoffError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotConnected(msg) => write!(f, "Handoff connection error: {msg}"),
            Self::AgentUnavailable(agent_id) => write!(f, "Agent {agent_id} is unavailable"),
            Self::RequestNotFound(id) => write!(f, "Handoff request {id} not found"),
            Self::InvalidTransition { from, to } => {
                write!(f, "Invalid handoff transition from {from} to {to}")
            }
            Self::StorageError(msg) => write!(f, "Storage error: {msg}"),
        }
    }
}

impl std::error::Error for HandoffError {}

pub struct HandoffService {
    requests: Vec<HandoffRequest>,
}

impl HandoffService {
    pub fn new() -> Self {
        Self {
            requests: Vec::new(),
        }
    }

    pub fn request_handoff(
        &mut self,
        session_id: String,
        bot_id: Uuid,
        user_id: Uuid,
        context_snapshot: ContextSnapshot,
        reason: String,
    ) -> HandoffRequest {
        let request = HandoffRequest {
            id: Uuid::new_v4(),
            session_id,
            bot_id,
            user_id,
            context_snapshot,
            reason,
            assigned_agent: None,
            status: HandoffStatus::Requested,
            created_at: Utc::now(),
        };
        self.requests.push(request.clone());
        request
    }

    pub fn assign_agent(
        &mut self,
        request_id: Uuid,
        agent_id: Uuid,
    ) -> Result<HandoffRequest, HandoffError> {
        let request = self
            .requests
            .iter_mut()
            .find(|r| r.id == request_id)
            .ok_or(HandoffError::RequestNotFound(request_id))?;

        if request.status != HandoffStatus::Requested {
            return Err(HandoffError::InvalidTransition {
                from: request.status.clone(),
                to: HandoffStatus::Accepted,
            });
        }

        request.assigned_agent = Some(agent_id);
        request.status = HandoffStatus::Accepted;
        Ok(request.clone())
    }

    pub fn get_context_history(&self, request_id: Uuid) -> Result<&ContextSnapshot, HandoffError> {
        let request = self
            .requests
            .iter()
            .find(|r| r.id == request_id)
            .ok_or(HandoffError::RequestNotFound(request_id))?;
        Ok(&request.context_snapshot)
    }

    pub fn complete_handoff(
        &mut self,
        request_id: Uuid,
    ) -> Result<HandoffRequest, HandoffError> {
        let request = self
            .requests
            .iter_mut()
            .find(|r| r.id == request_id)
            .ok_or(HandoffError::RequestNotFound(request_id))?;

        if request.status != HandoffStatus::Accepted {
            return Err(HandoffError::InvalidTransition {
                from: request.status.clone(),
                to: HandoffStatus::Completed,
            });
        }

        request.status = HandoffStatus::Completed;
        Ok(request.clone())
    }

    pub fn escalate_handoff(
        &mut self,
        request_id: Uuid,
    ) -> Result<HandoffRequest, HandoffError> {
        let request = self
            .requests
            .iter_mut()
            .find(|r| r.id == request_id)
            .ok_or(HandoffError::RequestNotFound(request_id))?;

        request.status = HandoffStatus::Escalated;
        Ok(request.clone())
    }

    pub fn list_by_user(&self, user_id: Uuid) -> Vec<&HandoffRequest> {
        self.requests
            .iter()
            .filter(|r| r.user_id == user_id)
            .collect()
    }

    pub fn list_pending(&self) -> Vec<&HandoffRequest> {
        self.requests
            .iter()
            .filter(|r| r.status == HandoffStatus::Requested)
            .collect()
    }
}

impl Default for HandoffService {
    fn default() -> Self {
        Self::new()
    }
}
