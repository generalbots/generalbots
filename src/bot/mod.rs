mod ui;

use crate::config::ConfigManager;
use crate::drive_monitor::DriveMonitor;
use crate::llm_models;
use crate::nvidia::get_system_metrics;
use crate::bot::ui::BotUI;
use crate::shared::models::{BotResponse, Suggestion, UserMessage, UserSession};
use crate::shared::state::AppState;
use actix_web::{web, HttpRequest, HttpResponse, Result};
use actix_ws::Message as WsMessage;
use chrono::Utc;
use diesel::PgConnection;
use log::{error, info, trace, warn};
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::Mutex as AsyncMutex;
use tokio::time::Instant;
use uuid::Uuid;

pub fn get_default_bot(conn: &mut PgConnection) -> (Uuid, String) {
    use crate::shared::models::schema::bots::dsl::*;
    use diesel::prelude::*;

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
            error!("Failed to query default bot: {}", e);
            (Uuid::nil(), "default".to_string())
        }
    }
}

pub struct BotOrchestrator {
    pub state: Arc<AppState>,
    pub mounted_bots: Arc<AsyncMutex<HashMap<String, Arc<DriveMonitor>>>>,
}

impl BotOrchestrator {
    pub fn new(state: Arc<AppState>) -> Self {
        let orchestrator = Self {
            state,
            mounted_bots: Arc::new(AsyncMutex::new(HashMap::new())),
        };

        // Spawn internal automation to run compact prompt every minute if enabled
        // Compact automation disabled to avoid Send issues in background task

        orchestrator
    }

    pub async fn mount_all_bots(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use crate::shared::models::schema::bots::dsl::*;
        use diesel::prelude::*;

        let mut db_conn = self.state.conn.lock().unwrap();
        let active_bots = bots
            .filter(is_active.eq(true))
            .select(id)
            .load::<uuid::Uuid>(&mut *db_conn)
            .map_err(|e| {
                error!("Failed to query active bots: {}", e);
                e
            })?;

        for bot_guid in active_bots {
            let state_clone = self.state.clone();
            let mounted_bots_clone = self.mounted_bots.clone();
            let bot_guid_str = bot_guid.to_string();

            tokio::spawn(async move {
                if let Err(e) =
                    Self::mount_bot_task(state_clone, mounted_bots_clone, bot_guid_str.clone())
                        .await
                {
                    error!("Failed to mount bot {}: {}", bot_guid_str, e);
                }
            });
        }

        Ok(())
    }

    async fn mount_bot_task(
        state: Arc<AppState>,
        mounted_bots: Arc<AsyncMutex<HashMap<String, Arc<DriveMonitor>>>>,
        bot_guid: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use crate::shared::models::schema::bots::dsl::*;
        use diesel::prelude::*;

        let bot_name: String = {
            let mut db_conn = state.conn.lock().unwrap();
            bots.filter(id.eq(Uuid::parse_str(&bot_guid)?))
                .select(name)
                .first(&mut *db_conn)
                .map_err(|e| {
                    error!("Failed to query bot name for {}: {}", bot_guid, e);
                    e
                })?
        };

        let bucket_name = format!("{}.gbai", bot_name);

        {
            let mounted = mounted_bots.lock().await;
            if mounted.contains_key(&bot_guid) {
                warn!("Bot {} is already mounted", bot_guid);
                return Ok(());
            }
        }

        let bot_id = Uuid::parse_str(&bot_guid)?;
        let drive_monitor = Arc::new(DriveMonitor::new(state.clone(), bucket_name, bot_id));
        let _handle = drive_monitor.clone().spawn().await;

        {
            let mut mounted = mounted_bots.lock().await;
            mounted.insert(bot_guid.clone(), drive_monitor);
        }

        Ok(())
    }

    pub async fn create_bot(
        &self,
        _bot_name: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }

    pub async fn mount_bot(
        &self,
        bot_guid: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let bot_guid = bot_guid
            .strip_suffix(".gbai")
            .unwrap_or(bot_guid)
            .to_string();

        use crate::shared::models::schema::bots::dsl::*;
        use diesel::prelude::*;

        let bot_name: String = {
            let mut db_conn = self.state.conn.lock().unwrap();
            bots.filter(id.eq(Uuid::parse_str(&bot_guid)?))
                .select(name)
                .first(&mut *db_conn)
                .map_err(|e| {
                    error!("Failed to query bot name for {}: {}", bot_guid, e);
                    e
                })?
        };

        let bucket_name = format!("{}.gbai", bot_name);

        {
            let mounted_bots = self.mounted_bots.lock().await;
            if mounted_bots.contains_key(&bot_guid) {
                warn!("Bot {} is already mounted", bot_guid);
                return Ok(());
            }
        }

        let bot_id = Uuid::parse_str(&bot_guid)?;
        let drive_monitor = Arc::new(DriveMonitor::new(self.state.clone(), bucket_name, bot_id));
        let _handle = drive_monitor.clone().spawn().await;

        {
            let mut mounted_bots = self.mounted_bots.lock().await;
            mounted_bots.insert(bot_guid.clone(), drive_monitor);
        }

        Ok(())
    }

    pub async fn handle_user_input(
        &self,
        session_id: Uuid,
        user_input: &str,
    ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        trace!(
            "Handling user input for session {}: '{}'",
            session_id,
            user_input
        );

        let mut session_manager = self.state.session_manager.lock().await;
        session_manager.provide_input(session_id, user_input.to_string())?;

        Ok(None)
    }

    pub async fn register_response_channel(
        &self,
        session_id: String,
        sender: mpsc::Sender<BotResponse>,
    ) {
        self.state
            .response_channels
            .lock()
            .await
            .insert(session_id.clone(), sender);
    }

    pub async fn unregister_response_channel(&self, session_id: &str) {
        self.state.response_channels.lock().await.remove(session_id);
    }

    pub async fn send_event(
        &self,
        user_id: &str,
        bot_id: &str,
        session_id: &str,
        channel: &str,
        event_type: &str,
        data: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        trace!(
            "Sending event '{}' to session {} on channel {}",
            event_type,
            session_id,
            channel
        );

        let event_response = BotResponse {
            bot_id: bot_id.to_string(),
            user_id: user_id.to_string(),
            session_id: session_id.to_string(),
            channel: channel.to_string(),
            content: serde_json::to_string(&serde_json::json!({
                "event": event_type,
                "data": data
            }))?,
            message_type: 2,
            stream_token: None,
            is_complete: true,
            suggestions: Vec::new(),
            context_name: None,
            context_length: 0,
            context_max_length: 0,
        };

        if let Some(adapter) = self.state.channels.lock().unwrap().get(channel) {
            adapter.send_message(event_response).await?;
        } else {
            warn!("No channel adapter found for channel: {}", channel);
        }

        Ok(())
    }

    pub async fn handle_context_change(
        &self,
        user_id: &str,
        bot_id: &str,
        session_id: &str,
        channel: &str,
        context_name: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        trace!(
            "Changing context for session {} to {}",
            session_id,
            context_name
        );

        let session_uuid = Uuid::parse_str(session_id).map_err(|e| {
            error!("Failed to parse session_id: {}", e);
            e
        })?;

        let user_uuid = Uuid::parse_str(user_id).map_err(|e| {
            error!("Failed to parse user_id: {}", e);
            e
        })?;

        if let Err(e) = self
            .state
            .session_manager
            .lock()
            .await
            .update_session_context(&session_uuid, &user_uuid, context_name.to_string())
            .await
        {
            error!("Failed to update session context: {}", e);
        }

        let confirmation = BotResponse {
            bot_id: bot_id.to_string(),
            user_id: user_id.to_string(),
            session_id: session_id.to_string(),
            channel: channel.to_string(),
            content: "Context changed".to_string(),
            message_type: 5,
            stream_token: None,
            is_complete: true,
            suggestions: Vec::new(),
            context_name: Some(context_name.to_string()),
            context_length: 0,
            context_max_length: 0,
        };

        if let Some(adapter) = self.state.channels.lock().unwrap().get(channel) {
            adapter.send_message(confirmation).await?;
        }

        Ok(())
    }

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

        let suggestions = if let Some(redis) = &self.state.cache {
            let mut conn = redis.get_multiplexed_async_connection().await?;
            let redis_key = format!("suggestions:{}:{}", message.user_id, message.session_id);

            let suggestions: Vec<String> = redis::cmd("LRANGE")
                .arg(&redis_key)
                .arg(0)
                .arg(-1)
                .query_async(&mut conn)
                .await?;

            let mut seen = std::collections::HashSet::new();
            suggestions
                .into_iter()
                .filter_map(|s| serde_json::from_str::<Suggestion>(&s).ok())
                .filter(|s| seen.insert((s.text.clone(), s.context.clone())))
                .collect()
        } else {
            Vec::new()
        };

        let user_id = Uuid::parse_str(&message.user_id).map_err(|e| {
            error!("Invalid user ID: {}", e);
            e
        })?;

        // Acquire lock briefly for DB access, then release before awaiting
        let session_id = Uuid::parse_str(&message.session_id).map_err(|e| {
            error!("Invalid session ID: {}", e);
            e
        })?;
        let session = {
            let mut sm = self.state.session_manager.lock().await;
            sm.get_session_by_id(session_id)?
        }
        .ok_or_else(|| "Failed to create session")?;

        // Save user message to history
        {
            let mut sm = self.state.session_manager.lock().await;
            sm.save_message(session.id, user_id, 1, &message.content, 1)?;
        }

        if message.message_type == 4 {
            if let Some(context_name) = &message.context_name {
                let _ = self
                    .handle_context_change(
                        &message.user_id,
                        &message.bot_id,
                        &message.session_id,
                        &message.channel,
                        context_name,
                    )
                    .await;
            }
        }

        let system_prompt = std::env::var("SYSTEM_PROMPT").unwrap_or_default();

        // Acquire lock briefly for context retrieval
        let context_data = {
            let sm = self.state.session_manager.lock().await;
            sm.get_session_context_data(&session.id, &session.user_id)
                .await?
        };

        // Get history limit from bot config (default -1 for unlimited)
        let history_limit = {
            let config_manager = ConfigManager::new(Arc::clone(&self.state.conn));
            config_manager
                .get_config(
                    &Uuid::parse_str(&message.bot_id).unwrap_or_default(),
                    "prompt-history",
                    None,
                )
                .unwrap_or_default()
                .parse::<i32>()
                .unwrap_or(-1)
        };

        // Acquire lock briefly for history retrieval with configurable limit
        let history = {
            let mut sm = self.state.session_manager.lock().await;
            let mut history = sm.get_conversation_history(session.id, user_id)?;

            // Skip all messages before the most recent compacted message (type 9)
            if let Some(last_compacted_index) = history
                .iter()
                .rposition(|(role, content)| role == "COMPACTED" || content.starts_with("SUMMARY:"))
            {
                history = history.split_off(last_compacted_index);
            }

            if history_limit > 0 && history.len() > history_limit as usize {
                let start = history.len() - history_limit as usize;
                history.drain(0..start);
            }
            history
        };

        let mut prompt = String::new();
        if !system_prompt.is_empty() {
            prompt.push_str(&format!("SYSTEM: *** {} *** \n", system_prompt));
        }
        if !context_data.is_empty() {
            prompt.push_str(&format!("CONTEXT: *** {} *** \n", context_data));
        }
        for (role, content) in &history {
            prompt.push_str(&format!("{}:{}\n", role, content));
        }
        prompt.push_str(&format!("Human: {}\nBot:", message.content));

        trace!(
            "Stream prompt constructed with {} history entries",
            history.len()
        );

        let (stream_tx, mut stream_rx) = mpsc::channel::<String>(100);
        let llm = self.state.llm_provider.clone();

        if message.channel == "web" {
            self.send_event(
                &message.user_id,
                &message.bot_id,
                &message.session_id,
                &message.channel,
                "thinking_start",
                serde_json::json!({}),
            )
            .await?;
        } else {
            let thinking_response = BotResponse {
                bot_id: message.bot_id.clone(),
                user_id: message.user_id.clone(),
                session_id: message.session_id.clone(),
                channel: message.channel.clone(),
                content: "Thinking...".to_string(),
                message_type: 1,
                stream_token: None,
                is_complete: true,
                suggestions: Vec::new(),
                context_name: None,
                context_length: 0,
                context_max_length: 0,
            };
            response_tx.send(thinking_response).await?;
        }

        let prompt_clone = prompt.clone();
        tokio::spawn(async move {
            if let Err(e) = llm
                .generate_stream(&prompt_clone, &serde_json::Value::Null, stream_tx)
                .await
            {
                error!("LLM streaming error: {}", e);
            }
        });

        let mut full_response = String::new();
        let mut analysis_buffer = String::new();
        let mut in_analysis = false;
        let mut chunk_count = 0;
        let mut first_word_received = false;
        let mut last_progress_update = Instant::now();
        let progress_interval = Duration::from_secs(1);

        // Calculate initial token count
        let initial_tokens = crate::shared::utils::estimate_token_count(&prompt);
        let config_manager = ConfigManager::new(Arc::clone(&self.state.conn));
        let max_context_size = config_manager
            .get_config(
                &Uuid::parse_str(&message.bot_id).unwrap_or_default(),
                "llm-server-ctx-size",
                None,
            )
            .unwrap_or_default()
            .parse::<usize>()
            .unwrap_or(0);

        // Show initial progress
        if let Ok(metrics) = get_system_metrics(initial_tokens, max_context_size) {
        }
        let model = config_manager
            .get_config(
                &Uuid::parse_str(&message.bot_id).unwrap_or_default(),
                "llm-model",
                None,
            )
            .unwrap_or_default();
        let handler = llm_models::get_handler(&model);

        while let Some(chunk) = stream_rx.recv().await {
            chunk_count += 1;

            if !first_word_received && !chunk.trim().is_empty() {
                first_word_received = true;
            }

            analysis_buffer.push_str(&chunk);

            if handler.has_analysis_markers(&analysis_buffer) && !in_analysis {
                in_analysis = true;
            }

            if in_analysis && handler.is_analysis_complete(&analysis_buffer) {
                in_analysis = false;
                analysis_buffer.clear();

                if message.channel == "web" {
                    let orchestrator = BotOrchestrator::new(Arc::clone(&self.state));
                    orchestrator
                        .send_event(
                            &message.user_id,
                            &message.bot_id,
                            &message.session_id,
                            &message.channel,
                            "thinking_end",
                            serde_json::json!({"user_id": message.user_id.clone()}),
                        )
                        .await
                        .ok();
                }
                continue;
            }

            if !in_analysis {
                full_response.push_str(&chunk);

                // Update progress if interval elapsed
                if last_progress_update.elapsed() >= progress_interval {
                    let current_tokens =
                        initial_tokens + crate::shared::utils::estimate_token_count(&full_response);
                    if let Ok(metrics) = get_system_metrics(current_tokens, max_context_size) {
                        let gpu_bar =
                            "█".repeat((metrics.gpu_usage.unwrap_or(0.0) / 5.0).round() as usize);
                        let cpu_bar = "█".repeat((metrics.cpu_usage / 5.0).round() as usize);
                        let token_ratio = current_tokens as f64 / max_context_size.max(1) as f64;
                        let token_bar = "█".repeat((token_ratio * 20.0).round() as usize);
                        let mut ui = BotUI::new().unwrap();
                        ui.render_progress(current_tokens, max_context_size).unwrap();
                    }
                    last_progress_update = Instant::now();
                }

                let partial = BotResponse {
                    bot_id: message.bot_id.clone(),
                    user_id: message.user_id.clone(),
                    session_id: message.session_id.clone(),
                    channel: message.channel.clone(),
                    content: chunk,
                    message_type: 1,
                    stream_token: None,
                    is_complete: false,
                    suggestions: suggestions.clone(),
                    context_name: None,
                    context_length: 0,
                    context_max_length: 0,
                };

                if response_tx.send(partial).await.is_err() {
                    break;
                }
            }
        }

        trace!(
            "Stream processing completed, {} chunks processed",
            chunk_count
        );

        // Sum tokens from all p.push context builds before submission
        let total_tokens = crate::shared::utils::estimate_token_count(&prompt)
            + crate::shared::utils::estimate_token_count(&context_data)
            + crate::shared::utils::estimate_token_count(&full_response);
        info!(
            "Total tokens (context + prompt + response): {}",
            total_tokens
        );

        // Trigger compact prompt if enabled
        let config_manager = ConfigManager::new(Arc::clone(&self.state.conn));
        let compact_enabled = config_manager
            .get_config(
                &Uuid::parse_str(&message.bot_id).unwrap_or_default(),
                "prompt-compact",
                None,
            )
            .unwrap_or_default()
            .parse::<i32>()
            .unwrap_or(0);
        if compact_enabled > 0 {
            let state = self.state.clone();
            tokio::task::spawn_blocking(move || loop {
                if let Err(e) = tokio::runtime::Handle::current()
                    .block_on(crate::automation::execute_compact_prompt(state.clone()))
                {
                    error!("Failed to execute compact prompt: {}", e);
                }
                std::thread::sleep(Duration::from_secs(60));
            });
        }

        // Save final message with short lock scope
        {
            let mut sm = self.state.session_manager.lock().await;
            sm.save_message(session.id, user_id, 2, &full_response, 1)?;
        }

        let config_manager = ConfigManager::new(Arc::clone(&self.state.conn));
        let max_context_size = config_manager
            .get_config(
                &Uuid::parse_str(&message.bot_id).unwrap_or_default(),
                "llm-server-ctx-size",
                None,
            )
            .unwrap_or_default()
            .parse::<usize>()
            .unwrap_or(0);

        let current_context_length = crate::shared::utils::estimate_token_count(&context_data);

        let final_msg = BotResponse {
            bot_id: message.bot_id,
            user_id: message.user_id,
            session_id: message.session_id,
            channel: message.channel,
            content: String::new(),
            message_type: 1,
            stream_token: None,
            is_complete: true,
            suggestions,
            context_name: None,
            context_length: current_context_length,
            context_max_length: max_context_size,
        };

        response_tx.send(final_msg).await?;
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
        trace!(
            "Getting conversation history for session {} user {}",
            session_id,
            user_id
        );

        let mut session_manager = self.state.session_manager.lock().await;
        let history = session_manager.get_conversation_history(session_id, user_id)?;
        Ok(history)
    }

    pub async fn run_start_script(
        session: &UserSession,
        state: Arc<AppState>,
        token: Option<String>,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        trace!(
            "Running start script for session: {} with token: {:?}",
            session.id,
            token
        );

        use crate::shared::models::schema::bots::dsl::*;
        use diesel::prelude::*;

        let bot_id = session.bot_id;

        let bot_name: String = {
            let mut db_conn = state.conn.lock().unwrap();
            bots.filter(id.eq(Uuid::parse_str(&bot_id.to_string())?))
                .select(name)
                .first(&mut *db_conn)
                .map_err(|e| {
                    error!("Failed to query bot name for {}: {}", bot_id, e);
                    e
                })?
        };

        let start_script_path = format!("./work/{}.gbai/{}.gbdialog/start.ast", bot_name, bot_name);

        let start_script = match std::fs::read_to_string(&start_script_path) {
            Ok(content) => content,
            Err(_) => {
                warn!("start.bas not found at {}, skipping", start_script_path);
                return Ok(true);
            }
        };

        trace!(
            "Start script content for session {}: {}",
            session.id,
            start_script
        );

        let session_clone = session.clone();
        let state_clone = state.clone();
        let script_service = crate::basic::ScriptService::new(state_clone, session_clone.clone());

        match tokio::time::timeout(std::time::Duration::from_secs(10), async {
            script_service
                .compile(&start_script)
                .and_then(|ast| script_service.run(&ast))
        })
        .await
        {
            Ok(Ok(result)) => {
                info!(
                    "Start script executed successfully for session {}, result: {}",
                    session_clone.id, result
                );
                Ok(true)
            }
            Ok(Err(e)) => {
                error!(
                    "Failed to run start script for session {}: {}",
                    session_clone.id, e
                );
                Ok(false)
            }
            Err(_) => {
                error!("Start script timeout for session {}", session_clone.id);
                Ok(false)
            }
        }
    }

    pub async fn send_warning(
        &self,
        session_id: &str,
        channel: &str,
        message: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        warn!(
            "Sending warning to session {} on channel {}: {}",
            session_id, channel, message
        );

        let mut ui = BotUI::new().unwrap();
        ui.render_warning(message).unwrap();
        Ok(())
    }

    pub async fn trigger_auto_welcome(
        &self,
        session_id: &str,
        user_id: &str,
        _bot_id: &str,
        token: Option<String>,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        trace!(
            "Triggering auto welcome for user: {}, session: {}, token: {:?}",
            user_id,
            session_id,
            token
        );

        let session_uuid = Uuid::parse_str(session_id).map_err(|e| {
            error!("Invalid session ID: {}", e);
            e
        })?;

        let session = {
            let mut session_manager = self.state.session_manager.lock().await;
            match session_manager.get_session_by_id(session_uuid)? {
                Some(session) => session,
                None => {
                    error!("Failed to create session for auto welcome");
                    return Ok(false);
                }
            }
        };

        let result = match tokio::time::timeout(
            std::time::Duration::from_secs(5),
            Self::run_start_script(&session, Arc::clone(&self.state), token),
        )
        .await
        {
            Ok(Ok(result)) => result,
            Ok(Err(e)) => {
                error!("Auto welcome script error: {}", e);
                false
            }
            Err(_) => {
                error!("Auto welcome timeout for session: {}", session_id);
                false
            }
        };

        info!(
            "Auto welcome completed for session: {} with result: {}",
            session_id, result
        );
        Ok(result)
    }
}

impl Default for BotOrchestrator {
    fn default() -> Self {
        panic!("BotOrchestrator::default is not supported; instantiate with BotOrchestrator::new(state)");
    }
}

#[actix_web::get("/ws")]
async fn websocket_handler(
    req: HttpRequest,
    stream: web::Payload,
    data: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    let query = web::Query::<HashMap<String, String>>::from_query(req.query_string()).unwrap();

    let session_id = query.get("session_id").cloned().unwrap();
    let user_id_string = query
        .get("user_id")
        .cloned()
        .unwrap_or_else(|| Uuid::new_v4().to_string())
        .replace("undefined", &Uuid::new_v4().to_string());

    // Acquire lock briefly, then release before performing blocking DB operations
    let user_id = {
        let user_uuid = Uuid::parse_str(&user_id_string).unwrap_or_else(|_| Uuid::new_v4());
        let result = {
            let mut sm = data.session_manager.lock().await;
            sm.get_or_create_anonymous_user(Some(user_uuid))
        };
        match result {
            Ok(uid) => uid.to_string(),
            Err(e) => {
                error!("Failed to ensure user exists for WebSocket: {}", e);
                user_id_string
            }
        }
    };

    let (res, mut session, mut msg_stream) = actix_ws::handle(&req, stream)?;
    let (tx, mut rx) = mpsc::channel::<BotResponse>(100);

    let orchestrator = BotOrchestrator::new(Arc::clone(&data));
    orchestrator
        .register_response_channel(session_id.clone(), tx.clone())
        .await;

    data.web_adapter
        .add_connection(session_id.clone(), tx.clone())
        .await;

    data.voice_adapter
        .add_connection(session_id.clone(), tx.clone())
        .await;

    let bot_id: String = {
        use crate::shared::models::schema::bots::dsl::*;
        use diesel::prelude::*;

        let mut db_conn = data.conn.lock().unwrap();
        match bots
            .filter(is_active.eq(true))
            .select(id)
            .first::<Uuid>(&mut *db_conn)
            .optional()
        {
            Ok(Some(first_bot_id)) => first_bot_id.to_string(),
            Ok(None) => {
                warn!("No active bots found");
                Uuid::nil().to_string()
            }
            Err(e) => {
                error!("DB error: {}", e);
                Uuid::nil().to_string()
            }
        }
    };

    orchestrator
        .send_event(
            &user_id,
            &bot_id,
            &session_id,
            "web",
            "session_start",
            serde_json::json!({
                "session_id": session_id,
                "user_id": user_id,
                "timestamp": Utc::now().to_rfc3339()
            }),
        )
        .await
        .ok();

    info!(
        "WebSocket connection established for session: {}, user: {}",
        session_id, user_id
    );

    let orchestrator_clone = BotOrchestrator::new(Arc::clone(&data));
    let user_id_welcome = user_id.clone();
    let session_id_welcome = session_id.clone();
    let bot_id_welcome = bot_id.clone();

    actix_web::rt::spawn(async move {
        match tokio::time::timeout(
            std::time::Duration::from_secs(3),
            orchestrator_clone.trigger_auto_welcome(
                &session_id_welcome,
                &user_id_welcome,
                &bot_id_welcome,
                None,
            ),
        )
        .await
        {
            Ok(Ok(_)) => {
                trace!("Auto welcome completed successfully");
            }
            Ok(Err(e)) => {
                warn!("Failed to trigger auto welcome: {}", e);
            }
            Err(_) => {
                warn!("Auto welcome timeout");
            }
        }
    });

    let web_adapter = data.web_adapter.clone();
    let session_id_clone1 = session_id.clone();
    let session_id_clone2 = session_id.clone();
    let user_id_clone = user_id.clone();

    actix_web::rt::spawn(async move {
        trace!(
            "Starting WebSocket sender for session {}",
            session_id_clone1
        );
        let mut message_count = 0;

        while let Some(msg) = rx.recv().await {
            message_count += 1;
            if let Ok(json) = serde_json::to_string(&msg) {
                if let Err(e) = session.text(json).await {
                    warn!("Failed to send WebSocket message {}: {}", message_count, e);
                    break;
                }
            }
        }

        trace!(
            "WebSocket sender terminated for session {}, sent {} messages",
            session_id_clone1,
            message_count
        );
    });

    actix_web::rt::spawn(async move {
        trace!(
            "Starting WebSocket receiver for session {}",
            session_id_clone2
        );
        let mut message_count = 0;

        while let Some(Ok(msg)) = msg_stream.recv().await {
            match msg {
                WsMessage::Text(text) => {
                    message_count += 1;

                    let bot_id = {
                        use crate::shared::models::schema::bots::dsl::*;
                        use diesel::prelude::*;

                        let mut db_conn = data.conn.lock().unwrap();
                        match bots
                            .filter(is_active.eq(true))
                            .select(id)
                            .first::<Uuid>(&mut *db_conn)
                            .optional()
                        {
                            Ok(Some(first_bot_id)) => first_bot_id.to_string(),
                            Ok(None) => {
                                warn!("No active bots found");
                                Uuid::nil().to_string()
                            }
                            Err(e) => {
                                error!("DB error: {}", e);
                                Uuid::nil().to_string()
                            }
                        }
                    };

                    let json_value: serde_json::Value = match serde_json::from_str(&text) {
                        Ok(value) => value,
                        Err(e) => {
                            error!("Error parsing JSON message {}: {}", message_count, e);
                            continue;
                        }
                    };

                    let content = json_value["content"]
                        .as_str()
                        .map(|s| s.to_string())
                        .unwrap();

                    let user_message = UserMessage {
                        bot_id,
                        user_id: user_id_clone.clone(),
                        session_id: session_id_clone2.clone(),
                        channel: "web".to_string(),
                        content,
                        message_type: json_value["message_type"].as_u64().unwrap_or(1) as i32,
                        media_url: None,
                        timestamp: Utc::now(),
                        context_name: json_value["context_name"].as_str().map(|s| s.to_string()),
                    };

                    if let Err(e) = orchestrator.stream_response(user_message, tx.clone()).await {
                        error!("Failed to stream response: {}", e);
                    }
                }
                WsMessage::Close(reason) => {
                    trace!(
                        "WebSocket closing for session {} - reason: {:?}",
                        session_id_clone2,
                        reason
                    );

                    let bot_id = {
                        use crate::shared::models::schema::bots::dsl::*;
                        use diesel::prelude::*;

                        let mut db_conn = data.conn.lock().unwrap();
                        match bots
                            .filter(is_active.eq(true))
                            .select(id)
                            .first::<Uuid>(&mut *db_conn)
                            .optional()
                        {
                            Ok(Some(first_bot_id)) => first_bot_id.to_string(),
                            Ok(None) => {
                                error!("No active bots found");
                                "".to_string()
                            }
                            Err(e) => {
                                error!("Failed to query bots: {}", e);
                                "".to_string()
                            }
                        }
                    };

                    if let Err(e) = orchestrator
                        .send_event(
                            &user_id_clone,
                            &bot_id,
                            &session_id_clone2,
                            "web",
                            "session_end",
                            serde_json::json!({}),
                        )
                        .await
                    {
                        error!("Failed to send session_end event: {}", e);
                    }

                    web_adapter.remove_connection(&session_id_clone2).await;
                    orchestrator
                        .unregister_response_channel(&session_id_clone2)
                        .await;

                    if let Err(e) = data.llm_provider.cancel_job(&session_id_clone2).await {
                        warn!(
                            "Failed to cancel LLM job for session {}: {}",
                            session_id_clone2, e
                        );
                    }

                    break;
                }
                _ => {}
            }
        }

        trace!(
            "WebSocket receiver terminated for session {}, processed {} messages",
            session_id_clone2,
            message_count
        );
    });

    info!(
        "WebSocket handler setup completed for session {}",
        session_id
    );
    Ok(res)
}

#[actix_web::post("/api/bot/create")]
async fn create_bot_handler(
    data: web::Data<AppState>,
    info: web::Json<HashMap<String, String>>,
) -> Result<HttpResponse> {
    let bot_name = info
        .get("bot_name")
        .cloned()
        .unwrap_or("default".to_string());

    let orchestrator = BotOrchestrator::new(Arc::clone(&data));

    if let Err(e) = orchestrator.create_bot(&bot_name).await {
        error!("Failed to create bot: {}", e);
        return Ok(
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        );
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({"status": "bot_created"})))
}

#[actix_web::post("/api/bot/mount")]
async fn mount_bot_handler(
    data: web::Data<AppState>,
    info: web::Json<HashMap<String, String>>,
) -> Result<HttpResponse> {
    let bot_guid = info.get("bot_guid").cloned().unwrap_or_default();

    let orchestrator = BotOrchestrator::new(Arc::clone(&data));

    if let Err(e) = orchestrator.mount_bot(&bot_guid).await {
        error!("Failed to mount bot: {}", e);
        return Ok(
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        );
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({"status": "bot_mounted"})))
}

#[actix_web::post("/api/bot/input")]
async fn handle_user_input_handler(
    data: web::Data<AppState>,
    info: web::Json<HashMap<String, String>>,
) -> Result<HttpResponse> {
    let session_id = info.get("session_id").cloned().unwrap_or_default();
    let user_input = info.get("input").cloned().unwrap_or_default();

    let orchestrator = BotOrchestrator::new(Arc::clone(&data));
    let session_uuid = Uuid::parse_str(&session_id).unwrap_or(Uuid::nil());

    if let Err(e) = orchestrator
        .handle_user_input(session_uuid, &user_input)
        .await
    {
        error!("Failed to handle user input: {}", e);
        return Ok(
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        );
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({"status": "input_processed"})))
}

#[actix_web::get("/api/bot/sessions/{user_id}")]
async fn get_user_sessions_handler(
    data: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let user_id = path.into_inner();

    let orchestrator = BotOrchestrator::new(Arc::clone(&data));

    match orchestrator.get_user_sessions(user_id).await {
        Ok(sessions) => Ok(HttpResponse::Ok().json(sessions)),
        Err(e) => {
            error!("Failed to get user sessions: {}", e);
            Ok(HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": e.to_string()})))
        }
    }
}

#[actix_web::get("/api/bot/history/{session_id}/{user_id}")]
async fn get_conversation_history_handler(
    data: web::Data<AppState>,
    path: web::Path<(Uuid, Uuid)>,
) -> Result<HttpResponse> {
    let (session_id, user_id) = path.into_inner();

    let orchestrator = BotOrchestrator::new(Arc::clone(&data));

    match orchestrator
        .get_conversation_history(session_id, user_id)
        .await
    {
        Ok(history) => Ok(HttpResponse::Ok().json(history)),
        Err(e) => {
            error!("Failed to get conversation history: {}", e);
            Ok(HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": e.to_string()})))
        }
    }
}

#[actix_web::post("/api/warn")]
async fn send_warning_handler(
    data: web::Data<AppState>,
    info: web::Json<HashMap<String, String>>,
) -> Result<HttpResponse> {
    let default_session = "default".to_string();
    let default_channel = "web".to_string();
    let default_message = "Warning!".to_string();

    let session_id = info.get("session_id").unwrap_or(&default_session);
    let channel = info.get("channel").unwrap_or(&default_channel);
    let message = info.get("message").unwrap_or(&default_message);

    trace!(
        "Sending warning via API - session: {}, channel: {}",
        session_id,
        channel
    );

    let orchestrator = BotOrchestrator::new(Arc::clone(&data));

    if let Err(e) = orchestrator
        .send_warning(session_id, channel, message)
        .await
    {
        error!("Failed to send warning: {}", e);
        return Ok(
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        );
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({"status": "warning_sent"})))
}
