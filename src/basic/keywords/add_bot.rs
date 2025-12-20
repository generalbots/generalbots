//! ADD BOT keyword for multi-agent conversations
//!
//! Enables multiple bots to participate in a conversation based on triggers.
//!
//! Syntax:
//! - ADD BOT "name" WITH TRIGGER "keyword1, keyword2"
//! - ADD BOT "name" WITH TOOLS "tool1, tool2"
//! - ADD BOT "name" WITH SCHEDULE "cron_expression"
//! - REMOVE BOT "name"
//! - LIST BOTS
//! - SET BOT PRIORITY "name", priority

use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use diesel::prelude::*;
use log::{info, trace};
use rhai::{Dynamic, Engine};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Bot trigger types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TriggerType {
    Keyword,
    Tool,
    Schedule,
    Event,
    Always,
}

impl From<String> for TriggerType {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "keyword" => TriggerType::Keyword,
            "tool" => TriggerType::Tool,
            "schedule" => TriggerType::Schedule,
            "event" => TriggerType::Event,
            "always" => TriggerType::Always,
            _ => TriggerType::Keyword,
        }
    }
}

impl ToString for TriggerType {
    fn to_string(&self) -> String {
        match self {
            TriggerType::Keyword => "keyword".to_string(),
            TriggerType::Tool => "tool".to_string(),
            TriggerType::Schedule => "schedule".to_string(),
            TriggerType::Event => "event".to_string(),
            TriggerType::Always => "always".to_string(),
        }
    }
}

/// Bot trigger configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotTrigger {
    pub trigger_type: TriggerType,
    pub keywords: Option<Vec<String>>,
    pub tools: Option<Vec<String>>,
    pub schedule: Option<String>,
    pub event_name: Option<String>,
}

impl BotTrigger {
    pub fn from_keywords(keywords: Vec<String>) -> Self {
        Self {
            trigger_type: TriggerType::Keyword,
            keywords: Some(keywords),
            tools: None,
            schedule: None,
            event_name: None,
        }
    }

    pub fn from_tools(tools: Vec<String>) -> Self {
        Self {
            trigger_type: TriggerType::Tool,
            keywords: None,
            tools: Some(tools),
            schedule: None,
            event_name: None,
        }
    }

    pub fn from_schedule(cron: String) -> Self {
        Self {
            trigger_type: TriggerType::Schedule,
            keywords: None,
            tools: None,
            schedule: Some(cron),
            event_name: None,
        }
    }

    pub fn always() -> Self {
        Self {
            trigger_type: TriggerType::Always,
            keywords: None,
            tools: None,
            schedule: None,
            event_name: None,
        }
    }
}

/// Session bot association
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionBot {
    pub id: Uuid,
    pub session_id: Uuid,
    pub bot_id: Uuid,
    pub bot_name: String,
    pub trigger: BotTrigger,
    pub priority: i32,
    pub is_active: bool,
}

/// Register all bot-related keywords
pub fn register_bot_keywords(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    if let Err(e) = add_bot_with_trigger_keyword(state.clone(), user.clone(), engine) {
        log::error!("Failed to register ADD BOT WITH TRIGGER keyword: {}", e);
    }
    if let Err(e) = add_bot_with_tools_keyword(state.clone(), user.clone(), engine) {
        log::error!("Failed to register ADD BOT WITH TOOLS keyword: {}", e);
    }
    if let Err(e) = add_bot_with_schedule_keyword(state.clone(), user.clone(), engine) {
        log::error!("Failed to register ADD BOT WITH SCHEDULE keyword: {}", e);
    }
    if let Err(e) = remove_bot_keyword(state.clone(), user.clone(), engine) {
        log::error!("Failed to register REMOVE BOT keyword: {}", e);
    }
    if let Err(e) = list_bots_keyword(state.clone(), user.clone(), engine) {
        log::error!("Failed to register LIST BOTS keyword: {}", e);
    }
    if let Err(e) = set_bot_priority_keyword(state.clone(), user.clone(), engine) {
        log::error!("Failed to register SET BOT PRIORITY keyword: {}", e);
    }
    if let Err(e) = delegate_to_keyword(state.clone(), user.clone(), engine) {
        log::error!("Failed to register DELEGATE TO keyword: {}", e);
    }
}

/// ADD BOT "name" WITH TRIGGER "keywords"
fn add_bot_with_trigger_keyword(
    state: Arc<AppState>,
    user: UserSession,
    engine: &mut Engine,
) -> Result<(), rhai::ParseError> {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine.register_custom_syntax(
        &["ADD", "BOT", "$expr$", "WITH", "TRIGGER", "$expr$"],
        false,
        move |context, inputs| {
            let bot_name = context
                .eval_expression_tree(&inputs[0])?
                .to_string()
                .trim_matches('"')
                .to_string();
            let trigger_str = context
                .eval_expression_tree(&inputs[1])?
                .to_string()
                .trim_matches('"')
                .to_string();

            trace!(
                "ADD BOT '{}' WITH TRIGGER '{}' for session: {}",
                bot_name,
                trigger_str,
                user_clone.id
            );

            let keywords: Vec<String> = trigger_str
                .split(',')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect();

            let trigger = BotTrigger::from_keywords(keywords);
            let state_for_task = Arc::clone(&state_clone);
            let session_id = user_clone.id;
            let bot_id = user_clone.bot_id;
            let bot_name_clone = bot_name.clone();

            let (tx, rx) = std::sync::mpsc::channel();

            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
                let result = rt.block_on(async {
                    add_bot_to_session(
                        &state_for_task,
                        session_id,
                        bot_id,
                        &bot_name_clone,
                        trigger,
                    )
                    .await
                });
                let _ = tx.send(result);
            });

            match rx.recv_timeout(std::time::Duration::from_secs(30)) {
                Ok(Ok(msg)) => Ok(Dynamic::from(msg)),
                Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    e.into(),
                    rhai::Position::NONE,
                ))),
                Err(_) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    "ADD BOT timed out".into(),
                    rhai::Position::NONE,
                ))),
            }
        },
    )?;
    Ok(())
}

/// ADD BOT "name" WITH TOOLS "tool1, tool2"
fn add_bot_with_tools_keyword(
    state: Arc<AppState>,
    user: UserSession,
    engine: &mut Engine,
) -> Result<(), rhai::ParseError> {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine.register_custom_syntax(
        &["ADD", "BOT", "$expr$", "WITH", "TOOLS", "$expr$"],
        false,
        move |context, inputs| {
            let bot_name = context
                .eval_expression_tree(&inputs[0])?
                .to_string()
                .trim_matches('"')
                .to_string();
            let tools_str = context
                .eval_expression_tree(&inputs[1])?
                .to_string()
                .trim_matches('"')
                .to_string();

            trace!(
                "ADD BOT '{}' WITH TOOLS '{}' for session: {}",
                bot_name,
                tools_str,
                user_clone.id
            );

            let tools: Vec<String> = tools_str
                .split(',')
                .map(|s| s.trim().to_uppercase())
                .filter(|s| !s.is_empty())
                .collect();

            let trigger = BotTrigger::from_tools(tools);
            let state_for_task = Arc::clone(&state_clone);
            let session_id = user_clone.id;
            let bot_id = user_clone.bot_id;
            let bot_name_clone = bot_name.clone();

            let (tx, rx) = std::sync::mpsc::channel();

            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
                let result = rt.block_on(async {
                    add_bot_to_session(
                        &state_for_task,
                        session_id,
                        bot_id,
                        &bot_name_clone,
                        trigger,
                    )
                    .await
                });
                let _ = tx.send(result);
            });

            match rx.recv_timeout(std::time::Duration::from_secs(30)) {
                Ok(Ok(msg)) => Ok(Dynamic::from(msg)),
                Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    e.into(),
                    rhai::Position::NONE,
                ))),
                Err(_) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    "ADD BOT timed out".into(),
                    rhai::Position::NONE,
                ))),
            }
        },
    )?;
    Ok(())
}

/// ADD BOT "name" WITH SCHEDULE "cron"
fn add_bot_with_schedule_keyword(
    state: Arc<AppState>,
    user: UserSession,
    engine: &mut Engine,
) -> Result<(), rhai::ParseError> {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine.register_custom_syntax(
        &["ADD", "BOT", "$expr$", "WITH", "SCHEDULE", "$expr$"],
        false,
        move |context, inputs| {
            let bot_name = context
                .eval_expression_tree(&inputs[0])?
                .to_string()
                .trim_matches('"')
                .to_string();
            let schedule = context
                .eval_expression_tree(&inputs[1])?
                .to_string()
                .trim_matches('"')
                .to_string();

            trace!(
                "ADD BOT '{}' WITH SCHEDULE '{}' for session: {}",
                bot_name,
                schedule,
                user_clone.id
            );

            let trigger = BotTrigger::from_schedule(schedule);
            let state_for_task = Arc::clone(&state_clone);
            let session_id = user_clone.id;
            let bot_id = user_clone.bot_id;
            let bot_name_clone = bot_name.clone();

            let (tx, rx) = std::sync::mpsc::channel();

            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
                let result = rt.block_on(async {
                    add_bot_to_session(
                        &state_for_task,
                        session_id,
                        bot_id,
                        &bot_name_clone,
                        trigger,
                    )
                    .await
                });
                let _ = tx.send(result);
            });

            match rx.recv_timeout(std::time::Duration::from_secs(30)) {
                Ok(Ok(msg)) => Ok(Dynamic::from(msg)),
                Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    e.into(),
                    rhai::Position::NONE,
                ))),
                Err(_) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    "ADD BOT timed out".into(),
                    rhai::Position::NONE,
                ))),
            }
        },
    )?;
    Ok(())
}

/// REMOVE BOT "name"
fn remove_bot_keyword(
    state: Arc<AppState>,
    user: UserSession,
    engine: &mut Engine,
) -> Result<(), rhai::ParseError> {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine.register_custom_syntax(
        &["REMOVE", "BOT", "$expr$"],
        false,
        move |context, inputs| {
            let bot_name = context
                .eval_expression_tree(&inputs[0])?
                .to_string()
                .trim_matches('"')
                .to_string();

            trace!("REMOVE BOT '{}' from session: {}", bot_name, user_clone.id);

            let state_for_task = Arc::clone(&state_clone);
            let session_id = user_clone.id;

            let (tx, rx) = std::sync::mpsc::channel();

            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
                let result = rt.block_on(async {
                    remove_bot_from_session(&state_for_task, session_id, &bot_name).await
                });
                let _ = tx.send(result);
            });

            match rx.recv_timeout(std::time::Duration::from_secs(30)) {
                Ok(Ok(msg)) => Ok(Dynamic::from(msg)),
                Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    e.into(),
                    rhai::Position::NONE,
                ))),
                Err(_) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    "REMOVE BOT timed out".into(),
                    rhai::Position::NONE,
                ))),
            }
        },
    )?;
    Ok(())
}

/// LIST BOTS
fn list_bots_keyword(
    state: Arc<AppState>,
    user: UserSession,
    engine: &mut Engine,
) -> Result<(), rhai::ParseError> {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine.register_custom_syntax(&["LIST", "BOTS"], false, move |_context, _inputs| {
        trace!("LIST BOTS for session: {}", user_clone.id);

        let state_for_task = Arc::clone(&state_clone);
        let session_id = user_clone.id;

        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
            let result = rt.block_on(async { get_session_bots(&state_for_task, session_id).await });
            let _ = tx.send(result);
        });

        match rx.recv_timeout(std::time::Duration::from_secs(30)) {
            Ok(Ok(bots)) => {
                // Convert to Dynamic array
                let bot_list: Vec<Dynamic> = bots
                    .into_iter()
                    .map(|b| {
                        let mut map = rhai::Map::new();
                        map.insert("name".into(), Dynamic::from(b.bot_name));
                        map.insert("priority".into(), Dynamic::from(b.priority));
                        map.insert(
                            "trigger_type".into(),
                            Dynamic::from(b.trigger.trigger_type.to_string()),
                        );
                        map.insert("is_active".into(), Dynamic::from(b.is_active));
                        Dynamic::from(map)
                    })
                    .collect();
                Ok(Dynamic::from(bot_list))
            }
            Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                e.into(),
                rhai::Position::NONE,
            ))),
            Err(_) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                "LIST BOTS timed out".into(),
                rhai::Position::NONE,
            ))),
        }
    })?;
    Ok(())
}

/// SET BOT PRIORITY "name", priority
fn set_bot_priority_keyword(
    state: Arc<AppState>,
    user: UserSession,
    engine: &mut Engine,
) -> Result<(), rhai::ParseError> {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine.register_custom_syntax(
        &["SET", "BOT", "PRIORITY", "$expr$", ",", "$expr$"],
        false,
        move |context, inputs| {
            let bot_name = context
                .eval_expression_tree(&inputs[0])?
                .to_string()
                .trim_matches('"')
                .to_string();
            let priority = context
                .eval_expression_tree(&inputs[1])?
                .as_int()
                .unwrap_or(0) as i32;

            trace!(
                "SET BOT PRIORITY '{}' to {} for session: {}",
                bot_name,
                priority,
                user_clone.id
            );

            let state_for_task = Arc::clone(&state_clone);
            let session_id = user_clone.id;

            let (tx, rx) = std::sync::mpsc::channel();

            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
                let result = rt.block_on(async {
                    set_bot_priority(&state_for_task, session_id, &bot_name, priority).await
                });
                let _ = tx.send(result);
            });

            match rx.recv_timeout(std::time::Duration::from_secs(30)) {
                Ok(Ok(msg)) => Ok(Dynamic::from(msg)),
                Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    e.into(),
                    rhai::Position::NONE,
                ))),
                Err(_) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    "SET BOT PRIORITY timed out".into(),
                    rhai::Position::NONE,
                ))),
            }
        },
    )?;
    Ok(())
}

/// DELEGATE TO "bot" WITH CONTEXT
fn delegate_to_keyword(
    state: Arc<AppState>,
    user: UserSession,
    engine: &mut Engine,
) -> Result<(), rhai::ParseError> {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine.register_custom_syntax(
        &["DELEGATE", "TO", "$expr$"],
        false,
        move |context, inputs| {
            let bot_name = context
                .eval_expression_tree(&inputs[0])?
                .to_string()
                .trim_matches('"')
                .to_string();

            trace!("DELEGATE TO '{}' for session: {}", bot_name, user_clone.id);

            let state_for_task = Arc::clone(&state_clone);
            let session_id = user_clone.id;

            let (tx, rx) = std::sync::mpsc::channel();

            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
                let result = rt.block_on(async {
                    delegate_to_bot(&state_for_task, session_id, &bot_name).await
                });
                let _ = tx.send(result);
            });

            match rx.recv_timeout(std::time::Duration::from_secs(60)) {
                Ok(Ok(response)) => Ok(Dynamic::from(response)),
                Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    e.into(),
                    rhai::Position::NONE,
                ))),
                Err(_) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    "DELEGATE TO timed out".into(),
                    rhai::Position::NONE,
                ))),
            }
        },
    )?;
    Ok(())
}

// Database Operations

/// Add a bot to the session
async fn add_bot_to_session(
    state: &AppState,
    session_id: Uuid,
    _parent_bot_id: Uuid,
    bot_name: &str,
    trigger: BotTrigger,
) -> Result<String, String> {
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {}", e))?;

    // Check if bot exists
    let bot_exists: bool = diesel::sql_query(
        "SELECT EXISTS(SELECT 1 FROM bots WHERE name = $1 AND is_active = true) as exists",
    )
    .bind::<diesel::sql_types::Text, _>(bot_name)
    .get_result::<BoolResult>(&mut *conn)
    .map(|r| r.exists)
    .unwrap_or(false);

    // If bot doesn't exist, try to find it in templates or create a placeholder
    let bot_id: String = if bot_exists {
        diesel::sql_query("SELECT id FROM bots WHERE name = $1 AND is_active = true")
            .bind::<diesel::sql_types::Text, _>(bot_name)
            .get_result::<UuidResult>(&mut *conn)
            .map(|r| r.id)
            .map_err(|e| format!("Failed to get bot ID: {}", e))?
    } else {
        // Create a new bot entry
        let new_bot_id = Uuid::new_v4();
        diesel::sql_query(
            "INSERT INTO bots (id, name, description, is_active, created_at)
             VALUES ($1, $2, $3, true, NOW())
             ON CONFLICT (name) DO UPDATE SET is_active = true
             RETURNING id",
        )
        .bind::<diesel::sql_types::Text, _>(new_bot_id.to_string())
        .bind::<diesel::sql_types::Text, _>(bot_name)
        .bind::<diesel::sql_types::Text, _>(format!("Bot agent: {}", bot_name))
        .execute(&mut *conn)
        .map_err(|e| format!("Failed to create bot: {}", e))?;

        new_bot_id.to_string()
    };

    // Serialize trigger to JSON
    let trigger_json = serde_json::to_string(&trigger)
        .map_err(|e| format!("Failed to serialize trigger: {}", e))?;

    // Add bot to session
    let association_id = Uuid::new_v4();
    diesel::sql_query(
        "INSERT INTO session_bots (id, session_id, bot_id, bot_name, trigger_config, priority, is_active, joined_at)
         VALUES ($1, $2, $3, $4, $5, 0, true, NOW())
         ON CONFLICT (session_id, bot_name)
         DO UPDATE SET trigger_config = $5, is_active = true, joined_at = NOW()",
    )
    .bind::<diesel::sql_types::Text, _>(association_id.to_string())
    .bind::<diesel::sql_types::Text, _>(session_id.to_string())
    .bind::<diesel::sql_types::Text, _>(bot_id.to_string())
    .bind::<diesel::sql_types::Text, _>(bot_name)
    .bind::<diesel::sql_types::Text, _>(&trigger_json)
    .execute(&mut *conn)
    .map_err(|e| format!("Failed to add bot to session: {}", e))?;

    info!(
        "Bot '{}' added to session {} with trigger type: {:?}",
        bot_name, session_id, trigger.trigger_type
    );

    Ok(format!("Bot '{}' added to conversation", bot_name))
}

/// Remove a bot from the session
async fn remove_bot_from_session(
    state: &AppState,
    session_id: Uuid,
    bot_name: &str,
) -> Result<String, String> {
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {}", e))?;

    let affected = diesel::sql_query(
        "UPDATE session_bots SET is_active = false WHERE session_id = $1 AND bot_name = $2",
    )
    .bind::<diesel::sql_types::Text, _>(session_id.to_string())
    .bind::<diesel::sql_types::Text, _>(bot_name)
    .execute(&mut *conn)
    .map_err(|e| format!("Failed to remove bot: {}", e))?;

    if affected > 0 {
        info!("Bot '{}' removed from session {}", bot_name, session_id);
        Ok(format!("Bot '{}' removed from conversation", bot_name))
    } else {
        Ok(format!("Bot '{}' was not in the conversation", bot_name))
    }
}

/// Get all bots in a session
async fn get_session_bots(state: &AppState, session_id: Uuid) -> Result<Vec<SessionBot>, String> {
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {}", e))?;

    let results: Vec<SessionBotRow> = diesel::sql_query(
        "SELECT id, session_id, bot_id, bot_name, trigger_config, priority, is_active
         FROM session_bots
         WHERE session_id = $1 AND is_active = true
         ORDER BY priority DESC, joined_at ASC",
    )
    .bind::<diesel::sql_types::Text, _>(session_id.to_string())
    .load(&mut *conn)
    .map_err(|e| format!("Failed to get session bots: {}", e))?;

    let bots = results
        .into_iter()
        .filter_map(|row| {
            let trigger: BotTrigger =
                serde_json::from_str(&row.trigger_config).unwrap_or(BotTrigger::always());
            Some(SessionBot {
                id: Uuid::parse_str(&row.id).ok()?,
                session_id: Uuid::parse_str(&row.session_id).ok()?,
                bot_id: Uuid::parse_str(&row.bot_id).ok()?,
                bot_name: row.bot_name,
                trigger,
                priority: row.priority,
                is_active: row.is_active,
            })
        })
        .collect();

    Ok(bots)
}

/// Set bot priority in session
async fn set_bot_priority(
    state: &AppState,
    session_id: Uuid,
    bot_name: &str,
    priority: i32,
) -> Result<String, String> {
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {}", e))?;

    diesel::sql_query(
        "UPDATE session_bots SET priority = $1 WHERE session_id = $2 AND bot_name = $3",
    )
    .bind::<diesel::sql_types::Integer, _>(priority)
    .bind::<diesel::sql_types::Text, _>(session_id.to_string())
    .bind::<diesel::sql_types::Text, _>(bot_name)
    .execute(&mut *conn)
    .map_err(|e| format!("Failed to set priority: {}", e))?;

    Ok(format!("Bot '{}' priority set to {}", bot_name, priority))
}

/// Delegate current conversation to another bot
async fn delegate_to_bot(
    state: &AppState,
    session_id: Uuid,
    bot_name: &str,
) -> Result<String, String> {
    // Get the bot's configuration
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {}", e))?;

    let bot_config: Option<BotConfigRow> = diesel::sql_query(
        "SELECT id, name, system_prompt, model_config FROM bots WHERE name = $1 AND is_active = true",
    )
    .bind::<diesel::sql_types::Text, _>(bot_name)
    .get_result(&mut *conn)
    .ok();

    let config = match bot_config {
        Some(cfg) => cfg,
        None => return Err(format!("Bot '{}' not found", bot_name)),
    };

    // Log delegation details for debugging
    trace!(
        "Delegating to bot: id={}, name={}, has_system_prompt={}, has_model_config={}",
        config.id,
        config.name,
        config.system_prompt.is_some(),
        config.model_config.is_some()
    );

    // Mark delegation in session with bot ID for proper tracking
    diesel::sql_query("UPDATE sessions SET delegated_to = $1, delegated_at = NOW() WHERE id = $2")
        .bind::<diesel::sql_types::Text, _>(&config.id)
        .bind::<diesel::sql_types::Text, _>(session_id.to_string())
        .execute(&mut *conn)
        .map_err(|e| format!("Failed to delegate: {}", e))?;

    // Build response message with bot info
    let response = if let Some(ref prompt) = config.system_prompt {
        format!(
            "Conversation delegated to '{}' (specialized: {})",
            config.name,
            prompt.chars().take(50).collect::<String>()
        )
    } else {
        format!("Conversation delegated to '{}'", config.name)
    };

    Ok(response)
}

// Multi-Agent Message Processing

/// Check if a message matches any bot triggers
pub fn match_bot_triggers(message: &str, bots: &[SessionBot]) -> Vec<SessionBot> {
    let message_lower = message.to_lowercase();
    let mut matching_bots = Vec::new();

    for bot in bots {
        if !bot.is_active {
            continue;
        }

        let matches = match bot.trigger.trigger_type {
            TriggerType::Keyword => {
                if let Some(keywords) = &bot.trigger.keywords {
                    keywords
                        .iter()
                        .any(|kw| message_lower.contains(&kw.to_lowercase()))
                } else {
                    false
                }
            }
            TriggerType::Tool => {
                // Tool triggers are checked separately when tools are invoked
                false
            }
            TriggerType::Schedule => {
                // Schedule triggers are checked by the scheduler
                false
            }
            TriggerType::Event => {
                // Event triggers are checked when events occur
                false
            }
            TriggerType::Always => true,
        };

        if matches {
            matching_bots.push(bot.clone());
        }
    }

    // Sort by priority (higher first)
    matching_bots.sort_by(|a, b| b.priority.cmp(&a.priority));
    matching_bots
}

/// Check if a tool invocation matches any bot triggers
pub fn match_tool_triggers(tool_name: &str, bots: &[SessionBot]) -> Vec<SessionBot> {
    let tool_upper = tool_name.to_uppercase();
    let mut matching_bots = Vec::new();

    for bot in bots {
        if !bot.is_active {
            continue;
        }

        if bot.trigger.trigger_type == TriggerType::Tool {
            if let Some(tools) = &bot.trigger.tools {
                if tools.iter().any(|t| t.to_uppercase() == tool_upper) {
                    matching_bots.push(bot.clone());
                }
            }
        }
    }

    matching_bots.sort_by(|a, b| b.priority.cmp(&a.priority));
    matching_bots
}

// Helper Types for Diesel Queries

#[derive(QueryableByName)]
struct BoolResult {
    #[diesel(sql_type = diesel::sql_types::Bool)]
    exists: bool,
}

#[derive(QueryableByName)]
struct UuidResult {
    #[diesel(sql_type = diesel::sql_types::Text)]
    id: String,
}

#[derive(QueryableByName)]
struct SessionBotRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    id: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    session_id: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    bot_id: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    bot_name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    trigger_config: String,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    priority: i32,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    is_active: bool,
}

#[derive(QueryableByName)]
struct BotConfigRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    id: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    system_prompt: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    model_config: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trigger_from_keywords() {
        let trigger = BotTrigger::from_keywords(vec!["finance".to_string(), "money".to_string()]);
        assert_eq!(trigger.trigger_type, TriggerType::Keyword);
        assert_eq!(trigger.keywords.unwrap().len(), 2);
    }

    #[test]
    fn test_match_bot_triggers() {
        let bots = vec![
            SessionBot {
                id: Uuid::new_v4(),
                session_id: Uuid::new_v4(),
                bot_id: Uuid::new_v4(),
                bot_name: "finance-bot".to_string(),
                trigger: BotTrigger::from_keywords(vec!["money".to_string(), "budget".to_string()]),
                priority: 1,
                is_active: true,
            },
            SessionBot {
                id: Uuid::new_v4(),
                session_id: Uuid::new_v4(),
                bot_id: Uuid::new_v4(),
                bot_name: "hr-bot".to_string(),
                trigger: BotTrigger::from_keywords(vec![
                    "vacation".to_string(),
                    "employee".to_string(),
                ]),
                priority: 0,
                is_active: true,
            },
        ];

        let matches = match_bot_triggers("How much money do I have?", &bots);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].bot_name, "finance-bot");

        let matches = match_bot_triggers("I need to request vacation", &bots);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].bot_name, "hr-bot");

        let matches = match_bot_triggers("Hello world", &bots);
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_match_tool_triggers() {
        let bots = vec![SessionBot {
            id: Uuid::new_v4(),
            session_id: Uuid::new_v4(),
            bot_id: Uuid::new_v4(),
            bot_name: "data-bot".to_string(),
            trigger: BotTrigger::from_tools(vec!["AGGREGATE".to_string(), "CHART".to_string()]),
            priority: 1,
            is_active: true,
        }];

        let matches = match_tool_triggers("aggregate", &bots);
        assert_eq!(matches.len(), 1);

        let matches = match_tool_triggers("SEND", &bots);
        assert_eq!(matches.len(), 0);
    }
}
