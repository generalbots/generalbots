use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Queryable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = crate::core::shared::schema::core::workflow_executions)]
pub struct WorkflowExecution {
    pub id: Uuid,
    pub bot_id: Uuid,
    pub workflow_name: String,
    pub current_step: i32,
    pub state_json: String,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Queryable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = crate::core::shared::schema::core::workflow_events)]
pub struct WorkflowEvent {
    pub id: Uuid,
    pub workflow_id: Option<Uuid>,
    pub event_name: String,
    pub event_data_json: Option<String>,
    pub processed: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Queryable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = crate::core::shared::schema::core::bot_shared_memory)]
pub struct BotSharedMemory {
    pub id: Uuid,
    pub source_bot_id: Uuid,
    pub target_bot_id: Uuid,
    pub memory_key: String,
    pub memory_value: String,
    pub shared_at: chrono::DateTime<chrono::Utc>,
}
