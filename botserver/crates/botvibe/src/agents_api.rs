//! HTTP surface for the #1172 public Agent API: register, list, execute,
//! trace, and meter agents under `/api/vibe/agents/**`.

use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Serialize;
use uuid::Uuid;

use botsecurity_auth::auth_api::types::AuthenticatedUser;

use crate::agents::{AgentDef, AgentRun, AgentUsage, AgentsRef, ExecAgentRequest, RegisterAgentRequest};

#[derive(Debug, Serialize)]
pub struct AgentsResponse {
    pub success: bool,
    pub agent: Option<AgentDef>,
    pub agents: Option<Vec<AgentDef>>,
    pub run: Option<AgentRun>,
    pub runs: Option<Vec<AgentRun>>,
    pub usage: Option<AgentUsage>,
    pub error: Option<String>,
}

type ApiResult = (StatusCode, Json<AgentsResponse>);

fn ok_agent(agent: AgentDef) -> ApiResult {
    (
        StatusCode::OK,
        Json(AgentsResponse { success: true, agent: Some(agent), agents: None, run: None, runs: None, usage: None, error: None }),
    )
}

fn ok_agents(agents: Vec<AgentDef>) -> ApiResult {
    (
        StatusCode::OK,
        Json(AgentsResponse { success: true, agent: None, agents: Some(agents), run: None, runs: None, usage: None, error: None }),
    )
}

fn ok_run(run: AgentRun) -> ApiResult {
    (
        StatusCode::OK,
        Json(AgentsResponse { success: true, agent: None, agents: None, run: Some(run), runs: None, usage: None, error: None }),
    )
}

fn ok_runs(runs: Vec<AgentRun>) -> ApiResult {
    (
        StatusCode::OK,
        Json(AgentsResponse { success: true, agent: None, agents: None, run: None, runs: Some(runs), usage: None, error: None }),
    )
}

fn ok_usage(usage: AgentUsage) -> ApiResult {
    (
        StatusCode::OK,
        Json(AgentsResponse { success: true, agent: None, agents: None, run: None, runs: None, usage: Some(usage), error: None }),
    )
}

fn err(msg: String) -> ApiResult {
    log::error!("Vibe agents API error: {msg}");
    (
        StatusCode::OK,
        Json(AgentsResponse { success: false, agent: None, agents: None, run: None, runs: None, usage: None, error: Some(msg) }),
    )
}

pub fn agents_router(agents: AgentsRef) -> Router {
    Router::new()
        .route("/api/vibe/agents", get(list_agents).post(register_agent))
        .route("/api/vibe/agents/:agent_id", get(get_agent).delete(delete_agent))
        .route("/api/vibe/agents/:agent_id/exec", post(exec_agent))
        .route("/api/vibe/agents/:agent_id/runs", get(list_runs))
        .route("/api/vibe/agents/:agent_id/usage", get(get_usage))
        .layer(Extension(agents))
}

async fn register_agent(
    Extension(agents): Extension<AgentsRef>,
    Extension(_user): Extension<AuthenticatedUser>,
    Json(req): Json<RegisterAgentRequest>,
) -> ApiResult {
    match agents.register(&req).await {
        Ok(agent) => ok_agent(agent),
        Err(e) => err(e),
    }
}

async fn list_agents(
    Extension(agents): Extension<AgentsRef>,
    Extension(_user): Extension<AuthenticatedUser>,
) -> ApiResult {
    ok_agents(agents.list().await)
}

async fn get_agent(
    Extension(agents): Extension<AgentsRef>,
    Extension(_user): Extension<AuthenticatedUser>,
    Path(agent_id): Path<Uuid>,
) -> ApiResult {
    match agents.get(&agent_id).await {
        Some(agent) => ok_agent(agent),
        None => err(format!("agent {agent_id} not found")),
    }
}

async fn delete_agent(
    Extension(agents): Extension<AgentsRef>,
    Extension(_user): Extension<AuthenticatedUser>,
    Path(agent_id): Path<Uuid>,
) -> ApiResult {
    if agents.delete(&agent_id).await {
        (
            StatusCode::OK,
            Json(AgentsResponse { success: true, agent: None, agents: None, run: None, runs: None, usage: None, error: None }),
        )
    } else {
        err(format!("agent {agent_id} not found"))
    }
}

async fn exec_agent(
    Extension(agents): Extension<AgentsRef>,
    Extension(_user): Extension<AuthenticatedUser>,
    Path(agent_id): Path<Uuid>,
    Json(req): Json<ExecAgentRequest>,
) -> ApiResult {
    if req.input.trim().is_empty() {
        return err("input is required".to_string());
    }
    match agents.exec(&agent_id, &req).await {
        Ok(run) => ok_run(run),
        Err(e) => err(e),
    }
}

async fn list_runs(
    Extension(agents): Extension<AgentsRef>,
    Extension(_user): Extension<AuthenticatedUser>,
    Path(agent_id): Path<Uuid>,
) -> ApiResult {
    ok_runs(agents.runs(&agent_id).await)
}

async fn get_usage(
    Extension(agents): Extension<AgentsRef>,
    Extension(_user): Extension<AuthenticatedUser>,
    Path(agent_id): Path<Uuid>,
) -> ApiResult {
    match agents.usage(&agent_id).await {
        Some(usage) => ok_usage(usage),
        None => err(format!("agent {agent_id} not found")),
    }
}
