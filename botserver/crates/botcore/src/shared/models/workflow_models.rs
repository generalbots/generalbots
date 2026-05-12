use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use uuid::Uuid;

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = crate::shared::schema::core::workflow_executions, check_for_backend(diesel::pg::Pg))]
pub struct WorkflowExecution {
    pub id: Uuid,
    pub bot_id: Uuid,
    pub workflow_name: String,
    pub current_step: Option<i32>,
    pub state_json: Option<JsonValue>,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = crate::shared::schema::core::workflow_events, check_for_backend(diesel::pg::Pg))]
pub struct WorkflowEvent {
    pub id: Uuid,
    pub execution_id: Uuid,
    pub workflow_id: Uuid,
    pub event_name: String,
    pub event_type: String,
    pub payload: JsonValue,
    pub event_data_json: Option<JsonValue>,
    pub processed: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = crate::shared::schema::core::bot_shared_memory, check_for_backend(diesel::pg::Pg))]
pub struct BotSharedMemory {
    pub id: Uuid,
    pub source_bot_id: Uuid,
    pub target_bot_id: Uuid,
    pub memory_key: String,
    pub memory_value: String,
    pub shared_at: chrono::DateTime<chrono::Utc>,
}
