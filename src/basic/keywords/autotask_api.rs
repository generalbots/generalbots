//! Auto Task API Handlers
//!
//! This module provides the HTTP API endpoints for the Auto Task system,
//! enabling the UI to interact with the Intent Compiler, execution engine,
//! safety layer, and MCP client.

use crate::basic::keywords::auto_task::{
    AutoTask, AutoTaskStatus, ExecutionMode, PendingApproval, PendingDecision, TaskPriority,
};
use crate::basic::keywords::intent_compiler::{CompiledIntent, IntentCompiler};
use crate::basic::keywords::mcp_client::McpClient;
use crate::basic::keywords::safety_layer::{SafetyLayer, SimulationResult};
use crate::shared::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use log::{error, info, trace};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

// =============================================================================
// REQUEST/RESPONSE TYPES
// =============================================================================

/// Request to compile an intent into an executable plan
#[derive(Debug, Deserialize)]
pub struct CompileIntentRequest {
    pub intent: String,
    pub execution_mode: Option<String>,
    pub priority: Option<String>,
}

/// Response from intent compilation
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

/// Request to execute a compiled plan
#[derive(Debug, Deserialize)]
pub struct ExecutePlanRequest {
    pub plan_id: String,
    pub execution_mode: Option<String>,
    pub priority: Option<String>,
}

/// Response from plan execution
#[derive(Debug, Serialize)]
pub struct ExecutePlanResponse {
    pub success: bool,
    pub task_id: Option<String>,
    pub status: Option<String>,
    pub error: Option<String>,
}

/// Query parameters for listing tasks
#[derive(Debug, Deserialize)]
pub struct ListTasksQuery {
    pub filter: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

/// Auto task stats response
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

/// Task action response
#[derive(Debug, Serialize)]
pub struct TaskActionResponse {
    pub success: bool,
    pub message: Option<String>,
    pub error: Option<String>,
}

/// Decision submission request
#[derive(Debug, Deserialize)]
pub struct DecisionRequest {
    pub decision_id: String,
    pub option_id: Option<String>,
    pub skip: Option<bool>,
}

/// Approval action request
#[derive(Debug, Deserialize)]
pub struct ApprovalRequest {
    pub approval_id: String,
    pub action: String, // "approve", "reject", "defer"
    pub comment: Option<String>,
}

/// Simulation response
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

// =============================================================================
// API HANDLERS
// =============================================================================

/// POST /api/autotask/compile - Compile an intent into an execution plan
pub async fn compile_intent_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CompileIntentRequest>,
) -> impl IntoResponse {
    info!("Compiling intent: {}", &request.intent[..request.intent.len().min(100)]);

    // Get session from state (in real implementation, extract from auth)
    let session = match get_current_session(&state).await {
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

    // Create intent compiler
    let compiler = IntentCompiler::new(Arc::clone(&state));

    // Compile the intent
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
                    llm_tokens: 0, // TODO: Track LLM tokens
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

/// POST /api/autotask/execute - Execute a compiled plan
pub async fn execute_plan_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ExecutePlanRequest>,
) -> impl IntoResponse {
    info!("Executing plan: {}", request.plan_id);

    let session = match get_current_session(&state).await {
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

    // Parse execution mode
    let execution_mode = match request.execution_mode.as_deref() {
        Some("fully-automatic") => ExecutionMode::FullyAutomatic,
        Some("supervised") => ExecutionMode::Supervised,
        Some("manual") => ExecutionMode::Manual,
        Some("dry-run") => ExecutionMode::DryRun,
        _ => ExecutionMode::SemiAutomatic,
    };

    // Parse priority
    let priority = match request.priority.as_deref() {
        Some("critical") => TaskPriority::Critical,
        Some("high") => TaskPriority::High,
        Some("low") => TaskPriority::Low,
        Some("background") => TaskPriority::Background,
        _ => TaskPriority::Medium,
    };

    // Create the auto task from the compiled plan
    match create_auto_task_from_plan(&state, &session, &request.plan_id, execution_mode, priority).await {
        Ok(task) => {
            // Start execution
            match start_task_execution(&state, &task.id).await {
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
            }
        }
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

/// GET /api/autotask/list - List auto tasks
pub async fn list_tasks_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListTasksQuery>,
) -> impl IntoResponse {
    let filter = query.filter.as_deref().unwrap_or("all");
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    match list_auto_tasks(&state, filter, limit, offset).await {
        Ok(tasks) => {
            // Render as HTML for HTMX
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
            (StatusCode::INTERNAL_SERVER_ERROR, axum::response::Html(html))
        }
    }
}

/// GET /api/autotask/stats - Get auto task statistics
pub async fn get_stats_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match get_auto_task_stats(&state).await {
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

/// POST /api/autotask/:task_id/pause - Pause a task
pub async fn pause_task_handler(
    State(state): State<Arc<AppState>>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    match update_task_status(&state, &task_id, AutoTaskStatus::Paused).await {
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

/// POST /api/autotask/:task_id/resume - Resume a paused task
pub async fn resume_task_handler(
    State(state): State<Arc<AppState>>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    match update_task_status(&state, &task_id, AutoTaskStatus::Running).await {
        Ok(_) => {
            // Restart execution
            let _ = start_task_execution(&state, &task_id).await;
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

/// POST /api/autotask/:task_id/cancel - Cancel a task
pub async fn cancel_task_handler(
    State(state): State<Arc<AppState>>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    match update_task_status(&state, &task_id, AutoTaskStatus::Cancelled).await {
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

/// POST /api/autotask/:task_id/simulate - Simulate task execution
pub async fn simulate_task_handler(
    State(state): State<Arc<AppState>>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    let session = match get_current_session(&state).await {
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

    match simulate_task_execution(&state, &safety_layer, &task_id, &session).await {
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
                        failure_modes: s.failure_modes.iter().map(|f| f.failure_type.clone()).collect(),
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
                        estimated_duration_seconds: result.impact.time_impact.estimated_duration_seconds,
                        blocking: result.impact.time_impact.blocking,
                    },
                    security_impact: SecurityImpactResponse {
                        risk_level: format!("{}", result.impact.security_impact.risk_level),
                        credentials_accessed: result.impact.security_impact.credentials_accessed.clone(),
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

/// GET /api/autotask/:task_id/decisions - Get pending decisions for a task
pub async fn get_decisions_handler(
    State(state): State<Arc<AppState>>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    match get_pending_decisions(&state, &task_id).await {
        Ok(decisions) => (StatusCode::OK, Json(decisions)),
        Err(e) => {
            error!("Failed to get decisions: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(Vec::<PendingDecision>::new()))
        }
    }
}

/// POST /api/autotask/:task_id/decide - Submit a decision
pub async fn submit_decision_handler(
    State(state): State<Arc<AppState>>,
    Path(task_id): Path<String>,
    Json(request): Json<DecisionRequest>,
) -> impl IntoResponse {
    match submit_decision(&state, &task_id, &request).await {
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

/// GET /api/autotask/:task_id/approvals - Get pending approvals for a task
pub async fn get_approvals_handler(
    State(state): State<Arc<AppState>>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    match get_pending_approvals(&state, &task_id).await {
        Ok(approvals) => (StatusCode::OK, Json(approvals)),
        Err(e) => {
            error!("Failed to get approvals: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(Vec::<PendingApproval>::new()))
        }
    }
}

/// POST /api/autotask/:task_id/approve - Submit an approval decision
pub async fn submit_approval_handler(
    State(state): State<Arc<AppState>>,
    Path(task_id): Path<String>,
    Json(request): Json<ApprovalRequest>,
) -> impl IntoResponse {
    match submit_approval(&state, &task_id, &request).await {
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

/// POST /api/autotask/simulate/:plan_id - Simulate a plan before execution
pub async fn simulate_plan_handler(
    State(state): State<Arc<AppState>>,
    Path(plan_id): Path<String>,
) -> impl IntoResponse {
    let session = match get_current_session(&state).await {
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
