use crate::types::{AutoTaskState, BotDatabaseOps, ConfigOps, DriveOps, LlmProviderOps, UserSession};
use crate::task_types::{ExecutionMode, TaskPriority};
use crate::intent_compiler::{CompiledIntent, IntentCompiler};
use crate::app_generator::AppGenerator;
use crate::safety_layer::SafetyLayer;
use crate::app_logs::{log_generator_error, log_generator_info};
use log::{info, warn, error};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorResult {
    pub success: bool,
    pub task_id: String,
    pub status: String,
    pub message: String,
    pub app_url: Option<String>,
    pub created_resources: Vec<CreatedResourceInfo>,
    pub pending_items: Vec<PendingItemInfo>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatedResourceInfo {
    pub resource_type: String,
    pub name: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingItemInfo {
    pub id: String,
    pub label: String,
    pub config_key: String,
    pub reason: Option<String>,
}

pub struct Orchestrator {
    state: Arc<dyn AutoTaskState>,
    config_ops: Arc<dyn ConfigOps>,
    llm_ops: Arc<dyn LlmProviderOps>,
    db_ops: Arc<dyn BotDatabaseOps>,
    drive_ops: Arc<dyn DriveOps>,
    compiler: IntentCompiler,
    generator: AppGenerator,
    safety: SafetyLayer,
}

impl Orchestrator {
    pub fn new(
        state: Arc<dyn AutoTaskState>,
        config_ops: Arc<dyn ConfigOps>,
        llm_ops: Arc<dyn LlmProviderOps>,
        db_ops: Arc<dyn BotDatabaseOps>,
        drive_ops: Arc<dyn DriveOps>,
    ) -> Self {
        let compiler = IntentCompiler::new(state.clone(), config_ops.clone(), llm_ops.clone());
        let generator = AppGenerator::new(state.clone());
        let safety = SafetyLayer::new(state.db_pool().clone());
        Self { state, config_ops, llm_ops, db_ops, drive_ops, compiler, generator, safety }
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

    pub fn db_ops(&self) -> &Arc<dyn BotDatabaseOps> {
        &self.db_ops
    }

    pub fn drive_ops(&self) -> &Arc<dyn DriveOps> {
        &self.drive_ops
    }

    pub async fn execute(
        &self,
        intent: &str,
        session: &UserSession,
        execution_mode: Option<ExecutionMode>,
        priority: Option<TaskPriority>,
    ) -> Result<OrchestratorResult, Box<dyn std::error::Error + Send + Sync>> {
        let task_id = Uuid::new_v4().to_string();
        info!("Orchestrator executing task {task_id}: {}", &intent[..intent.len().min(80)]);

        self.state.emit_task_started(&task_id, &format!("Processing: {intent}"), 4);

        let compiled = self.compiler.compile(intent, execution_mode, priority).await?;
        info!("Intent compiled: {} steps, risk={}", compiled.steps.len(), compiled.risk_level);

        if compiled.requires_approval {
            return Ok(OrchestratorResult {
                success: false,
                task_id,
                status: "pending_approval".to_string(),
                message: "This action requires approval before execution".to_string(),
                app_url: None,
                created_resources: Vec::new(),
                pending_items: Vec::new(),
                error: None,
            });
        }

        let sim_result = self.safety.simulate_execution(&task_id, session)?;
        if !sim_result.success {
            let reason = sim_result.impact.summary.clone();
            warn!("Safety check failed for task {task_id}: {reason}");
            return Ok(OrchestratorResult {
                success: false,
                task_id,
                status: "blocked".to_string(),
                message: format!("Safety check failed: {reason}"),
                app_url: None,
                created_resources: Vec::new(),
                pending_items: Vec::new(),
                error: Some(reason),
            });
        }

        self.state.emit_activity(&task_id, "generating", "Generating application", 2, 4,
            crate::types::AgentActivity::new("generating"));

        let app_result = self.generator.generate(intent, session).await;
        let created_resources = match app_result {
            Ok(app) => {
                log_generator_info(&task_id, &format!("app_generated: {}", app.name));
                let mut resources = Vec::new();
                for page in &app.pages {
                    resources.push(CreatedResourceInfo {
                        resource_type: "file".to_string(),
                        name: page.filename.clone(),
                        path: Some(page.filename.clone()),
                    });
                }
                for table in &app.tables {
                    resources.push(CreatedResourceInfo {
                        resource_type: "table".to_string(),
                        name: table.name.clone(),
                        path: None,
                    });
                }
                resources
            }
            Err(e) => {
                error!("App generation failed for task {task_id}: {e}");
                log_generator_error(&task_id, "generation_failed", &e.to_string());
                return Ok(OrchestratorResult {
                    success: false,
                    task_id,
                    status: "failed".to_string(),
                    message: format!("App generation failed: {e}"),
                    app_url: None,
                    created_resources: Vec::new(),
                    pending_items: Vec::new(),
                    error: Some(e.to_string()),
                });
            }
        };

        self.state.emit_activity(&task_id, "completed", "Task completed", 4, 4,
            crate::types::AgentActivity::new("completed"));

        Ok(OrchestratorResult {
            success: true,
            task_id,
            status: "completed".to_string(),
            message: "Task completed successfully".to_string(),
            app_url: None,
            created_resources,
            pending_items: Vec::new(),
            error: None,
        })
    }

    pub async fn execute_compiled(
        &self,
        compiled: &CompiledIntent,
        _session: &UserSession,
    ) -> Result<OrchestratorResult, Box<dyn std::error::Error + Send + Sync>> {
        let task_id = Uuid::new_v4().to_string();
        info!("Orchestrator executing compiled plan: {}", compiled.plan_name);

        self.state.emit_task_started(&task_id, &compiled.plan_description, compiled.steps.len() as u8);

        for (i, step) in compiled.steps.iter().enumerate() {
            self.state.emit_activity(
                &task_id, &step.name, &step.description,
                i as u8 + 1, compiled.steps.len() as u8,
                crate::types::AgentActivity::new(&step.name),
            );
        }

        Ok(OrchestratorResult {
            success: true,
            task_id,
            status: "completed".to_string(),
            message: format!("Executed plan: {}", compiled.plan_name),
            app_url: None,
            created_resources: Vec::new(),
            pending_items: Vec::new(),
            error: None,
        })
    }
}
