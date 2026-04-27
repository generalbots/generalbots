#[cfg(any(feature = "research", feature = "llm"))]
pub mod kb_context;
#[cfg(any(feature = "research", feature = "llm"))]
use kb_context::inject_kb_context;
pub mod tool_context;
use tool_context::get_session_tools;
pub mod tool_executor;
use tool_executor::ToolExecutor;
#[cfg(feature = "llm")]
use crate::core::config::ConfigManager;

#[cfg(feature = "drive")]
use crate::drive::drive_monitor::{DriveMonitor};
#[cfg(feature = "llm")]
use crate::llm::llm_models;
#[cfg(feature = "llm")]
use crate::llm::OpenAIClient;
#[cfg(feature = "nvidia")]
use crate::nvidia::get_system_metrics;
use crate::core::shared::message_types::MessageType;
use crate::core::shared::models::{BotResponse, UserMessage, UserSession};
#[cfg(not(feature = "chat"))]
use crate::core::shared::models::Switcher;
use crate::core::shared::state::AppState;
#[cfg(feature = "chat")]
use crate::basic::keywords::add_suggestion::get_suggestions;
#[cfg(feature = "chat")]
use crate::basic::keywords::switcher::{get_switchers, resolve_active_switchers};
use html2md::parse_html;

use axum::extract::ws::{Message, WebSocket};
use axum::{
    extract::{ws::WebSocketUpgrade, Extension, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use diesel::ExpressionMethods;
use diesel::PgConnection;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use futures::{sink::SinkExt, stream::StreamExt};
#[cfg(feature = "llm")]
use log::trace;
use log::{debug, error, info, warn};
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::broadcast;
use regex;
#[cfg(feature = "drive")]
#[cfg(feature = "drive")]
use tokio::sync::Mutex as AsyncMutex;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use diesel::OptionalExtension;

pub mod channels;
pub mod multimedia;

/// Check if a user has access to a bot
/// Returns Ok(()) if access is allowed, Err with status code and message if not
pub fn check_bot_access(
    conn: &mut PgConnection,
    bot_id: Uuid,
    user_id: Uuid,
) -> Result<(), (StatusCode, String)> {
    use crate::core::shared::models::schema::bots;

    if bot_id == Uuid::nil() {
        return Err((StatusCode::NOT_FOUND, "Bot not found".to_string()));
    }

    let bot_result = bots::table
        .filter(bots::id.eq(bot_id))
        .select((bots::is_public, bots::org_id))
        .first::<(bool, Option<Uuid>)>(conn)
        .optional();

    match bot_result {
        Ok(Some((public, bot_org_id))) => {
            if public {
                return Ok(());
            }

            if let Some(org_id) = bot_org_id {
                use crate::core::shared::models::schema::user_organizations;

                let is_member = user_organizations::table
                    .filter(user_organizations::user_id.eq(user_id))
                    .filter(user_organizations::org_id.eq(org_id))
                    .select(user_organizations::id)
                    .first::<Uuid>(conn)
                    .optional()
                    .unwrap_or(None)
                    .is_some();

                if is_member {
                    return Ok(());
                }

                Err((StatusCode::FORBIDDEN, "Access denied - not a member of this organization".to_string()))
            } else {
                Err((StatusCode::FORBIDDEN, "Access denied - bot is private".to_string()))
            }
        }
        _ => Err((StatusCode::NOT_FOUND, "Bot not found".to_string())),
    }
}

pub fn get_default_bot(conn: &mut PgConnection) -> (Uuid, String) {
    use crate::core::shared::models::schema::bots::dsl::*;
    use diesel::prelude::*;

    // First try to get the bot named "default"
    match bots
        .filter(name.eq("default"))
        .filter(is_active.eq(true))
        .select((id, name))
        .first::<(Uuid, String)>(conn)
        .optional()
    {
        Ok(Some((bot_id, bot_name))) => (bot_id, bot_name),
        Ok(None) => {
            warn!("Bot named 'default' not found, falling back to first active bot");
            // Fall back to first active bot
            match bots
                .filter(is_active.eq(true))
                .select((id, name))
                .first::<(Uuid, String)>(conn)
                .optional()
            {
                Ok(Some((bot_id, bot_name))) => (bot_id, bot_name),
                Ok(None) => {
                    warn!("No active bots found, using nil UUID");
                    (Uuid::nil(), "default".to_string())
                }
                Err(e) => {
                    error!("Failed to query fallback bot: {}", e);
                    (Uuid::nil(), "default".to_string())
                }
            }
        }
        Err(e) => {
            error!("Failed to query default bot: {}", e);
            (Uuid::nil(), "default".to_string())
        }
    }
}

/// Get bot ID by name from database
pub fn get_bot_id_by_name(conn: &mut PgConnection, bot_name: &str) -> Result<Uuid, String> {
    use crate::core::shared::models::schema::bots::dsl::*;
    use diesel::prelude::*;

    bots
        .filter(name.eq(bot_name))
        .select(id)
        .first::<Uuid>(conn)
        .map_err(|e| format!("Bot '{}' not found: {}", bot_name, e))
}

#[derive(Debug)]
pub struct BotOrchestrator {
    pub state: Arc<AppState>,
    #[cfg(feature = "drive")]
    pub mounted_bots: Arc<AsyncMutex<HashMap<String, Arc<DriveMonitor>>>>,
}

#[derive(Debug, Deserialize)]
pub struct BotConfigQuery {
    pub bot_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BotConfigResponse {
    pub public: bool,
    pub theme_color1: Option<String>,
    pub theme_color2: Option<String>,
    pub theme_title: Option<String>,
    pub theme_logo: Option<String>,
    pub theme_logo_text: Option<String>,
}

/// Get bot configuration endpoint
/// Returns bot's public setting and other configuration values
pub async fn get_bot_config(
    Query(params): Query<BotConfigQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<BotConfigResponse>, StatusCode> {
    let bot_name = params.bot_name.unwrap_or_else(|| "default".to_string());

    let mut conn = match state.conn.get() {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Get bot_id and is_public from bots table
    let (target_bot_id, is_public) = match get_bot_id_by_name(&mut conn, &bot_name) {
        Ok(found_id) => {
            // Query is_public from bots table
            use crate::core::shared::models::schema::bots::dsl::*;
            use diesel::OptionalExtension;
            let public_result = bots
                .filter(id.eq(found_id))
                .select(is_public)
                .first::<bool>(&mut conn)
                .optional();

            match public_result {
                Ok(Some(p)) => (found_id, p),
                Ok(None) => (found_id, false),
                Err(e) => {
                    warn!("Failed to query is_public for bot '{}': {}", bot_name, e);
                    (found_id, false)
                }
            }
        }
        Err(e) => {
            warn!("Failed to find bot ID for name '{}': {}", bot_name, e);
            return Ok(Json(BotConfigResponse {
                public: false,
                theme_color1: None,
                theme_color2: None,
                theme_title: None,
                theme_logo: None,
                theme_logo_text: None,
            }));
        }
    };

    let mut theme_color1: Option<String> = None;
    let mut theme_color2: Option<String> = None;
    let mut theme_title: Option<String> = None;
    let mut theme_logo: Option<String> = None;
    let mut theme_logo_text: Option<String> = None;

    // Query theme config values from bot_configuration table
    use crate::core::shared::models::schema::bot_configuration::dsl::*;

    match bot_configuration
        .filter(bot_id.eq(target_bot_id))
        .select((config_key, config_value))
        .load::<(String, String)>(&mut conn)
    {
        Ok(configs) => {
            info!("Config query returned {} entries for bot '{}'", configs.len(), bot_name);
            for (key, value) in configs {
                // Try to strip bot_name prefix, use original if no prefix
                let clean_key = key.strip_prefix(&format!("{}.", bot_name))
                    .or_else(|| key.strip_prefix(&format!("{}_", bot_name)))
                    .unwrap_or(&key);

                // Check if key is for this bot (either prefixed or not)
                let key_for_bot = clean_key == key || key.starts_with(&format!("{}.", bot_name)) || key.starts_with(&format!("{}_", bot_name));

                info!("Key '{}' -> clean_key '{}' -> key_for_bot: {}", key, clean_key, key_for_bot);

                if !key_for_bot {
                    info!("Skipping key '{}' - not for bot '{}'", key, bot_name);
                    continue;
                }

                match clean_key.to_lowercase().as_str() {
                    "public" => {
                        // Also check config table for backward compatibility
                        // But is_public from bots table takes precedence
                        info!("Found 'public' in config table: {}", value);
                    }
                    "theme-color1" => {
                        theme_color1 = Some(value);
                    }
                    "theme-color2" => {
                        theme_color2 = Some(value);
                    }
                    "theme-title" => {
                        theme_title = Some(value);
                    }
                    "theme-logo" => {
                        theme_logo = Some(value);
                    }
                    "theme-logo-text" => {
                        theme_logo_text = Some(value);
                    }
                    _ => {}
                }
            }
            info!("Retrieved config for bot '{}': public={}, theme_color1={:?}, theme_color2={:?}, theme_title={:?}",
                bot_name, is_public, theme_color1, theme_color2, theme_title);
        }
        Err(e) => {
            warn!("Failed to load config for bot '{}': {}", bot_name, e);
            // Return defaults (not public, no theme)
        }
    }

    let config_response = BotConfigResponse {
        public: is_public,
        theme_color1,
        theme_color2,
        theme_title,
        theme_logo,
        theme_logo_text,
    };

    Ok(Json(config_response))
}

impl BotOrchestrator {
    pub fn new(state: Arc<AppState>) -> Self {
        Self {
            state,
            #[cfg(feature = "drive")]
            mounted_bots: Arc::new(AsyncMutex::new(HashMap::new())),
        }
    }

    pub fn mount_all_bots(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Scanning drive for .gbai files to mount bots...");

        let mut bots_mounted = 0;

        let directories_to_scan: Vec<std::path::PathBuf> = vec![
            self.state
                .config
                .as_ref()
                .map(|c| c.site_path.clone())
                .unwrap_or_else(|| format!("{}/sites", crate::core::shared::utils::get_stack_path()))
                .into(),
            "./templates".into(),
            "../bottemplates".into(),
        ];

        for dir_path in directories_to_scan {
            info!("Checking directory for bots: {}", dir_path.display());

            if !dir_path.exists() {
                info!("Directory does not exist, skipping: {}", dir_path.display());
                continue;
            }

            match self.scan_directory(&dir_path, &mut bots_mounted) {
                Ok(()) => {}
                Err(e) => {
                    error!("Failed to scan directory {}: {}", dir_path.display(), e);
                }
            }
        }

        info!(
            "BotServer ready - {} bots loaded",
            bots_mounted
        );
        log::debug!(
            "Bot mounting details: {} bots mounted",
            bots_mounted
        );

        Ok(())
    }

    fn scan_directory(
        &self,
        dir_path: &std::path::Path,
        bots_mounted: &mut i32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let entries =
            std::fs::read_dir(dir_path).map_err(|e| format!("Failed to read directory: {}", e))?;

        for entry in entries.flatten() {
            let name = entry.file_name();

            let bot_name = match name.to_str() {
                Some(n) if n.ends_with(".gbai") => n.trim_end_matches(".gbai"),
                _ => continue,
            };

            info!("Found .gbai file: {}", bot_name);

            match self.ensure_bot_exists(bot_name) {
                Ok(true) => {
                    info!("Bot '{}' already exists in database, mounting", bot_name);
                    *bots_mounted += 1;
                }
                Ok(false) => {
                    {
                        info!(
                            "Bot '{}' does not exist in database, skipping (run import to create)",
                            bot_name
                        );
                    }
                }
                Err(e) => {
                    error!("Failed to check if bot '{}' exists: {}", bot_name, e);
                }
            }
        }

        Ok(())
    }

    fn ensure_bot_exists(
        &self,
        bot_name: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        use diesel::sql_query;

        let mut conn = self
            .state
            .conn
            .get()
            .map_err(|e| format!("Failed to get database connection: {e}"))?;

        #[derive(diesel::QueryableByName)]
        #[diesel(check_for_backend(diesel::pg::Pg))]
        struct BotExistsResult {
            #[diesel(sql_type = diesel::sql_types::Bool)]
            exists: bool,
        }

        let exists: BotExistsResult = sql_query(
            "SELECT EXISTS(SELECT 1 FROM bots WHERE name = $1 AND is_active = true) as exists",
        )
        .bind::<diesel::sql_types::Text, _>(bot_name)
        .get_result(&mut conn)
        .map_err(|e| format!("Failed to check if bot exists: {e}"))?;

        Ok(exists.exists)
    }


#[cfg(feature = "llm")]
pub async fn stream_response(
    &self,
    mut message: UserMessage,
    response_tx: mpsc::Sender<BotResponse>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        trace!(
            "Streaming response for user: {}, session: {}",
            message.user_id,
            message.session_id
        );

    let user_id = Uuid::parse_str(&message.user_id)?;
    let session_id = Uuid::parse_str(&message.session_id)?;

// Handle direct tool execution via TOOL_EXEC message type (invisible to user)
    if message.message_type == MessageType::TOOL_EXEC {
        let tool_name = message.content.trim();
            if !tool_name.is_empty() {
                info!("tool_exec: Direct tool execution: {}", tool_name);
                
                // Get bot name from bot_id
                let bot_name = if let Ok(bot_uuid) = Uuid::parse_str(&message.bot_id) {
                    let conn = self.state.conn.get().ok();
                    conn.and_then(|mut db_conn| {
                        use crate::core::shared::models::schema::bots::dsl::*;
                        bots.filter(id.eq(bot_uuid))
                            .select(name)
                            .first::<String>(&mut db_conn)
                            .ok()
                    }).unwrap_or_else(|| "default".to_string())
                } else {
                    "default".to_string()
                };
                
                let tool_result = ToolExecutor::execute_tool_by_name(
                    &self.state,
                    &bot_name,
                    tool_name,
                    &session_id,
                    &user_id,
                ).await;

                let response_content = if tool_result.success {
                    tool_result.result
                } else {
                    format!("Erro ao executar '{}': {}", tool_name, tool_result.error.unwrap_or_default())
                };

    // Direct tool execution — return result immediately, no LLM call
    let mut suggestions = vec![];
    let mut switchers = vec![];
    if let Some(cache) = &self.state.cache {
        #[cfg(feature = "chat")]
        {
            // Try to restore existing suggestions so they don't disappear in the UI
            suggestions = get_suggestions(Some(cache), &message.bot_id, &message.session_id);
            switchers = get_switchers(Some(cache), &message.bot_id, &message.session_id);
        }
    }

    let final_response = BotResponse {
        bot_id: message.bot_id.clone(),
        user_id: message.user_id.clone(),
        session_id: message.session_id.clone(),
        channel: message.channel.clone(),
        content: response_content,
        message_type: MessageType::BOT_RESPONSE,
        stream_token: None,
        is_complete: true,
        suggestions,
        switchers,
        context_name: None,
        context_length: 0,
        context_max_length: 0,
    };

                let _ = response_tx.send(final_response).await;
                return Ok(());
            }
        }

    // Handle SYSTEM messages (type 7) - no longer saved to DB, just acknowledge
    if message.message_type == MessageType::SYSTEM {
        trace!("SYSTEM message received for session {} (deprecated - switchers now via active_switchers field)", session_id);
        return Ok(());
    }

    // Handle SWITCHER_TOGGLE (type 8) - user clicked a switcher chip
    // Re-process last user message with the active switchers injected into system prompt
    // Mutates message in-place to avoid recursive async call
    // Replays are NOT saved to message_history, so the DB always has the last original user question
    // When user types a new message (e.g. "faz azul"), it IS saved and becomes the new base for switchers
    let mut is_switcher_replay = false;
    if message.message_type == MessageType::SWITCHER_TOGGLE {
        let last_user_content: Option<String> = {
            let conn = self.state.conn.get().ok();
            let session_id_for_query = session_id;
            conn.and_then(|mut db_conn| {
                use crate::core::shared::models::schema::message_history::dsl::*;
                message_history
                    .filter(session_id.eq(session_id_for_query))
                    .filter(role.eq(1))
                    .order(created_at.desc())
                    .select(content_encrypted)
                    .first::<String>(&mut db_conn)
                    .ok()
            })
        };

        if let Some(last_content) = last_user_content {
            message.content = last_content;
            message.message_type = MessageType::USER;
            is_switcher_replay = true;
        } else {
            let empty_response = BotResponse {
                bot_id: message.bot_id.clone(),
                user_id: message.user_id.clone(),
                session_id: message.session_id.clone(),
                channel: message.channel.clone(),
                content: String::new(),
                message_type: MessageType::BOT_RESPONSE,
                stream_token: None,
                is_complete: true,
                suggestions: Vec::new(),
                switchers: Vec::new(),
                context_name: None,
                context_length: 0,
                context_max_length: 0,
            };
            let _ = response_tx.send(empty_response).await;
            return Ok(());
        }
    }

    let message_content = message.content.clone();

    // Legacy: Handle direct tool invocation via __TOOL__: prefix
        if message_content.starts_with("__TOOL__:") {
            let tool_name = message_content.trim_start_matches("__TOOL__:").trim();
            if !tool_name.is_empty() {
                info!("Direct tool invocation via WS: {}", tool_name);
                
                let tool_result = ToolExecutor::execute_tool_by_name(
                    &self.state,
                    &message.bot_id,
                    tool_name,
                    &session_id,
                    &user_id,
                ).await;
                
                let response_content = if tool_result.success {
                    tool_result.result
                } else {
                    format!("Erro ao executar tool '{}': {}", tool_name, tool_result.error.unwrap_or_default())
                };
                
                let final_response = BotResponse {
                    bot_id: message.bot_id.clone(),
                    user_id: message.user_id.clone(),
                    session_id: message.session_id.clone(),
                    channel: message.channel.clone(),
                    content: response_content,
                    message_type: MessageType::BOT_RESPONSE,
                    stream_token: None,
                    is_complete: true,
            suggestions: vec![],
            switchers: Vec::new(),
            context_name: None,
            context_length: 0,
            context_max_length: 0,
        };

                
                if let Err(e) = response_tx.send(final_response).await {
                    error!("Failed to send tool response: {}", e);
                }
                return Ok(());
            }
        }

        // If a HEAR is blocking the script thread for this session, deliver the input
        // directly and return — the script continues from where it paused.
        if crate::basic::keywords::hearing::deliver_hear_input(
            &self.state,
            session_id,
            message_content.clone(),
        ) {
            trace!("HEAR: delivered input to blocking script for session {session_id}");
            return Ok(());
        }

        let (session, context_data, history, model, key, system_prompt, bot_llm_url, explicit_llm_provider, bot_endpoint_path) = {
            let state_clone = self.state.clone();
            tokio::task::spawn_blocking(
                move || -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                    let mut session = {
                        let mut sm = state_clone.session_manager.blocking_lock();
                        sm.get_session_by_id(session_id)?
                    }
                    .ok_or("Session not found")?;

                    // Store WebSocket session_id in context for TALK routing
                    if let serde_json::Value::Object(ref mut map) = session.context_data {
                        map.insert("websocket_session_id".to_string(), serde_json::Value::String(session_id.to_string()));
                    } else {
                        let mut map = serde_json::Map::new();
                        map.insert("websocket_session_id".to_string(), serde_json::Value::String(session_id.to_string()));
                        session.context_data = serde_json::Value::Object(map);
                    }

    if !message.content.trim().is_empty() && !is_switcher_replay {
        let mut sm = state_clone.session_manager.blocking_lock();
        sm.save_message(session.id, user_id, 1, &message.content, 1)?;
    }

                    let context_data = {
                        let sm = state_clone.session_manager.blocking_lock();
                        sm.get_session_context_data(&session.id, &session.user_id)?
                    };

                    let config_manager = ConfigManager::new(state_clone.conn.clone());

                    let history_limit = config_manager
                        .get_bot_config_value(&session.bot_id, "history-limit")
                        .ok()
                        .and_then(|v| v.parse::<i64>().ok());

                    let history = {
                        let mut sm = state_clone.session_manager.blocking_lock();
                        sm.get_conversation_history(session.id, user_id, history_limit)?
                    };

                    // For local LLM server, use the actual model name
                    // Default to DeepSeek model if not configured
                    let model = config_manager
                        .get_config(&session.bot_id, "llm-model", Some("DeepSeek-R1-Distill-Qwen-1.5B-Q3_K_M.gguf"))
                        .unwrap_or_else(|_| "DeepSeek-R1-Distill-Qwen-1.5B-Q3_K_M.gguf".to_string());

                    let key = config_manager
                        .get_config(&session.bot_id, "llm-key", Some(""))
                        .unwrap_or_default();

                    // Load bot-specific llm-url (may differ from global default)
                    let bot_llm_url = config_manager
                        .get_bot_config_value(&session.bot_id, "llm-url")
                        .ok();

                    // Load explicit llm-provider from config.csv (e.g., "openai", "bedrock", "claude")
                    // This allows overriding auto-detection from URL
                    let explicit_llm_provider = config_manager
                        .get_bot_config_value(&session.bot_id, "llm-provider")
                        .ok();

                    // Load bot-specific llm-endpoint-path
                    let bot_endpoint_path = config_manager
                        .get_bot_config_value(&session.bot_id, "llm-endpoint-path")
                        .ok();

                    // Load system-prompt from config.csv, fallback to default
                    // Load system-prompt: auto-detect PROMPT.md, PROMPT.txt, prompt.md, prompt.txt in .gbot folder
                    // Ignore system-prompt-file config to avoid double .gbot path bug
                    let bot_id = session.bot_id;
                    let bot_name = {
                        let conn = state_clone.conn.get().ok();
                        if let Some(mut db_conn) = conn {
                            use crate::core::shared::models::schema::bots::dsl::*;
                            bots.filter(id.eq(bot_id))
                                .select(name)
                                .first::<String>(&mut db_conn)
                                .unwrap_or_else(|_| "default".to_string())
                        } else {
                            "default".to_string()
                        }
                    };
    let work_dir = crate::core::shared::utils::get_work_path();
    let gbot_dir = format!("{}/{}.gbai/{}.gbot/",
        work_dir, bot_name, bot_name);
                    
                    let system_prompt = std::fs::read_to_string(format!("{}PROMPT.md", gbot_dir))
                        .or_else(|_| std::fs::read_to_string(format!("{}prompt.md", gbot_dir)))
                        .or_else(|_| std::fs::read_to_string(format!("{}PROMPT.txt", gbot_dir)))
                        .or_else(|_| std::fs::read_to_string(format!("{}prompt.txt", gbot_dir)))
                        .unwrap_or_else(|_| {
                            config_manager
                                .get_config(&session.bot_id, "system-prompt", Some("You are a helpful assistant with access to tools that can help you complete tasks. When a user's request matches one of your available tools, use the appropriate tool instead of providing a generic response."))
                                .unwrap_or_else(|_| "You are a helpful General Bots assistant.".to_string())
                        });

                    info!("Loaded system-prompt for bot {}: {}", session.bot_id, system_prompt.chars().take(500).collect::<String>());

                    Ok((session, context_data, history, model, key, system_prompt, bot_llm_url, explicit_llm_provider, bot_endpoint_path))
                },
            )
    .await??
    };

let system_prompt = if !message.active_switchers.is_empty() {
                log::debug!("Switchers active: {:?}", message.active_switchers);
                let switcher_prompts = resolve_active_switchers(
                    self.state.cache.as_ref(),
                    &session.bot_id.to_string(),
                    &session.id.to_string(),
                    &message.active_switchers,
                );
                log::debug!("Switcher prompts: {}", switcher_prompts);
                if switcher_prompts.is_empty() {
                    system_prompt
                } else {
                    format!("{system_prompt}\n\n{switcher_prompts}")
                }
            } else {
                log::debug!("No active switchers for this message");
                system_prompt
            };

    let mut messages = OpenAIClient::build_messages(&system_prompt, &context_data, &history);

        trace!("Built messages array with {} items, first message role: {:?}",
            messages.as_array().map(|a| a.len()).unwrap_or(0),
            messages.as_array().and_then(|a| a.first()).and_then(|m| m.get("role")));

        // Get bot name for KB and tool injection
        let bot_name_for_context = {
            let conn = self.state.conn.get().ok();
            if let Some(mut db_conn) = conn {
                use crate::core::shared::models::schema::bots::dsl::*;
                bots.filter(id.eq(session.bot_id))
                    .select(name)
                    .first::<String>(&mut db_conn)
                    .unwrap_or_else(|_| "default".to_string())
            } else {
                "default".to_string()
            }
        };

        #[cfg(any(feature = "research", feature = "llm"))]
        {
            // Execute start.bas on first message - ONLY run once per session to load suggestions
            let actual_session_id = session.id.to_string();

            // Check if start.bas has already been executed for this session
            let start_bas_key = format!("start_bas_executed:{}", actual_session_id);
            let should_execute_start_bas = if let Some(cache) = &self.state.cache {
                if let Ok(mut conn) = cache.get_multiplexed_async_connection().await {
                    let executed: Result<Option<String>, redis::RedisError> = redis::cmd("GET")
                        .arg(&start_bas_key)
                        .query_async(&mut conn)
                        .await;
                    matches!(executed, Ok(None))
                } else {
                    true // If cache fails, try to execute
                }
            } else {
                true // If no cache, try to execute
            };

        if should_execute_start_bas {
            // Execute start.bas from work directory
            let work_path = crate::core::shared::utils::get_work_path();
            let start_script_path = format!("{}/{}.gbai/{}.gbdialog/start.bas", work_path, bot_name_for_context, bot_name_for_context);

            trace!("Executing start.bas for session {} at: {}", actual_session_id, start_script_path);

            // Load pre-compiled .ast only (compilation happens in Drive Monitor)
            let ast_path = start_script_path.replace(".bas", ".ast");
            let ast_content = match tokio::fs::read_to_string(&ast_path).await {
                Ok(content) if !content.is_empty() => content,
                _ => {
                    let content = tokio::fs::read_to_string(&start_script_path).await.unwrap_or_default();
                    if content.is_empty() {
                        trace!("No start.bas/start.ast found for bot {}", bot_name_for_context);
                        return Ok(());
                    }
                    content
                }
            };

            let state_clone = self.state.clone();
            let actual_session_id_for_task = session.id;
            let bot_id_clone = session.bot_id;

            // Execute start.bas synchronously (blocking)
            let result = tokio::task::spawn_blocking(move || {
                let session_result = {
                    let mut sm = state_clone.session_manager.blocking_lock();
                    sm.get_session_by_id(actual_session_id_for_task)
                };

                let sess = match session_result {
                    Ok(Some(s)) => s,
                    Ok(None) => {
                        return Err(format!("Session {} not found during start.bas execution", actual_session_id_for_task));
                    }
                    Err(e) => return Err(format!("Failed to get session: {}", e)),
                };

                let mut script_service = crate::basic::ScriptService::new(
                    state_clone.clone(),
                    sess
                );
                script_service.load_bot_config_params(&state_clone, bot_id_clone);

                match script_service.run(&ast_content) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(format!("Script execution error: {}", e)),
                }
            }).await;

            match result {
                Ok(Ok(())) => {
                    trace!("start.bas completed successfully for session {}", actual_session_id);

                    // Mark start.bas as executed for this session to prevent re-running
                    if let Some(cache) = &self.state.cache {
                        if let Ok(mut conn) = cache.get_multiplexed_async_connection().await {
                            let _: Result<(), redis::RedisError> = redis::cmd("SET")
                                .arg(&start_bas_key)
                                .arg("1")
                                .arg("EX")
                                .arg("86400") // Expire after 24 hours
                                .query_async(&mut conn)
                                .await;
                        }
                    }
                }
                Ok(Err(e)) => {
                    error!("start.bas error for session {}: {}", actual_session_id, e);
                }
                Err(e) => {
                    error!("start.bas task error for session {}: {}", actual_session_id, e);
                }
            }
        } // End of if should_execute_start_bas

            // If message content is empty, we stop here after potentially running start.bas.
            // This happens when the bot is activated by its name in WhatsApp, where an empty string is sent as a signal.
            if message_content.trim().is_empty() {
                let bot_id_str = message.bot_id.clone();
                let session_id_str = message.session_id.clone();
                
        #[cfg(feature = "chat")]
        let suggestions = get_suggestions(self.state.cache.as_ref(), &bot_id_str, &session_id_str);
        #[cfg(not(feature = "chat"))]
        let suggestions: Vec<crate::core::shared::models::Suggestion> = Vec::new();

        #[cfg(feature = "chat")]
        let switchers = get_switchers(self.state.cache.as_ref(), &bot_id_str, &session_id_str);
        #[cfg(not(feature = "chat"))]
        let switchers: Vec<Switcher> = Vec::new();

        let final_response = BotResponse {
            bot_id: message.bot_id,
            user_id: message.user_id,
            session_id: message.session_id,
            channel: message.channel,
            content: String::new(),
            message_type: MessageType::BOT_RESPONSE,
            stream_token: None,
            is_complete: true,
            suggestions,
            switchers,
            context_name: None,
                    context_length: 0,
                    context_max_length: 0,
                };

                if let Err(e) = response_tx.send(final_response).await {
                    warn!("Failed to send final response for empty content: {}", e);
                }
                return Ok(());
            }

            // Inject KB context for normal messages
            if let Some(kb_manager) = self.state.kb_manager.as_ref() {
                let context = crate::core::bot::kb_context::KbInjectionContext {
                    session_id,
                    bot_id: session.bot_id,
                    bot_name: &bot_name_for_context,
                    user_query: &message_content,
                    messages: &mut messages,
                    max_context_tokens: 16000,
                };
                if let Err(e) = inject_kb_context(
                    kb_manager.clone(),
                    self.state.conn.clone(),
                    context,
                )
                .await
                {
                    error!("Failed to inject KB context: {}", e);
                }
            }
        }

        // Sanitize user message to remove any UTF-16 surrogate characters
        let sanitized_message_content = message_content
            .chars()
            .filter(|c| {
                let cp = *c as u32;
                !(0xD800..=0xDBFF).contains(&cp) && !(0xDC00..=0xDFFF).contains(&cp)
            })
            .collect::<String>();

        // Add the current user message to the messages array
        if let Some(msgs_array) = messages.as_array_mut() {
            msgs_array.push(serde_json::json!({
                "role": "user",
                "content": sanitized_message_content
            }));
        }

        let (stream_tx, mut stream_rx) = mpsc::channel::<String>(100);

        // Use bot-specific LLM provider if the bot has its own llm-url configured
        let llm: std::sync::Arc<dyn crate::llm::LLMProvider> = if let Some(ref url) = bot_llm_url {
            trace!("Bot has custom llm-url: {}, creating per-bot LLM provider", url);
            // Parse explicit provider type if configured (e.g., "openai", "bedrock", "claude")
            let explicit_type = explicit_llm_provider.as_ref().map(|p| {
                let parsed: crate::llm::LLMProviderType = p.as_str().into();
                trace!("Using explicit llm-provider config: {:?} for bot {}", parsed, session.bot_id);
                parsed
            });
            crate::llm::create_llm_provider_from_url(url, Some(model.clone()), bot_endpoint_path, explicit_type)
        } else {
            self.state.llm_provider.clone()
        };

        let model_clone = model.clone();
        let key_clone = key.clone();

        // Retrieve session tools for tool calling (use actual session.id after potential creation)
        let session_tools = get_session_tools(&self.state.conn, &bot_name_for_context, &session.id);
        let tools_for_llm = match session_tools {
            Ok(tools) => {
                if !tools.is_empty() {
                    Some(tools)
                } else {
                    None
                }
            }
            Err(e) => {
                warn!("Failed to load session tools: {}", e);
                None
            }
        };

        // Clone messages for the async task
        let messages_clone = messages.clone();

        // REMOVED: LLM streaming lock was causing deadlocks
        // #[cfg(feature = "drive")]
        // set_llm_streaming(true);

        let stream_tx_clone = stream_tx.clone();

        // Create cancellation channel for this streaming session
        let (cancel_tx, mut cancel_rx) = broadcast::channel::<()>(1);
        let session_id_str = session.id.to_string();

        // Register this streaming session for potential cancellation
        {
            let mut active_streams = self.state.active_streams.lock().await;
            active_streams.insert(session_id_str.clone(), cancel_tx);
        }

        // Wrap the LLM task in a JoinHandle so we can abort it
        let mut cancel_rx_for_abort = cancel_rx.resubscribe();
        let llm_task = tokio::spawn(async move {
            if let Err(e) = llm
                .generate_stream("", &messages_clone, stream_tx_clone, &model_clone, &key_clone, tools_for_llm.as_ref())
                .await
            {
                error!("LLM streaming error: {}", e);
            }
        });

        // Drop the original stream_tx so stream_rx.recv() loop ends
        // when the LLM task finishes and drops its clone.
        drop(stream_tx);

        // Wait for cancellation to abort LLM task
        tokio::spawn(async move {
            if cancel_rx_for_abort.recv().await.is_ok() {
                trace!("Aborting LLM task for session {}", session_id_str);
                llm_task.abort();
            }
        });

        let mut full_response = String::new();
        let mut analysis_buffer = String::new();
        let mut in_analysis = false;
        let mut tool_call_buffer = String::new(); // Accumulate potential tool call JSON chunks
        let mut accumulating_tool_call = false; // Track if we're currently accumulating a tool call
        let mut html_buffer = String::new(); // Buffer for HTML content
        let _handler = llm_models::get_handler(&model);

        trace!("Using model handler for {}", model);
        info!("llm_start: Starting LLM streaming for session {}", session.id);
        trace!("Receiving LLM stream chunks...");
        let mut chunk_count: usize = 0;

        #[cfg(feature = "nvidia")]
        {
            let initial_tokens = crate::core::shared::utils::estimate_token_count(&context_data);
            let config_manager = ConfigManager::new(self.state.conn.clone());
            let max_context_size = config_manager
                .get_config(&session.bot_id, "llm-server-ctx-size", None)
                .unwrap_or_default()
                .parse::<usize>()
                .unwrap_or(0);

            if let Ok(metrics) = get_system_metrics() {
                eprintln!(
                    "\nNVIDIA: {:.1}% | CPU: {:.1}% | Tokens: {}/{}",
                    metrics.gpu_usage.unwrap_or(0.0),
                    metrics.cpu_usage,
                    initial_tokens,
                    max_context_size
                );
            }
        }

while let Some(chunk) = stream_rx.recv().await {
    // Check if cancellation was requested (user sent new message)
    match cancel_rx.try_recv() {
        Ok(_) => {
            info!("stream_exit: Cancelled for session {}", session.id);
            break;
        }
        Err(broadcast::error::TryRecvError::Empty) => {}
        Err(broadcast::error::TryRecvError::Closed) => {
            info!("stream_exit: Cancel channel closed for session {}", session.id);
            break;
        }
        Err(broadcast::error::TryRecvError::Lagged(_)) => {}
    }

            chunk_count += 1;
            if chunk_count <= 3 || chunk_count % 50 == 0 {
                trace!("LLM chunk #{} received for session {} (len={})", chunk_count, session.id, chunk.len());
            }

            // ===== GENERIC TOOL EXECUTION =====
            // Add chunk to tool_call_buffer and try to parse
            // Tool calls arrive as JSON that can span multiple chunks

        // Strip GPT-oSS thinking content from chunks
        // Thinking content appears as JSON objects: {"content":"...","type":"thinking"}
        // We remove these completely as they are internal reasoning not meant for display
        let thinking_regex = regex::Regex::new(r#"\{"content":"((?:[^"\\]|\\.)*?)"\s*,\s*"type":"thinking"\}|\{"type":"thinking"\s*,\s*"content":"((?:[^"\\]|\\.)*?)"\}|\{"type":"thinking_clear"\}"#).unwrap();
        let mut cleaned_chunk = chunk.clone();
        let mut found_thinking = false;

        for mat in thinking_regex.find_iter(&chunk) {
            found_thinking = true;
            let thinking_signal = mat.as_str();
            // Remove the thinking signal from the cleaned chunk
            cleaned_chunk = cleaned_chunk.replace(thinking_signal, "");
        }

        // If the chunk contained only thinking signals, skip to next iteration
        if found_thinking && cleaned_chunk.trim().is_empty() {
            continue;
        }

        // Use the cleaned chunk for further processing
        let chunk = cleaned_chunk;

            // Check if this chunk contains a tool call start
            // We only accumulate if it strongly resembles a tool call to avoid swallowing regular JSON/text
            let looks_like_tool_start = (chunk.trim().starts_with('{') || chunk.trim().starts_with('[')) && 
                                        (chunk.contains("\"id\":\"call_") || chunk.contains("\"type\":\"tool_call\"") || chunk.contains("\"function\":"));

            let chunk_in_tool_buffer = if accumulating_tool_call {
                // Already accumulating - add entire chunk to buffer
                tool_call_buffer.push_str(&chunk);
                true
            } else if looks_like_tool_start {
                // Check if { appears in the middle of the chunk (mixed text + JSON)
                let json_start = chunk.find('{').or_else(|| chunk.find('['));

                if let Some(pos) = json_start {
                    if pos > 0 {
                        // Send the part before { as regular content
                        let regular_part = &chunk[..pos];
                        if !regular_part.trim().is_empty() {
                            full_response.push_str(regular_part);

                            let response = BotResponse {
                                bot_id: message.bot_id.clone(),
                                user_id: message.user_id.clone(),
                                session_id: message.session_id.clone(),
                                channel: message.channel.clone(),
                                content: regular_part.to_string(),
                                message_type: MessageType::BOT_RESPONSE,
                                stream_token: None,
                                is_complete: false,
            suggestions: Vec::new(),
            switchers: Vec::new(),
            context_name: None,
                                context_length: 0,
                                context_max_length: 0,
                            };

                    if response_tx.send(response).await.is_err() {
                        warn!("stream_exit: Response channel closed for session {}", session.id);
                        break;
                    }
                        }

                        // Start accumulating from { onwards
                        accumulating_tool_call = true;
                        tool_call_buffer.push_str(&chunk[pos..]);
                        true
                    } else {
                        // Chunk starts with { or [
                        accumulating_tool_call = true;
                        tool_call_buffer.push_str(&chunk);
                        true
                    }
                } else {
                    // Contains {/[ but find() failed - shouldn't happen, but send as regular content
                    false
                }
            } else {
                false
            };

            // Try to parse tool call from accumulated buffer
            let tool_call = if chunk_in_tool_buffer {
                ToolExecutor::parse_tool_call(&tool_call_buffer)
            } else {
                None
            };

            if let Some(tc) = tool_call {
                let execution_result = ToolExecutor::execute_tool_call(
                    &self.state,
                    &bot_name_for_context,
                    &tc,
                    &session_id,
                    &user_id,
                )
                .await;

                if execution_result.success {
                    info!(
                        "[TOOL_EXEC] Tool '{}' executed successfully: {}",
                        tc.tool_name, execution_result.result
                    );

                    // Send tool execution result to user
                    let response = BotResponse {
                        bot_id: message.bot_id.clone(),
                        user_id: message.user_id.clone(),
                        session_id: message.session_id.clone(),
                        channel: message.channel.clone(),
                        content: execution_result.result,
                        message_type: MessageType::BOT_RESPONSE,
                        stream_token: None,
                        is_complete: false,
                        suggestions: Vec::new(),
                        switchers: Vec::new(),
                        context_name: None,
                        context_length: 0,
                        context_max_length: 0,
                    };

                    if response_tx.send(response).await.is_err() {
                        warn!("stream_exit: Response channel closed during tool execution for session {}", session.id);
                        break;
                    }
                } else {
                    error!(
                        "tool_exec: Tool {} execution failed: {:?}",
                        tc.tool_name, execution_result.error
                    );

                    // Send error to user
                    let error_msg = format!(
                        "Erro ao executar ferramenta '{}': {:?}",
                        tc.tool_name,
                        execution_result.error
                    );

                    let response = BotResponse {
                        bot_id: message.bot_id.clone(),
                        user_id: message.user_id.clone(),
                        session_id: message.session_id.clone(),
                        channel: message.channel.clone(),
                        content: error_msg,
                        message_type: MessageType::BOT_RESPONSE,
                        stream_token: None,
                        is_complete: false,
                        suggestions: Vec::new(),
                        switchers: Vec::new(),
                        context_name: None,
                        context_length: 0,
                        context_max_length: 0,
                    };

                    if response_tx.send(response).await.is_err() {
                        warn!("stream_exit: Response channel closed during tool error for session {}", session.id);
                        break;
                    }
                }

                // Don't add tool_call JSON to full_response or analysis_buffer
                // Clear the tool_call_buffer since we found and executed a tool call
                tool_call_buffer.clear();
                accumulating_tool_call = false; // Reset accumulation flag
                // Continue to next chunk
                continue;
            }

            // Clear tool_call_buffer if it's getting too large and no tool call was found
            // This prevents memory issues from accumulating JSON fragments
            // Increased limit to 50000 to handle large tool calls with many parameters
            if tool_call_buffer.len() > 50000 {
                // Flush accumulated content to client since it's too large to be a tool call
                full_response.push_str(&tool_call_buffer);

                let response = BotResponse {
                    bot_id: message.bot_id.clone(),
                    user_id: message.user_id.clone(),
                    session_id: message.session_id.clone(),
                    channel: message.channel.clone(),
                    content: tool_call_buffer.clone(),
                    message_type: MessageType::BOT_RESPONSE,
                    stream_token: None,
                    is_complete: false,
                    suggestions: Vec::new(),
                    switchers: Vec::new(),
                    context_name: None,
                    context_length: 0,
                    context_max_length: 0,
                };

                tool_call_buffer.clear();
                accumulating_tool_call = false; // Reset accumulation flag after flush

                if response_tx.send(response).await.is_err() {
                    warn!("stream_exit: Response channel closed for session {}", session.id);
                    break;
                }
            }

            // If this chunk was added to tool_call_buffer and no tool call was found yet,
            // skip processing (it's part of an incomplete tool call JSON)
            if chunk_in_tool_buffer {
                continue;
            }
            // ===== END TOOL EXECUTION =====

            analysis_buffer.push_str(&chunk);

            // TEMP DISABLED: Thinking detection causing deadlock
            // Just pass content through directly for now
            /*
            if !in_analysis && handler.has_analysis_markers(&analysis_buffer) {
                in_analysis = true;
                ANALYSIS_START_TIME.store(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    Ordering::SeqCst,
                );
                log::debug!(
                    "Detected start of thinking/analysis content for model {}",
                    model
                );

                // Send thinking indicator
                let thinking_msg = BotResponse {
                    bot_id: message.bot_id.clone(),
                    user_id: message.user_id.clone(),
                    session_id: message.session_id.clone(),
                    channel: message.channel.clone(),
                    content: "🤔 Pensando...".to_string(),
                    message_type: MessageType::BOT_RESPONSE,
                    stream_token: None,
                    is_complete: false,
                    suggestions: Vec::new(),
                    switchers: Vec::new(),
                    context_name: None,
                    context_length: 0,
                    context_max_length: 0,
                };
                
                if response_tx.send(thinking_msg).await.is_err() {
                    warn!("stream_exit: Response channel closed for session {}", session.id);
                    break;
                }
                continue;
            }

            if in_analysis {
                let elapsed = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    - ANALYSIS_START_TIME.load(Ordering::SeqCst);
                if elapsed > ANALYSIS_TIMEOUT_SECS {
                    warn!("Analysis timeout after {}s, forcing exit", elapsed);
                    in_analysis = false;
                }
            }

            if in_analysis && handler.is_analysis_complete(&analysis_buffer) {
                in_analysis = false;
                trace!("Detected end of thinking for model {}", model);
                let processed = handler.process_content(&analysis_buffer);
                if !processed.is_empty() {
                    full_response.push_str(&processed);
                    let response = BotResponse {
                        bot_id: message.bot_id.clone(),
                        user_id: message.user_id.clone(),
                        session_id: message.session_id.clone(),
                        channel: message.channel.clone(),
                        content: processed,
                        message_type: MessageType::BOT_RESPONSE,
                        stream_token: None,
                        is_complete: false,
                        suggestions: Vec::new(),
                        switchers: Vec::new(),
                        context_name: None,
                        context_length: 0,
                        context_max_length: 0,
                    };
                    if response_tx.send(response).await.is_err() {
                        warn!("stream_exit: Response channel closed for session {}", session.id);
                        break;
                    }
                }
                analysis_buffer.clear();
                continue;
            }

            if in_analysis {
                trace!("Accumulating thinking content, not sending to user");
                continue;
            }
            */

            // If in analysis mode from previous chunks, just clear and continue (TEMPORARY)
            if in_analysis {
                in_analysis = false;
                trace!("Cleared leftover in_analysis state");
            }

            if !in_analysis {
                full_response.push_str(&chunk);
                html_buffer.push_str(&chunk);

                // Check if we should flush the buffer:
                // 1. HTML tag pair completed (e.g., </div>, </h1>, </p>, </ul>, </li>)
                // 2. Buffer is large enough (> 500 chars)
                // 3. This is the last chunk (is_complete will be true next iteration)
                let should_flush = html_buffer.len() > 500 
                    || html_buffer.contains("</div>")
                    || html_buffer.contains("</h1>")
                    || html_buffer.contains("</h2>")
                    || html_buffer.contains("</p>")
                    || html_buffer.contains("</ul>")
                    || html_buffer.contains("</ol>")
                    || html_buffer.contains("</li>")
                    || html_buffer.contains("</section>")
                    || html_buffer.contains("</header>")
                    || html_buffer.contains("</footer>");

                if should_flush {
                    let content_to_send = html_buffer.clone();
                    html_buffer.clear();

                    let response = BotResponse {
                        bot_id: message.bot_id.clone(),
                        user_id: message.user_id.clone(),
                        session_id: message.session_id.clone(),
                        channel: message.channel.clone(),
                        content: content_to_send,
                        message_type: MessageType::BOT_RESPONSE,
                        stream_token: None,
                        is_complete: false,
                        suggestions: Vec::new(),
                        switchers: Vec::new(),
                        context_name: None,
                        context_length: 0,
                        context_max_length: 0,
                    };

                    if response_tx.send(response).await.is_err() {
                        warn!("Response channel closed");
                        break;
                    }
                }
            }
        }

        info!("llm_end: Streaming loop ended for session {}, chunk_count={}, full_response_len={}", session.id, chunk_count, full_response.len());

        let has_html = full_response.contains("</") || full_response.contains("<!--");
        let has_div = full_response.contains("<div") || full_response.contains("</div>");
        let has_style = full_response.contains("<style");
        let is_truncated = !full_response.trim_end().ends_with("</div>") && has_div;
        let preview = if full_response.len() > 800 {
            format!("{}... ({} chars total)", full_response.split_at(800).0, full_response.len())
        } else {
            full_response.clone()
        };
        info!("llm_output: session={} has_html={} has_div={} has_style={} is_truncated={} len={} preview=\"{}\"",
            session_id, has_html, has_div, has_style, is_truncated, full_response.len(), 
            preview.replace('\n', "\\n"));

        let full_response_len = full_response.len();
        let is_html = full_response.contains("<") && full_response.contains(">");
        let content_for_save = if is_html {
            let parsed = parse_html(&full_response);
            // Fallback to original if parsing returns empty
            if parsed.trim().is_empty() {
                full_response.clone()
            } else {
                parsed
            }
        } else {
            full_response.clone()
        };
        let history_preview = if content_for_save.len() > 100 {
            format!("{}...", content_for_save.split_at(100).0)
        } else {
            content_for_save.clone()
        };
        info!("history_save: session_id={} user_id={} full_response_len={} is_html={} content_len={} preview={}",
            session.id, user_id, full_response_len, is_html, content_for_save.len(), history_preview);
        
        let state_for_save = self.state.clone();
        let content_for_save_owned = content_for_save;
        let session_id_for_save = session.id;
        let user_id_for_save = user_id;
        
        let save_result = tokio::task::spawn_blocking(
            move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                let mut sm = state_for_save.session_manager.blocking_lock();
                sm.save_message(session_id_for_save, user_id_for_save, 2, &content_for_save_owned, 2)?;
                Ok(())
            },
        )
        .await;
        
        match save_result {
            Ok(Ok(())) => {
                trace!("history_save: Assistant message saved for session {}", session_id_for_save);
            }
            Ok(Err(e)) => {
                error!("history_save: Failed to save assistant message for session {}: {}", session_id_for_save, e);
            }
            Err(e) => {
                error!("history_save: Spawn blocking failed for session {}: {}", session_id_for_save, e);
            }
        }

        // Extract bot_id and session_id before moving them into BotResponse
        let bot_id_str = message.bot_id.clone();
        let session_id_str = message.session_id.clone();

        #[cfg(feature = "chat")]
        let suggestions = get_suggestions(self.state.cache.as_ref(), &bot_id_str, &session_id_str);
        #[cfg(not(feature = "chat"))]
        let suggestions: Vec<crate::core::shared::models::Suggestion> = Vec::new();

        #[cfg(feature = "chat")]
        let switchers = get_switchers(self.state.cache.as_ref(), &bot_id_str, &session_id_str);
        #[cfg(not(feature = "chat"))]
        let switchers: Vec<Switcher> = Vec::new();

        // Flush any remaining HTML buffer before sending final response
        if !html_buffer.is_empty() {
            trace!("Flushing remaining {} chars in HTML buffer", html_buffer.len());
            let final_chunk = BotResponse {
                bot_id: message.bot_id.clone(),
                user_id: message.user_id.clone(),
                session_id: message.session_id.clone(),
                channel: message.channel.clone(),
                content: html_buffer.clone(),
                message_type: MessageType::BOT_RESPONSE,
                stream_token: None,
            is_complete: false,
            suggestions: Vec::new(),
            switchers: Vec::new(),
            context_name: None,
            context_length: 0,
            context_max_length: 0,
        };
        let _ = response_tx.send(final_chunk).await;
            html_buffer.clear();
        }

        // Content was already sent as streaming chunks.
        // Sending full_response again would duplicate it (especially for WhatsApp which accumulates buffer).
        // The final response is just a signal that streaming is complete - it should not contain content.
        let final_content = String::new();

        let final_response = BotResponse {
            bot_id: message.bot_id,
            user_id: message.user_id,
            session_id: message.session_id,
            channel: message.channel,
            content: final_content,
            message_type: MessageType::BOT_RESPONSE,
            stream_token: None,
            is_complete: true,
    suggestions,
    switchers,
    context_name: None,
    context_length: 0,
    context_max_length: 0,
};

        response_tx.send(final_response).await?;
        Ok(())
    }

    #[cfg(not(feature = "llm"))]
    pub async fn stream_response(
        &self,
        message: UserMessage,
        response_tx: mpsc::Sender<BotResponse>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        warn!("LLM feature not enabled, cannot stream response");

        let error_response = BotResponse {
            bot_id: message.bot_id,
            user_id: message.user_id,
            session_id: message.session_id,
            channel: message.channel,
            content: "LLM feature is not enabled in this build".to_string(),
            message_type: MessageType::BOT_RESPONSE,
            stream_token: None,
            is_complete: true,
            suggestions: Vec::new(),
            switchers: Vec::new(),
            context_name: None,
            context_length: 0,
            context_max_length: 0,
        };

        response_tx.send(error_response).await?;
        Ok(())
    }

    pub async fn get_user_sessions(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<UserSession>, Box<dyn std::error::Error + Send + Sync>> {
        let mut session_manager = self.state.session_manager.lock().await;
        let sessions = session_manager.get_user_sessions(user_id)?;
        Ok(sessions)
    }

    pub async fn get_conversation_history(
        &self,
        session_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<(String, String)>, Box<dyn std::error::Error + Send + Sync>> {
        let mut session_manager = self.state.session_manager.lock().await;
        let history = session_manager.get_conversation_history(session_id, user_id, None)?;
        Ok(history)
    }
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    info!("websocket_handler: Received request with params: {:?}", params);
    let session_id = params
        .get("session_id")
        .and_then(|s| Uuid::parse_str(s).ok());
    let user_id = params.get("user_id").and_then(|s| Uuid::parse_str(s).ok());

    // Extract bot_name from query params
    let bot_name = params
        .get("bot_name")
        .cloned()
        .unwrap_or_else(|| "default".to_string());

    // Allow anonymous connections for desktop UI - create UUIDs if not provided
    let session_id = session_id.unwrap_or_else(Uuid::new_v4);
    let user_id = user_id.unwrap_or_else(Uuid::new_v4);

    info!("WebSocket: session_id from params = {:?}, user_id = {:?}", session_id, user_id);

    // Look up bot_id from bot_name
    let (bot_id, _bot_is_public) = {
        let conn = state.conn.get().ok();
        if let Some(mut db_conn) = conn {
            use crate::core::shared::models::schema::bots::dsl::*;

            // Try to parse as UUID first, if that fails treat as bot name
            let result: Result<(Uuid, bool), _> = if let Ok(uuid) = Uuid::parse_str(&bot_name) {
                // Parameter is a UUID, look up by id
                bots.filter(id.eq(uuid))
                    .select((id, is_public))
                    .first(&mut db_conn)
            } else {
                // Parameter is a bot name, look up by name
                bots.filter(name.eq(&bot_name))
                    .select((id, is_public))
                    .first(&mut db_conn)
            };

            result.unwrap_or_else(|_| {
                log::warn!("Bot not found: {}, using nil bot_id", bot_name);
                (Uuid::nil(), false)
            })
        } else {
            log::warn!("Could not get database connection, using nil bot_id");
            (Uuid::nil(), false)
        }
    };

    // Check bot access before upgrading WebSocket
    if bot_id != Uuid::nil() {
        let conn = state.conn.get().ok();
        if let Some(mut db_conn) = conn {
            if let Err((status, msg)) = check_bot_access(&mut db_conn, bot_id, user_id) {
                return (status, msg).into_response();
            }
        } else {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    }

    ws.on_upgrade(move |socket| handle_websocket(socket, state, session_id, user_id, bot_id))
        .into_response()
}

pub async fn websocket_handler_with_bot(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Path(bot_name): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let mut params = params;
    if !bot_name.is_empty() {
        params.insert("bot_name".to_string(), bot_name);
    }
    websocket_handler(ws, State(state), Query(params)).await
}

async fn handle_websocket(
    socket: WebSocket,
    state: Arc<AppState>,
    session_id: Uuid,
    user_id: Uuid,
    bot_id: Uuid,
) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::channel::<BotResponse>(100);

    state
        .web_adapter
        .add_connection(session_id.to_string(), tx.clone())
        .await;

    {
        let mut channels = state.response_channels.lock().await;
        channels.insert(session_id.to_string(), tx.clone());
    }

    info!(
        "WebSocket connected for session: {}, user: {}, bot: {}",
        session_id, user_id, bot_id
    );

    // Get bot_name for tools loading
    let bot_name_result = {
        let conn = state.conn.get().ok();
        if let Some(mut db_conn) = conn {
            use crate::core::shared::models::schema::bots::dsl::*;
            bots.filter(id.eq(bot_id))
                .select(name)
                .first::<String>(&mut db_conn)
                .ok()
        } else {
            None
        }
    };

    // Load session tools
    let tools = if let Some(bot_name) = bot_name_result {
        match get_session_tools(&state.conn, &bot_name, &session_id) {
            Ok(tools_vec) => {
                info!(
                    "[WEBSOCKET] Loaded {} session tools for bot {}, session {}",
                    tools_vec.len(),
                    bot_name,
                    session_id
                );
                tools_vec
            }
            Err(e) => {
                error!(
                    "[WEBSOCKET] Failed to load session tools for bot {}, session {}: {}",
                    bot_name, session_id, e
                );
                vec![]
            }
        }
    } else {
        warn!(
            "[WEBSOCKET] Could not get bot name for bot_id {}, no session tools loaded",
            bot_id
        );
        vec![]
    };

    let welcome = serde_json::json!({
        "type": "connected",
        "session_id": session_id,
        "user_id": user_id,
        "bot_id": bot_id,
        "message": "Connected to bot server",
        "tools": tools
    });

    if let Ok(welcome_str) = serde_json::to_string(&welcome) {
        if sender.send(Message::Text(welcome_str)).await.is_err() {
            error!("Failed to send welcome message");
        }
    }

    let (send_ready_tx, send_ready_rx) = tokio::sync::mpsc::channel::<()>(1);

    // Execute start.bas automatically on connection (similar to auth.ast pattern)
    {
        let bot_name_result = {
            let conn = state.conn.get().ok();
            if let Some(mut db_conn) = conn {
                use crate::core::shared::models::schema::bots::dsl::*;
                bots.filter(id.eq(bot_id))
                    .select(name)
                    .first::<String>(&mut db_conn)
                    .ok()
            } else {
                None
            }
        };

        // DEBUG: Log start script execution attempt
        info!(
            "Checking for start.bas: bot_id={}, bot_name_result={:?}",
            bot_id,
            bot_name_result
        );

        if let Some(bot_name) = bot_name_result {
            // Web clients expect start.bas to execute their first screen every time they connect/reload.
            // We always run it, but we SET the start_bas_key flag right after so stream_response skips execution.
            let start_bas_key = format!("start_bas_executed:{}", session_id);
            let should_execute_start_bas = true;

            if should_execute_start_bas {
                let work_path = crate::core::shared::utils::get_work_path();
                let start_script_path = format!("{}/{}.gbai/{}.gbdialog/start.bas", work_path, bot_name, bot_name);

                info!("Looking for start.bas at: {}", start_script_path);

            // Load pre-compiled .ast only (compilation happens in Drive Monitor)
            let ast_path = start_script_path.replace(".bas", ".ast");
            let ast_content = match tokio::fs::read_to_string(&ast_path).await {
                Ok(content) if !content.is_empty() => content,
                _ => {
                    let content = tokio::fs::read_to_string(&start_script_path).await.unwrap_or_default();
                    if content.is_empty() {
                        info!("No start.bas/start.ast found for bot {}", bot_name);
                        String::new()
                    } else {
                        content
                    }
                }
            };

                if !ast_content.is_empty() {
                    info!(
                        "Executing start.bas for bot {} on session {}",
                        bot_name, session_id
                    );

            let state_for_start = state.clone();
            let tx_for_start = tx.clone();
            let bot_id_str = bot_id.to_string();
            let session_id_str = session_id.to_string();
            let mut send_ready_rx = send_ready_rx;

                    tokio::spawn(async move {
                        let _ = send_ready_rx.recv().await;

                        let session_result = {
                            let mut sm = state_for_start.session_manager.lock().await;
                            let by_id = sm.get_session_by_id(session_id);
                            match by_id {
                                Ok(Some(s)) => Ok(Some(s)),
                                _ => sm.get_or_create_user_session(user_id, bot_id, "Chat Session"),
                            }
                        };

                        if let Ok(Some(mut session)) = session_result {
                            info!("start.bas: Found session {} for websocket session {}", session.id, session_id);
                            
                            // Save session ID before session is moved into closure
                            let session_id_for_redis = session.id.to_string();

                            // Store WebSocket session_id in context so TALK can route messages correctly
                            if let serde_json::Value::Object(ref mut map) = session.context_data {
                                map.insert("websocket_session_id".to_string(), serde_json::Value::String(session_id.to_string()));
                            } else {
                                let mut map = serde_json::Map::new();
                                map.insert("websocket_session_id".to_string(), serde_json::Value::String(session_id.to_string()));
                                session.context_data = serde_json::Value::Object(map);
                            }

                            // Clone state_for_start for use in Redis SET after execution
                            let state_for_redis = state_for_start.clone();

            let result = tokio::task::spawn_blocking(move || {
                info!("start.bas: Creating ScriptService with session.id={}", session.id);
                let mut script_service = crate::basic::ScriptService::new(
                    state_for_start.clone(),
                    session.clone()
                );
                script_service.load_bot_config_params(&state_for_start, bot_id);

                match script_service.run(&ast_content) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(format!("Script execution error: {}", e)),
                }
            }).await;

                                match result {
                                    Ok(Ok(())) => {
                                        info!("start.bas executed successfully for bot {}", bot_name);

                                        // Mark start.bas as executed for this session to prevent re-running
                                        if let Some(cache) = &state_for_redis.cache {
                                            if let Ok(mut conn) = cache.get_multiplexed_async_connection().await {
                                                let _: Result<(), redis::RedisError> = redis::cmd("SET")
                                                    .arg(&start_bas_key)
                                                    .arg("1")
                                                    .arg("EX")
                                                    .arg("86400") // Expire after 24 hours
                                                    .query_async(&mut conn)
                                                    .await;
                                                info!("Marked start.bas as executed for session {}", session_id);
                                            }
                                        }

        // Fetch suggestions and switchers from Redis and send to frontend
        // Use session_id_for_redis (DB session) not session_id_str (WebSocket session) for Redis key consistency
        let user_id_str = user_id.to_string();
        let suggestions = get_suggestions(state_for_redis.cache.as_ref(), &bot_id_str, &session_id_for_redis);
        let switchers = get_switchers(state_for_redis.cache.as_ref(), &bot_id_str, &session_id_for_redis);
        if !suggestions.is_empty() || !switchers.is_empty() {
                                            info!("Sending {} suggestions to frontend for session {}", suggestions.len(), session_id);
                                            let response = BotResponse {
                                                bot_id: bot_id_str.clone(),
                                                user_id: user_id_str.clone(),
                                                session_id: session_id_str.clone(),
                                                channel: "Chat".to_string(),
                                                content: String::new(),
                                                message_type: MessageType::BOT_RESPONSE,
                                                stream_token: None,
            is_complete: true,
            suggestions,
            switchers,
            context_name: None,
                                                context_length: 0,
                                                context_max_length: 0,
                                            };
                                            let _ = tx_for_start.send(response).await;


                                        }
                                    }
                                    Ok(Err(e)) => {
                                        error!("start.bas error for bot {}: {}", bot_name, e);
                                    }
                                    Err(e) => {
                    error!("start.bas task error for bot {}: {}", bot_name, e);
                        }
            }
        }
    });
    }
}
} // End of if should_execute_start_bas
}

let mut send_task = tokio::spawn(async move {
        while let Some(response) = rx.recv().await {
            if let Ok(json_str) = serde_json::to_string(&response) {
                if sender.send(Message::Text(json_str)).await.is_err() {
                    break;
                }
            }
        }
    });

    let _ = send_ready_tx.send(()).await;

    let state_clone = state.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    debug!("WebSocket received text: {}", text);
                    // Add immediate trace
                    info!("Processing message for session {}", session_id);
                    
                    if let Ok(user_msg) = serde_json::from_str::<UserMessage>(&text) {
                        // Get session first, outside any lock scope
                        let session_result = {
                            let mut sm = state_clone.session_manager.lock().await;
                            sm.get_session_by_id(session_id)
                        };

                        let session = match session_result {
                            Ok(Some(sess)) => sess,
                            Ok(None) => {
                                let mut sm = state_clone.session_manager.lock().await;
                                match sm.create_session(user_id, bot_id, "WebSocket Chat") {
                                    Ok(new_session) => new_session,
                                    Err(e) => {
                                        error!("Failed to create session: {}", e);
                                        continue;
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Error getting session: {}", e);
                                continue;
                            }
                        };

                        // Get response channel sender out of lock scope
                        let tx_opt = {
                            let channels = state_clone.response_channels.lock().await;
                            channels.get(&session_id.to_string()).cloned()
                        };

        if let Some(tx_clone) = tx_opt {
            // CANCEL any existing streaming for this session first
            let session_id_str = session_id.to_string();
            {
                let mut active_streams = state_clone.active_streams.lock().await;
                if let Some(cancel_tx) = active_streams.remove(&session_id_str) {
                    info!("Cancelling existing streaming for session {}", session_id);
                    let _ = cancel_tx.send(());
                    // Give a moment for the streaming to stop
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }
                    
                    let corrected_msg = UserMessage {
                        bot_id: bot_id.to_string(),
                        user_id: session.user_id.to_string(),
                        session_id: session.id.to_string(),
                        ..user_msg
                    };
                    info!("Calling orchestrator for session {}", session_id);

                    // Spawn LLM in its own task so recv_task stays free to handle
                    // new messages — prevents one hung LLM from locking the session.
                    let orch = BotOrchestrator::new(state_clone.clone());
                    tokio::spawn(async move {
                        if let Err(e) = orch
                        .stream_response(corrected_msg, tx_clone)
                        .await
                        {
                            error!("Failed to stream response: {}", e);
                        }
                    });
                        } else {
                            warn!("Response channel NOT found for session: {}", session_id);
                        }
                    } else {
                        warn!("Failed to parse UserMessage from: {}", text);
                    }
                }
                Message::Close(_) => {
                    trace!("WebSocket close message received");
                    break;
                }
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => { recv_task.abort(); }
        _ = (&mut recv_task) => { send_task.abort(); }
    }

    state
        .web_adapter
        .remove_connection(&session_id.to_string())
        .await;

    {
        let mut channels = state.response_channels.lock().await;
        channels.remove(&session_id.to_string());
    }

    info!("WebSocket disconnected for session: {}", session_id);
}

pub fn create_bot_handler(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<HashMap<String, String>>,
) -> impl IntoResponse {
    let bot_name = payload
        .get("bot_name")
        .cloned()
        .unwrap_or_else(|| "default".to_string());

    let orchestrator = BotOrchestrator::new(state);
    if let Err(e) = orchestrator.mount_all_bots() {
        error!("Failed to mount bots: {}", e);
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({ "status": format!("bot '{}' created", bot_name) })),
    )
}

pub fn mount_bot_handler(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<HashMap<String, String>>,
) -> impl IntoResponse {
    let bot_guid = payload.get("bot_guid").cloned().unwrap_or_default();

    let orchestrator = BotOrchestrator::new(state);
    if let Err(e) = orchestrator.mount_all_bots() {
        error!("Failed to mount bot: {}", e);
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({ "status": format!("bot '{}' mounted", bot_guid) })),
    )
}

pub async fn handle_user_input_handler(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<HashMap<String, String>>,
) -> impl IntoResponse {
    let session_id = payload.get("session_id").cloned().unwrap_or_default();
    let user_input = payload.get("input").cloned().unwrap_or_default();

    info!(
        "Processing user input: {} for session: {}",
        user_input, session_id
    );

    let orchestrator = BotOrchestrator::new(state);
    if let Ok(sessions) = orchestrator.get_user_sessions(Uuid::nil()).await {
        info!("Found {} sessions", sessions.len());
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({ "status": format!("processed: {}", user_input) })),
    )
}

pub async fn get_user_sessions_handler(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<HashMap<String, String>>,
) -> impl IntoResponse {
    let user_id = payload
        .get("user_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .unwrap_or_else(Uuid::nil);

    let orchestrator = BotOrchestrator::new(state);
    match orchestrator.get_user_sessions(user_id).await {
        Ok(sessions) => (
            StatusCode::OK,
            Json(serde_json::json!({ "sessions": sessions })),
        ),
        Err(e) => {
            error!("Failed to get sessions: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        }
    }
}

pub async fn get_conversation_history_handler(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<HashMap<String, String>>,
) -> impl IntoResponse {
    let session_id = payload
        .get("session_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .unwrap_or_else(Uuid::nil);
    let user_id = payload
        .get("user_id")
        .and_then(|id| Uuid::parse_str(id).ok())
        .unwrap_or_else(Uuid::nil);

    let orchestrator = BotOrchestrator::new(state);
    match orchestrator
        .get_conversation_history(session_id, user_id)
        .await
    {
        Ok(history) => (
            StatusCode::OK,
            Json(serde_json::json!({ "history": history })),
        ),
        Err(e) => {
            error!("Failed to get history: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        }
    }
}

pub async fn send_warning_handler(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<HashMap<String, String>>,
) -> impl IntoResponse {
    let message = payload
        .get("message")
        .cloned()
        .unwrap_or_else(|| "Warning".to_string());
    let session_id = payload.get("session_id").cloned().unwrap_or_default();

    warn!("Warning for session {}: {}", session_id, message);

    let orchestrator = BotOrchestrator::new(state);
    info!("Orchestrator created for warning");

    if let Ok(sessions) = orchestrator.get_user_sessions(Uuid::nil()).await {
        info!("Current active sessions: {}", sessions.len());
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({ "status": "warning sent", "message": message })),
    )
}
