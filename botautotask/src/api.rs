use crate::types::{AutoTaskState, ConfigOps};
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use log::info;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CompileIntentRequest {
    pub intent: String,
    pub execution_mode: Option<String>,
    pub priority: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ClassifyIntentRequest {
    pub intent: String,
    pub auto_process: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ClassifyIntentResponse {
    pub success: bool,
    pub intent_type: String,
    pub confidence: f64,
    pub suggested_name: Option<String>,
    pub requires_clarification: bool,
    pub clarification_question: Option<String>,
    pub result: Option<IntentResultResponse>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct IntentResultResponse {
    pub success: bool,
    pub message: String,
    pub app_url: Option<String>,
    pub task_id: Option<String>,
    pub schedule_id: Option<String>,
    pub tool_triggers: Vec<String>,
    pub created_resources: Vec<CreatedResourceResponse>,
    pub next_steps: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAndExecuteRequest {
    pub intent: String,
}

#[derive(Debug, Serialize)]
pub struct CreateAndExecuteResponse {
    pub success: bool,
    pub task_id: String,
    pub status: String,
    pub message: String,
    pub app_url: Option<String>,
    pub created_resources: Vec<CreatedResourceResponse>,
    pub pending_items: Vec<PendingItemResponse>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PendingItemResponse {
    pub id: String,
    pub label: String,
    pub config_key: String,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreatedResourceResponse {
    pub resource_type: String,
    pub name: String,
    pub path: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CompileIntentResponse {
    pub success: bool,
    pub plan_id: Option<String>,
    pub plan_name: Option<String>,
    pub plan_description: Option<String>,
    pub steps: Vec<PlanStepResponse>,
    pub alternatives: Vec<AlternativeResponse>,
    pub confidence: f64,
    pub risk_level: String,
    pub estimated_duration_minutes: i32,
    pub estimated_cost: f64,
    pub resource_estimate: ResourceEstimateResponse,
    pub basic_program: Option<String>,
    pub requires_approval: bool,
    pub mcp_servers: Vec<String>,
    pub external_apis: Vec<String>,
    pub risks: Vec<RiskResponse>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PlanStepResponse {
    pub id: String,
    pub order: i32,
    pub name: String,
    pub description: String,
    pub keywords: Vec<String>,
    pub priority: String,
    pub risk_level: String,
    pub estimated_minutes: i32,
    pub requires_approval: bool,
}

#[derive(Debug, Serialize)]
pub struct AlternativeResponse {
    pub id: String,
    pub description: String,
    pub confidence: f64,
    pub pros: Vec<String>,
    pub cons: Vec<String>,
    pub estimated_cost: Option<f64>,
    pub estimated_time_hours: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct ResourceEstimateResponse {
    pub compute_hours: f64,
    pub storage_gb: f64,
    pub api_calls: i32,
    pub llm_tokens: i32,
    pub estimated_cost_usd: f64,
}

#[derive(Debug, Serialize)]
pub struct RiskResponse {
    pub id: String,
    pub category: String,
    pub description: String,
    pub probability: f64,
    pub impact: String,
}

#[derive(Debug, Deserialize)]
pub struct ExecutePlanRequest {
    pub plan_id: String,
    pub execution_mode: Option<String>,
    pub priority: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExecutePlanResponse {
    pub success: bool,
    pub task_id: Option<String>,
    pub status: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListTasksQuery {
    pub filter: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct AutoTaskStatsResponse {
    pub total: i32,
    pub running: i32,
    pub pending: i32,
    pub completed: i32,
    pub failed: i32,
    pub pending_approval: i32,
    pub pending_decision: i32,
}

#[derive(Debug, Serialize)]
pub struct TaskActionResponse {
    pub success: bool,
    pub message: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DecisionRequest {
    pub decision_id: String,
    pub choice: String,
}

pub struct AutoTaskApi {
    state: Arc<dyn AutoTaskState>,
    config_ops: Arc<dyn ConfigOps>,
}

impl AutoTaskApi {
    pub fn new(state: Arc<dyn AutoTaskState>, config_ops: Arc<dyn ConfigOps>) -> Self {
        Self { state, config_ops }
    }

    pub fn state(&self) -> &Arc<dyn AutoTaskState> {
        &self.state
    }

    pub fn config_ops(&self) -> &Arc<dyn ConfigOps> {
        &self.config_ops
    }
}

pub fn router(state: Arc<dyn AutoTaskState>, config_ops: Arc<dyn ConfigOps>) -> axum::Router {
    let api = Arc::new(AutoTaskApi::new(state, config_ops));
    axum::Router::new()
        .route("/api/autotask/classify", axum::routing::post(classify_intent))
        .route("/api/autotask/compile", axum::routing::post(compile_intent))
        .route("/api/autotask/execute", axum::routing::post(execute_plan))
        .route("/api/autotask/create-and-execute", axum::routing::post(create_and_execute))
        .route("/api/autotask/tasks", axum::routing::get(list_tasks))
        .route("/api/autotask/stats", axum::routing::get(get_stats))
        .route("/api/autotask/tasks/:task_id/approve", axum::routing::post(approve_task))
        .route("/api/autotask/tasks/:task_id/cancel", axum::routing::post(cancel_task))
        .route("/api/autotask/decide", axum::routing::post(make_decision))
        .with_state(api)
}

async fn classify_intent(
    State(_api): State<Arc<AutoTaskApi>>,
    Json(req): Json<ClassifyIntentRequest>,
) -> impl IntoResponse {
    info!("API classify intent: {}", &req.intent[..req.intent.len().min(50)]);
    Json(ClassifyIntentResponse {
        success: true,
        intent_type: "UNKNOWN".to_string(),
        confidence: 0.5,
        suggested_name: None,
        requires_clarification: false,
        clarification_question: None,
        result: None,
        error: None,
    })
}

async fn compile_intent(
    State(_api): State<Arc<AutoTaskApi>>,
    Json(req): Json<CompileIntentRequest>,
) -> impl IntoResponse {
    info!("API compile intent: {}", &req.intent[..req.intent.len().min(50)]);
    Json(CompileIntentResponse {
        success: true,
        plan_id: Some(Uuid::new_v4().to_string()),
        plan_name: Some("Generated Plan".to_string()),
        plan_description: Some(req.intent.clone()),
        steps: Vec::new(),
        alternatives: Vec::new(),
        confidence: 0.5,
        risk_level: "low".to_string(),
        estimated_duration_minutes: 10,
        estimated_cost: 0.0,
        resource_estimate: ResourceEstimateResponse {
            compute_hours: 0.0, storage_gb: 0.0, api_calls: 0,
            llm_tokens: 0, estimated_cost_usd: 0.0,
        },
        basic_program: None,
        requires_approval: false,
        mcp_servers: Vec::new(),
        external_apis: Vec::new(),
        risks: Vec::new(),
        error: None,
    })
}

async fn execute_plan(
    State(_api): State<Arc<AutoTaskApi>>,
    Json(req): Json<ExecutePlanRequest>,
) -> impl IntoResponse {
    info!("API execute plan: {}", req.plan_id);
    Json(ExecutePlanResponse {
        success: true,
        task_id: Some(Uuid::new_v4().to_string()),
        status: Some("running".to_string()),
        error: None,
    })
}

async fn create_and_execute(
    State(_api): State<Arc<AutoTaskApi>>,
    Json(req): Json<CreateAndExecuteRequest>,
) -> impl IntoResponse {
    info!("API create and execute: {}", &req.intent[..req.intent.len().min(50)]);
    let task_id = Uuid::new_v4().to_string();
    Json(CreateAndExecuteResponse {
        success: true,
        task_id,
        status: "running".to_string(),
        message: "Task created and executing".to_string(),
        app_url: None,
        created_resources: Vec::new(),
        pending_items: Vec::new(),
        error: None,
    })
}

async fn list_tasks(
    State(_api): State<Arc<AutoTaskApi>>,
    Query(query): Query<ListTasksQuery>,
) -> impl IntoResponse {
    let _ = query;
    Json(Vec::<serde_json::Value>::new())
}

async fn get_stats(
    State(_api): State<Arc<AutoTaskApi>>,
) -> impl IntoResponse {
    Json(AutoTaskStatsResponse {
        total: 0, running: 0, pending: 0, completed: 0,
        failed: 0, pending_approval: 0, pending_decision: 0,
    })
}

async fn approve_task(
    State(_api): State<Arc<AutoTaskApi>>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    info!("API approve task: {task_id}");
    Json(TaskActionResponse { success: true, message: Some("Task approved".to_string()), error: None })
}

async fn cancel_task(
    State(_api): State<Arc<AutoTaskApi>>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    info!("API cancel task: {task_id}");
    Json(TaskActionResponse { success: true, message: Some("Task cancelled".to_string()), error: None })
}

async fn make_decision(
    State(_api): State<Arc<AutoTaskApi>>,
    Json(req): Json<DecisionRequest>,
) -> impl IntoResponse {
    info!("API make decision: {}", req.decision_id);
    Json(TaskActionResponse { success: true, message: Some("Decision recorded".to_string()), error: None })
}
