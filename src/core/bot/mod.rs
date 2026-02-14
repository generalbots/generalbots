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
use crate::drive::drive_monitor::DriveMonitor;
#[cfg(feature = "llm")]
use crate::llm::llm_models;
#[cfg(feature = "llm")]
use crate::llm::OpenAIClient;
#[cfg(feature = "nvidia")]
use crate::nvidia::get_system_metrics;
use crate::core::shared::message_types::MessageType;
use crate::core::shared::models::{BotResponse, UserMessage, UserSession};
use crate::core::shared::state::AppState;
#[cfg(feature = "chat")]
use crate::basic::keywords::add_suggestion::get_suggestions;
use axum::extract::ws::{Message, WebSocket};
use axum::{
    extract::{ws::WebSocketUpgrade, Extension, Query, State},
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
use log::{error, info, warn};
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex as AsyncMutex;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

pub mod channels;
pub mod multimedia;

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

    // Query bot_configuration table for this bot's configuration
    use crate::core::shared::models::schema::bot_configuration::dsl::*;

    let mut is_public = false;
    let mut theme_color1: Option<String> = None;
    let mut theme_color2: Option<String> = None;
    let mut theme_title: Option<String> = None;
    let mut theme_logo: Option<String> = None;
    let mut theme_logo_text: Option<String> = None;

    // Query all config values (no prefix filter - will match in code)
    match bot_configuration
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
                        is_public = value.eq_ignore_ascii_case("true") || value == "1";
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
            mounted_bots: Arc::new(AsyncMutex::new(HashMap::new())),
        }
    }

    pub fn mount_all_bots(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Scanning drive for .gbai files to mount bots...");

        let mut bots_mounted = 0;
        let mut bots_created = 0;

        let data_dir = "/opt/gbo/data";

        let directories_to_scan: Vec<std::path::PathBuf> = vec![
            self.state
                .config
                .as_ref()
                .map(|c| c.site_path.clone())
                .unwrap_or_else(|| "./botserver-stack/sites".to_string())
                .into(),
            "./templates".into(),
            "../bottemplates".into(),
            data_dir.into(),
        ];

        for dir_path in directories_to_scan {
            info!("Checking directory for bots: {}", dir_path.display());

            if !dir_path.exists() {
                info!("Directory does not exist, skipping: {}", dir_path.display());
                continue;
            }

            match self.scan_directory(&dir_path, &mut bots_mounted, &mut bots_created) {
                Ok(()) => {}
                Err(e) => {
                    error!("Failed to scan directory {}: {}", dir_path.display(), e);
                }
            }
        }

        info!(
            "Bot mounting complete: {} bots processed ({} created, {} already existed)",
            bots_mounted,
            bots_created,
            bots_mounted - bots_created
        );

        Ok(())
    }

    fn scan_directory(
        &self,
        dir_path: &std::path::Path,
        bots_mounted: &mut i32,
        bots_created: &mut i32,
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
                    // Auto-create bots found in /opt/gbo/data
                    if dir_path.to_string_lossy().contains("/data") {
                        info!("Auto-creating bot '{}' from /opt/gbo/data", bot_name);
                        match self.create_bot_simple(bot_name) {
                            Ok(_) => {
                                info!("Bot '{}' created successfully", bot_name);
                                *bots_created += 1;
                                *bots_mounted += 1;
                            }
                            Err(e) => {
                                error!("Failed to create bot '{}': {}", bot_name, e);
                            }
                        }
                    } else {
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

    fn create_bot_simple(&self, bot_name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use diesel::sql_query;
        use uuid::Uuid;

        let mut conn = self
            .state
            .conn
            .get()
            .map_err(|e| format!("Failed to get database connection: {e}"))?;

        // Check if bot already exists
        let exists = self.ensure_bot_exists(bot_name)?;
        if exists {
            info!("Bot '{}' already exists, skipping creation", bot_name);
            return Ok(());
        }

        let bot_id = Uuid::new_v4();

        sql_query(
            "INSERT INTO bots (id, name, llm_provider, context_provider, is_active, created_at, updated_at)
             VALUES ($1, $2, 'openai', 'website', true, NOW(), NOW())"
        )
        .bind::<diesel::sql_types::Uuid, _>(bot_id)
        .bind::<diesel::sql_types::Text, _>(bot_name)
        .execute(&mut conn)
        .map_err(|e| format!("Failed to create bot: {e}"))?;

        info!("Created bot '{}' with ID '{}'", bot_name, bot_id);
        Ok(())
    }

    #[cfg(feature = "llm")]
    pub async fn stream_response(
        &self,
        message: UserMessage,
        response_tx: mpsc::Sender<BotResponse>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        trace!(
            "Streaming response for user: {}, session: {}",
            message.user_id,
            message.session_id
        );

        let user_id = Uuid::parse_str(&message.user_id)?;
        let session_id = Uuid::parse_str(&message.session_id)?;
        let message_content = message.content.clone();

        let (session, context_data, history, model, key) = {
            let state_clone = self.state.clone();
            tokio::task::spawn_blocking(
                move || -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                    let session = {
                        let mut sm = state_clone.session_manager.blocking_lock();
                        sm.get_session_by_id(session_id)?
                    }
                    .ok_or("Session not found")?;

                    {
                        let mut sm = state_clone.session_manager.blocking_lock();
                        sm.save_message(session.id, user_id, 1, &message.content, 1)?;
                    }

                    let context_data = {
                        let sm = state_clone.session_manager.blocking_lock();
                        sm.get_session_context_data(&session.id, &session.user_id)?
                    };

                    let history = {
                        let mut sm = state_clone.session_manager.blocking_lock();
                        sm.get_conversation_history(session.id, user_id)?
                    };

                    let config_manager = ConfigManager::new(state_clone.conn.clone());

                    // DEBUG: Log which bot we're getting config for
                    info!("[CONFIG_TRACE] Getting LLM config for bot_id: {}", session.bot_id);

                    let model = config_manager
                        .get_config(&session.bot_id, "llm-model", Some("gpt-3.5-turbo"))
                        .unwrap_or_else(|_| "gpt-3.5-turbo".to_string());

                    let key = config_manager
                        .get_config(&session.bot_id, "llm-key", Some(""))
                        .unwrap_or_default();

                    // DEBUG: Log the exact config values retrieved
                    info!("[CONFIG_TRACE] Model: '{}'", model);
                    info!("[CONFIG_TRACE] API Key: '{}' ({} chars)", key, key.len());
                    info!("[CONFIG_TRACE] API Key first 10 chars: '{}'", &key.chars().take(10).collect::<String>());
                    info!("[CONFIG_TRACE] API Key last 10 chars: '{}'", &key.chars().rev().take(10).collect::<String>());

                    Ok((session, context_data, history, model, key))
                },
            )
            .await??
        };

        let system_prompt = "You are a helpful assistant with access to tools that can help you complete tasks. When a user's request matches one of your available tools, use the appropriate tool instead of providing a generic response.".to_string();
        let mut messages = OpenAIClient::build_messages(&system_prompt, &context_data, &history);

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
                // Always execute start.bas for this session (blocking - wait for completion)
                let data_dir = "/opt/gbo/data";
                let start_script_path = format!("{}/{}.gbai/{}.gbdialog/start.bas", data_dir, bot_name_for_context, bot_name_for_context);

                info!("[START_BAS] Executing start.bas for session {} at: {}", actual_session_id, start_script_path);

                if let Ok(metadata) = tokio::fs::metadata(&start_script_path).await {
                    if metadata.is_file() {
                        info!("[START_BAS] Found start.bas, executing for session {}", actual_session_id);

                        if let Ok(start_script) = tokio::fs::read_to_string(&start_script_path).await {
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

                                match script_service.compile(&start_script) {
                                    Ok(ast) => match script_service.run(&ast) {
                                        Ok(_) => Ok(()),
                                        Err(e) => Err(format!("Script execution error: {}", e)),
                                    },
                                    Err(e) => Err(format!("Script compilation error: {}", e)),
                                }
                            }).await;

                            match result {
                                Ok(Ok(())) => {
                                    info!("[START_BAS] start.bas completed successfully for session {}", actual_session_id);

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
                                            info!("[START_BAS] Marked start.bas as executed for session {}", actual_session_id);
                                        }
                                    }
                                }
                                Ok(Err(e)) => {
                                    error!("[START_BAS] start.bas error for session {}: {}", actual_session_id, e);
                                }
                                Err(e) => {
                                    error!("[START_BAS] start.bas task error for session {}: {}", actual_session_id, e);
                                }
                            }
                        }
                    }
                }
            } // End of if should_execute_start_bas

            if let Some(kb_manager) = self.state.kb_manager.as_ref() {
                if let Err(e) = inject_kb_context(
                    kb_manager.clone(),
                    self.state.conn.clone(),
                    session_id,
                    &bot_name_for_context,
                    &message_content,
                    &mut messages,
                    8000,
                )
                .await
                {
                    error!("Failed to inject KB context: {}", e);
                }
            }
        }

        // Add the current user message to the messages array
        if let Some(msgs_array) = messages.as_array_mut() {
            msgs_array.push(serde_json::json!({
                "role": "user",
                "content": message_content
            }));
        }

        // DEBUG: Log messages before sending to LLM
        info!("[LLM_CALL] Messages before LLM: {}", serde_json::to_string_pretty(&messages).unwrap_or_default());
        info!("[LLM_CALL] message_content: '{}'", message_content);

        let (stream_tx, mut stream_rx) = mpsc::channel::<String>(100);
        info!("[STREAM_SETUP] Channel created, starting LLM stream");
        let llm = self.state.llm_provider.clone();

        let model_clone = model.clone();
        let key_clone = key.clone();

        // Retrieve session tools for tool calling (use actual session.id after potential creation)
        let session_tools = get_session_tools(&self.state.conn, &bot_name_for_context, &session.id);
        let tools_for_llm = match session_tools {
            Ok(tools) => {
                if !tools.is_empty() {
                    info!("[TOOLS] Loaded {} tools for session {}", tools.len(), session.id);
                    Some(tools)
                } else {
                    info!("[TOOLS] No tools associated with session {}", session.id);
                    None
                }
            }
            Err(e) => {
                warn!("[TOOLS] Failed to load session tools: {}", e);
                None
            }
        };

        // Clone messages for the async task
        let messages_clone = messages.clone();

        // DEBUG: Log exact values being passed to LLM
        info!("[LLM_CALL] Calling generate_stream with:");
        info!("[LLM_CALL]   Model: '{}'", model_clone);
        info!("[LLM_CALL]   Key length: {} chars", key_clone.len());
        info!("[LLM_CALL]   Key preview: '{}...{}'",
            &key_clone.chars().take(8).collect::<String>(),
            &key_clone.chars().rev().take(8).collect::<String>()
        );

        tokio::spawn(async move {
            info!("[SPAWN_TASK] LLM stream task started");
            if let Err(e) = llm
                .generate_stream("", &messages_clone, stream_tx, &model_clone, &key_clone, tools_for_llm.as_ref())
                .await
            {
                error!("LLM streaming error: {}", e);
            }
            info!("[SPAWN_TASK] LLM stream task completed");
        });

        let mut full_response = String::new();
        let mut analysis_buffer = String::new();
        let mut in_analysis = false;
        let handler = llm_models::get_handler(&model);

        info!("[STREAM_START] Entering stream processing loop for model: {}", model);
        info!("[STREAM_START] About to enter while loop, stream_rx is valid");

        trace!("Using model handler for {}", model);

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
            info!("[STREAM_DEBUG] Received chunk: '{}', len: {}", chunk, chunk.len());
            trace!("Received LLM chunk: {:?}", chunk);

            // ===== GENERIC TOOL EXECUTION =====
            // Check if this chunk contains a tool call (works with all LLM providers)
            if let Some(tool_call) = ToolExecutor::parse_tool_call(&chunk) {
                info!(
                    "[TOOL_CALL] Detected tool '{}' from LLM, executing...",
                    tool_call.tool_name
                );

                let execution_result = ToolExecutor::execute_tool_call(
                    &self.state,
                    &bot_name_for_context,
                    &tool_call,
                    &session_id,
                    &user_id,
                )
                .await;

                if execution_result.success {
                    info!(
                        "[TOOL_EXEC] Tool '{}' executed successfully: {}",
                        tool_call.tool_name, execution_result.result
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
                        context_name: None,
                        context_length: 0,
                        context_max_length: 0,
                    };

                    if response_tx.send(response).await.is_err() {
                        warn!("Response channel closed during tool execution");
                        break;
                    }
                } else {
                    error!(
                        "[TOOL_EXEC] Tool '{}' execution failed: {:?}",
                        tool_call.tool_name, execution_result.error
                    );

                    // Send error to user
                    let error_msg = format!(
                        "Erro ao executar ferramenta '{}': {:?}",
                        tool_call.tool_name,
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
                        context_name: None,
                        context_length: 0,
                        context_max_length: 0,
                    };

                    if response_tx.send(response).await.is_err() {
                        warn!("Response channel closed during tool error");
                        break;
                    }
                }

                // Don't add tool_call JSON to full_response or analysis_buffer
                // Continue to next chunk
                continue;
            }
            // ===== END TOOL EXECUTION =====

            analysis_buffer.push_str(&chunk);

            if !in_analysis && handler.has_analysis_markers(&analysis_buffer) {
                in_analysis = true;
                log::debug!(
                    "Detected start of thinking/analysis content for model {}",
                    model
                );

                let processed = handler.process_content(&analysis_buffer);
                if !processed.is_empty() && processed != analysis_buffer {
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
                        context_name: None,
                        context_length: 0,
                        context_max_length: 0,
                    };

                    if response_tx.send(response).await.is_err() {
                        warn!("Response channel closed");
                        break;
                    }
                }
                continue;
            }

            if in_analysis && handler.is_analysis_complete(&analysis_buffer) {
                in_analysis = false;
                info!(
                    "[ANALYSIS] Detected end of thinking for model {}. Buffer: '{}'",
                    model, analysis_buffer
                );

                let processed = handler.process_content(&analysis_buffer);
                info!("[ANALYSIS] Processed content: '{}'", processed);
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
                        context_name: None,
                        context_length: 0,
                        context_max_length: 0,
                    };

                    if response_tx.send(response).await.is_err() {
                        warn!("Response channel closed");
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

            if !in_analysis {
                info!("[STREAM_CONTENT] Sending chunk: '{}', len: {}", chunk, chunk.len());
                full_response.push_str(&chunk);

                let response = BotResponse {
                    bot_id: message.bot_id.clone(),
                    user_id: message.user_id.clone(),
                    session_id: message.session_id.clone(),
                    channel: message.channel.clone(),
                    content: chunk,
                    message_type: MessageType::BOT_RESPONSE,
                    stream_token: None,
                    is_complete: false,
                    suggestions: Vec::new(),
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

        info!("[STREAM_END] While loop exited. full_response length: {}", full_response.len());

        let state_for_save = self.state.clone();
        let full_response_clone = full_response.clone();
        tokio::task::spawn_blocking(
            move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                let mut sm = state_for_save.session_manager.blocking_lock();
                sm.save_message(session.id, user_id, 2, &full_response_clone, 2)?;
                Ok(())
            },
        )
        .await??;

        // Extract user_id and session_id before moving them into BotResponse
        let user_id_str = message.user_id.clone();
        let session_id_str = message.session_id.clone();

        #[cfg(feature = "chat")]
        let suggestions = get_suggestions(self.state.cache.as_ref(), &user_id_str, &session_id_str);
        #[cfg(not(feature = "chat"))]
        let suggestions: Vec<crate::core::shared::models::Suggestion> = Vec::new();

        let final_response = BotResponse {
            bot_id: message.bot_id,
            user_id: message.user_id,
            session_id: message.session_id,
            channel: message.channel,
            content: full_response,
            message_type: MessageType::BOT_RESPONSE,
            stream_token: None,
            is_complete: true,
            suggestions,
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
        let history = session_manager.get_conversation_history(session_id, user_id)?;
        Ok(history)
    }
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let session_id = params
        .get("session_id")
        .and_then(|s| Uuid::parse_str(s).ok());
    let user_id = params.get("user_id").and_then(|s| Uuid::parse_str(s).ok());

    // Extract bot_name from query params
    let bot_name = params
        .get("bot_name")
        .cloned()
        .unwrap_or_else(|| "default".to_string());

    if session_id.is_none() || user_id.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "session_id and user_id are required" })),
        )
            .into_response();
    }

    let session_id = session_id.unwrap_or_default();
    let user_id = user_id.unwrap_or_default();

    // Look up bot_id from bot_name
    let bot_id = {
        let conn = state.conn.get().ok();
        if let Some(mut db_conn) = conn {
            use crate::core::shared::models::schema::bots::dsl::*;

            // Try to parse as UUID first, if that fails treat as bot name
            let result: Result<Uuid, _> = if let Ok(uuid) = Uuid::parse_str(&bot_name) {
                // Parameter is a UUID, look up by id
                bots.filter(id.eq(uuid)).select(id).first(&mut db_conn)
            } else {
                // Parameter is a bot name, look up by name
                bots.filter(name.eq(&bot_name))
                    .select(id)
                    .first(&mut db_conn)
            };

            result.unwrap_or_else(|_| {
                log::warn!("Bot not found: {}, using nil bot_id", bot_name);
                Uuid::nil()
            })
        } else {
            log::warn!("Could not get database connection, using nil bot_id");
            Uuid::nil()
        }
    };

    ws.on_upgrade(move |socket| handle_websocket(socket, state, session_id, user_id, bot_id))
        .into_response()
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

    let welcome = serde_json::json!({
        "type": "connected",
        "session_id": session_id,
        "user_id": user_id,
        "bot_id": bot_id,
        "message": "Connected to bot server"
    });

    if let Ok(welcome_str) = serde_json::to_string(&welcome) {
        if sender.send(Message::Text(welcome_str)).await.is_err() {
            error!("Failed to send welcome message");
        }
    }

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
            // Check if start.bas has already been executed for this session
            let start_bas_key = format!("start_bas_executed:{}", session_id);
            let should_execute_start_bas = if let Some(cache) = &state.cache {
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
                let data_dir = "/opt/gbo/data";
                let start_script_path = format!("{}/{}.gbai/{}.gbdialog/start.bas", data_dir, bot_name, bot_name);

                info!("Looking for start.bas at: {}", start_script_path);

                if let Ok(metadata) = tokio::fs::metadata(&start_script_path).await {
                    if metadata.is_file() {
                        info!("Found start.bas file, reading contents...");
                        if let Ok(start_script) = tokio::fs::read_to_string(&start_script_path).await {
                        info!(
                            "Executing start.bas for bot {} on session {}",
                            bot_name, session_id
                        );

                        let state_for_start = state.clone();
                        let _tx_for_start = tx.clone();

                        tokio::spawn(async move {
                            let session_result = {
                                let mut sm = state_for_start.session_manager.lock().await;
                                sm.get_session_by_id(session_id)
                            };

                            if let Ok(Some(session)) = session_result {
                                info!("Executing start.bas for bot {} on session {}", bot_name, session_id);

                                // Clone state_for_start for use in Redis SET after execution
                                let state_for_redis = state_for_start.clone();

                                let result = tokio::task::spawn_blocking(move || {
                                    let mut script_service = crate::basic::ScriptService::new(
                                        state_for_start.clone(),
                                        session.clone()
                                    );
                                    script_service.load_bot_config_params(&state_for_start, bot_id);

                                    match script_service.compile(&start_script) {
                                        Ok(ast) => match script_service.run(&ast) {
                                            Ok(_) => Ok(()),
                                            Err(e) => Err(format!("Script execution error: {}", e)),
                                        },
                                        Err(e) => Err(format!("Script compilation error: {}", e)),
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
            }
            } // End of if should_execute_start_bas
        }
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

    let state_clone = state.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    info!("Received WebSocket message: {}", text);
                    if let Ok(user_msg) = serde_json::from_str::<UserMessage>(&text) {
                        let orchestrator = BotOrchestrator::new(state_clone.clone());
                        info!("[WS_DEBUG] Looking up response channel for session: {}", session_id);
                        if let Some(tx_clone) = state_clone
                            .response_channels
                            .lock()
                            .await
                            .get(&session_id.to_string())
                        {
                            info!("[WS_DEBUG] Response channel found, calling stream_response");

                            // Ensure session exists - create if not
                            let session_result = {
                                let mut sm = state_clone.session_manager.lock().await;
                                sm.get_session_by_id(session_id)
                            };

                            let session = match session_result {
                                Ok(Some(sess)) => {
                                    info!("[WS_DEBUG] Session exists: {}", session_id);
                                    sess
                                }
                                Ok(None) => {
                                    info!("[WS_DEBUG] Session not found, creating via session manager");
                                    // Use session manager to create session (will generate new UUID)
                                    let mut sm = state_clone.session_manager.lock().await;
                                    match sm.create_session(user_id, bot_id, "WebSocket Chat") {
                                        Ok(new_session) => {
                                            info!("[WS_DEBUG] Session created: {} (note: different from WebSocket session_id)", new_session.id);
                                            new_session
                                        }
                                        Err(e) => {
                                            error!("[WS_DEBUG] Failed to create session: {}", e);
                                            continue;
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("[WS_DEBUG] Error getting session: {}", e);
                                    continue;
                                }
                            };

                            // Use bot_id from WebSocket connection instead of from message
                            let corrected_msg = UserMessage {
                                bot_id: bot_id.to_string(),
                                user_id: session.user_id.to_string(),
                                session_id: session.id.to_string(),
                                ..user_msg
                            };
                            if let Err(e) = orchestrator
                                .stream_response(corrected_msg, tx_clone.clone())
                                .await
                            {
                                error!("Failed to stream response: {}", e);
                            }
                        } else {
                            warn!("[WS_DEBUG] Response channel NOT found for session: {}", session_id);
                        }
                    } else {
                        warn!("[WS_DEBUG] Failed to parse UserMessage from: {}", text);
                    }
                }
                Message::Close(_) => {
                    info!("WebSocket close message received");
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
