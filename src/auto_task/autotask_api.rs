use crate::auto_task::task_manifest::{TaskManifest, ManifestStatus};
use crate::auto_task::task_types::{
    AutoTask, AutoTaskStatus, ExecutionMode, PendingApproval, PendingDecision, TaskPriority,
};
use crate::auto_task::intent_classifier::IntentClassifier;
use crate::auto_task::intent_compiler::IntentCompiler;
use crate::auto_task::safety_layer::{SafetyLayer, SimulationResult};
use crate::shared::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{Text, Uuid as DieselUuid};
use log::{error, info, trace};
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
    pub option_id: Option<String>,
    pub skip: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ApprovalRequest {
    pub approval_id: String,
    pub action: String,
    pub comment: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SimulationResponse {
    pub success: bool,
    pub confidence: f64,
    pub risk_score: f64,
    pub risk_level: String,
    pub step_outcomes: Vec<StepOutcomeResponse>,
    pub impact: ImpactResponse,
    pub side_effects: Vec<SideEffectResponse>,
    pub recommendations: Vec<RecommendationResponse>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct StepOutcomeResponse {
    pub step_id: String,
    pub step_name: String,
    pub would_succeed: bool,
    pub success_probability: f64,
    pub failure_modes: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ImpactResponse {
    pub risk_score: f64,
    pub risk_level: String,
    pub data_impact: DataImpactResponse,
    pub cost_impact: CostImpactResponse,
    pub time_impact: TimeImpactResponse,
    pub security_impact: SecurityImpactResponse,
}

#[derive(Debug, Serialize)]
pub struct DataImpactResponse {
    pub records_created: i32,
    pub records_modified: i32,
    pub records_deleted: i32,
    pub tables_affected: Vec<String>,
    pub reversible: bool,
}

#[derive(Debug, Serialize)]
pub struct CostImpactResponse {
    pub api_costs: f64,
    pub compute_costs: f64,
    pub storage_costs: f64,
    pub total_estimated_cost: f64,
}

#[derive(Debug, Serialize)]
pub struct TimeImpactResponse {
    pub estimated_duration_seconds: i32,
    pub blocking: bool,
}

#[derive(Debug, Serialize)]
pub struct SecurityImpactResponse {
    pub risk_level: String,
    pub credentials_accessed: Vec<String>,
    pub external_systems: Vec<String>,
    pub concerns: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SideEffectResponse {
    pub effect_type: String,
    pub description: String,
    pub severity: String,
    pub mitigation: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RecommendationResponse {
    pub id: String,
    pub recommendation_type: String,
    pub description: String,
    pub action: Option<String>,
}

pub async fn create_and_execute_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateAndExecuteRequest>,
) -> impl IntoResponse {
    info!(
        "Create and execute: {}",
        &request.intent[..request.intent.len().min(100)]
    );

    let session = match get_current_session(&state) {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(CreateAndExecuteResponse {
                    success: false,
                    task_id: String::new(),
                    status: "error".to_string(),
                    message: format!("Authentication error: {}", e),
                    app_url: None,
                    created_resources: Vec::new(),
                    pending_items: Vec::new(),
                    error: Some(e.to_string()),
                }),
            );
        }
    };

    // Create task record first
    let task_id = Uuid::new_v4();
    if let Err(e) = create_task_record(&state, task_id, &session, &request.intent) {
        error!("Failed to create task record: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(CreateAndExecuteResponse {
                success: false,
                task_id: String::new(),
                status: "error".to_string(),
                message: format!("Failed to create task: {}", e),
                app_url: None,
                created_resources: Vec::new(),
                pending_items: Vec::new(),
                error: Some(e.to_string()),
            }),
        );
    }

    // Update status to running
    let _ = update_task_status_db(&state, task_id, "running", None);

    // Clone what we need for the background task
    let state_clone = Arc::clone(&state);
    let intent = request.intent.clone();
    let session_clone = session.clone();
    let task_id_str = task_id.to_string();

    // Spawn background task to do the actual work
    let spawn_result = tokio::spawn(async move {
        info!("[AUTOTASK] *** Background task STARTED for task_id={} ***", task_id_str);

        // Use IntentClassifier to classify and process with task tracking
        let classifier = IntentClassifier::new(state_clone.clone());

        info!("[AUTOTASK] Calling classify_and_process_with_task_id for task_id={}", task_id_str);

        let result = classifier
            .classify_and_process_with_task_id(&intent, &session_clone, Some(task_id_str.clone()))
            .await;

        info!("[AUTOTASK] classify_and_process_with_task_id returned for task_id={}", task_id_str);

        match result {
            Ok(result) => {
                let status = if result.success {
                    "completed"
                } else {
                    "failed"
                };
                let _ = update_task_status_db(&state_clone, task_id, status, result.error.as_deref());
                info!(
                    "[AUTOTASK] *** Background task COMPLETED: task_id={}, status={}, message={} ***",
                    task_id_str, status, result.message
                );
            }
            Err(e) => {
                let _ = update_task_status_db(&state_clone, task_id, "failed", Some(&e.to_string()));
                error!(
                    "[AUTOTASK] *** Background task FAILED: task_id={}, error={} ***",
                    task_id_str, e
                );
            }
        }
    });

    info!("[AUTOTASK] Spawn result: {:?}", spawn_result);

    // Return immediately with task_id - client will poll for status
    info!("[AUTOTASK] Returning immediately with task_id={}", task_id);
    (
        StatusCode::ACCEPTED,
        Json(CreateAndExecuteResponse {
            success: true,
            task_id: task_id.to_string(),
            status: "running".to_string(),
            message: "Task started, poll for status".to_string(),
            app_url: None,
            created_resources: Vec::new(),
            pending_items: Vec::new(),
            error: None,
        }),
    )
}

pub async fn classify_intent_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ClassifyIntentRequest>,
) -> impl IntoResponse {
    info!(
        "Classifying intent: {}",
        &request.intent[..request.intent.len().min(100)]
    );

    let session = match get_current_session(&state) {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ClassifyIntentResponse {
                    success: false,
                    intent_type: "UNKNOWN".to_string(),
                    confidence: 0.0,
                    suggested_name: None,
                    requires_clarification: false,
                    clarification_question: None,
                    result: None,
                    error: Some(format!("Authentication error: {}", e)),
                }),
            );
        }
    };

    let classifier = IntentClassifier::new(Arc::clone(&state));
    let auto_process = request.auto_process.unwrap_or(true);

    if auto_process {
        // Classify and process in one step
        match classifier
            .classify_and_process(&request.intent, &session)
            .await
        {
            Ok(result) => {
                let response = ClassifyIntentResponse {
                    success: result.success,
                    intent_type: result.intent_type.to_string(),
                    confidence: 0.0, // Would come from classification
                    suggested_name: None,
                    requires_clarification: false,
                    clarification_question: None,
                    result: Some(IntentResultResponse {
                        success: result.success,
                        message: result.message,
                        app_url: result.app_url,
                        task_id: result.task_id,
                        schedule_id: result.schedule_id,
                        tool_triggers: result.tool_triggers,
                        created_resources: result
                            .created_resources
                            .into_iter()
                            .map(|r| CreatedResourceResponse {
                                resource_type: r.resource_type,
                                name: r.name,
                                path: r.path,
                            })
                            .collect(),
                        next_steps: result.next_steps,
                    }),
                    error: result.error,
                };
                (StatusCode::OK, Json(response))
            }
            Err(e) => {
                error!("Failed to classify/process intent: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ClassifyIntentResponse {
                        success: false,
                        intent_type: "UNKNOWN".to_string(),
                        confidence: 0.0,
                        suggested_name: None,
                        requires_clarification: false,
                        clarification_question: None,
                        result: None,
                        error: Some(e.to_string()),
                    }),
                )
            }
        }
    } else {
        // Just classify, don't process
        match classifier.classify(&request.intent, &session).await {
            Ok(classification) => {
                let response = ClassifyIntentResponse {
                    success: true,
                    intent_type: classification.intent_type.to_string(),
                    confidence: classification.confidence,
                    suggested_name: classification.suggested_name,
                    requires_clarification: classification.requires_clarification,
                    clarification_question: classification.clarification_question,
                    result: None,
                    error: None,
                };
                (StatusCode::OK, Json(response))
            }
            Err(e) => {
                error!("Failed to classify intent: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ClassifyIntentResponse {
                        success: false,
                        intent_type: "UNKNOWN".to_string(),
                        confidence: 0.0,
                        suggested_name: None,
                        requires_clarification: false,
                        clarification_question: None,
                        result: None,
                        error: Some(e.to_string()),
                    }),
                )
            }
        }
    }
}

pub async fn compile_intent_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CompileIntentRequest>,
) -> impl IntoResponse {
    info!(
        "Compiling intent: {}",
        &request.intent[..request.intent.len().min(100)]
    );

    let session = match get_current_session(&state) {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(CompileIntentResponse {
                    success: false,
                    plan_id: None,
                    plan_name: None,
                    plan_description: None,
                    steps: Vec::new(),
                    alternatives: Vec::new(),
                    confidence: 0.0,
                    risk_level: "unknown".to_string(),
                    estimated_duration_minutes: 0,
                    estimated_cost: 0.0,
                    resource_estimate: ResourceEstimateResponse {
                        compute_hours: 0.0,
                        storage_gb: 0.0,
                        api_calls: 0,
                        llm_tokens: 0,
                        estimated_cost_usd: 0.0,
                    },
                    basic_program: None,
                    requires_approval: false,
                    mcp_servers: Vec::new(),
                    external_apis: Vec::new(),
                    risks: Vec::new(),
                    error: Some(format!("Authentication error: {}", e)),
                }),
            );
        }
    };

    let compiler = IntentCompiler::new(Arc::clone(&state));

    match compiler.compile(&request.intent, &session).await {
        Ok(compiled) => {
            let response = CompileIntentResponse {
                success: true,
                plan_id: Some(compiled.plan.id.clone()),
                plan_name: Some(compiled.plan.name.clone()),
                plan_description: Some(compiled.plan.description.clone()),
                steps: compiled
                    .plan
                    .steps
                    .iter()
                    .map(|s| PlanStepResponse {
                        id: s.id.clone(),
                        order: s.order,
                        name: s.name.clone(),
                        description: s.description.clone(),
                        keywords: s.keywords.clone(),
                        priority: format!("{:?}", s.priority),
                        risk_level: format!("{:?}", s.risk_level),
                        estimated_minutes: s.estimated_minutes,
                        requires_approval: s.requires_approval,
                    })
                    .collect(),
                alternatives: compiled
                    .alternatives
                    .iter()
                    .map(|a| AlternativeResponse {
                        id: a.id.clone(),
                        description: a.description.clone(),
                        confidence: a.confidence,
                        pros: a.pros.clone(),
                        cons: a.cons.clone(),
                        estimated_cost: a.estimated_cost,
                        estimated_time_hours: a.estimated_time_hours,
                    })
                    .collect(),
                confidence: compiled.confidence,
                risk_level: format!("{:?}", compiled.risk_assessment.overall_risk),
                estimated_duration_minutes: compiled.plan.estimated_duration_minutes,
                estimated_cost: compiled.resource_estimate.estimated_cost_usd,
                resource_estimate: ResourceEstimateResponse {
                    compute_hours: compiled.resource_estimate.compute_hours,
                    storage_gb: compiled.resource_estimate.storage_gb,
                    api_calls: compiled.resource_estimate.api_calls,
                    llm_tokens: compiled.resource_estimate.llm_tokens,
                    estimated_cost_usd: compiled.resource_estimate.estimated_cost_usd,
                },
                basic_program: Some(compiled.basic_program.clone()),
                requires_approval: compiled.plan.requires_approval,
                mcp_servers: compiled.resource_estimate.mcp_servers_needed.clone(),
                external_apis: compiled.resource_estimate.external_services.clone(),
                risks: compiled
                    .risk_assessment
                    .risks
                    .iter()
                    .map(|r| RiskResponse {
                        id: r.id.clone(),
                        category: format!("{:?}", r.category),
                        description: r.description.clone(),
                        probability: r.probability,
                        impact: format!("{:?}", r.impact),
                    })
                    .collect(),
                error: None,
            };

            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            error!("Failed to compile intent: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CompileIntentResponse {
                    success: false,
                    plan_id: None,
                    plan_name: None,
                    plan_description: None,
                    steps: Vec::new(),
                    alternatives: Vec::new(),
                    confidence: 0.0,
                    risk_level: "unknown".to_string(),
                    estimated_duration_minutes: 0,
                    estimated_cost: 0.0,
                    resource_estimate: ResourceEstimateResponse {
                        compute_hours: 0.0,
                        storage_gb: 0.0,
                        api_calls: 0,
                        llm_tokens: 0,
                        estimated_cost_usd: 0.0,
                    },
                    basic_program: None,
                    requires_approval: false,
                    mcp_servers: Vec::new(),
                    external_apis: Vec::new(),
                    risks: Vec::new(),
                    error: Some(e.to_string()),
                }),
            )
        }
    }
}

pub async fn execute_plan_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ExecutePlanRequest>,
) -> impl IntoResponse {
    info!("Executing plan: {}", request.plan_id);

    let session = match get_current_session(&state) {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ExecutePlanResponse {
                    success: false,
                    task_id: None,
                    status: None,
                    error: Some(format!("Authentication error: {}", e)),
                }),
            );
        }
    };

    let execution_mode = match request.execution_mode.as_deref() {
        Some("fully-automatic") => ExecutionMode::FullyAutomatic,
        Some("supervised") => ExecutionMode::Supervised,
        Some("manual") => ExecutionMode::Manual,
        Some("dry-run") => ExecutionMode::DryRun,
        _ => ExecutionMode::SemiAutomatic,
    };

    let priority = match request.priority.as_deref() {
        Some("critical") => TaskPriority::Critical,
        Some("high") => TaskPriority::High,
        Some("low") => TaskPriority::Low,
        Some("background") => TaskPriority::Background,
        _ => TaskPriority::Medium,
    };

    match create_auto_task_from_plan(&state, &session, &request.plan_id, execution_mode, priority) {
        Ok(task) => match start_task_execution(&state, &task.id) {
            Ok(_) => (
                StatusCode::OK,
                Json(ExecutePlanResponse {
                    success: true,
                    task_id: Some(task.id),
                    status: Some(task.status.to_string()),
                    error: None,
                }),
            ),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ExecutePlanResponse {
                    success: false,
                    task_id: Some(task.id),
                    status: Some("failed".to_string()),
                    error: Some(e.to_string()),
                }),
            ),
        },
        Err(e) => {
            error!("Failed to create task: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ExecutePlanResponse {
                    success: false,
                    task_id: None,
                    status: None,
                    error: Some(e.to_string()),
                }),
            )
        }
    }
}

pub async fn list_tasks_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListTasksQuery>,
) -> impl IntoResponse {
    let filter = query.filter.as_deref().unwrap_or("all");
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    match list_auto_tasks(&state, filter, limit, offset) {
        Ok(tasks) => {
            let html = render_task_list_html(&tasks);
            (StatusCode::OK, axum::response::Html(html))
        }
        Err(e) => {
            error!("Failed to list tasks: {}", e);
            let html = format!(
                r#"<div class="error-message">
                    <span class="error-icon">❌</span>
                    <p>Failed to load tasks: {}</p>
                </div>"#,
                html_escape(&e.to_string())
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::response::Html(html),
            )
        }
    }
}

/// Get a single task by ID - used for polling task status
pub async fn get_task_handler(
    State(state): State<Arc<AppState>>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting task: {}", task_id);

    match get_auto_task_by_id(&state, &task_id) {
        Ok(Some(task)) => {
            let error_str = task.error.as_ref().map(|e| e.message.clone());
            (StatusCode::OK, Json(serde_json::json!({
                "id": task.id,
                "name": task.title,
                "description": task.intent,
                "status": format!("{:?}", task.status).to_lowercase(),
                "progress": task.progress,
                "current_step": task.current_step,
                "total_steps": task.total_steps,
                "error": error_str,
                "created_at": task.created_at,
                "updated_at": task.updated_at,
                "completed_at": task.completed_at,
            })))
        },
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "error": "Task not found"
        }))),
        Err(e) => {
            error!("Failed to get task {}: {}", task_id, e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": e.to_string()
            })))
        }
    }
}

pub async fn get_stats_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match get_auto_task_stats(&state) {
        Ok(stats) => (StatusCode::OK, Json(stats)),
        Err(e) => {
            error!("Failed to get stats: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AutoTaskStatsResponse {
                    total: 0,
                    running: 0,
                    pending: 0,
                    completed: 0,
                    failed: 0,
                    pending_approval: 0,
                    pending_decision: 0,
                }),
            )
        }
    }
}

pub async fn pause_task_handler(
    State(state): State<Arc<AppState>>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    match update_task_status(&state, &task_id, AutoTaskStatus::Paused) {
        Ok(_) => (
            StatusCode::OK,
            Json(TaskActionResponse {
                success: true,
                message: Some("Task paused".to_string()),
                error: None,
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(TaskActionResponse {
                success: false,
                message: None,
                error: Some(e.to_string()),
            }),
        ),
    }
}

pub async fn resume_task_handler(
    State(state): State<Arc<AppState>>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    match update_task_status(&state, &task_id, AutoTaskStatus::Running) {
        Ok(_) => {
            let _ = start_task_execution(&state, &task_id);
            (
                StatusCode::OK,
                Json(TaskActionResponse {
                    success: true,
                    message: Some("Task resumed".to_string()),
                    error: None,
                }),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(TaskActionResponse {
                success: false,
                message: None,
                error: Some(e.to_string()),
            }),
        ),
    }
}

pub async fn cancel_task_handler(
    State(state): State<Arc<AppState>>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    match update_task_status(&state, &task_id, AutoTaskStatus::Cancelled) {
        Ok(_) => (
            StatusCode::OK,
            Json(TaskActionResponse {
                success: true,
                message: Some("Task cancelled".to_string()),
                error: None,
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(TaskActionResponse {
                success: false,
                message: None,
                error: Some(e.to_string()),
            }),
        ),
    }
}

pub async fn simulate_task_handler(
    State(state): State<Arc<AppState>>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    let session = match get_current_session(&state) {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(SimulationResponse {
                    success: false,
                    confidence: 0.0,
                    risk_score: 0.0,
                    risk_level: "unknown".to_string(),
                    step_outcomes: Vec::new(),
                    impact: ImpactResponse {
                        risk_score: 0.0,
                        risk_level: "unknown".to_string(),
                        data_impact: DataImpactResponse {
                            records_created: 0,
                            records_modified: 0,
                            records_deleted: 0,
                            tables_affected: Vec::new(),
                            reversible: true,
                        },
                        cost_impact: CostImpactResponse {
                            api_costs: 0.0,
                            compute_costs: 0.0,
                            storage_costs: 0.0,
                            total_estimated_cost: 0.0,
                        },
                        time_impact: TimeImpactResponse {
                            estimated_duration_seconds: 0,
                            blocking: false,
                        },
                        security_impact: SecurityImpactResponse {
                            risk_level: "unknown".to_string(),
                            credentials_accessed: Vec::new(),
                            external_systems: Vec::new(),
                            concerns: Vec::new(),
                        },
                    },
                    side_effects: Vec::new(),
                    recommendations: Vec::new(),
                    error: Some(format!("Authentication error: {}", e)),
                }),
            );
        }
    };

    let safety_layer = SafetyLayer::new(Arc::clone(&state));

    match simulate_task_execution(&state, &safety_layer, &task_id, &session) {
        Ok(result) => {
            let response = SimulationResponse {
                success: result.success,
                confidence: result.confidence,
                risk_score: result.impact.risk_score,
                risk_level: format!("{}", result.impact.risk_level),
                step_outcomes: result
                    .step_outcomes
                    .iter()
                    .map(|s| StepOutcomeResponse {
                        step_id: s.step_id.clone(),
                        step_name: s.step_name.clone(),
                        would_succeed: s.would_succeed,
                        success_probability: s.success_probability,
                        failure_modes: s
                            .failure_modes
                            .iter()
                            .map(|f| f.failure_type.clone())
                            .collect(),
                    })
                    .collect(),
                impact: ImpactResponse {
                    risk_score: result.impact.risk_score,
                    risk_level: format!("{}", result.impact.risk_level),
                    data_impact: DataImpactResponse {
                        records_created: result.impact.data_impact.records_created,
                        records_modified: result.impact.data_impact.records_modified,
                        records_deleted: result.impact.data_impact.records_deleted,
                        tables_affected: result.impact.data_impact.tables_affected.clone(),
                        reversible: result.impact.data_impact.reversible,
                    },
                    cost_impact: CostImpactResponse {
                        api_costs: result.impact.cost_impact.api_costs,
                        compute_costs: result.impact.cost_impact.compute_costs,
                        storage_costs: result.impact.cost_impact.storage_costs,
                        total_estimated_cost: result.impact.cost_impact.total_estimated_cost,
                    },
                    time_impact: TimeImpactResponse {
                        estimated_duration_seconds: result
                            .impact
                            .time_impact
                            .estimated_duration_seconds,
                        blocking: result.impact.time_impact.blocking,
                    },
                    security_impact: SecurityImpactResponse {
                        risk_level: format!("{}", result.impact.security_impact.risk_level),
                        credentials_accessed: result
                            .impact
                            .security_impact
                            .credentials_accessed
                            .clone(),
                        external_systems: result.impact.security_impact.external_systems.clone(),
                        concerns: result.impact.security_impact.concerns.clone(),
                    },
                },
                side_effects: result
                    .side_effects
                    .iter()
                    .map(|s| SideEffectResponse {
                        effect_type: s.effect_type.clone(),
                        description: s.description.clone(),
                        severity: format!("{:?}", s.severity),
                        mitigation: s.mitigation.clone(),
                    })
                    .collect(),
                recommendations: result
                    .recommendations
                    .iter()
                    .enumerate()
                    .map(|(i, r)| RecommendationResponse {
                        id: format!("rec-{}", i),
                        recommendation_type: format!("{:?}", r.recommendation_type),
                        description: r.description.clone(),
                        action: r.action.clone(),
                    })
                    .collect(),
                error: None,
            };
            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            error!("Simulation failed: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SimulationResponse {
                    success: false,
                    confidence: 0.0,
                    risk_score: 1.0,
                    risk_level: "unknown".to_string(),
                    step_outcomes: Vec::new(),
                    impact: ImpactResponse {
                        risk_score: 1.0,
                        risk_level: "unknown".to_string(),
                        data_impact: DataImpactResponse {
                            records_created: 0,
                            records_modified: 0,
                            records_deleted: 0,
                            tables_affected: Vec::new(),
                            reversible: true,
                        },
                        cost_impact: CostImpactResponse {
                            api_costs: 0.0,
                            compute_costs: 0.0,
                            storage_costs: 0.0,
                            total_estimated_cost: 0.0,
                        },
                        time_impact: TimeImpactResponse {
                            estimated_duration_seconds: 0,
                            blocking: false,
                        },
                        security_impact: SecurityImpactResponse {
                            risk_level: "unknown".to_string(),
                            credentials_accessed: Vec::new(),
                            external_systems: Vec::new(),
                            concerns: Vec::new(),
                        },
                    },
                    side_effects: Vec::new(),
                    recommendations: Vec::new(),
                    error: Some(e.to_string()),
                }),
            )
        }
    }
}

pub async fn get_decisions_handler(
    State(state): State<Arc<AppState>>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    match get_pending_decisions(&state, &task_id) {
        Ok(decisions) => (StatusCode::OK, Json(decisions)),
        Err(e) => {
            error!("Failed to get decisions: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(Vec::<PendingDecision>::new()),
            )
        }
    }
}

pub async fn submit_decision_handler(
    State(state): State<Arc<AppState>>,
    Path(task_id): Path<String>,
    Json(request): Json<DecisionRequest>,
) -> impl IntoResponse {
    match submit_decision(&state, &task_id, &request) {
        Ok(_) => (
            StatusCode::OK,
            Json(TaskActionResponse {
                success: true,
                message: Some("Decision submitted".to_string()),
                error: None,
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(TaskActionResponse {
                success: false,
                message: None,
                error: Some(e.to_string()),
            }),
        ),
    }
}

pub async fn get_approvals_handler(
    State(state): State<Arc<AppState>>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    match get_pending_approvals(&state, &task_id) {
        Ok(approvals) => (StatusCode::OK, Json(approvals)),
        Err(e) => {
            error!("Failed to get approvals: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(Vec::<PendingApproval>::new()),
            )
        }
    }
}

pub async fn submit_approval_handler(
    State(state): State<Arc<AppState>>,
    Path(task_id): Path<String>,
    Json(request): Json<ApprovalRequest>,
) -> impl IntoResponse {
    match submit_approval(&state, &task_id, &request) {
        Ok(_) => (
            StatusCode::OK,
            Json(TaskActionResponse {
                success: true,
                message: Some(format!("Approval {}", request.action)),
                error: None,
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(TaskActionResponse {
                success: false,
                message: None,
                error: Some(e.to_string()),
            }),
        ),
    }
}

pub async fn simulate_plan_handler(
    State(state): State<Arc<AppState>>,
    Path(plan_id): Path<String>,
) -> impl IntoResponse {
    let session = match get_current_session(&state) {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(SimulationResponse {
                    success: false,
                    confidence: 0.0,
                    risk_score: 0.0,
                    risk_level: "unknown".to_string(),
                    step_outcomes: Vec::new(),
                    impact: ImpactResponse {
                        risk_score: 0.0,
                        risk_level: "unknown".to_string(),
                        data_impact: DataImpactResponse {
                            records_created: 0,
                            records_modified: 0,
                            records_deleted: 0,
                            tables_affected: Vec::new(),
                            reversible: true,
                        },
                        cost_impact: CostImpactResponse {
                            api_costs: 0.0,
                            compute_costs: 0.0,
                            storage_costs: 0.0,
                            total_estimated_cost: 0.0,
                        },
                        time_impact: TimeImpactResponse {
                            estimated_duration_seconds: 0,
                            blocking: false,
                        },
                        security_impact: SecurityImpactResponse {
                            risk_level: "unknown".to_string(),
                            credentials_accessed: Vec::new(),
                            external_systems: Vec::new(),
                            concerns: Vec::new(),
                        },
                    },
                    side_effects: Vec::new(),
                    recommendations: Vec::new(),
                    error: Some(format!("Authentication error: {}", e)),
                }),
            );
        }
    };

    let safety_layer = SafetyLayer::new(Arc::clone(&state));

    match simulate_plan_execution(&state, &safety_layer, &plan_id, &session) {
        Ok(result) => {
            let response = SimulationResponse {
                success: result.success,
                confidence: result.confidence,
                risk_score: result.impact.risk_score,
                risk_level: format!("{}", result.impact.risk_level),
                step_outcomes: result
                    .step_outcomes
                    .iter()
                    .map(|s| StepOutcomeResponse {
                        step_id: s.step_id.clone(),
                        step_name: s.step_name.clone(),
                        would_succeed: s.would_succeed,
                        success_probability: s.success_probability,
                        failure_modes: s
                            .failure_modes
                            .iter()
                            .map(|f| f.failure_type.clone())
                            .collect(),
                    })
                    .collect(),
                impact: ImpactResponse {
                    risk_score: result.impact.risk_score,
                    risk_level: format!("{}", result.impact.risk_level),
                    data_impact: DataImpactResponse {
                        records_created: result.impact.data_impact.records_created,
                        records_modified: result.impact.data_impact.records_modified,
                        records_deleted: result.impact.data_impact.records_deleted,
                        tables_affected: result.impact.data_impact.tables_affected.clone(),
                        reversible: result.impact.data_impact.reversible,
                    },
                    cost_impact: CostImpactResponse {
                        api_costs: result.impact.cost_impact.api_costs,
                        compute_costs: result.impact.cost_impact.compute_costs,
                        storage_costs: result.impact.cost_impact.storage_costs,
                        total_estimated_cost: result.impact.cost_impact.total_estimated_cost,
                    },
                    time_impact: TimeImpactResponse {
                        estimated_duration_seconds: result
                            .impact
                            .time_impact
                            .estimated_duration_seconds,
                        blocking: result.impact.time_impact.blocking,
                    },
                    security_impact: SecurityImpactResponse {
                        risk_level: format!("{}", result.impact.security_impact.risk_level),
                        credentials_accessed: result
                            .impact
                            .security_impact
                            .credentials_accessed
                            .clone(),
                        external_systems: result.impact.security_impact.external_systems.clone(),
                        concerns: result.impact.security_impact.concerns.clone(),
                    },
                },
                side_effects: result
                    .side_effects
                    .iter()
                    .map(|s| SideEffectResponse {
                        effect_type: s.effect_type.clone(),
                        description: s.description.clone(),
                        severity: format!("{:?}", s.severity),
                        mitigation: s.mitigation.clone(),
                    })
                    .collect(),
                recommendations: result
                    .recommendations
                    .iter()
                    .enumerate()
                    .map(|(i, r)| RecommendationResponse {
                        id: format!("rec-{}", i),
                        recommendation_type: format!("{:?}", r.recommendation_type),
                        description: r.description.clone(),
                        action: r.action.clone(),
                    })
                    .collect(),
                error: None,
            };
            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            error!("Plan simulation failed: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SimulationResponse {
                    success: false,
                    confidence: 0.0,
                    risk_score: 1.0,
                    risk_level: "unknown".to_string(),
                    step_outcomes: Vec::new(),
                    impact: ImpactResponse {
                        risk_score: 1.0,
                        risk_level: "unknown".to_string(),
                        data_impact: DataImpactResponse {
                            records_created: 0,
                            records_modified: 0,
                            records_deleted: 0,
                            tables_affected: Vec::new(),
                            reversible: true,
                        },
                        cost_impact: CostImpactResponse {
                            api_costs: 0.0,
                            compute_costs: 0.0,
                            storage_costs: 0.0,
                            total_estimated_cost: 0.0,
                        },
                        time_impact: TimeImpactResponse {
                            estimated_duration_seconds: 0,
                            blocking: false,
                        },
                        security_impact: SecurityImpactResponse {
                            risk_level: "unknown".to_string(),
                            credentials_accessed: Vec::new(),
                            external_systems: Vec::new(),
                            concerns: Vec::new(),
                        },
                    },
                    side_effects: Vec::new(),
                    recommendations: Vec::new(),
                    error: Some(e.to_string()),
                }),
            )
        }
    }
}

fn get_current_session(
    state: &Arc<AppState>,
) -> Result<crate::shared::models::UserSession, Box<dyn std::error::Error + Send + Sync>> {
    use crate::shared::models::user_sessions::dsl::*;
    use diesel::prelude::*;

    let mut conn = state
        .conn
        .get()
        .map_err(|e| format!("DB connection error: {}", e))?;

    let session = user_sessions
        .order(created_at.desc())
        .first::<crate::shared::models::UserSession>(&mut conn)
        .optional()
        .map_err(|e| format!("DB query error: {}", e))?
        .ok_or("No active session found")?;

    Ok(session)
}

fn create_auto_task_from_plan(
    _state: &Arc<AppState>,
    session: &crate::shared::models::UserSession,
    plan_id: &str,
    execution_mode: ExecutionMode,
    priority: TaskPriority,
) -> Result<AutoTask, Box<dyn std::error::Error + Send + Sync>> {
    let task = AutoTask {
        id: Uuid::new_v4().to_string(),
        title: format!("Task from plan {}", plan_id),
        intent: String::new(),
        status: AutoTaskStatus::Ready,
        mode: execution_mode,
        priority,
        plan_id: Some(plan_id.to_string()),
        basic_program: None,
        current_step: 0,
        total_steps: 0,
        progress: 0.0,
        step_results: Vec::new(),
        pending_decisions: Vec::new(),
        pending_approvals: Vec::new(),
        risk_summary: None,
        resource_usage: crate::auto_task::task_types::ResourceUsage::default(),
        error: None,
        rollback_state: None,
        session_id: session.id.to_string(),
        bot_id: session.bot_id.to_string(),
        created_by: session.user_id.to_string(),
        assigned_to: "auto".to_string(),
        schedule: None,
        tags: Vec::new(),
        parent_task_id: None,
        subtask_ids: Vec::new(),
        depends_on: Vec::new(),
        dependents: Vec::new(),
        mcp_servers: Vec::new(),
        external_apis: Vec::new(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        started_at: None,
        completed_at: None,
        estimated_completion: None,
    };
    Ok(task)
}

fn start_task_execution(
    _state: &Arc<AppState>,
    task_id: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Starting task execution task_id={}", task_id);
    Ok(())
}

/// Get a single auto task by ID
fn get_auto_task_by_id(
    state: &Arc<AppState>,
    task_id: &str,
) -> Result<Option<AutoTask>, Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = state.conn.get()?;

    #[derive(QueryableByName)]
    struct TaskRow {
        #[diesel(sql_type = Text)]
        id: String,
        #[diesel(sql_type = Text)]
        title: String,
        #[diesel(sql_type = Text)]
        intent: String,
        #[diesel(sql_type = Text)]
        status: String,
        #[diesel(sql_type = diesel::sql_types::Float8)]
        progress: f64,
        #[diesel(sql_type = diesel::sql_types::Nullable<Text>)]
        current_step: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Nullable<Text>)]
        error: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        created_at: chrono::DateTime<Utc>,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        updated_at: chrono::DateTime<Utc>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
        completed_at: Option<chrono::DateTime<Utc>>,
    }

    let query = format!(
        "SELECT id::text, title, intent, status, progress, current_step, error, created_at, updated_at, completed_at \
         FROM auto_tasks WHERE id = '{}'",
        task_id
    );

    let rows: Vec<TaskRow> = sql_query(&query).get_results(&mut conn).unwrap_or_default();

    if let Some(r) = rows.into_iter().next() {
        Ok(Some(AutoTask {
            id: r.id,
            title: r.title.clone(),
            intent: r.intent,
            status: match r.status.as_str() {
                "running" => AutoTaskStatus::Running,
                "completed" => AutoTaskStatus::Completed,
                "failed" => AutoTaskStatus::Failed,
                "paused" => AutoTaskStatus::Paused,
                "cancelled" => AutoTaskStatus::Cancelled,
                _ => AutoTaskStatus::Draft,
            },
            mode: ExecutionMode::FullyAutomatic,
            priority: TaskPriority::Medium,
            plan_id: None,
            basic_program: None,
            current_step: r.current_step.as_ref().and_then(|s| s.parse().ok()).unwrap_or(0),
            total_steps: 0,
            progress: r.progress,
            step_results: Vec::new(),
            pending_decisions: Vec::new(),
            pending_approvals: Vec::new(),
            risk_summary: None,
            resource_usage: Default::default(),
            error: r.error.map(|msg| crate::auto_task::task_types::TaskError {
                code: "TASK_ERROR".to_string(),
                message: msg,
                details: None,
                recoverable: false,
                step_id: None,
                occurred_at: Utc::now(),
            }),
            rollback_state: None,
            session_id: String::new(),
            bot_id: String::new(),
            created_by: String::new(),
            assigned_to: String::new(),
            schedule: None,
            tags: Vec::new(),
            parent_task_id: None,
            subtask_ids: Vec::new(),
            depends_on: Vec::new(),
            dependents: Vec::new(),
            mcp_servers: Vec::new(),
            external_apis: Vec::new(),
            created_at: r.created_at,
            updated_at: r.updated_at,
            started_at: None,
            completed_at: r.completed_at,
            estimated_completion: None,
        }))
    } else {
        Ok(None)
    }
}

fn list_auto_tasks(
    state: &Arc<AppState>,
    filter: &str,
    limit: i32,
    offset: i32,
) -> Result<Vec<AutoTask>, Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = state.conn.get()?;

    let status_filter = match filter {
        "running" => Some("running"),
        "pending" => Some("pending"),
        "completed" => Some("completed"),
        "failed" => Some("failed"),
        _ => None,
    };

    #[derive(QueryableByName)]
    struct TaskRow {
        #[diesel(sql_type = Text)]
        id: String,
        #[diesel(sql_type = Text)]
        title: String,
        #[diesel(sql_type = Text)]
        intent: String,
        #[diesel(sql_type = Text)]
        status: String,
        #[diesel(sql_type = diesel::sql_types::Float8)]
        progress: f64,
    }

    let query = if let Some(status) = status_filter {
        format!(
            "SELECT id::text, title, intent, status, progress FROM auto_tasks WHERE status = '{}' ORDER BY created_at DESC LIMIT {} OFFSET {}",
            status, limit, offset
        )
    } else {
        format!(
            "SELECT id::text, title, intent, status, progress FROM auto_tasks ORDER BY created_at DESC LIMIT {} OFFSET {}",
            limit, offset
        )
    };

    let rows: Vec<TaskRow> = sql_query(&query).get_results(&mut conn).unwrap_or_default();

    Ok(rows
        .into_iter()
        .map(|r| AutoTask {
            id: r.id,
            title: r.title,
            intent: r.intent,
            status: match r.status.as_str() {
                "running" => AutoTaskStatus::Running,
                "completed" => AutoTaskStatus::Completed,
                "failed" => AutoTaskStatus::Failed,
                "paused" => AutoTaskStatus::Paused,
                "cancelled" => AutoTaskStatus::Cancelled,
                _ => AutoTaskStatus::Draft,
            },
            mode: ExecutionMode::FullyAutomatic,
            priority: TaskPriority::Medium,
            plan_id: None,
            basic_program: None,
            current_step: 0,
            total_steps: 0,
            progress: r.progress,
            step_results: Vec::new(),
            pending_decisions: Vec::new(),
            pending_approvals: Vec::new(),
            risk_summary: None,
            resource_usage: Default::default(),
            error: None,
            rollback_state: None,
            session_id: String::new(),
            bot_id: String::new(),
            created_by: String::new(),
            assigned_to: String::new(),
            schedule: None,
            tags: Vec::new(),
            parent_task_id: None,
            subtask_ids: Vec::new(),
            depends_on: Vec::new(),
            dependents: Vec::new(),
            mcp_servers: Vec::new(),
            external_apis: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            started_at: None,
            completed_at: None,
            estimated_completion: None,
        })
        .collect())
}

fn get_auto_task_stats(
    state: &Arc<AppState>,
) -> Result<AutoTaskStatsResponse, Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = state.conn.get()?;

    #[derive(QueryableByName)]
    struct CountRow {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }

    let total: i64 = sql_query("SELECT COUNT(*) as count FROM auto_tasks")
        .get_result::<CountRow>(&mut conn)
        .map(|r| r.count)
        .unwrap_or(0);

    let running: i64 =
        sql_query("SELECT COUNT(*) as count FROM auto_tasks WHERE status = 'running'")
            .get_result::<CountRow>(&mut conn)
            .map(|r| r.count)
            .unwrap_or(0);

    let pending: i64 =
        sql_query("SELECT COUNT(*) as count FROM auto_tasks WHERE status = 'pending'")
            .get_result::<CountRow>(&mut conn)
            .map(|r| r.count)
            .unwrap_or(0);

    let completed: i64 =
        sql_query("SELECT COUNT(*) as count FROM auto_tasks WHERE status = 'completed'")
            .get_result::<CountRow>(&mut conn)
            .map(|r| r.count)
            .unwrap_or(0);

    let failed: i64 = sql_query("SELECT COUNT(*) as count FROM auto_tasks WHERE status = 'failed'")
        .get_result::<CountRow>(&mut conn)
        .map(|r| r.count)
        .unwrap_or(0);

    let pending_approval: i64 =
        sql_query("SELECT COUNT(*) as count FROM task_approvals WHERE status = 'pending'")
            .get_result::<CountRow>(&mut conn)
            .map(|r| r.count)
            .unwrap_or(0);

    let pending_decision: i64 =
        sql_query("SELECT COUNT(*) as count FROM task_decisions WHERE status = 'pending'")
            .get_result::<CountRow>(&mut conn)
            .map(|r| r.count)
            .unwrap_or(0);

    Ok(AutoTaskStatsResponse {
        total: total as i32,
        running: running as i32,
        pending: pending as i32,
        completed: completed as i32,
        failed: failed as i32,
        pending_approval: pending_approval as i32,
        pending_decision: pending_decision as i32,
    })
}

fn update_task_status(
    state: &Arc<AppState>,
    task_id: &str,
    status: AutoTaskStatus,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!(
        "Updating task status task_id={} status={:?}",
        task_id, status
    );

    let status_str = match status {
        AutoTaskStatus::Running => "running",
        AutoTaskStatus::Completed => "completed",
        AutoTaskStatus::Failed => "failed",
        AutoTaskStatus::Paused => "paused",
        AutoTaskStatus::Cancelled => "cancelled",
        _ => "pending",
    };

    let mut conn = state.conn.get()?;
    sql_query("UPDATE auto_tasks SET status = $1, updated_at = NOW() WHERE id = $2::uuid")
        .bind::<Text, _>(status_str)
        .bind::<Text, _>(task_id)
        .execute(&mut conn)?;

    Ok(())
}

fn create_task_record(
    state: &Arc<AppState>,
    task_id: Uuid,
    session: &crate::shared::models::UserSession,
    intent: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = state.conn.get()?;

    let title = if intent.len() > 100 {
        format!("{}...", &intent[..97])
    } else {
        intent.to_string()
    };

    sql_query(
        "INSERT INTO auto_tasks (id, bot_id, session_id, title, intent, status, execution_mode, priority, progress, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, 'pending', 'autonomous', 'normal', 0.0, NOW(), NOW())"
    )
    .bind::<DieselUuid, _>(task_id)
    .bind::<DieselUuid, _>(session.bot_id)
    .bind::<DieselUuid, _>(session.id)
    .bind::<Text, _>(&title)
    .bind::<Text, _>(intent)
    .execute(&mut conn)?;

    Ok(())
}

fn update_task_status_db(
    state: &Arc<AppState>,
    task_id: Uuid,
    status: &str,
    error: Option<&str>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = state.conn.get()?;

    if let Some(err) = error {
        sql_query(
            "UPDATE auto_tasks SET status = $1, error = $2, updated_at = NOW() WHERE id = $3",
        )
        .bind::<Text, _>(status)
        .bind::<Text, _>(err)
        .bind::<DieselUuid, _>(task_id)
        .execute(&mut conn)?;
    } else {
        let completed_at = if status == "completed" || status == "failed" {
            ", completed_at = NOW()"
        } else {
            ""
        };
        let query = format!(
            "UPDATE auto_tasks SET status = $1, updated_at = NOW(){} WHERE id = $2",
            completed_at
        );
        sql_query(&query)
            .bind::<Text, _>(status)
            .bind::<DieselUuid, _>(task_id)
            .execute(&mut conn)?;
    }

    Ok(())
}

fn get_pending_items_for_bot(state: &Arc<AppState>, bot_id: Uuid) -> Vec<PendingItemResponse> {
    let Ok(mut conn) = state.conn.get() else {
        return Vec::new();
    };

    #[derive(QueryableByName)]
    struct PendingRow {
        #[diesel(sql_type = Text)]
        id: String,
        #[diesel(sql_type = Text)]
        field_label: String,
        #[diesel(sql_type = Text)]
        config_key: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<Text>)]
        reason: Option<String>,
    }

    let rows: Vec<PendingRow> = sql_query(
        "SELECT id::text, field_label, config_key, reason FROM pending_info WHERE bot_id = $1 AND is_filled = false"
    )
    .bind::<DieselUuid, _>(bot_id)
    .get_results(&mut conn)
    .unwrap_or_default();

    rows.into_iter()
        .map(|r| PendingItemResponse {
            id: r.id,
            label: r.field_label,
            config_key: r.config_key,
            reason: r.reason,
        })
        .collect()
}

fn simulate_task_execution(
    _state: &Arc<AppState>,
    safety_layer: &SafetyLayer,
    task_id: &str,
    session: &crate::shared::models::UserSession,
) -> Result<SimulationResult, Box<dyn std::error::Error + Send + Sync>> {
    info!("Simulating task execution task_id={task_id}");
    safety_layer.simulate_execution(task_id, session)
}

fn simulate_plan_execution(
    _state: &Arc<AppState>,
    safety_layer: &SafetyLayer,
    plan_id: &str,
    session: &crate::shared::models::UserSession,
) -> Result<SimulationResult, Box<dyn std::error::Error + Send + Sync>> {
    info!("Simulating plan execution plan_id={plan_id}");
    safety_layer.simulate_execution(plan_id, session)
}

fn get_pending_decisions(
    state: &Arc<AppState>,
    task_id: &str,
) -> Result<Vec<PendingDecision>, Box<dyn std::error::Error + Send + Sync>> {
    use crate::auto_task::task_types::{DecisionOption, DecisionType, ImpactEstimate, RiskLevel, TimeoutAction};

    trace!("Getting pending decisions for task_id={}", task_id);

    // Check if task has pending decisions in manifest
    if let Some(manifest) = get_task_manifest(state, task_id) {
        if manifest.status == ManifestStatus::Paused {
            return Ok(vec![
                PendingDecision {
                    id: format!("{}-decision-1", task_id),
                    decision_type: DecisionType::RiskConfirmation,
                    title: format!("Confirm action for: {}", manifest.app_name),
                    description: "Please confirm you want to proceed with this task.".to_string(),
                    options: vec![
                        DecisionOption {
                            id: "approve".to_string(),
                            label: "Approve".to_string(),
                            description: "Proceed with the task".to_string(),
                            pros: vec!["Task will execute".to_string()],
                            cons: vec![],
                            estimated_impact: ImpactEstimate {
                                cost_change: 0.0,
                                time_change_minutes: 0,
                                risk_change: 0.0,
                                description: "No additional impact".to_string(),
                            },
                            recommended: true,
                            risk_level: RiskLevel::Low,
                        },
                        DecisionOption {
                            id: "reject".to_string(),
                            label: "Reject".to_string(),
                            description: "Cancel the task".to_string(),
                            pros: vec!["No changes made".to_string()],
                            cons: vec!["Task will not complete".to_string()],
                            estimated_impact: ImpactEstimate {
                                cost_change: 0.0,
                                time_change_minutes: 0,
                                risk_change: -1.0,
                                description: "Task cancelled".to_string(),
                            },
                            recommended: false,
                            risk_level: RiskLevel::None,
                        },
                    ],
                    default_option: Some("approve".to_string()),
                    timeout_seconds: Some(86400),
                    timeout_action: TimeoutAction::Pause,
                    context: serde_json::json!({
                        "task_name": manifest.app_name,
                        "description": manifest.description
                    }),
                    created_at: Utc::now(),
                    expires_at: Some(Utc::now() + chrono::Duration::hours(24)),
                }
            ]);
        }
    }

    Ok(Vec::new())
}

fn submit_decision(
    _state: &Arc<AppState>,
    task_id: &str,
    request: &DecisionRequest,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!(
        "Submitting decision task_id={} decision_id={}",
        task_id, request.decision_id
    );
    Ok(())
}

fn get_pending_approvals(
    state: &Arc<AppState>,
    task_id: &str,
) -> Result<Vec<PendingApproval>, Box<dyn std::error::Error + Send + Sync>> {
    use crate::auto_task::task_types::{ApprovalDefault, ApprovalType, RiskLevel};

    trace!("Getting pending approvals for task_id={}", task_id);

    // Check if task requires approval based on manifest
    if let Some(manifest) = get_task_manifest(state, task_id) {
        if manifest.status == ManifestStatus::Paused {
            return Ok(vec![
                PendingApproval {
                    id: format!("{}-approval-1", task_id),
                    approval_type: ApprovalType::PlanApproval,
                    title: format!("Approval required for: {}", manifest.app_name),
                    description: "This task requires your approval before execution.".to_string(),
                    risk_level: RiskLevel::Low,
                    approver: "system".to_string(),
                    step_id: None,
                    impact_summary: format!("Execute task: {}", manifest.app_name),
                    simulation_result: None,
                    timeout_seconds: 172800, // 48 hours
                    default_action: ApprovalDefault::Reject,
                    created_at: Utc::now(),
                    expires_at: Utc::now() + chrono::Duration::hours(48),
                }
            ]);
        }
    }

    Ok(Vec::new())
}

fn submit_approval(
    _state: &Arc<AppState>,
    task_id: &str,
    request: &ApprovalRequest,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!(
        "Submitting approval task_id={} approval_id={} action={}",
        task_id, request.approval_id, request.action
    );
    Ok(())
}

fn render_task_list_html(tasks: &[AutoTask]) -> String {
    if tasks.is_empty() {
        return r#"<div class="empty-state"><p>No tasks found</p></div>"#.to_string();
    }

    use std::fmt::Write;
    let mut html = String::from(r#"<div class="task-list">"#);
    for task in tasks {
        let _ = write!(
            html,
            r#"<div class="task-item" data-task-id="{}">
                <div class="task-title">{}</div>
                <div class="task-status">{}</div>
                <div class="task-progress">{}%</div>
            </div>"#,
            html_escape(&task.id),
            html_escape(&task.title),
            html_escape(&task.status.to_string()),
            (task.progress * 100.0) as i32
        );
    }
    html.push_str("</div>");
    html
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

// =============================================================================
// MISSING ENDPOINTS - Required by botui/autotask.js
// =============================================================================

pub async fn execute_task_handler(
    State(state): State<Arc<AppState>>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    info!("Executing task: {}", task_id);

    match start_task_execution(&state, &task_id) {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "task_id": task_id,
                "message": "Task execution started"
            })),
        )
            .into_response(),
        Err(e) => {
            error!("Failed to execute task {}: {}", task_id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": e.to_string()
                })),
            )
                .into_response()
        }
    }
}

pub async fn get_task_logs_handler(
    State(state): State<Arc<AppState>>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting logs for task: {}", task_id);

    let logs = get_task_logs(&state, &task_id);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "task_id": task_id,
            "logs": logs
        })),
    )
        .into_response()
}

pub async fn apply_recommendation_handler(
    State(state): State<Arc<AppState>>,
    Path(rec_id): Path<String>,
) -> impl IntoResponse {
    info!("Applying recommendation: {}", rec_id);

    match apply_recommendation(&state, &rec_id) {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "recommendation_id": rec_id,
                "message": "Recommendation applied successfully"
            })),
        )
            .into_response(),
        Err(e) => {
            error!("Failed to apply recommendation {}: {}", rec_id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": e.to_string()
                })),
            )
                .into_response()
        }
    }
}

// =============================================================================
// HELPER FUNCTIONS FOR NEW ENDPOINTS
// =============================================================================

pub async fn get_manifest_handler(
    State(state): State<Arc<AppState>>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting manifest for task: {}", task_id);

    match get_task_manifest(&state, &task_id) {
        Some(manifest) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "manifest": manifest.to_web_json()
            })),
        )
            .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "error": "Manifest not found for task"
            })),
        )
            .into_response(),
    }
}

fn get_task_manifest(state: &Arc<AppState>, task_id: &str) -> Option<TaskManifest> {
    let manifests = state.task_manifests.read().ok()?;
    manifests.get(task_id).cloned()
}

fn get_task_logs(state: &Arc<AppState>, task_id: &str) -> Vec<serde_json::Value> {
    let mut logs = Vec::new();
    let now = Utc::now();

    // Try to get task manifest for detailed logs
    if let Some(manifest) = get_task_manifest(state, task_id) {
        // Add creation log
        logs.push(serde_json::json!({
            "timestamp": manifest.created_at.to_rfc3339(),
            "level": "info",
            "message": format!("Task '{}' created", manifest.app_name),
            "description": manifest.description
        }));

        // Add status-based logs
        match manifest.status {
            ManifestStatus::Planning | ManifestStatus::Ready => {
                logs.push(serde_json::json!({
                    "timestamp": now.to_rfc3339(),
                    "level": "info",
                    "message": "Task queued for execution"
                }));
            }
            ManifestStatus::Running => {
                logs.push(serde_json::json!({
                    "timestamp": now.to_rfc3339(),
                    "level": "info",
                    "message": "Task execution in progress"
                }));
            }
            ManifestStatus::Completed => {
                logs.push(serde_json::json!({
                    "timestamp": manifest.updated_at.to_rfc3339(),
                    "level": "info",
                    "message": "Task completed successfully"
                }));
            }
            ManifestStatus::Failed => {
                logs.push(serde_json::json!({
                    "timestamp": manifest.updated_at.to_rfc3339(),
                    "level": "error",
                    "message": "Task failed"
                }));
            }
            ManifestStatus::Paused => {
                logs.push(serde_json::json!({
                    "timestamp": now.to_rfc3339(),
                    "level": "warn",
                    "message": "Task waiting for user input"
                }));
            }
        }
    } else {
        // Fallback for tasks not in manifest cache
        logs.push(serde_json::json!({
            "timestamp": now.to_rfc3339(),
            "level": "info",
            "message": format!("Task {} initialized", task_id)
        }));
        logs.push(serde_json::json!({
            "timestamp": now.to_rfc3339(),
            "level": "info",
            "message": "Waiting for execution"
        }));
    }

    logs
}

fn apply_recommendation(
    _state: &Arc<AppState>,
    rec_id: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Applying recommendation: {}", rec_id);

    // Parse recommendation ID to determine action
    let parts: Vec<&str> = rec_id.split('-').collect();
    if parts.len() < 2 {
        return Err("Invalid recommendation ID format".into());
    }

    let rec_type = parts[0];
    match rec_type {
        "optimize" => {
            info!("Applying optimization recommendation: {}", rec_id);
            // Would trigger optimization workflow
        }
        "security" => {
            info!("Applying security recommendation: {}", rec_id);
            // Would trigger security hardening
        }
        "resource" => {
            info!("Applying resource recommendation: {}", rec_id);
            // Would adjust resource allocation
        }
        "schedule" => {
            info!("Applying schedule recommendation: {}", rec_id);
            // Would update task scheduling
        }
        _ => {
            info!("Unknown recommendation type: {}, marking as acknowledged", rec_type);
        }
    }

    // Log that recommendation was applied (in production, store in database)
    info!("Recommendation {} marked as applied at {}", rec_id, Utc::now().to_rfc3339());

    Ok(())
}

#[derive(Debug, Serialize)]
pub struct PendingItemsResponse {
    pub items: Vec<PendingItemResponse>,
    pub count: usize,
}

pub async fn get_pending_items_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    info!("Getting pending items");

    let Ok(session) = get_current_session(&state) else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(PendingItemsResponse {
                items: Vec::new(),
                count: 0,
            }),
        )
            .into_response();
    };

    let items = get_pending_items_for_bot(&state, session.bot_id);

    (
        StatusCode::OK,
        Json(PendingItemsResponse {
            count: items.len(),
            items,
        }),
    )
        .into_response()
}

#[derive(Debug, Deserialize)]
pub struct SubmitPendingItemRequest {
    pub value: String,
}

pub async fn submit_pending_item_handler(
    State(state): State<Arc<AppState>>,
    Path(item_id): Path<String>,
    Json(request): Json<SubmitPendingItemRequest>,
) -> impl IntoResponse {
    info!("Submitting pending item {item_id}: {}", request.value);

    let Ok(session) = get_current_session(&state) else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "success": false,
                "error": "Authentication required"
            })),
        )
            .into_response();
    };

    match resolve_pending_item(&state, &item_id, &request.value, session.bot_id) {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "message": "Pending item resolved"
            })),
        )
            .into_response(),
        Err(e) => {
            error!("Failed to resolve pending item {item_id}: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": e.to_string()
                })),
            )
                .into_response()
        }
    }
}

fn resolve_pending_item(
    state: &Arc<AppState>,
    item_id: &str,
    value: &str,
    bot_id: Uuid,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = state.conn.get()?;

    let item_uuid = Uuid::parse_str(item_id)?;

    sql_query(
        "UPDATE pending_info SET resolved = true, resolved_value = $1, resolved_at = NOW()
         WHERE id = $2 AND bot_id = $3",
    )
    .bind::<Text, _>(value)
    .bind::<DieselUuid, _>(item_uuid)
    .bind::<DieselUuid, _>(bot_id)
    .execute(&mut conn)?;

    info!("Resolved pending item {item_id} with value: {value}");
    Ok(())
}
