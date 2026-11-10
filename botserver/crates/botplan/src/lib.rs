

pub mod auth;
pub mod collaboration;

use axum::{response::IntoResponse, Json};
use botllm::provider_catalog::LLMProviderType;
use botllm::provider_factory::create_llm_provider;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub assignee: Option<String>,
    pub start: Option<i64>,
    pub end: Option<i64>,
    pub progress: u8,
    pub tags: Vec<String>,
    pub depends_on: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    /// Verified subject of the creator (#2136). Never `"anonymous"`.
    pub owner_id: String,
    /// Tenant that minted the plan. `None` for plans created by a branchless
    /// suite session (and for pre-#2136 snapshots).
    #[serde(default)]
    pub org_id: Option<String>,
    #[serde(default)]
    pub branch_id: Option<String>,
    pub members: Vec<String>,
    pub tasks: HashMap<String, Task>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Body of `POST /api/plan`. Ids are minted server-side, so the client never
/// chooses (or reuses) a plan id.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreatePlanRequest {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateTaskRequest {
    pub plan_id: String,
    pub title: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub priority: Option<String>,
    #[serde(default)]
    pub assignee: Option<String>,
    #[serde(default)]
    pub start: Option<i64>,
    #[serde(default)]
    pub end: Option<i64>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateTaskRequest {
    pub plan_id: String,
    pub task_id: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub priority: Option<String>,
    #[serde(default)]
    pub assignee: Option<String>,
    #[serde(default)]
    pub start: Option<i64>,
    #[serde(default)]
    pub end: Option<i64>,
    #[serde(default)]
    pub progress: Option<u8>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub depends_on: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AddMemberRequest {
    pub user_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeleteTaskRequest {
    pub plan_id: String,
    pub task_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SetDependenciesRequest {
    pub plan_id: String,
    pub task_id: String,
    pub depends_on: Vec<String>,
}

pub type PlanStore = Arc<RwLock<HashMap<String, Plan>>>;

static STORE: std::sync::OnceLock<PlanStore> = std::sync::OnceLock::new();

/// Postgres pool shared by persistence operations (set once at server mount).
pub use botlib::db_pool::DbPool;
static POOL: std::sync::OnceLock<Option<DbPool>> = std::sync::OnceLock::new();

#[derive(Debug, Clone, Deserialize)]
pub struct PlanAiRequest {
    pub instruction: String,
    #[serde(default)]
    pub context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanAiSuggestion {
    pub title: String,
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(default = "default_priority")]
    pub priority: String,
    #[serde(default)]
    pub assignee: Option<String>,
}

mod default_status;
mod get_store;
mod handlers;
mod routes;

#[cfg(test)]
#[path = "test_support_tests.rs"]
pub(crate) mod test_support;

pub use auth::*;
pub use default_status::*;
pub use get_store::*;
pub use handlers::*;
pub use routes::*;
