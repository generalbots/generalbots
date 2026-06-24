pub mod collaboration;

use axum::{
    extract::Path,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use collaboration::{
    handle_get_plan_collaborators, handle_get_plan_presence, handle_get_plan_typing,
    handle_plan_websocket,
};

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
    pub owner_id: String,
    pub members: Vec<String>,
    pub tasks: HashMap<String, Task>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
}

pub type PlanStore = Arc<RwLock<HashMap<String, Plan>>>;

static STORE: std::sync::OnceLock<PlanStore> = std::sync::OnceLock::new();

pub fn get_store() -> &'static PlanStore {
    STORE.get_or_init(|| Arc::new(RwLock::new(HashMap::new())))
}

pub fn configure_plan_routes() -> Router {
    Router::new()
        .route("/ws/plan/{plan_id}", get(handle_plan_websocket))
        .route(
            "/api/plan/{plan_id}/collaborators",
            get(handle_get_plan_collaborators),
        )
        .route(
            "/api/plan/{plan_id}/presence",
            get(handle_get_plan_presence),
        )
        .route("/api/plan/{plan_id}/typing", get(handle_get_plan_typing))
        .route("/api/plan/{plan_id}", get(handle_get_plan))
        .route("/api/plan/task", post(handle_create_task))
        .route("/api/plan/task/update", post(handle_update_task))
        .route("/api/plan/task/delete", post(handle_delete_task))
}

pub async fn handle_get_plan(Path(plan_id): Path<String>) -> impl IntoResponse {
    let store = get_store().read().await;
    if let Some(plan) = store.get(&plan_id) {
        Json(serde_json::json!({ "ok": true, "plan": plan }))
    } else {
        let plan = Plan {
            id: plan_id.clone(),
            title: "Novo Plano".to_string(),
            description: None,
            owner_id: "anonymous".to_string(),
            members: Vec::new(),
            tasks: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        drop(store);
        let mut w = get_store().write().await;
        w.insert(plan_id.clone(), plan.clone());
        Json(serde_json::json!({ "ok": true, "plan": plan, "created": true }))
    }
}

pub async fn handle_create_task(Json(req): Json<CreateTaskRequest>) -> impl IntoResponse {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let task = Task {
        id: id.clone(),
        title: req.title,
        description: None,
        status: req.status.unwrap_or_else(|| "todo".to_string()),
        priority: req.priority.unwrap_or_else(|| "medium".to_string()),
        assignee: req.assignee,
        start: req.start,
        end: req.end,
        progress: 0,
        tags: req.tags.unwrap_or_default(),
        depends_on: Vec::new(),
        created_at: now,
        updated_at: now,
        created_by: None,
    };
    let mut store = get_store().write().await;
    let plan = store
        .entry(req.plan_id.clone())
        .or_insert_with(|| Plan {
            id: req.plan_id.clone(),
            title: "Plano".to_string(),
            description: None,
            owner_id: "anonymous".to_string(),
            members: Vec::new(),
            tasks: HashMap::new(),
            created_at: now,
            updated_at: now,
        });
    plan.tasks.insert(id.clone(), task.clone());
    plan.updated_at = now;
    Json(serde_json::json!({ "ok": true, "task": task }))
}

pub async fn handle_update_task(Json(req): Json<UpdateTaskRequest>) -> impl IntoResponse {
    let mut store = get_store().write().await;
    if let Some(plan) = store.get_mut(&req.plan_id) {
        if let Some(t) = plan.tasks.get_mut(&req.task_id) {
            if let Some(v) = req.title { t.title = v; }
            if let Some(v) = req.status { t.status = v; }
            if let Some(v) = req.priority { t.priority = v; }
            if let Some(v) = req.assignee { t.assignee = Some(v); }
            if let Some(v) = req.start { t.start = Some(v); }
            if let Some(v) = req.end { t.end = Some(v); }
            if let Some(v) = req.progress { t.progress = v.min(100); }
            if let Some(v) = req.tags { t.tags = v; }
            t.updated_at = Utc::now();
            plan.updated_at = Utc::now();
            let updated = t.clone();
            return Json(serde_json::json!({ "ok": true, "task": updated }));
        }
    }
    Json(serde_json::json!({ "ok": false, "error": "task not found" }))
}

pub async fn handle_delete_task(Json(req): Json<serde_json::Value>) -> impl IntoResponse {
    let plan_id = req
        .get("plan_id")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let task_id = req
        .get("task_id")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let mut store = get_store().write().await;
    if let Some(plan) = store.get_mut(&plan_id) {
        plan.tasks.remove(&task_id);
        plan.updated_at = Utc::now();
        return Json(serde_json::json!({ "ok": true }));
    }
    Json(serde_json::json!({ "ok": false, "error": "plan not found" }))
}
