//! Row and request models for the browser policy crate.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::browser_tasks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct BrowserTask {
    pub id: Uuid,
    pub user_id: Uuid,
    pub org_id: Option<Uuid>,
    pub bot_id: Option<Uuid>,
    pub goal: String,
    pub domains: serde_json::Value,
    pub budget_steps: i32,
    pub status: String,
    pub plan: Option<serde_json::Value>,
    pub progress: Option<serde_json::Value>,
    pub result: Option<serde_json::Value>,
    pub citations: Option<serde_json::Value>,
    pub error: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schema::browser_tasks)]
pub struct NewBrowserTask {
    pub id: Uuid,
    pub user_id: Uuid,
    pub org_id: Option<Uuid>,
    pub bot_id: Option<Uuid>,
    pub goal: String,
    pub domains: serde_json::Value,
    pub budget_steps: i32,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::page_facts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PageFact {
    pub id: Uuid,
    pub user_id: Uuid,
    pub url: String,
    pub title: Option<String>,
    pub facts: serde_json::Value,
    pub visit_count: i32,
    pub last_seen: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schema::page_facts)]
pub struct NewPageFact {
    pub id: Uuid,
    pub user_id: Uuid,
    pub url: String,
    pub title: Option<String>,
    pub facts: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::browse_sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct BrowseSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub task_id: Option<Uuid>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schema::browse_sessions)]
pub struct NewBrowseSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub task_id: Option<Uuid>,
}

/// One recorded step inside the `progress` JSONB document of a browser task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressStep {
    pub n: i32,
    pub action: String,
    pub url: String,
    pub ok: bool,
    #[serde(default)]
    pub note: String,
    pub ts: DateTime<Utc>,
}

/// Progress document shape persisted in `browser_tasks.progress`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProgressDoc {
    #[serde(default)]
    pub steps: Vec<ProgressStep>,
    #[serde(default)]
    pub cost_milli: u64,
}

impl ProgressDoc {
    pub fn parse(value: Option<&serde_json::Value>) -> Self {
        match value {
            Some(v) => serde_json::from_value::<Self>(v.clone()).unwrap_or_default(),
            None => Self::default(),
        }
    }

    pub fn to_value(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or_else(|_| serde_json::json!({ "steps": [] }))
    }
}
