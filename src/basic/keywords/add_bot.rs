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
use std::fmt;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
            "keyword" => Self::Keyword,
            "tool" => Self::Tool,
            "schedule" => Self::Schedule,
            "event" => Self::Event,
            "always" => Self::Always,
            _ => Self::Keyword,
        }
    }
}

impl fmt::Display for TriggerType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Keyword => write!(f, "keyword"),
            Self::Tool => write!(f, "tool"),
            Self::Schedule => write!(f, "schedule"),
            Self::Event => write!(f, "event"),
            Self::Always => write!(f, "always"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotTrigger {
    pub trigger_type: TriggerType,
    pub keywords: Option<Vec<String>>,
    pub tools: Option<Vec<String>>,
    pub schedule: Option<String>,
    pub event_name: Option<String>,
}

impl BotTrigger {
    #[must_use]
    pub fn from_keywords(keywords: Vec<String>) -> Self {
        Self {
            trigger_type: TriggerType::Keyword,
            keywords: Some(keywords),
            tools: None,
            schedule: None,
            event_name: None,
        }
    }

    #[must_use]
    pub fn from_tools(tools: Vec<String>) -> Self {
        Self {
            trigger_type: TriggerType::Tool,
            keywords: None,
            tools: Some(tools),
            schedule: None,
            event_name: None,
        }
    }

    #[must_use]
    pub fn from_schedule(cron: String) -> Self {
        Self {
            trigger_type: TriggerType::Schedule,
            keywords: None,
            tools: None,
            schedule: Some(cron),
            event_name: None,
        }
    }

    #[must_use]
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

pub fn register_bot_keywords(state: &Arc<AppState>, user: &UserSession, engine: &mut Engine) {
    if let Err(e) = add_bot_with_trigger_keyword(Arc::clone(state), user.clone(), engine) {
        log::error!("Failed to register ADD BOT WITH TRIGGER keyword: {e}");
    }
    if let Err(e) = add_bot_with_tools_keyword(Arc::clone(state), user.clone(), engine) {
        log::error!("Failed to register ADD BOT WITH TOOLS keyword: {e}");
    }
    if let Err(e) = add_bot_with_schedule_keyword(Arc::clone(state), user.clone(), engine) {
        log::error!("Failed to register ADD BOT WITH SCHEDULE keyword: {e}");
    }
    if let Err(e) = remove_bot_keyword(Arc::clone(state), user.clone(), engine) {
        log::error!("Failed to register REMOVE BOT keyword: {e}");
    }
    if let Err(e) = list_bots_keyword(Arc::clone(state), user.clone(), engine) {
        log::error!("Failed to register LIST BOTS keyword: {e}");
    }
    if let Err(e) = set_bot_priority_keyword(Arc::clone(state), user.clone(), engine) {
        log::error!("Failed to register SET BOT PRIORITY keyword: {e}");
    }
    if let Err(e) = delegate_to_keyword(Arc::clone(state), user.clone(), engine) {
        log::error!("Failed to register DELEGATE TO keyword: {e}");
    }
}

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
                "ADD BOT '{bot_name}' WITH TRIGGER '{trigger_str}' for session: {}",
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

            let (tx, rx) = std::sync::mpsc::channel();

            std::thread::spawn(move || {
                let rt = match tokio::runtime::Runtime::new() {
                    Ok(rt) => rt,
                    Err(e) => {
                        let _ = tx.send(Err(format!("Failed to create runtime: {e}")));
                        return;
                    }
                };
                let result = rt.block_on(async {
                    add_bot_to_session(&state_for_task, session_id, bot_id, &bot_name, trigger)
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
                "ADD BOT '{bot_name}' WITH TOOLS '{tools_str}' for session: {}",
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

            let (tx, rx) = std::sync::mpsc::channel();

            std::thread::spawn(move || {
                let rt = match tokio::runtime::Runtime::new() {
                    Ok(rt) => rt,
                    Err(e) => {
                        let _ = tx.send(Err(format!("Failed to create runtime: {e}")));
                        return;
                    }
                };
                let result = rt.block_on(async {
                    add_bot_to_session(&state_for_task, session_id, bot_id, &bot_name, trigger)
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
                "ADD BOT '{bot_name}' WITH SCHEDULE '{schedule}' for session: {}",
                user_clone.id
            );

            let trigger = BotTrigger::from_schedule(schedule);
            let state_for_task = Arc::clone(&state_clone);
            let session_id = user_clone.id;
            let bot_id = user_clone.bot_id;

            let (tx, rx) = std::sync::mpsc::channel();

            std::thread::spawn(move || {
                let rt = match tokio::runtime::Runtime::new() {
                    Ok(rt) => rt,
                    Err(e) => {
                        let _ = tx.send(Err(format!("Failed to create runtime: {e}")));
                        return;
                    }
                };
                let result = rt.block_on(async {
                    add_bot_to_session(&state_for_task, session_id, bot_id, &bot_name, trigger)
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

            trace!("REMOVE BOT '{bot_name}' from session: {}", user_clone.id);

            let state_for_task = Arc::clone(&state_clone);
            let session_id = user_clone.id;

            let (tx, rx) = std::sync::mpsc::channel();

            std::thread::spawn(move || {
                let rt = match tokio::runtime::Runtime::new() {
                    Ok(rt) => rt,
                    Err(e) => {
                        let _ = tx.send(Err(format!("Failed to create runtime: {e}")));
                        return;
                    }
                };
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
            let rt = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(e) => {
                    let _ = tx.send(Err(format!("Failed to create runtime: {e}")));
                    return;
                }
            };
            let result = rt.block_on(async { get_session_bots(&state_for_task, session_id).await });
            let _ = tx.send(result);
        });

        match rx.recv_timeout(std::time::Duration::from_secs(30)) {
            Ok(Ok(bots)) => {
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
            #[allow(clippy::cast_possible_truncation)]
            let priority = context
                .eval_expression_tree(&inputs[1])?
                .as_int()
                .unwrap_or(0) as i32;

            trace!(
                "SET BOT PRIORITY '{bot_name}' to {priority} for session: {}",
                user_clone.id
            );

            let state_for_task = Arc::clone(&state_clone);
            let session_id = user_clone.id;

            let (tx, rx) = std::sync::mpsc::channel();

            std::thread::spawn(move || {
                let rt = match tokio::runtime::Runtime::new() {
                    Ok(rt) => rt,
                    Err(e) => {
                        let _ = tx.send(Err(format!("Failed to create runtime: {e}")));
                        return;
                    }
                };
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

            trace!("DELEGATE TO '{bot_name}' for session: {}", user_clone.id);

            let state_for_task = Arc::clone(&state_clone);
            let session_id = user_clone.id;

            let (tx, rx) = std::sync::mpsc::channel();

            std::thread::spawn(move || {
                let rt = match tokio::runtime::Runtime::new() {
                    Ok(rt) => rt,
                    Err(e) => {
                        let _ = tx.send(Err(format!("Failed to create runtime: {e}")));
                        return;
                    }
                };
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

async fn add_bot_to_session(
    state: &AppState,
    session_id: Uuid,
    _parent_bot_id: Uuid,
    bot_name: &str,
    trigger: BotTrigger,
) -> Result<String, String> {
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;

    let bot_exists: bool = diesel::sql_query(
        "SELECT EXISTS(SELECT 1 FROM bots WHERE name = $1 AND is_active = true) as exists",
    )
    .bind::<diesel::sql_types::Text, _>(bot_name)
    .get_result::<BoolResult>(&mut *conn)
    .map(|r| r.exists)
    .unwrap_or(false);

    let bot_id: String = if bot_exists {
        diesel::sql_query("SELECT id FROM bots WHERE name = $1 AND is_active = true")
            .bind::<diesel::sql_types::Text, _>(bot_name)
            .get_result::<UuidResult>(&mut *conn)
            .map(|r| r.id)
            .map_err(|e| format!("Failed to get bot ID: {e}"))?
    } else {
        let new_bot_id = Uuid::new_v4();
        diesel::sql_query(
            "INSERT INTO bots (id, name, description, is_active, created_at)
             VALUES ($1, $2, $3, true, NOW())
             ON CONFLICT (name) DO UPDATE SET is_active = true
             RETURNING id",
        )
        .bind::<diesel::sql_types::Text, _>(new_bot_id.to_string())
        .bind::<diesel::sql_types::Text, _>(bot_name)
        .bind::<diesel::sql_types::Text, _>(format!("Bot agent: {bot_name}"))
        .execute(&mut *conn)
        .map_err(|e| format!("Failed to create bot: {e}"))?;

        new_bot_id.to_string()
    };

    let trigger_json =
        serde_json::to_string(&trigger).map_err(|e| format!("Failed to serialize trigger: {e}"))?;

    let association_id = Uuid::new_v4();
    diesel::sql_query(
        "INSERT INTO session_bots (id, session_id, bot_id, bot_name, trigger_config, priority, is_active, joined_at)
         VALUES ($1, $2, $3, $4, $5, 0, true, NOW())
         ON CONFLICT (session_id, bot_name)
         DO UPDATE SET trigger_config = $5, is_active = true, joined_at = NOW()",
    )
    .bind::<diesel::sql_types::Text, _>(association_id.to_string())
    .bind::<diesel::sql_types::Text, _>(session_id.to_string())
    .bind::<diesel::sql_types::Text, _>(bot_id)
    .bind::<diesel::sql_types::Text, _>(bot_name)
    .bind::<diesel::sql_types::Text, _>(&trigger_json)
    .execute(&mut *conn)
    .map_err(|e| format!("Failed to add bot to session: {e}"))?;

    info!(
        "Bot '{bot_name}' added to session {session_id} with trigger type: {:?}",
        trigger.trigger_type
    );

    Ok(format!("Bot '{bot_name}' added to conversation"))
}

async fn remove_bot_from_session(
    state: &AppState,
    session_id: Uuid,
    bot_name: &str,
) -> Result<String, String> {
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;

    let affected = diesel::sql_query(
        "UPDATE session_bots SET is_active = false WHERE session_id = $1 AND bot_name = $2",
    )
    .bind::<diesel::sql_types::Text, _>(session_id.to_string())
    .bind::<diesel::sql_types::Text, _>(bot_name)
    .execute(&mut *conn)
    .map_err(|e| format!("Failed to remove bot: {e}"))?;

    if affected > 0 {
        info!("Bot '{bot_name}' removed from session {session_id}");
        Ok(format!("Bot '{bot_name}' removed from conversation"))
    } else {
        Ok(format!("Bot '{bot_name}' was not in the conversation"))
    }
}

async fn get_session_bots(state: &AppState, session_id: Uuid) -> Result<Vec<SessionBot>, String> {
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;

    let results: Vec<SessionBotRow> = diesel::sql_query(
        "SELECT id, session_id, bot_id, bot_name, trigger_config, priority, is_active
         FROM session_bots
         WHERE session_id = $1 AND is_active = true
         ORDER BY priority DESC, joined_at ASC",
    )
    .bind::<diesel::sql_types::Text, _>(session_id.to_string())
    .load(&mut *conn)
    .map_err(|e| format!("Failed to get session bots: {e}"))?;

    let bots = results
        .into_iter()
        .filter_map(|row| {
            let trigger: BotTrigger =
                serde_json::from_str(&row.trigger_config).unwrap_or_else(|_| BotTrigger::always());
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

async fn set_bot_priority(
    state: &AppState,
    session_id: Uuid,
    bot_name: &str,
    priority: i32,
) -> Result<String, String> {
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;

    diesel::sql_query(
        "UPDATE session_bots SET priority = $1 WHERE session_id = $2 AND bot_name = $3",
    )
    .bind::<diesel::sql_types::Integer, _>(priority)
    .bind::<diesel::sql_types::Text, _>(session_id.to_string())
    .bind::<diesel::sql_types::Text, _>(bot_name)
    .execute(&mut *conn)
    .map_err(|e| format!("Failed to set priority: {e}"))?;

    Ok(format!("Bot '{bot_name}' priority set to {priority}"))
}

async fn delegate_to_bot(
    state: &AppState,
    session_id: Uuid,
    bot_name: &str,
) -> Result<String, String> {
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;

    let bot_config: Option<BotConfigRow> = diesel::sql_query(
        "SELECT id, name, system_prompt, model_config FROM bots WHERE name = $1 AND is_active = true",
    )
    .bind::<diesel::sql_types::Text, _>(bot_name)
    .get_result(&mut *conn)
    .ok();

    let Some(config) = bot_config else {
        return Err(format!("Bot '{bot_name}' not found"));
    };

    trace!(
        "Delegating to bot: id={}, name={}, has_system_prompt={}, has_model_config={}",
        config.id,
        config.name,
        config.system_prompt.is_some(),
        config.model_config.is_some()
    );

    diesel::sql_query("UPDATE sessions SET delegated_to = $1, delegated_at = NOW() WHERE id = $2")
        .bind::<diesel::sql_types::Text, _>(&config.id)
        .bind::<diesel::sql_types::Text, _>(session_id.to_string())
        .execute(&mut *conn)
        .map_err(|e| format!("Failed to delegate: {e}"))?;

    let response = config.system_prompt.as_ref().map_or_else(
        || format!("Conversation delegated to '{}'", config.name),
        |prompt| {
            format!(
                "Conversation delegated to '{}' (specialized: {})",
                config.name,
                prompt.chars().take(50).collect::<String>()
            )
        },
    );

    Ok(response)
}

#[must_use]
pub fn match_bot_triggers(message: &str, bots: &[SessionBot]) -> Vec<SessionBot> {
    let message_lower = message.to_lowercase();
    let mut matching_bots = Vec::new();

    for bot in bots {
        if !bot.is_active {
            continue;
        }

        let matches = match bot.trigger.trigger_type {
            TriggerType::Keyword => bot.trigger.keywords.as_ref().map_or(false, |keywords| {
                keywords
                    .iter()
                    .any(|kw| message_lower.contains(&kw.to_lowercase()))
            }),
            TriggerType::Tool | TriggerType::Schedule | TriggerType::Event => false,
            TriggerType::Always => true,
        };

        if matches {
            matching_bots.push(bot.clone());
        }
    }

    matching_bots.sort_by(|a, b| b.priority.cmp(&a.priority));
    matching_bots
}

#[must_use]
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
        assert!(matches.is_empty());
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
        assert!(matches.is_empty());
    }
}
