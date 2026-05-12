use botschema::workflow_executions;
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

pub fn register_orchestrate_workflow(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
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
    state: &Arc<dyn BasicRuntime>,
    user: &UserSession,
    workflow_name: &str,
) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = state.db_pool().get()?;

    let bot_uuid = Uuid::parse_str(&user.bot_id.to_string())?;

    let initial_state = WorkflowState {
        current_step: 1,
        variables: std::collections::HashMap::new(),
        status: "running".to_string(),
    };

    let state_json = serde_json::to_value(&initial_state)?;
    let workflow_id = Uuid::new_v4();
    let now = chrono::Utc::now();

    diesel::sql_query(
        "INSERT INTO workflow_executions (id, bot_id, workflow_name, current_step, state_json, status, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
    )
        .bind::<diesel::sql_types::Uuid, _>(&workflow_id)
        .bind::<diesel::sql_types::Uuid, _>(&bot_uuid)
        .bind::<diesel::sql_types::Text, _>(workflow_name)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Int4>, _>(&Some(1i32))
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Jsonb>, _>(&Some(state_json))
        .bind::<diesel::sql_types::Text, _>("running")
        .bind::<diesel::sql_types::Timestamptz, _>(&now)
        .bind::<diesel::sql_types::Timestamptz, _>(&now)
        .execute(&mut conn)?;

    Ok(workflow_id)
}

pub fn register_step_keyword(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
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
    state: &Arc<dyn BasicRuntime>,
    user: &UserSession,
    step_number: i32,
    bot_name: &str,
    action: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = state.db_pool().get()?;
    
    let bot_uuid = Uuid::parse_str(&user.bot_id.to_string())?;
    
    #[derive(QueryableByName)]
    struct WorkflowRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Jsonb>)]
        state_json: Option<serde_json::Value>,
    }

    let workflow = diesel::sql_query(
        "SELECT id, state_json FROM workflow_executions WHERE bot_id = $1 AND status = 'running' ORDER BY created_at DESC LIMIT 1"
    )
        .bind::<diesel::sql_types::Uuid, _>(bot_uuid)
        .get_result::<WorkflowRow>(&mut conn)
        .optional()?
        .ok_or_else(|| -> Box<dyn std::error::Error + Send + Sync> { "No running workflow found".into() })?;
    
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
    state: Arc<dyn BasicRuntime>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = state.db_pool().get()?;
    
    #[derive(QueryableByName)]
    struct WorkflowRowList {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Jsonb>)]
        state_json: Option<serde_json::Value>,
    }

    let running_workflows: Vec<WorkflowRowList> = diesel::sql_query(
        "SELECT id, state_json FROM workflow_executions WHERE status = 'running'"
    )
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
    _app_state: Arc<dyn BasicRuntime>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    log::info!("Resuming workflow {workflow_id}");
    Ok(())
}

use botbasic_types::BasicRuntime;
use botbasic_types::UserSession;
use diesel::prelude::*;
use diesel::PgConnection;
use std::sync::Arc;
use uuid::Uuid;