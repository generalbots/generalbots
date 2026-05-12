use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerKind {
    Scheduled = 0,
    TableUpdate = 1,
    TableInsert = 2,
    TableDelete = 3,
    Webhook = 4,
    EmailReceived = 5,
    FolderChange = 6,
    DealStageChange = 7,
    ContactChange = 8,
    EmailOpened = 9,
}

impl TriggerKind {
    #[must_use]
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Scheduled),
            1 => Some(Self::TableUpdate),
            2 => Some(Self::TableInsert),
            3 => Some(Self::TableDelete),
            4 => Some(Self::Webhook),
            5 => Some(Self::EmailReceived),
            6 => Some(Self::FolderChange),
            7 => Some(Self::DealStageChange),
            8 => Some(Self::ContactChange),
            9 => Some(Self::EmailOpened),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecution {
    pub id: Uuid,
    pub bot_id: Uuid,
    pub workflow_name: String,
    pub current_step: Option<i32>,
    pub state_json: Option<serde_json::Value>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
