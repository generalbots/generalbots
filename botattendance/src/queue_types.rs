use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueItem {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub bot_id: Uuid,
    pub channel: String,
    pub user_name: String,
    pub user_email: Option<String>,
    pub last_message: String,
    pub last_message_time: String,
    pub waiting_time_seconds: i64,
    pub priority: i32,
    pub status: QueueStatus,
    pub assigned_to: Option<Uuid>,
    pub assigned_to_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueueStatus {
    Waiting,
    Assigned,
    Active,
    Resolved,
    Abandoned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendantStats {
    pub attendant_id: String,
    pub attendant_name: String,
    pub channel: String,
    pub preferences: String,
    pub active_conversations: i32,
    pub total_handled_today: i32,
    pub avg_response_time_seconds: i32,
    pub status: AttendantStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendantCSV {
    pub id: String,
    pub name: String,
    pub channel: String,
    pub preferences: String,
    pub department: Option<String>,
    pub aliases: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub teams: Option<String>,
    pub google: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttendantStatus {
    Online,
    Busy,
    Away,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignRequest {
    pub session_id: Uuid,
    pub attendant_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferRequest {
    pub session_id: Uuid,
    pub from_attendant_id: Uuid,
    pub to_attendant_id: Uuid,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueFilters {
    pub channel: Option<String>,
    pub status: Option<String>,
    pub assigned_to: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct KanbanColumn {
    pub id: String,
    pub title: String,
    pub items: Vec<QueueItem>,
}

#[derive(Debug, Serialize)]
pub struct KanbanBoard {
    pub columns: Vec<KanbanColumn>,
}

#[derive(Debug, Deserialize)]
pub struct KanbanQuery {
    pub bot_id: Option<Uuid>,
    pub channel: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SkillBasedAssignRequest {
    pub session_id: Uuid,
    pub required_skills: Vec<String>,
    pub channel: Option<String>,
}
