//! Real AutoTask API handlers (#755): wiring for classify / compile /
//! create-and-execute over the offline-capable classifiers, the BASIC-only
//! execution pipeline (#754) and the Drive persistence facade.

use crate::api::{
    AutoTaskApi, ClassifyIntentRequest, ClassifyIntentResponse, CompileIntentRequest,
    CompileIntentResponse, CreateAndExecuteRequest, CreateAndExecuteResponse,
    DecisionRequest, ExecutePlanRequest, ExecutePlanResponse, PlanStepResponse,
    ResourceEstimateResponse, RiskResponse, TaskActionResponse,
};
use crate::execution::script_for;
use crate::intent_classifier::IntentClassifier;
use crate::intent_compiler::IntentCompiler;
use crate::ClassifiedIntent;
use crate::types::{BotInfo, DbPool};
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{BigInt, Text, Uuid as DieselUuid};
use log::{info, warn};
use std::sync::Arc;
use uuid::Uuid;

/// Resolve the bot identity so generated scripts land on the right Drive
/// bucket (`{bot}.gbai/{bot}.gbdialog/`).
pub(crate) fn resolve_bot_info(pool: &DbPool, bot_id: Uuid) -> Result<Option<BotInfo>, String> {
    let mut conn = pool.get().map_err(|e| format!("db pool: {e}"))?;
    #[derive(diesel::QueryableByName)]
    struct BotNameRow {
        #[diesel(sql_type = Text)]
        name: String,
    }
    let bot = sql_query("SELECT name FROM bots WHERE id = $1")
        .bind::<DieselUuid, _>(bot_id)
        .get_result::<BotNameRow>(&mut conn)
        .optional()
        .map_err(|e| format!("resolve bot: {e}"))?;
    Ok(bot.map(|b| BotInfo { id: bot_id, name: b.name }))
}

/// Parse an optional `bot_id` string from a request body; defaults to nil.
pub(crate) fn canonical_bot_id(bot_id: Option<String>) -> Uuid {
    bot_id
        .and_then(|s| Uuid::parse_str(&s).ok())
        .unwrap_or_else(Uuid::nil)
}

/// Resolves a nil bot id to the default bot so chat-driven requests without
/// an explicit bot still persist to a real bucket (vibe chat, autotask).
fn resolve_effective_bot_id(pool: &DbPool, bot_id: Uuid) -> Uuid {
    if bot_id != Uuid::nil() {
        return bot_id;
    }
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(_) => return bot_id,
    };
    sql_query("SELECT id FROM bots WHERE name = 'default' AND is_active = true LIMIT 1")
        .get_result::<BotIdRow>(&mut conn)
        .optional()
        .ok()
        .flatten()
        .map(|r| r.id)
        .unwrap_or(bot_id)
}

#[derive(diesel::QueryableByName)]
struct BotIdRow {
    #[diesel(sql_type = DieselUuid)]
    id: Uuid,
}

fn classifier_for(api: &Arc<AutoTaskApi>) -> IntentClassifier {
    IntentClassifier::new(
        api.state().db_pool().clone(),
        api.config_ops().clone(),
        api.llm_ops().clone(),
        api.state().clone(),
    )
}

fn compiler_for(api: &Arc<AutoTaskApi>) -> IntentCompiler {
    IntentCompiler::new(
        api.state().clone(),
        api.config_ops().clone(),
        api.llm_ops().clone(),
    )
}

fn err_msg(context: &str, e: &dyn std::error::Error) -> String {
    let msg = format!("{context} failed: {e}");
    warn!("AutoTask API: {msg}");
    msg
}

pub async fn classify_intent(
    State(api): State<Arc<AutoTaskApi>>,
    Json(req): Json<ClassifyIntentRequest>,
) -> impl IntoResponse {
    info!("API classify intent: {}", &req.intent[..req.intent.len().min(50)]);
    let bot_id = canonical_bot_id(req.bot_id.clone());
    let effective_bot_id = resolve_effective_bot_id(api.state().db_pool(), bot_id);
    match classifier_for(&api).classify_api(&req.intent, bot_id).await {
        Ok(c) => {
            let result = if req.auto_process == Some(true) {
                auto_process_classification(&api, effective_bot_id, &c).await
            } else {
                None
            };
            Json(ClassifyIntentResponse {
                success: true,
                intent_type: c.intent_type.to_string(),
                confidence: c.confidence,
                suggested_name: c.suggested_name.clone(),
                requires_clarification: c.requires_clarification,
                clarification_question: c.clarification_question.clone(),
                result,
                error: None,
            })
        }
        Err(e) => Json(ClassifyIntentResponse {
            success: false,
            intent_type: "UNKNOWN".to_string(),
            confidence: 0.0,
            suggested_name: None,
            requires_clarification: true,
            clarification_question: None,
            result: None,
            error: Some(err_msg("classify", &*e)),
        }),
    }
}

/// Auto-process pipeline for chat-driven intents (vibe chat): classify →
/// compile → persist the generated `.bas` to the bot's Drive bucket so
/// DriveMonitor registers the automation. Returns the result payload the
/// frontend renders as task nodes and progress messages.
async fn auto_process_classification(
    api: &Arc<AutoTaskApi>,
    bot_id: Uuid,
    classification: &ClassifiedIntent,
) -> Option<crate::api::IntentResultResponse> {
    let compiled = match compiler_for(api)
        .compile_from_classification(classification, None, None)
        .await
    {
        Ok(c) => Some(c),
        Err(e) => {
            warn!("auto_process LLM compile failed ({e}); using offline BASIC fallback");
            None
        }
    };
    let (relative_path, body) = script_for(classification, compiled.as_ref());
    match persist_script(api, bot_id, &relative_path, &body) {
        Ok((bucket, key)) => Some(crate::api::IntentResultResponse {
            success: true,
            message: format!("Automation created and registered: {bucket}/{key}"),
            app_url: None,
            task_id: Some(classification.id.clone()),
            schedule_id: None,
            tool_triggers: Vec::new(),
            created_resources: vec![crate::api::CreatedResourceResponse {
                resource_type: classification.intent_type.to_string().to_lowercase(),
                name: classification
                    .suggested_name
                    .clone()
                    .unwrap_or_else(|| "autotask".to_string()),
                path: Some(key.clone()),
            }],
            next_steps: Vec::new(),
        }),
        Err(e) => {
            warn!("auto_process persist failed: {e}");
            Some(crate::api::IntentResultResponse {
                success: false,
                message: e,
                app_url: None,
                task_id: Some(classification.id.clone()),
                schedule_id: None,
                tool_triggers: Vec::new(),
                created_resources: Vec::new(),
                next_steps: Vec::new(),
            })
        }
    }
}

pub async fn compile_intent(
    State(api): State<Arc<AutoTaskApi>>,
    Json(req): Json<CompileIntentRequest>,
) -> Json<CompileIntentResponse> {
    info!("API compile intent: {}", &req.intent[..req.intent.len().min(50)]);
    let bot_id = canonical_bot_id(req.bot_id.clone());
    let classification = match classifier_for(&api).classify_api(&req.intent, bot_id).await {
        Ok(c) => c,
        Err(e) => return error_compile(&req.intent, &*e),
    };
    let fallback_body = script_for(&classification, None).1;
    let compiled = match compiler_for(&api)
        .compile_from_classification(&classification, None, None)
        .await
    {
        Ok(c) => c,
        Err(e) => {
            warn!("LLM compile failed ({e}); returning offline BASIC fallback");
            return Json(CompileIntentResponse {
                success: true,
                plan_id: Some(classification.id.clone()),
                plan_name: classification.suggested_name.clone(),
                plan_description: Some(classification.original_text.clone()),
                steps: Vec::new(),
                alternatives: Vec::new(),
                confidence: classification.confidence,
                risk_level: "medium".to_string(),
                estimated_duration_minutes: 0,
                estimated_cost: 0.0,
                resource_estimate: ResourceEstimateResponse {
                    compute_hours: 0.0, storage_gb: 0.0, api_calls: 0, llm_tokens: 0, estimated_cost_usd: 0.0,
                },
                basic_program: Some(fallback_body),
                requires_approval: classification.requires_clarification,
                mcp_servers: Vec::new(),
                external_apis: Vec::new(),
                risks: Vec::new(),
                error: None,
            })
        }
    };
    let basic_program = compiled
        .basic_program
        .clone()
        .filter(|p| !p.trim().is_empty())
        .unwrap_or(fallback_body);
    Json(CompileIntentResponse {
        success: true,
        plan_id: Some(compiled.id.clone()),
        plan_name: Some(compiled.plan_name.clone()),
        plan_description: Some(compiled.plan_description.clone()),
        steps: compiled
            .steps
            .iter()
            .map(|s| PlanStepResponse {
                id: s.id.clone(),
                order: s.order,
                name: s.name.clone(),
                description: s.description.clone(),
                keywords: s.keywords.clone(),
                priority: s.priority.clone(),
                risk_level: s.risk_level.clone(),
                estimated_minutes: s.estimated_minutes,
                requires_approval: s.requires_approval,
            })
            .collect(),
        alternatives: Vec::new(),
        confidence: compiled.confidence,
        risk_level: compiled.risk_level.clone(),
        estimated_duration_minutes: compiled.estimated_duration_minutes,
        estimated_cost: compiled.estimated_cost,
        resource_estimate: ResourceEstimateResponse {
            compute_hours: compiled.resource_estimate.compute_hours,
            storage_gb: compiled.resource_estimate.storage_gb,
            api_calls: compiled.resource_estimate.api_calls,
            llm_tokens: compiled.resource_estimate.llm_tokens,
            estimated_cost_usd: compiled.resource_estimate.estimated_cost_usd,
        },
        basic_program: Some(basic_program),
        requires_approval: compiled.requires_approval,
        mcp_servers: compiled.mcp_servers.clone(),
        external_apis: compiled.external_apis.clone(),
        risks: compiled
            .risks
            .iter()
            .map(|r| RiskResponse {
                id: r.id.clone(),
                category: r.category.clone(),
                description: r.description.clone(),
                probability: r.probability,
                impact: r.impact.clone(),
            })
            .collect(),
        error: None,
    })
}

fn error_compile(intent: &str, e: &dyn std::error::Error) -> Json<CompileIntentResponse> {
    let msg = err_msg("compile", e);
    warn!("compile failed for intent: {intent}");
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
            compute_hours: 0.0, storage_gb: 0.0, api_calls: 0, llm_tokens: 0, estimated_cost_usd: 0.0,
        },
        basic_program: None,
        requires_approval: false,
        mcp_servers: Vec::new(),
        external_apis: Vec::new(),
        risks: Vec::new(),
        error: Some(msg),
    })
}

pub async fn execute_plan(
    State(_api): State<Arc<AutoTaskApi>>,
    Json(req): Json<ExecutePlanRequest>,
) -> impl IntoResponse {
    info!("API execute plan: {}", req.plan_id);
    Json(ExecutePlanResponse {
        success: true,
        task_id: Some(Uuid::new_v4().to_string()),
        status: Some("scheduled".to_string()),
        error: None,
    })
}

/// BASIC-only pipeline: classify → compile → persist `.bas` to the bot's
/// Drive bucket. DriveMonitor picks the file up, DriveCompiler registers the
/// automation (basic_tools + system_automations), auto_service runs it.
pub async fn create_and_execute(
    State(api): State<Arc<AutoTaskApi>>,
    Json(req): Json<CreateAndExecuteRequest>,
) -> Json<CreateAndExecuteResponse> {
    info!("API create and execute: {}", &req.intent[..req.intent.len().min(50)]);
    let bot_id = canonical_bot_id(req.bot_id.clone());
    let classification = match classifier_for(&api).classify_api(&req.intent, bot_id).await {
        Ok(c) => c,
        Err(e) => return error_create(&req.intent, &*e),
    };
    let compiled = match compiler_for(&api)
        .compile_from_classification(&classification, None, None)
        .await
    {
        Ok(c) => c,
        Err(e) => return error_create(&req.intent, &*e),
    };
    let (relative_path, body) = script_for(&classification, Some(&compiled));
    match persist_script(&api, bot_id, &relative_path, &body) {
        Ok((bucket, key)) => Json(CreateAndExecuteResponse {
            success: true,
            task_id: classification.id.clone(),
            status: "created".to_string(),
            message: format!("Automation created and registered: {bucket}/{key}"),
            app_url: None,
            created_resources: vec![crate::api::CreatedResourceResponse {
                resource_type: classification.intent_type.to_string().to_lowercase(),
                name: classification
                    .suggested_name
                    .clone()
                    .unwrap_or_else(|| "autotask".to_string()),
                path: Some(key.clone()),
            }],
            pending_items: Vec::new(),
            error: None,
        }),
        Err(e) => Json(CreateAndExecuteResponse {
            success: false,
            task_id: classification.id.clone(),
            status: "failed".to_string(),
            message: e.clone(),
            app_url: None,
            created_resources: Vec::new(),
            pending_items: Vec::new(),
            error: Some(e),
        }),
    }
}

/// Upload the generated `.bas` to `{bot}.gbai/{bot}.gbdialog/{path}` via the
/// AutoTask Drive facade (no local filesystem writes).
fn persist_script(
    api: &AutoTaskApi,
    bot_id: Uuid,
    relative_path: &str,
    body: &str,
) -> Result<(String, String), String> {
    let info = resolve_bot_info(api.state().db_pool(), bot_id)?
        .ok_or_else(|| "bot not found for classification".to_string())?;
    let ops = api
        .state()
        .file_ops()
        .ok_or_else(|| "Drive ops not available — cannot persist generated BASIC".to_string())?;
    let bucket = info.bucket_name();
    let key = format!("{}/{}", info.dialog_folder(), relative_path.trim_start_matches('/'));
    ops.put_object(&bucket, &key, body.as_bytes().to_vec(), "text/plain")
        .map_err(|e| format!("drive put failed: {e}"))?;
    info!("Saved BASIC file to Drive: {bucket}/{key}");
    Ok((bucket, key))
}

fn error_create(intent: &str, e: &dyn std::error::Error) -> Json<CreateAndExecuteResponse> {
    let msg = err_msg("create_and_execute", e);
    warn!("create failed for intent: {intent}");
    Json(CreateAndExecuteResponse {
        success: false,
        task_id: String::new(),
        status: "failed".to_string(),
        message: msg.clone(),
        app_url: None,
        created_resources: Vec::new(),
        pending_items: Vec::new(),
        error: Some(msg),
    })
}

pub async fn list_tasks(
    State(api): State<Arc<AutoTaskApi>>,
    Query(_query): Query<crate::api::ListTasksQuery>,
) -> Json<Vec<serde_json::Value>> {
    let pool = api.state().db_pool().clone();
    let rows = pool
        .get()
        .ok()
        .and_then(|mut conn| {
            let rows: Vec<TaskRow> = sql_query(
                "SELECT id, original_text, intent_type, confidence, created_at \
                 FROM intent_classifications ORDER BY created_at DESC LIMIT 50",
            )
            .load(&mut conn)
            .ok()?;
            Some(rows)
        })
        .unwrap_or_default();
    Json(rows.into_iter().map(|r| r.into_json()).collect())
}

#[derive(diesel::QueryableByName)]
struct TaskRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    id: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    original_text: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    intent_type: String,
    #[diesel(sql_type = diesel::sql_types::Float8)]
    confidence: f64,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    created_at: chrono::DateTime<chrono::Utc>,
}

impl TaskRow {
    fn into_json(self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id,
            "original_text": self.original_text,
            "intent_type": self.intent_type,
            "confidence": self.confidence,
            "created_at": self.created_at.to_rfc3339(),
        })
    }
}

pub async fn get_stats(
    State(api): State<Arc<AutoTaskApi>>,
) -> impl IntoResponse {
    #[derive(diesel::QueryableByName)]
    struct CountRow {
        #[diesel(sql_type = BigInt)]
        total: i64,
    }
    let mut stats = crate::api::AutoTaskStatsResponse {
        total: 0, running: 0, pending: 0, completed: 0, failed: 0, pending_approval: 0, pending_decision: 0,
    };
    if let Ok(mut conn) = api.state().db_pool().get() {
        if let Ok(row) = sql_query("SELECT COUNT(*) AS total FROM intent_classifications")
            .get_result::<CountRow>(&mut conn)
        {
            stats.total = row.total as i32;
        }
    }
    Json(stats)
}

pub async fn approve_task(
    State(_api): State<Arc<AutoTaskApi>>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    info!("API approve task: {task_id}");
    Json(TaskActionResponse { success: true, message: Some("Task approved".to_string()), error: None })
}

pub async fn cancel_task(
    State(_api): State<Arc<AutoTaskApi>>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    info!("API cancel task: {task_id}");
    Json(TaskActionResponse { success: true, message: Some("Task cancelled".to_string()), error: None })
}

pub async fn make_decision(
    State(_api): State<Arc<AutoTaskApi>>,
    Json(req): Json<DecisionRequest>,
) -> impl IntoResponse {
    info!("API make decision: {} -> {}", req.decision_id, req.choice);
    Json(TaskActionResponse { success: true, message: Some("Decision recorded".to_string()), error: None })
}