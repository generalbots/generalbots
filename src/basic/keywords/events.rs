use crate::core::shared::models::{workflow_events, WorkflowEvent};
use crate::core::shared::state::AppState;
use crate::basic::UserSession;
use diesel::prelude::*;
use rhai::{Dynamic, Engine};
use std::sync::Arc;
use uuid::Uuid;
#[cfg(feature = "cache")]
use redis::AsyncCommands;

const ALLOWED_EVENTS: &[&str] = &[
    "workflow_step_complete",
    "approval_received", 
    "approval_denied",
    "timeout_occurred",
    "bot_response_ready",
];

pub fn register_on_event(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user;

    if let Err(e) = engine.register_custom_syntax(
        ["ON", "EVENT", "$string$", "DO"],
        false,
        move |context, inputs| {
            let event_name = context.eval_expression_tree(&inputs[0])?.to_string();
            
            if !ALLOWED_EVENTS.contains(&event_name.as_str()) {
                return Err(format!("Invalid event name: {event_name}").into());
            }
            
            let state_for_spawn = Arc::clone(&state_clone);
            let user_clone_spawn = user_clone.clone();
            
            tokio::spawn(async move {
                if let Err(e) = register_event_handler(&state_for_spawn, &user_clone_spawn, &event_name).await {
                    log::error!("Failed to register event handler for {event_name}: {e}");
                }
            });

            Ok(Dynamic::UNIT)
        },
    ) {
        log::warn!("Failed to register ON EVENT syntax: {e}");
    }
}

pub fn register_publish_event(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user;

    if let Err(e) = engine.register_custom_syntax(
        ["PUBLISH", "EVENT", "$string$"],
        false,
        move |context, inputs| {
            let event_name = context.eval_expression_tree(&inputs[0])?.to_string();
            
            if !ALLOWED_EVENTS.contains(&event_name.as_str()) {
                return Err(format!("Invalid event name: {event_name}").into());
            }
            
            let state_for_spawn = Arc::clone(&state_clone);
            let user_clone_spawn = user_clone.clone();
            
            tokio::spawn(async move {
                if let Err(e) = publish_event(&state_for_spawn, &user_clone_spawn, &event_name, &serde_json::Value::Null).await {
                    log::error!("Failed to publish event {event_name}: {e}");
                }
            });

            Ok(Dynamic::UNIT)
        },
    ) {
        log::warn!("Failed to register PUBLISH EVENT syntax: {e}");
    }
}

pub fn register_wait_for_event(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user;

    if let Err(e) = engine.register_custom_syntax(
        ["WAIT", "FOR", "EVENT", "$string$", "TIMEOUT", "$int$"],
        false,
        move |context, inputs| {
            let event_name = context.eval_expression_tree(&inputs[0])?.to_string();
            let timeout_seconds = context.eval_expression_tree(&inputs[1])?.as_int()?;
            
            if !ALLOWED_EVENTS.contains(&event_name.as_str()) {
                return Err(format!("Invalid event name: {event_name}").into());
            }
            
            let state_for_spawn = Arc::clone(&state_clone);
            let user_clone_spawn = user_clone.clone();
            
            tokio::spawn(async move {
                if let Err(e) = wait_for_event(&state_for_spawn, &user_clone_spawn, &event_name, timeout_seconds as u64).await {
                    log::error!("Failed to wait for event {event_name}: {e}");
                }
            });

            Ok(Dynamic::UNIT)
        },
    ) {
        log::warn!("Failed to register WAIT FOR EVENT syntax: {e}");
    }
}

async fn register_event_handler(
    _state: &Arc<AppState>,
    user: &UserSession,
    event_name: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let bot_uuid = Uuid::parse_str(&user.bot_id.to_string())?;
    
    log::info!("Registered event handler for {event_name} on bot {bot_uuid}");
    
    Ok(())
}

async fn publish_event(
    state: &Arc<AppState>,
    _user: &UserSession,
    event_name: &str,
    event_data: &serde_json::Value,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = state.conn.get()?;
    
    let event_data_json = serde_json::to_string(event_data)?;
    
    let new_event = WorkflowEvent {
        id: Uuid::new_v4(),
        workflow_id: None,
        event_name: event_name.to_string(),
        event_data_json: Some(event_data_json),
        processed: false,
        created_at: chrono::Utc::now(),
    };
    
    diesel::insert_into(workflow_events::table)
        .values(&new_event)
        .execute(&mut conn)?;
    
    #[cfg(feature = "cache")]
    if let Some(redis_client) = &state.cache {
        if let Ok(mut redis_conn) = redis_client.get_multiplexed_async_connection().await {
            let channel = format!("events:{event_name}");
            let _: Result<(), _> = redis_conn.publish(&channel, new_event.id.to_string()).await;
        }
    }
    
    Ok(())
}

async fn wait_for_event(
    state: &Arc<AppState>,
    _user: &UserSession,
    event_name: &str,
    timeout_seconds: u64,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let timeout = tokio::time::Duration::from_secs(timeout_seconds);
    let start_time = std::time::Instant::now();
    
    while start_time.elapsed() < timeout {
        let mut conn = state.conn.get()?;
        
        let pending_events: Vec<WorkflowEvent> = workflow_events::table
            .filter(workflow_events::event_name.eq(event_name))
            .filter(workflow_events::processed.eq(false))
            .load(&mut conn)?;
        
        if !pending_events.is_empty() {
            diesel::update(workflow_events::table.filter(workflow_events::id.eq(pending_events[0].id)))
                .set(workflow_events::processed.eq(true))
                .execute(&mut conn)?;
            
            return Ok(true);
        }
        
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    
    publish_event(state, _user, "timeout_occurred", &serde_json::json!({
        "original_event": event_name,
        "timeout_seconds": timeout_seconds
    })).await?;
    
    Ok(false)
}
