use crate::schema::user_sessions;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Selectable)]
#[diesel(table_name = user_sessions)]
pub struct UserSession {
    pub id: Uuid,
    pub branch_id: Uuid,
    pub bot_id: Uuid,
    pub session_id: String,
    pub user_id: Option<String>,
    pub data: Option<serde_json::Value>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub data: String,
}
