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
use crate::intent_classifier::{ClassifiedEntities, IntentClassifier, IntentType};
use crate::intent_compiler::IntentCompiler;
use crate::templates::match_shipped_template;
use crate::ClassifiedIntent;
use crate::types::{BotInfo, DbPool};
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{BigInt, Float8, Nullable as NullableSql, Text, Uuid as DieselUuid};
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

pub(crate) fn err_msg(context: &str, e: &dyn std::error::Error) -> String {
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
        .compile_from_classification(bot_id, classification, None, None)
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
        .compile_from_classification(bot_id, &classification, None, None)
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

/// Persisted classification row backing an executable plan.
#[derive(diesel::QueryableByName)]
struct PlanRow {
    #[diesel(sql_type = DieselUuid)]
    id: Uuid,
    #[diesel(sql_type = DieselUuid)]
    bot_id: Uuid,
    #[diesel(sql_type = Text)]
    original_text: String,
    #[diesel(sql_type = Text)]
    intent_type: String,
    #[diesel(sql_type = Float8)]
    confidence: f64,
    #[diesel(sql_type = diesel::sql_types::Nullable<Text>)]
    suggested_name: Option<String>,
}

fn load_plan(api: &AutoTaskApi, plan_id: Uuid) -> Result<Option<PlanRow>, String> {
    let mut conn = api.state().db_pool().get().map_err(|e| format!("db pool: {e}"))?;
    sql_query(
        "SELECT id, bot_id, original_text, intent_type, confidence, suggested_name \
         FROM intent_classifications WHERE id = $1",
    )
    .bind::<DieselUuid, _>(plan_id)
    .get_result::<PlanRow>(&mut conn)
    .optional()
    .map_err(|e| format!("load plan: {e}"))
}

/// Execute a previously classified plan for real.
///
/// The plan id identifies a persisted `intent_classifications` row; the row is
/// rebuilt into a `ClassifiedIntent`, the offline pipeline regenerates the
/// BASIC, and the script is uploaded to the bot's Drive `gbdialog` folder —
/// exactly the path `create_and_execute` takes, so DriveMonitor compiles and
/// runs the automation. The response reports the real outcome; it never
/// fabricates a `scheduled` status with no side effect.
pub async fn execute_plan(
    State(api): State<Arc<AutoTaskApi>>,
    Json(req): Json<ExecutePlanRequest>,
) -> impl IntoResponse {
    info!(
        "API execute plan: {} (mode: {:?}, priority: {:?})",
        req.plan_id, req.execution_mode, req.priority
    );
    let plan_uuid = match Uuid::parse_str(&req.plan_id) {
        Ok(u) => u,
        Err(_) => {
            return Json(ExecutePlanResponse {
                success: false,
                task_id: None,
                status: Some("invalid".to_string()),
                error: Some("Invalid plan id".to_string()),
            });
        }
    };

    let row = match load_plan(&api, plan_uuid) {
        Ok(Some(row)) => row,
        Ok(None) => {
            return Json(ExecutePlanResponse {
                success: false,
                task_id: Some(req.plan_id.clone()),
                status: Some("not_found".to_string()),
                error: Some("Plan not found".to_string()),
            });
        }
        Err(e) => {
            warn!("execute_plan lookup failed for {}: {e}", req.plan_id);
            return Json(ExecutePlanResponse {
                success: false,
                task_id: Some(req.plan_id.clone()),
                status: Some("failed".to_string()),
                error: Some(e),
            });
        }
    };

    let plan_id = row.id.to_string();
    let classification = ClassifiedIntent {
        id: plan_id.clone(),
        original_text: row.original_text,
        intent_type: IntentType::from(row.intent_type.as_str()),
        confidence: row.confidence,
        entities: ClassifiedEntities::default(),
        suggested_name: row.suggested_name,
        requires_clarification: false,
        clarification_question: None,
        alternative_types: Vec::new(),
        classified_at: chrono::Utc::now(),
    };
    let (relative_path, body) = script_for(&classification, None);
    match persist_script(&api, row.bot_id, &relative_path, &body) {
        Ok((bucket, key)) => {
            info!("execute_plan persisted automation to {bucket}/{key}");
            Json(ExecutePlanResponse {
                success: true,
                task_id: Some(plan_id),
                status: Some("created".to_string()),
                error: None,
            })
        }
        Err(e) => {
            warn!("execute_plan persist failed: {e}");
            Json(ExecutePlanResponse {
                success: false,
                task_id: Some(plan_id),
                status: Some("failed".to_string()),
                error: Some(e),
            })
        }
    }
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

    // Shipped-template fast path: intents that name an implementation the
    // product already ships (e.g. media classification/filing) persist the
    // vetted template verbatim — deterministic, no LLM, no compile stall.
    if let Some(template) = match_shipped_template(&req.intent) {
        return persist_shipped_template(&api, bot_id, &req.intent, template).await;
    }

    let classification = match classifier_for(&api).classify_api(&req.intent, bot_id).await {
        Ok(c) => c,
        Err(e) => return error_create(&req.intent, &*e),
    };
    let compiled = match compiler_for(&api)
        .compile_from_classification(bot_id, &classification, None, None)
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
    let bucket = info.bucket_name();
    // Reform #1505 — a git-owned bot's `.gbdialog` is its repository: commit the
    // generated tool there so the pull monitor compiles it and the task leaves a
    // real artifact. Drive is the legacy fallback for bots without a project.
    if let Some(result) = crate::source_persist::persist_to_git(
        api.state().as_ref(),
        &info,
        bot_id,
        relative_path.trim_start_matches('/'),
        body,
        None,
        "autotask: add generated tool",
    ) {
        let persisted = result?;
        info!(
            "Committed generated BASIC to ALM: {}/{}",
            persisted.bucket, persisted.tool_key
        );
        record_auto_task(
            api.state().db_pool(),
            bot_id,
            relative_path.trim_start_matches('/'),
            "generated tool",
            "completed",
            Some(&persisted.tool_key),
        );
        return Ok((persisted.bucket, persisted.tool_key));
    }
    let ops = api
        .state()
        .file_ops()
        .ok_or_else(|| "Drive ops not available — cannot persist generated BASIC".to_string())?;
    let key = format!("{}/{}", info.dialog_folder(), relative_path.trim_start_matches('/'));
    ops.put_object(&bucket, &key, body.as_bytes().to_vec(), "text/plain")
        .map_err(|e| format!("drive put failed: {e}"))?;
    info!("Saved BASIC file to Drive: {bucket}/{key}");
    Ok((bucket, key))
}

/// Persist a shipped template's files to `{bot}.gbai/{bot}.gbdialog/` and
/// return the standard create-and-execute response.
async fn persist_shipped_template(
    api: &Arc<AutoTaskApi>,
    bot_id: Uuid,
    intent: &str,
    template: crate::templates::ShippedTemplate,
) -> Json<CreateAndExecuteResponse> {
    info!(
        "create-and-execute matched shipped template '{}' for intent: {}",
        template.name,
        &intent[..intent.len().min(80)]
    );
    let info = match resolve_bot_info(api.state().db_pool(), bot_id) {
        Ok(Some(info)) => info,
        Ok(None) => {
            let e: Box<dyn std::error::Error + Send + Sync> = "bot not found for classification".into();
            return error_create(intent, &*e);
        }
        Err(e) => {
            let e: Box<dyn std::error::Error + Send + Sync> = e.into();
            return error_create(intent, &*e);
        }
    };
    // Reform #1505 — persist the template into the bot's repository (ALM),
    // the canonical `.gbdialog`; Drive is the legacy fallback.
    if let Some(result) = crate::source_persist::persist_to_git(
        api.state().as_ref(),
        &info,
        bot_id,
        template.tool_path,
        template.tool_source,
        template.manifest_path.zip(template.manifest_source),
        &format!("autotask: add '{}' template", template.name),
    ) {
        let persisted = match result {
            Ok(p) => p,
            Err(e) => {
                let e: Box<dyn std::error::Error + Send + Sync> = e.into();
                return error_create(intent, &*e);
            }
        };
        info!(
            "Committed shipped template '{}' to ALM: {}",
            template.name, persisted.tool_key
        );
        record_auto_task(
            api.state().db_pool(),
            bot_id,
            &format!("{} ({})", template.name, template.tool_path),
            intent,
            "completed",
            Some(&persisted.tool_key),
        );
        let mut created = vec![crate::api::CreatedResourceResponse {
            resource_type: "tool".to_string(),
            name: template
                .tool_path
                .trim_start_matches("tools/")
                .trim_end_matches(".bas")
                .to_string(),
            path: Some(persisted.tool_key.clone()),
        }];
        // The channel prompts are what make the model call the tool; without
        // them the tool is committed but never invoked, so they ship with it.
        if let Some(config) = crate::source_persist::persist_config_to_git(
            api.state().as_ref(),
            &info,
            bot_id,
            template.config_files,
            &format!("autotask: add '{}' channel prompts", template.name),
        ) {
            match config {
                Ok(keys) if !keys.is_empty() => {
                    info!(
                        "Committed '{}' channel prompts to ALM: {}",
                        template.name,
                        keys.join(", ")
                    );
                    created.push(crate::api::CreatedResourceResponse {
                        resource_type: "bot-config".to_string(),
                        name: keys
                            .iter()
                            .map(|k| k.rsplit('/').next().unwrap_or_default().to_string())
                            .collect::<Vec<String>>()
                            .join(", "),
                        path: Some(
                            keys[0]
                                .rsplit_once('/')
                                .map(|(folder, _)| format!("{folder}/"))
                                .unwrap_or_default(),
                        ),
                    });
                }
                Ok(_) => {}
                // The tool is already committed; report the config failure
                // instead of aborting, so the user can retry the prompts.
                Err(e) => warn!("autotask: channel prompts not committed: {e}"),
            }
        }
        if let Some(key) = &persisted.manifest_key {
            created.push(crate::api::CreatedResourceResponse {
                resource_type: "mcp-manifest".to_string(),
                name: key.rsplit('/').next().unwrap_or_default().to_string(),
                path: Some(key.clone()),
            });
        }
        let tables = if persisted.tables.is_empty() {
            String::new()
        } else {
            format!(" (tables attached to tables.bas: {})", persisted.tables.join(", "))
        };
        return Json(CreateAndExecuteResponse {
            success: true,
            task_id: Uuid::new_v4().to_string(),
            status: "created".to_string(),
            message: format!(
                "Automation created from shipped template '{}' in the bot repository: {}{tables}",
                template.name, persisted.tool_key
            ),
            app_url: None,
            created_resources: created,
            pending_items: Vec::new(),
            error: None,
        });
    }
    let ops = match api.state().file_ops() {
        Some(ops) => ops,
        None => {
            let e: Box<dyn std::error::Error + Send + Sync> =
                "Drive ops not available — cannot persist shipped template".into();
            return error_create(intent, &*e);
        }
    };
    let bucket = info.bucket_name();
    let dialog = info.dialog_folder();
    let mut created = Vec::new();
    let tool_key = format!("{dialog}/{}", template.tool_path);
    if let Err(e) = ops.put_object(
        &bucket,
        &tool_key,
        template.tool_source.as_bytes().to_vec(),
        "text/plain",
    ) {
        let e: Box<dyn std::error::Error + Send + Sync> = format!("drive put failed: {e}").into();
        return error_create(intent, &*e);
    }
    info!("Saved BASIC file to Drive: {bucket}/{tool_key}");
    created.push(crate::api::CreatedResourceResponse {
        resource_type: "tool".to_string(),
        name: template
            .tool_path
            .trim_start_matches("tools/")
            .trim_end_matches(".bas")
            .to_string(),
        path: Some(tool_key.clone()),
    });
    if let (Some(rel), Some(src)) = (template.manifest_path, template.manifest_source) {
        // Persist the manifest next to the tool AND as the sibling root-level
        // copy the compiler reads: compile_file() regenerates a manifest with
        // an empty schema on every .bas compile, and the drive compiler syncs
        // the Drive copy over the generated one — without this root copy the
        // schema (and tool_exec's missing-argument defaulting) is lost.
        let manifest_key = format!("{dialog}/{rel}");
        let root_manifest_key = format!(
            "{dialog}/{}",
            rel.trim_start_matches("tools/")
        );
        let mut persisted = false;
        if let Err(e) = ops.put_object(
            &bucket,
            &manifest_key,
            src.as_bytes().to_vec(),
            "application/json",
        ) {
            warn!("shipped template manifest persist failed (tool kept): {e}");
        } else {
            persisted = true;
            info!("Saved MCP manifest to Drive: {bucket}/{manifest_key}");
            created.push(crate::api::CreatedResourceResponse {
                resource_type: "mcp-manifest".to_string(),
                name: rel.trim_start_matches("tools/").to_string(),
                path: Some(manifest_key.clone()),
            });
        }
        if persisted && root_manifest_key != manifest_key {
            if let Err(e) = ops.put_object(
                &bucket,
                &root_manifest_key,
                src.as_bytes().to_vec(),
                "application/json",
            ) {
                warn!("root MCP manifest persist failed (schema may be lost on recompile): {e}");
            } else {
                info!("Saved root MCP manifest to Drive: {bucket}/{root_manifest_key}");
            }
        }
    }
    Json(CreateAndExecuteResponse {
        success: true,
        task_id: Uuid::new_v4().to_string(),
        status: "created".to_string(),
        message: format!(
            "Automation created from shipped template '{}': {bucket}/{tool_key}",
            template.name
        ),
        app_url: None,
        created_resources: created,
        pending_items: Vec::new(),
        error: None,
    })
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

/// Insert an `auto_tasks` row recording a created automation. The row is what
/// makes an AutoTask item traceable: the list UI reads these rows, and the
/// ✏️ action opens the `.bas` it produced.
pub(crate) fn record_auto_task(
    pool: &DbPool,
    bot_id: Uuid,
    title: &str,
    intent: &str,
    status: &str,
    script_path: Option<&str>,
) {
    // branch_id is NOT NULL (6.5.23); resolve it from the bot's row.
    let outcome = pool.get().ok().map(|mut conn| {
        let branch_id: Option<Uuid> = sql_query("SELECT branch_id AS id FROM bots WHERE id = $1")
            .bind::<DieselUuid, _>(bot_id)
            .get_result::<BotBranchRow>(&mut conn)
            .ok()
            .and_then(|r| r.id);
        sql_query(
            "INSERT INTO auto_tasks (bot_id, branch_id, title, intent, status, basic_program, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW()) RETURNING id",
        )
        .bind::<DieselUuid, _>(bot_id)
        .bind::<NullableSql<DieselUuid>, _>(branch_id)
        .bind::<Text, _>(title)
        .bind::<Text, _>(intent)
        .bind::<Text, _>(status)
        .bind::<NullableSql<Text>, _>(script_path)
        .get_result::<TaskIdRow>(&mut conn)
    });
    match outcome {
        Some(Ok(row)) => info!("Recorded auto_tasks row {} for '{title}'", row.id),
        Some(Err(e)) => warn!("Failed to record auto_tasks row for '{title}': {e}"),
        None => warn!("DB pool unavailable — auto_tasks row for '{title}' not recorded"),
    }
}

#[derive(diesel::QueryableByName)]
struct TaskIdRow {
    #[diesel(sql_type = DieselUuid)]
    id: Uuid,
}

#[derive(diesel::QueryableByName)]
struct BotBranchRow {
    #[diesel(sql_type = NullableSql<DieselUuid>)]
    id: Option<Uuid>,
}

/// AutoTask items = rows of `auto_tasks`. The ✏️ action resolves the item's
/// `.bas` through /api/autotask/sources (the repository), so the row carries
/// the script path for display only.
#[derive(diesel::QueryableByName)]
struct AutoTaskRow {
    #[diesel(sql_type = DieselUuid)]
    id: Uuid,
    #[diesel(sql_type = DieselUuid)]
    bot_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    title: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    intent: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    status: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    basic_program: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    created_at: chrono::DateTime<chrono::Utc>,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl AutoTaskRow {
    fn into_json(self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id,
            "bot_id": self.bot_id,
            "title": self.title,
            "intent": self.intent,
            "status": self.status,
            "script_path": self.basic_program,
            "created_at": self.created_at.to_rfc3339(),
            "updated_at": self.updated_at.to_rfc3339(),
        })
    }
}

pub async fn list_tasks(
    State(api): State<Arc<AutoTaskApi>>,
    Query(query): Query<crate::api::ListTasksQuery>,
) -> Json<Vec<serde_json::Value>> {
    let pool = api.state().db_pool().clone();
    let bot_filter = query.bot_id.clone().and_then(|s| Uuid::parse_str(&s).ok());
    let rows = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        match bot_filter {
            Some(bot_id) => sql_query(
                "SELECT id, bot_id, title, intent, status, basic_program, created_at, updated_at \
                 FROM auto_tasks WHERE bot_id = $1 ORDER BY created_at DESC LIMIT 100",
            )
            .bind::<DieselUuid, _>(bot_id)
            .load::<AutoTaskRow>(&mut conn)
            .ok(),
            None => sql_query(
                "SELECT id, bot_id, title, intent, status, basic_program, created_at, updated_at \
                 FROM auto_tasks ORDER BY created_at DESC LIMIT 100",
            )
            .load::<AutoTaskRow>(&mut conn)
            .ok(),
        }
    })
    .await
    .ok()
    .flatten()
    .unwrap_or_default();
    Json(rows.into_iter().map(|r| r.into_json()).collect())
}

pub async fn get_stats(
    State(api): State<Arc<AutoTaskApi>>,
) -> impl IntoResponse {
    #[derive(diesel::QueryableByName)]
    struct CountRow {
        #[diesel(sql_type = BigInt)]
        total: i64,
        #[diesel(sql_type = BigInt)]
        running: i64,
        #[diesel(sql_type = BigInt)]
        pending: i64,
        #[diesel(sql_type = BigInt)]
        completed: i64,
        #[diesel(sql_type = BigInt)]
        failed: i64,
        #[diesel(sql_type = BigInt)]
        pending_approval: i64,
    }
    let mut stats = crate::api::AutoTaskStatsResponse {
        total: 0, running: 0, pending: 0, completed: 0, failed: 0, pending_approval: 0, pending_decision: 0,
    };
    if let Ok(mut conn) = api.state().db_pool().get() {
        // #1505 — the item list is auto_tasks, so the stats must count the same
        // rows (intent_classifications counted heuristic runs, not items).
        if let Ok(row) = sql_query(
            "SELECT COUNT(*) AS total, \
                    COUNT(*) FILTER (WHERE status = 'running') AS running, \
                    COUNT(*) FILTER (WHERE status IN ('pending','ready','paused')) AS pending, \
                    COUNT(*) FILTER (WHERE status = 'completed') AS completed, \
                    COUNT(*) FILTER (WHERE status = 'failed') AS failed, \
                    COUNT(*) FILTER (WHERE status = 'waiting_approval') AS pending_approval \
             FROM auto_tasks",
        )
        .get_result::<CountRow>(&mut conn)
        {
            stats.total = row.total as i32;
            stats.running = row.running as i32;
            stats.pending = row.pending as i32;
            stats.completed = row.completed as i32;
            stats.failed = row.failed as i32;
            stats.pending_approval = row.pending_approval as i32;
        }
    }
    Json(stats)
}

pub async fn approve_task(
    State(api): State<Arc<AutoTaskApi>>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    info!("API approve task: {task_id}");
    let task_uuid = match uuid::Uuid::parse_str(&task_id) {
        Ok(u) => u,
        Err(_) => return Json(TaskActionResponse { success: false, message: None, error: Some("Invalid task id".to_string()) }),
    };
    let pool = api.state().db_pool().clone();
    let result = tokio::task::spawn_blocking(move || -> Result<(), String> {
        let mut conn = pool.get().map_err(|e| format!("DB connection failed: {e}"))?;
        diesel::sql_query(
            "UPDATE auto_tasks SET status = 'ready', updated_at = NOW() WHERE id = $1 AND status IN ('pending', 'waiting_approval')",
        )
        .bind::<diesel::sql_types::Uuid, _>(task_uuid)
        .execute(&mut conn)
        .map_err(|e| format!("Failed to approve task: {e}"))?;
        diesel::sql_query(
            "UPDATE task_approvals SET status = 'approved', decision = 'approve', decided_at = NOW() WHERE task_id = $1 AND status = 'pending'",
        )
        .bind::<diesel::sql_types::Uuid, _>(task_uuid)
        .execute(&mut conn)
        .map_err(|e| format!("Failed to approve task approvals: {e}"))?;
        Ok(())
    })
    .await;
    match result {
        Ok(Ok(())) => Json(TaskActionResponse { success: true, message: Some("Task approved".to_string()), error: None }),
        Ok(Err(e)) => {
            log::error!("Failed to approve task {task_id}: {e}");
            Json(TaskActionResponse { success: false, message: None, error: Some(e) })
        }
        Err(e) => {
            log::error!("Approve task task panicked: {e}");
            Json(TaskActionResponse { success: false, message: None, error: Some("Task execution failed".to_string()) })
        }
    }
}

pub async fn cancel_task(
    State(api): State<Arc<AutoTaskApi>>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    info!("API cancel task: {task_id}");
    let task_uuid = match uuid::Uuid::parse_str(&task_id) {
        Ok(u) => u,
        Err(_) => return Json(TaskActionResponse { success: false, message: None, error: Some("Invalid task id".to_string()) }),
    };
    let pool = api.state().db_pool().clone();
    let result = tokio::task::spawn_blocking(move || -> Result<(), String> {
        let mut conn = pool.get().map_err(|e| format!("DB connection failed: {e}"))?;
        diesel::sql_query(
            "UPDATE auto_tasks SET status = 'cancelled', updated_at = NOW() WHERE id = $1 AND status NOT IN ('completed', 'cancelled')",
        )
        .bind::<diesel::sql_types::Uuid, _>(task_uuid)
        .execute(&mut conn)
        .map_err(|e| format!("Failed to cancel task: {e}"))?;
        Ok(())
    })
    .await;
    match result {
        Ok(Ok(())) => Json(TaskActionResponse { success: true, message: Some("Task cancelled".to_string()), error: None }),
        Ok(Err(e)) => {
            log::error!("Failed to cancel task {task_id}: {e}");
            Json(TaskActionResponse { success: false, message: None, error: Some(e) })
        }
        Err(e) => {
            log::error!("Cancel task task panicked: {e}");
            Json(TaskActionResponse { success: false, message: None, error: Some("Task execution failed".to_string()) })
        }
    }
}

pub async fn make_decision(
    State(api): State<Arc<AutoTaskApi>>,
    Json(req): Json<DecisionRequest>,
) -> impl IntoResponse {
    info!("API make decision: {} -> {}", req.decision_id, req.choice);
    let decision_uuid = match uuid::Uuid::parse_str(&req.decision_id) {
        Ok(u) => u,
        Err(_) => return Json(TaskActionResponse { success: false, message: None, error: Some("Invalid decision id".to_string()) }),
    };
    let choice = req.choice.clone();
    let pool = api.state().db_pool().clone();
    let result = tokio::task::spawn_blocking(move || -> Result<(), String> {
        let mut conn = pool.get().map_err(|e| format!("DB connection failed: {e}"))?;
        diesel::sql_query(
            "UPDATE task_decisions SET status = 'answered', selected_option = $1, decided_at = NOW() WHERE id = $2 AND status = 'pending'",
        )
        .bind::<diesel::sql_types::Text, _>(&choice)
        .bind::<diesel::sql_types::Uuid, _>(decision_uuid)
        .execute(&mut conn)
        .map_err(|e| format!("Failed to record decision: {e}"))?;
        Ok(())
    })
    .await;
    match result {
        Ok(Ok(())) => Json(TaskActionResponse { success: true, message: Some("Decision recorded".to_string()), error: None }),
        Ok(Err(e)) => {
            log::error!("Failed to record decision {}: {e}", req.decision_id);
            Json(TaskActionResponse { success: false, message: None, error: Some(e) })
        }
        Err(e) => {
            log::error!("Decision task panicked: {e}");
            Json(TaskActionResponse { success: false, message: None, error: Some("Decision execution failed".to_string()) })
        }
    }
}