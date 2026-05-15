use crate::prompt_manager::VibePromptManager;
use crate::telemetry::VibeTelemetry;
use crate::tool_executor::{ToolDescriptor, VibeToolExecutor};
use crate::types::{VibeProgressEvent, VibeRun, VibeRunConfig, VibeRunState, VibeState, VibeUseCase};
use axum::{
    extract::{Extension, Path, Query},
    response::IntoResponse,
    Json,
};
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateRunRequest {
    pub intent: String,
    pub use_case: Option<String>,
    pub auto_approve: Option<bool>,
    pub max_tool_calls: Option<u32>,
    pub timeout_seconds: Option<u64>,
    pub model: Option<String>,
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

#[derive(Debug, Deserialize)]
pub struct CancelRunRequest {
    pub reason: Option<String>,
}

struct VibeApiInner {
    state: Arc<dyn VibeState>,
    prompt_manager: Arc<VibePromptManager>,
    tool_executor: Arc<VibeToolExecutor>,
    telemetry: Arc<VibeTelemetry>,
    runs: Arc<RwLock<HashMap<Uuid, VibeRun>>>,
}

pub struct VibeApi {
    inner: Arc<VibeApiInner>,
}

impl VibeApi {
    pub fn new(
        state: Arc<dyn VibeState>,
        prompt_manager: Arc<VibePromptManager>,
        tool_executor: Arc<VibeToolExecutor>,
        telemetry: Arc<VibeTelemetry>,
    ) -> Self {
        Self {
            inner: Arc::new(VibeApiInner {
                state,
                prompt_manager,
                tool_executor,
                telemetry,
                runs: Arc::new(RwLock::new(HashMap::new())),
            }),
        }
    }
}

pub fn router(
    state: Arc<dyn VibeState>,
    prompt_manager: Arc<VibePromptManager>,
    tool_executor: Arc<VibeToolExecutor>,
    telemetry: Arc<VibeTelemetry>,
) -> axum::Router {
    let api = Arc::new(VibeApi::new(state, prompt_manager, tool_executor, telemetry));
    axum::Router::new()
        .route("/api/vibe/run", axum::routing::post(create_run))
        .route("/api/vibe/run/{run_id}", axum::routing::get(get_run))
        .route("/api/vibe/run/{run_id}/cancel", axum::routing::post(cancel_run))
        .route("/api/vibe/run/{run_id}/approve", axum::routing::post(approve_run))
        .route("/api/vibe/runs", axum::routing::get(list_runs))
        .route("/api/vibe/tools", axum::routing::get(list_tools))
        .route("/api/vibe/tools/{use_case}", axum::routing::get(list_tools_for_use_case))
        .route("/api/vibe/metrics", axum::routing::get(get_global_metrics))
        .route("/api/vibe/metrics/{run_id}", axum::routing::get(get_run_metrics))
        .route("/api/vibe/events/{run_id}", axum::routing::get(get_run_events))
        .route("/api/vibe/run/{run_id}/execute", axum::routing::post(execute_run))
        .layer(axum::Extension(api))
}

async fn create_run(
    Extension(api): Extension<Arc<VibeApi>>,
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
        auto_approve: req.auto_approve.unwrap_or(false),
        max_tool_calls: req.max_tool_calls.unwrap_or(50),
        timeout_seconds: req.timeout_seconds.unwrap_or(300),
        model: req.model,
    };

    let run = VibeRun::new(Uuid::nil(), Uuid::nil(), Uuid::nil(), req.intent, config);
    let run_id = run.run_id;
    let state_str = run.state.to_string();
    let uc_str = run.use_case.to_string();

    let ctx = api.inner.prompt_manager.build_context(
        run.use_case,
        &run.intent,
        &[],
    );
    let system_prompt = ctx.system_prompt.clone();

    api.inner.telemetry.record_run_start(&run).await;

    {
        let mut runs = api.inner.runs.write().await;
        runs.insert(run_id, run);
    }

    api.inner.state.broadcast_progress(
        VibeProgressEvent::started(run_id.to_string(), "Vibe run created", 3),
    );

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
    Extension(api): Extension<Arc<VibeApi>>,
    Path(run_id): Path<Uuid>,
) -> impl IntoResponse {
    let runs = api.inner.runs.read().await;
    if let Some(run) = runs.get(&run_id) {
        Json(GetRunResponse {
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
        })
    } else {
        Json(GetRunResponse {
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
        })
    }
}

async fn cancel_run(
    Extension(api): Extension<Arc<VibeApi>>,
    Path(run_id): Path<Uuid>,
    Json(_req): Json<CancelRunRequest>,
) -> impl IntoResponse {
    let mut runs = api.inner.runs.write().await;
    if let Some(run) = runs.get_mut(&run_id) {
        run.transition(VibeRunState::Cancelled);
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
    Extension(api): Extension<Arc<VibeApi>>,
    Path(run_id): Path<Uuid>,
) -> impl IntoResponse {
    let mut runs = api.inner.runs.write().await;
    if let Some(run) = runs.get_mut(&run_id) {
        for tool_call in &mut run.tool_calls {
            if tool_call.requires_approval && !tool_call.approved {
                tool_call.approved = true;
            }
        }
        info!("Vibe run approved: {run_id}");
        Json(ActionResponse {
            success: true,
            message: Some("Pending tool calls approved".to_string()),
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
    Extension(api): Extension<Arc<VibeApi>>,
    Query(query): Query<ListRunsQuery>,
) -> impl IntoResponse {
    let runs = api.inner.runs.read().await;
    let limit = query.limit.unwrap_or(50).min(200) as usize;
    let offset = query.offset.unwrap_or(0) as usize;

    let filtered: Vec<GetRunResponse> = runs
        .values()
        .skip(offset)
        .take(limit)
        .filter(|r| {
            query
                .state
                .as_ref()
                .map_or(true, |f| r.state.to_string() == *f)
        })
        .filter(|r| {
            query
                .use_case
                .as_ref()
                .map_or(true, |f| r.use_case.to_string() == *f)
        })
        .map(|r| GetRunResponse {
            run_id: r.run_id,
            bot_id: r.bot_id,
            session_id: r.session_id,
            state: r.state.to_string(),
            use_case: r.use_case.to_string(),
            intent: r.intent.clone(),
            tool_call_count: r.tool_calls.len(),
            created_at: r.created_at.to_rfc3339(),
            completed_at: r.completed_at.map(|t| t.to_rfc3339()),
            error: r.error.clone(),
        })
        .collect();

    Json(filtered)
}

async fn list_tools(Extension(api): Extension<Arc<VibeApi>>) -> impl IntoResponse {
    let tools = api.inner.tool_executor.registry().list_tools().await;
    Json(ListToolsResponse { tools })
}

async fn list_tools_for_use_case(
    Extension(api): Extension<Arc<VibeApi>>,
    Path(use_case): Path<String>,
) -> impl IntoResponse {
    let uc = parse_use_case(&use_case).unwrap_or(VibeUseCase::SoftwareDevelopment);
    let tools = api.inner.tool_executor.registry().list_tools_for_use_case(uc).await;
    Json(ListToolsResponse { tools })
}

async fn get_global_metrics(Extension(api): Extension<Arc<VibeApi>>) -> impl IntoResponse {
    let metrics = api.inner.telemetry.get_global_metrics().await;
    Json(MetricsResponse {
        success: true,
        metrics: Some(serde_json::to_value(metrics).unwrap_or(serde_json::Value::Null)),
        error: None,
    })
}

async fn get_run_metrics(
    Extension(api): Extension<Arc<VibeApi>>,
    Path(run_id): Path<Uuid>,
) -> impl IntoResponse {
    match api.inner.telemetry.get_run_metrics(run_id).await {
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
    Extension(api): Extension<Arc<VibeApi>>,
    Path(run_id): Path<Uuid>,
) -> impl IntoResponse {
    let events = api.inner.telemetry.get_events_for_run(run_id, 100).await;
    Json(events)
}


async fn execute_run(
    Extension(api): Extension<Arc<VibeApi>>,
    Path(run_id): Path<Uuid>,
) -> impl IntoResponse {
    let runs = api.inner.runs.read().await;
    let run = runs.get(&run_id);
    
    if run.is_none() {
        return Json(ActionResponse {
            success: false,
            message: None,
            error: Some("Run not found".to_string()),
        });
    }
    
    let run = run.unwrap();
    let state_clone = api.inner.state.clone();
    
    // Execute pending tool calls
    for tool_call in &run.tool_calls.clone() {
        if !tool_call.approved && tool_call.requires_approval {
            return Json(ActionResponse {
                success: false,
                message: Some("Approval required".to_string()),
                error: None,
            });
        }
        
        let result = api.inner.tool_executor.execute(
            &mut tool_call.clone(),
            run.use_case,
            state_clone.as_ref(),
        ).await;
        
        match result {
            Ok(_) => info!("Tool executed successfully"),
            Err(e) => return Json(ActionResponse {
                success: false,
                message: None,
                error: Some(format!("Execution error: {}", e)),
            }),
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
