use botcore::shared::schema::workflow_executions;
use botcore::shared::models::WorkflowExecution;
use rhai::{Dynamic, Engine};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkflowState {
    #[serde(default)]
    pub current_step: i32,
    #[serde(default)]
    pub variables: std::collections::HashMap<String, String>,
    #[serde(default = "default_running")]
    pub status: String,
}

fn default_running() -> String {
    "running".to_string()
}

#[derive(Debug, Clone)]
pub struct WorkflowStep {
    pub step_number: i32,
    pub step_type: StepType,
}

#[derive(Debug, Clone)]
pub enum StepType {
    BotCall { bot_name: String, action: String },
    Condition { expression: String },
    Parallel { branches: Vec<String> },
}

pub fn register_orchestrate_workflow(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user;

    if let Err(e) = engine.register_custom_syntax(
        ["ORCHESTRATE", "WORKFLOW", "$string$"],
        false,
        move |context, inputs| {
            let workflow_name = context.eval_expression_tree(&inputs[0])?.to_string();
            let state_for_spawn = Arc::clone(&state_clone);
            let user_clone_spawn = user_clone.clone();
            
            tokio::spawn(async move {
                if let Err(e) = create_workflow(&state_for_spawn, &user_clone_spawn, &workflow_name).await {
                    log::error!("Failed to create workflow {workflow_name}: {e}");
                }
            });

            Ok(Dynamic::UNIT)
        },
    ) {
        log::warn!("Failed to register ORCHESTRATE WORKFLOW syntax: {e}");
    }
}

async fn create_workflow(
    state: &Arc<AppState>,
    user: &UserSession,
    workflow_name: &str,
) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = state.conn.get()?;
    
    let bot_uuid = Uuid::parse_str(&user.bot_id.to_string())?;
    
    let initial_state = WorkflowState {
        current_step: 1,
        variables: std::collections::HashMap::new(),
        status: "running".to_string(),
    };
    
    let state_json = serde_json::to_value(&initial_state)?;

    let new_workflow = WorkflowExecution {
        id: Uuid::new_v4(),
        bot_id: bot_uuid,
        workflow_name: workflow_name.to_string(),
        current_step: Some(1),
        state_json: Some(state_json),
        status: "running".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    diesel::insert_into(workflow_executions::table)
        .values(&new_workflow)
        .execute(&mut conn)?;
    
    Ok(new_workflow.id)
}

pub fn register_step_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user;

    if let Err(e) = engine.register_custom_syntax(
        ["STEP", "$int$", ":", "BOT", "$string$", "$string$"],
        false,
        move |context, inputs| {
            let step_number = context.eval_expression_tree(&inputs[0])?.as_int()?;
            let bot_name = context.eval_expression_tree(&inputs[1])?.to_string();
            let action = context.eval_expression_tree(&inputs[2])?.to_string();
            
            let state_for_spawn = Arc::clone(&state_clone);
            let user_clone_spawn = user_clone.clone();
            
            tokio::spawn(async move {
                if let Err(e) = execute_workflow_step(
                    &state_for_spawn,
                    &user_clone_spawn,
                    step_number as i32,
                    &bot_name,
                    &action,
                ).await {
                    log::error!("Failed to execute workflow step {step_number}: {e}");
                }
            });

            Ok(Dynamic::UNIT)
        },
    ) {
        log::warn!("Failed to register STEP syntax: {e}");
    }
}

async fn execute_workflow_step(
    state: &Arc<AppState>,
    user: &UserSession,
    step_number: i32,
    bot_name: &str,
    action: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = state.conn.get()?;
    
    let bot_uuid = Uuid::parse_str(&user.bot_id.to_string())?;
    
    let workflow: WorkflowExecution = workflow_executions::table
        .filter(workflow_executions::bot_id.eq(bot_uuid))
        .filter(workflow_executions::status.eq("running"))
        .first(&mut conn)?;
    
    let mut workflow_state: WorkflowState = match &workflow.state_json {
        Some(v) => serde_json::from_value(v.clone()).unwrap_or_default(),
        None => WorkflowState::default(),
    };
    
    if workflow_state.current_step == step_number {
        workflow_state.current_step = step_number + 1;
        workflow_state.variables.insert("last_bot".to_string(), bot_name.to_string());
        workflow_state.variables.insert("last_action".to_string(), action.to_string());
        
        save_workflow_state(workflow.id, &workflow_state, &mut conn)?;
    }
    
    Ok(())
}

fn save_workflow_state(
    workflow_id: Uuid,
    state: &WorkflowState,
    conn: &mut PgConnection,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let state_json: Option<serde_json::Value> = Some(serde_json::to_value(state)?);

    diesel::update(workflow_executions::table.filter(workflow_executions::id.eq(workflow_id)))
        .set((
            workflow_executions::state_json.eq(state_json),
            workflow_executions::current_step.eq(Some(state.current_step)),
            workflow_executions::updated_at.eq(chrono::Utc::now()),
        ))
        .execute(conn)?;
    
    Ok(())
}

pub async fn resume_workflows_on_startup(
    state: Arc<AppState>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = state.conn.get()?;
    
    let running_workflows: Vec<WorkflowExecution> = workflow_executions::table
        .filter(workflow_executions::status.eq("running"))
        .load(&mut conn)?;
    
    for workflow in running_workflows {
        let workflow_state: WorkflowState = workflow.state_json.as_ref().and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or_default();
        
        let state_clone = Arc::clone(&state);
        tokio::spawn(async move {
            if let Err(e) = resume_workflow_execution(workflow.id, workflow_state, state_clone).await {
                log::error!("Failed to resume workflow {}: {e}", workflow.id);
            }
        });
    }
    
    Ok(())
}

async fn resume_workflow_execution(
    workflow_id: Uuid,
    _state: WorkflowState,
    _app_state: Arc<AppState>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    log::info!("Resuming workflow {workflow_id}");
    Ok(())
}

use botcore::shared::state::AppState;
use crate::basic::UserSession;
use diesel::prelude::*;
use std::sync::Arc;
use uuid::Uuid;