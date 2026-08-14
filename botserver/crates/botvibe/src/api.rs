use crate::agent_loop::AgentLoop;
use crate::pipeline::{PipelineEngine, RunPipeline, StageStatus};
use crate::prompt_manager::VibePromptManager;
use crate::telemetry::VibeTelemetry;
use crate::tool_executor::{ToolDescriptor, VibeToolExecutor};
use crate::types::{VibeProgressEvent, VibeRun, VibeRunConfig, VibeRunState, VibeState, VibeUseCase};
use axum::{
    extract::{Extension, Path, Query},
    response::IntoResponse,
    Json,
};
use diesel::prelude::*;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateRunRequest {
    pub intent: String,
    pub use_case: Option<String>,
    pub lang: Option<String>,
    pub auto_approve: Option<bool>,
    pub max_tool_calls: Option<u32>,
    pub timeout_seconds: Option<u64>,
    pub model: Option<String>,
    pub budget_cents: Option<u64>,
    /// vibe33 #811 — when "deploy", the run executes through the graph
    /// (PipelineEngine, approval-gated deploy pipeline) instead of the
    /// agent loop.
    pub pipeline_mode: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateRunResponse {
    pub success: bool,
    pub run_id: Uuid,
    pub state: String,
    pub use_case: String,
    pub system_prompt: String,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GetRunResponse {
    pub run_id: Uuid,
    pub bot_id: Uuid,
    pub session_id: Uuid,
    pub state: String,
    pub use_case: String,
    pub intent: String,
    pub tool_call_count: usize,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ListToolsResponse {
    pub tools: Vec<ToolDescriptor>,
}

#[derive(Debug, Deserialize)]
pub struct ListRunsQuery {
    pub state: Option<String>,
    pub use_case: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// Resolves a nil bot id to the default bot so chat-driven vibe runs
/// (which carry no explicit bot) still resolve LLM config (Vault + config.csv).
fn resolve_effective_bot_id(pool: &crate::types::DbPool) -> Uuid {
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(_) => return Uuid::nil(),
    };
    #[derive(diesel::QueryableByName)]
    struct BotIdRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
    }
    diesel::sql_query("SELECT id FROM bots WHERE name = 'default' AND is_active = true LIMIT 1")
        .get_result::<BotIdRow>(&mut conn)
        .optional()
        .ok()
        .flatten()
        .map(|r| r.id)
        .unwrap_or(Uuid::nil())
}

#[derive(Debug, Serialize)]
pub struct ActionResponse {
    pub success: bool,
    pub message: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MetricsResponse {
    pub success: bool,
    pub metrics: Option<serde_json::Value>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CapabilitiesResponse {
    pub success: bool,
    pub capabilities: Vec<crate::capability_registry::Capability>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PipelineResponse {
    pub success: bool,
    pub pipeline: crate::pipeline::RunPipeline,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CancelRunRequest {
    pub reason: Option<String>,
}

pub(crate) struct VibeApiInner {
    state: Arc<dyn VibeState>,
    prompt_manager: Arc<VibePromptManager>,
    tool_executor: Arc<VibeToolExecutor>,
    telemetry: Arc<VibeTelemetry>,
    permissions: crate::permissions::PermissionEngineRef,
    skills: Arc<crate::skills::SkillStore>,
    runs: Arc<RwLock<HashMap<Uuid, VibeRun>>>,
    runs_store: crate::run_store::VibeRunStore,
}

impl crate::knowledge_graph::GraphDataSource for VibeApiInner {
    fn snapshot_runs(
        &self,
    ) -> crate::knowledge_graph::GraphFuture<Vec<crate::knowledge_graph::RunNodeInfo>> {
        let runs = Arc::clone(&self.runs);
        let runs_store = self.runs_store.clone();
        Box::pin(async move {
            let mut all: Vec<crate::knowledge_graph::RunNodeInfo> = Vec::new();
            // Persisted runs (survive restarts, issue #799).
            for r in runs_store.list_runs(500) {
                all.push(crate::knowledge_graph::RunNodeInfo {
                    run_id: r.run_id.to_string(),
                    use_case: r.use_case.to_string(),
                    state: r.state.to_string(),
                    intent: r.intent.clone(),
                    tool_names: r.tool_calls.iter().map(|c| c.tool_name.clone()).collect(),
                });
            }
            // In-memory runs not yet flushed to Postgres.
            for r in runs.read().await.values() {
                all.push(crate::knowledge_graph::RunNodeInfo {
                    run_id: r.run_id.to_string(),
                    use_case: r.use_case.to_string(),
                    state: r.state.to_string(),
                    intent: r.intent.clone(),
                    tool_names: r.tool_calls.iter().map(|c| c.tool_name.clone()).collect(),
                });
            }
            all
        })
    }
}

impl VibeApiInner {
    pub(crate) async fn grounding_for(&self, run_id: Uuid) -> Vec<crate::grounding::GroundingSource> {
        let live_run = {
            let runs = self.runs.read().await;
            runs.get(&run_id).cloned()
        };
        let run = live_run.or_else(|| self.runs_store.get_run(run_id));
        let events = self.telemetry.get_events_for_run(run_id, 100).await;
        crate::grounding::build_grounding(run.as_ref(), &events)
    }
}

pub fn router(
    state: Arc<dyn VibeState>,
    prompt_manager: Arc<VibePromptManager>,
    tool_executor: Arc<VibeToolExecutor>,
    telemetry: Arc<VibeTelemetry>,
    permissions: crate::permissions::PermissionEngineRef,
    skills: Arc<crate::skills::SkillStore>,
    pool: crate::types::DbPool,
) -> axum::Router {
    let api = Arc::new(VibeApiInner {
        state,
        prompt_manager,
        tool_executor,
        telemetry,
        permissions,
        skills,
        runs: Arc::new(RwLock::new(HashMap::new())),
        runs_store: crate::run_store::VibeRunStore::new(pool),
    });
    axum::Router::new()
        .route("/api/vibe/run", axum::routing::post(create_run))
        .route("/api/vibe/run/:run_id", axum::routing::get(get_run))
        .route("/api/vibe/run/:run_id/cancel", axum::routing::post(cancel_run))
        .route("/api/vibe/run/:run_id/approve", axum::routing::post(approve_run))
        .route("/api/vibe/runs", axum::routing::get(list_runs))
        .route("/api/vibe/tools", axum::routing::get(list_tools))
        .route("/api/vibe/tools/:use_case", axum::routing::get(list_tools_for_use_case))
        .route("/api/vibe/metrics", axum::routing::get(get_global_metrics))
        .route("/api/vibe/metrics/:run_id", axum::routing::get(get_run_metrics))
        .route("/api/vibe/events/:run_id", axum::routing::get(get_run_events))
        .route("/api/vibe/run/:run_id/grounding", axum::routing::get(crate::grounding::get_run_grounding))
        .route("/api/vibe/run/:run_id/execute", axum::routing::post(execute_run))
        .route("/api/vibe/graph/:use_case", axum::routing::get(crate::knowledge_graph::get_knowledge_graph))
        .route("/api/vibe/graph/run/:run_id", axum::routing::get(crate::knowledge_graph::get_run_graph))
        .route("/api/vibe/capabilities", axum::routing::get(list_capabilities))
        .route("/api/vibe/capabilities/:use_case", axum::routing::get(list_capabilities_for_use_case))
        .route("/api/vibe/pipeline/:use_case", axum::routing::get(get_pipeline))
        .layer(axum::Extension(api))
}

async fn create_run(
    Extension(api): Extension<Arc<VibeApiInner>>,
    Json(req): Json<CreateRunRequest>,
) -> impl IntoResponse {
    info!("Vibe create run: {}", &req.intent[..req.intent.len().min(80)]);

    let use_case = req
        .use_case
        .as_deref()
        .and_then(parse_use_case)
        .unwrap_or(VibeUseCase::SoftwareDevelopment);

    let config = VibeRunConfig {
        use_case,
        lang: req.lang.unwrap_or_else(|| "en".to_string()),
        auto_approve: req.auto_approve.unwrap_or(false),
        max_tool_calls: req.max_tool_calls.unwrap_or(50),
        timeout_seconds: req.timeout_seconds.unwrap_or(300),
        model: req.model,
        llm_key: None,
        llm_url: None,
        budget_cents: req.budget_cents.unwrap_or(0),
    };

    let run = VibeRun::new(resolve_effective_bot_id(api.state.db_pool()), Uuid::nil(), Uuid::nil(), req.intent, config);
    let run_id = run.run_id;
    let state_str = run.state.to_string();
    let uc_str = run.use_case.to_string();

    let ctx = api.prompt_manager.build_context(
        run.use_case,
        &run.config.lang,
        &run.intent,
        &[],
    );
    let system_prompt = ctx.system_prompt.clone();

    api.telemetry.record_run_start(&run).await;

    let agent_loop = Arc::new(
        AgentLoop::new(
            api.prompt_manager.clone(),
            api.tool_executor.clone(),
            api.telemetry.clone(),
            api.state.clone(),
        )
        .with_security(
            api.permissions.clone(),
            api.skills.clone(),
        ),
    );

    {
        let mut runs = api.runs.write().await;
        runs.insert(run_id, run);
    }

    api.state.broadcast_progress(
        VibeProgressEvent::started(run_id.to_string(), "Vibe run created", 3),
    );

    let pipeline_mode = req.pipeline_mode.clone();
    let api_clone = api.clone();
    tokio::spawn(async move {
        let run_opt = {
            let mut runs = api_clone.runs.write().await;
            runs.remove(&run_id)
        };
        if let Some(mut run) = run_opt {
            if pipeline_mode.as_deref() == Some("deploy") {
                // vibe33 #811 — graph execution path: the deploy pipeline
                // runs its stages through the tool executor with approval
                // gates and fail-fast (failed stage skips the rest).
                run.transition(VibeRunState::Running);
                let engine = PipelineEngine::new(api_clone.telemetry.clone());
                let pipeline = RunPipeline::deploy_pipeline(run.use_case);
                let report = engine
                    .run(
                        &pipeline,
                        run_id,
                        run.use_case,
                        &api_clone.tool_executor,
                        api_clone.state.as_ref(),
                        &run.intent,
                    )
                    .await;
                let failed = report
                    .stages
                    .iter()
                    .any(|s| s.status == StageStatus::Failed);
                let skipped = report
                    .stages
                    .iter()
                    .filter(|s| s.status == StageStatus::Skipped)
                    .count();
                if failed {
                    run.transition(VibeRunState::Failed);
                    run.error = Some(
                        report
                            .stages
                            .iter()
                            .find(|s| s.status == StageStatus::Failed)
                            .and_then(|s| s.error.clone())
                            .unwrap_or_else(|| "pipeline stage failed".to_string()),
                    );
                } else {
                    run.transition(VibeRunState::Completed);
                }
                info!(
                    "Vibe deploy pipeline {run_id}: {} stages, {skipped} skipped, failed={failed}",
                    report.stages.len()
                );
                api_clone
                    .telemetry
                    .record_run_completion(&run, 0, None, 0.0)
                    .await;
            } else {
                agent_loop.execute_run(&mut run).await;
            }
            if let Err(e) = api_clone.runs_store.save_run(&run) {
                error!("Vibe: persist run {run_id} failed: {e}");
            }
            let mut runs = api_clone.runs.write().await;
            runs.insert(run_id, run);
        }
    });

    Json(CreateRunResponse {
        success: true,
        run_id,
        state: state_str,
        use_case: uc_str,
        system_prompt,
        error: None,
    })
}

async fn get_run(
    Extension(api): Extension<Arc<VibeApiInner>>,
    Path(run_id): Path<Uuid>,
) -> impl IntoResponse {
    let runs = api.runs.read().await;
    if let Some(run) = runs.get(&run_id) {
        return Json(run_to_response(run));
    }
    drop(runs);
    // Fall back to the persisted store (Issue #793): runs survive restarts.
    match api.runs_store.get_run(run_id) {
        Some(run) => Json(run_to_response(&run)),
        None => Json(GetRunResponse {
            run_id,
            bot_id: Uuid::nil(),
            session_id: Uuid::nil(),
            state: "not_found".to_string(),
            use_case: String::new(),
            intent: String::new(),
            tool_call_count: 0,
            created_at: String::new(),
            completed_at: None,
            error: Some("Run not found".to_string()),
        }),
    }
}

fn run_to_response(run: &VibeRun) -> GetRunResponse {
    GetRunResponse {
        run_id: run.run_id,
        bot_id: run.bot_id,
        session_id: run.session_id,
        state: run.state.to_string(),
        use_case: run.use_case.to_string(),
        intent: run.intent.clone(),
        tool_call_count: run.tool_calls.len(),
        created_at: run.created_at.to_rfc3339(),
        completed_at: run.completed_at.map(|t| t.to_rfc3339()),
        error: run.error.clone(),
    }
}

async fn cancel_run(
    Extension(api): Extension<Arc<VibeApiInner>>,
    Path(run_id): Path<Uuid>,
    Json(_req): Json<CancelRunRequest>,
) -> impl IntoResponse {
    if let Some(tx) = api.state.run_signal_sender() {
        let _ = tx.send(crate::types::VibeRunSignal::Cancelled(run_id));
    }
    let mut runs = api.runs.write().await;
    if let Some(run) = runs.get_mut(&run_id) {
        run.transition(VibeRunState::Cancelled);
        let snapshot = run.clone();
        drop(runs);
        if let Err(e) = api.runs_store.save_run(&snapshot) {
            error!("Vibe: persist cancelled run {run_id} failed: {e}");
        }
        info!("Vibe run cancelled: {run_id}");
        Json(ActionResponse {
            success: true,
            message: Some("Run cancelled".to_string()),
            error: None,
        })
    } else {
        Json(ActionResponse {
            success: false,
            message: None,
            error: Some("Run not found".to_string()),
        })
    }
}

async fn approve_run(
    Extension(api): Extension<Arc<VibeApiInner>>,
    Path(run_id): Path<Uuid>,
) -> impl IntoResponse {
    if let Some(tx) = api.state.run_signal_sender() {
        let _ = tx.send(crate::types::VibeRunSignal::Approved(run_id));
    }
    let mut runs = api.runs.write().await;
    if let Some(run) = runs.get_mut(&run_id) {
        for tool_call in &mut run.tool_calls {
            if tool_call.requires_approval && !tool_call.approved {
                tool_call.approved = true;
            }
        }
        info!("Vibe run approved: {run_id}");
        run.transition(VibeRunState::Running);
        let snapshot = run.clone();
        drop(runs);
        if let Err(e) = api.runs_store.save_run(&snapshot) {
            error!("Vibe: persist approved run {run_id} failed: {e}");
        }
        Json(ActionResponse {
            success: true,
            message: Some("Pending tool calls approved and run resumed".to_string()),
            error: None,
        })
    } else {
        Json(ActionResponse {
            success: false,
            message: None,
            error: Some("Run not found".to_string()),
        })
    }
}

async fn list_runs(
    Extension(api): Extension<Arc<VibeApiInner>>,
    Query(query): Query<ListRunsQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(50).min(200) as usize;
    let offset = query.offset.unwrap_or(0) as usize;

    let (live, persisted) = {
        let runs = api.runs.read().await;
        (runs.clone(), api.runs_store.list_runs((limit + offset) as i64))
    };

    let mut merged: Vec<VibeRun> = persisted;
    for run in live.into_values() {
        if let Some(existing) = merged.iter_mut().find(|r| r.run_id == run.run_id) {
            *existing = run;
        } else {
            merged.push(run);
        }
    }
    merged.sort_by_key(|r| std::cmp::Reverse(r.created_at));

    let filtered: Vec<GetRunResponse> = merged
        .iter()
        .skip(offset)
        .take(limit)
        .filter(|r| {
            query
                .state
                .as_ref()
                .is_none_or(|f| r.state.to_string() == *f)
        })
        .filter(|r| {
            query
                .use_case
                .as_ref()
                .is_none_or(|f| r.use_case.to_string() == *f)
        })
        .map(run_to_response)
        .collect();

    Json(filtered)
}

async fn list_tools(Extension(api): Extension<Arc<VibeApiInner>>) -> impl IntoResponse {
    let tools = api.tool_executor.registry().list_tools().await;
    Json(ListToolsResponse { tools })
}

async fn list_capabilities(Extension(api): Extension<Arc<VibeApiInner>>) -> impl IntoResponse {
    let tools = api.tool_executor.registry().list_tools().await;
    let capabilities = crate::capability_registry::build_capabilities(&tools);
    Json(CapabilitiesResponse {
        success: true,
        capabilities,
        error: None,
    })
}

async fn list_capabilities_for_use_case(
    Extension(api): Extension<Arc<VibeApiInner>>,
    Path(use_case): Path<String>,
) -> impl IntoResponse {
    let uc = parse_use_case(&use_case).unwrap_or(VibeUseCase::SoftwareDevelopment);
    let tools = api.tool_executor.registry().list_tools().await;
    let capabilities = crate::capability_registry::build_capabilities(&tools);
    let filtered = crate::capability_registry::capabilities_for(&capabilities, uc);
    Json(CapabilitiesResponse {
        success: true,
        capabilities: filtered,
        error: None,
    })
}

#[derive(Debug, Deserialize)]
pub struct PipelineQuery {
    pub mode: Option<String>,
}

async fn get_pipeline(
    Path(use_case): Path<String>,
    Query(query): Query<PipelineQuery>,
) -> impl IntoResponse {
    let uc = parse_use_case(&use_case).unwrap_or(VibeUseCase::SoftwareDevelopment);
    let pipeline = if query.mode.as_deref() == Some("deploy") {
        crate::pipeline::RunPipeline::deploy_pipeline(uc)
    } else {
        crate::pipeline::RunPipeline::for_use_case(uc)
    };
    Json(PipelineResponse {
        success: true,
        pipeline,
        error: None,
    })
}

async fn list_tools_for_use_case(
    Extension(api): Extension<Arc<VibeApiInner>>,
    Path(use_case): Path<String>,
) -> impl IntoResponse {
    let uc = parse_use_case(&use_case).unwrap_or(VibeUseCase::SoftwareDevelopment);
    let tools = api.tool_executor.registry().list_tools_for_use_case(uc).await;
    Json(ListToolsResponse { tools })
}

async fn get_global_metrics(Extension(api): Extension<Arc<VibeApiInner>>) -> impl IntoResponse {
    let mut metrics = api.telemetry.get_global_metrics().await;

    let mut runs: Vec<VibeRun> = api.runs_store.list_runs(1000);
    for run in api.runs.read().await.values() {
        match runs.iter_mut().find(|r| r.run_id == run.run_id) {
            Some(existing) => *existing = run.clone(),
            None => runs.push(run.clone()),
        }
    }
    runs.sort_by_key(|r| r.created_at);

    metrics.total_runs = 0;
    metrics.completed_runs = 0;
    metrics.failed_runs = 0;
    for m in metrics.by_use_case.values_mut() {
        m.total_runs = 0;
        m.completed_runs = 0;
        m.failed_runs = 0;
    }
    for run in runs.iter().rev() {
        metrics.total_runs += 1;
        let m = metrics.by_use_case.entry(run.use_case).or_default();
        m.total_runs += 1;
        match run.state {
            VibeRunState::Completed => {
                metrics.completed_runs += 1;
                m.completed_runs += 1;
            }
            VibeRunState::Failed | VibeRunState::Cancelled => {
                metrics.failed_runs += 1;
                m.failed_runs += 1;
            }
            _ => {}
        }
    }

    Json(MetricsResponse {
        success: true,
        metrics: Some(serde_json::to_value(metrics).unwrap_or(serde_json::Value::Null)),
        error: None,
    })
}

async fn get_run_metrics(
    Extension(api): Extension<Arc<VibeApiInner>>,
    Path(run_id): Path<Uuid>,
) -> impl IntoResponse {
    match api.telemetry.get_run_metrics(run_id).await {
        Some(metrics) => Json(MetricsResponse {
            success: true,
            metrics: Some(serde_json::to_value(metrics).unwrap_or(serde_json::Value::Null)),
            error: None,
        }),
        None => Json(MetricsResponse {
            success: false,
            metrics: None,
            error: Some("No metrics found for run".to_string()),
        }),
    }
}

async fn get_run_events(
    Extension(api): Extension<Arc<VibeApiInner>>,
    Path(run_id): Path<Uuid>,
) -> impl IntoResponse {
    let events = api.telemetry.get_events_for_run(run_id, 100).await;
    Json(events)
}


async fn execute_run(
    Extension(api): Extension<Arc<VibeApiInner>>,
    Path(run_id): Path<Uuid>,
) -> impl IntoResponse {
    let runs = api.runs.read().await;
    let run = match runs.get(&run_id) {
        Some(r) => r,
        None => {
            return Json(ActionResponse {
                success: false,
                message: None,
                error: Some("Run not found".to_string()),
            });
        }
    };

    let use_case = run.use_case;
    let tool_calls = run.tool_calls.clone();
    drop(runs);
    let state_clone = api.state.clone();

    for tool_call in &tool_calls {
        if !tool_call.approved && tool_call.requires_approval {
            return Json(ActionResponse {
                success: false,
                message: Some("Approval required".to_string()),
                error: None,
            });
        }

        let mut owned = tool_call.clone();
        let result = api
            .tool_executor
            .execute(&mut owned, use_case, state_clone.as_ref())
            .await;

        match result {
            Ok(_) => info!("Tool executed successfully"),
            Err(e) => {
                return Json(ActionResponse {
                    success: false,
                    message: None,
                    error: Some(format!("Execution error: {e}")),
                });
            }
        }
    }

    Json(ActionResponse {
        success: true,
        message: Some("Run executed".to_string()),
        error: None,
    })
}

fn parse_use_case(s: &str) -> Option<VibeUseCase> {
    match s {
        "software_development" => Some(VibeUseCase::SoftwareDevelopment),
        "customer_support" => Some(VibeUseCase::CustomerSupport),
        "financial_analysis" => Some(VibeUseCase::FinancialAnalysis),
        _ => None,
    }
}
