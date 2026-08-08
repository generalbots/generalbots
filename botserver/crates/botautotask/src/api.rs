//! AutoTask HTTP surface (#755): request/response contracts and the router.
//! Handler implementations live in `handlers.rs`.

use crate::types::{AutoTaskState, ConfigOps, LlmProviderOps};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct CompileIntentRequest {
    pub intent: String,
    pub bot_id: Option<String>,
    pub execution_mode: Option<String>,
    pub priority: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ClassifyIntentRequest {
    pub intent: String,
    pub bot_id: Option<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<IntentResultResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
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
    pub bot_id: Option<String>,
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
    llm_ops: Arc<dyn LlmProviderOps>,
}

impl AutoTaskApi {
    pub fn new(
        state: Arc<dyn AutoTaskState>,
        config_ops: Arc<dyn ConfigOps>,
        llm_ops: Arc<dyn LlmProviderOps>,
    ) -> Self {
        Self { state, config_ops, llm_ops }
    }

    pub fn state(&self) -> &Arc<dyn AutoTaskState> {
        &self.state
    }

    pub fn config_ops(&self) -> &Arc<dyn ConfigOps> {
        &self.config_ops
    }

    pub fn llm_ops(&self) -> &Arc<dyn LlmProviderOps> {
        &self.llm_ops
    }
}

pub fn router(
    state: Arc<dyn AutoTaskState>,
    config_ops: Arc<dyn ConfigOps>,
    llm_ops: Arc<dyn LlmProviderOps>,
) -> axum::Router {
    use axum::routing::{get, post};
    let api = Arc::new(AutoTaskApi::new(state, config_ops, llm_ops));
    axum::Router::new()
        .route("/api/autotask/classify", post(crate::handlers::classify_intent))
        .route("/api/autotask/compile", post(crate::handlers::compile_intent))
        .route("/api/autotask/execute", post(crate::handlers::execute_plan))
        .route("/api/autotask/create-and-execute", post(crate::handlers::create_and_execute))
        .route("/api/autotask/tasks", get(crate::handlers::list_tasks))
        .route("/api/autotask/stats", get(crate::handlers::get_stats))
        .route("/api/autotask/tasks/:task_id/approve", post(crate::handlers::approve_task))
        .route("/api/autotask/tasks/:task_id/cancel", post(crate::handlers::cancel_task))
        .route("/api/autotask/decide", post(crate::handlers::make_decision))
        .with_state(api)
}