use crate::channels::ChannelAdapter;
use crate::config::ConfigManager;
use crate::context::langcache::get_langcache_client;
use crate::drive_monitor::DriveMonitor;
use crate::kb::embeddings::generate_embeddings;
use crate::kb::qdrant_client::{ensure_collection_exists, get_qdrant_client, QdrantPoint};
use crate::llm_models;
use crate::shared::models::{BotResponse, Suggestion, UserMessage, UserSession};
use crate::shared::state::AppState;
use actix_web::{web, HttpRequest, HttpResponse, Result};
use actix_ws::Message as WsMessage;
use chrono::Utc;
use diesel::PgConnection;
use log::{debug, error, info, warn};
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex as AsyncMutex;
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
        Self {
            state,
            mounted_bots: Arc::new(AsyncMutex::new(HashMap::new())),
        }
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

        info!("Bot {} mounted successfully", bot_guid);
        Ok(())
    }

    pub async fn create_bot(
        &self,
        bot_name: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Move logic to here after duplication refactor

        let bucket_name = format!("{}.gbai", bot_name);
        crate::create_bucket::create_bucket(&bucket_name)?;
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
        info!(
            "Handling user input for session {}: '{}'",
            session_id, user_input
        );
        let mut session_manager = self.state.session_manager.lock().await;
        session_manager.provide_input(session_id, user_input.to_string())?;
        Ok(None)
    }

    pub async fn is_waiting_for_input(&self, session_id: Uuid) -> bool {
        let session_manager = self.state.session_manager.lock().await;
        session_manager.is_waiting_for_input(&session_id)
    }

    pub fn add_channel(&self, channel_type: &str, adapter: Arc<dyn ChannelAdapter>) {
        self.state
            .channels
            .lock()
            .unwrap()
            .insert(channel_type.to_string(), adapter);
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

    pub async fn set_user_answer_mode(
        &self,
        user_id: &str,
        bot_id: &str,
        mode: i32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "Setting answer mode for user {} with bot {} to mode {}",
            user_id, bot_id, mode
        );
        let mut session_manager = self.state.session_manager.lock().await;
        session_manager.update_answer_mode(user_id, bot_id, mode)?;
        Ok(())
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
        info!(
            "Sending event '{}' to session {} on channel {}",
            event_type, session_id, channel
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

    pub async fn send_direct_message(
        &self,
        session_id: &str,
        channel: &str,
        content: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "Sending direct message to session {}: '{}'",
            session_id, content
        );
        let (bot_id, _) = get_default_bot(&mut self.state.conn.lock().unwrap());
        let bot_response = BotResponse {
            bot_id: bot_id.to_string(),
            user_id: "default_user".to_string(),
            session_id: session_id.to_string(),
            channel: channel.to_string(),
            content: content.to_string(),
            message_type: 1,
            stream_token: None,
            is_complete: true,
            suggestions: Vec::new(),
            context_name: None,
            context_length: 0,
            context_max_length: 0,
        };

        if let Some(adapter) = self.state.channels.lock().unwrap().get(channel) {
            adapter.send_message(bot_response).await?;
        } else {
            warn!(
                "No channel adapter found for direct message on channel: {}",
                channel
            );
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
        info!(
            "Changing context for session {} to {}",
            session_id, context_name
        );

        // Use session manager to update context
        let session_uuid = Uuid::parse_str(session_id).map_err(|e| {
            error!("Failed to parse session_id: {}", e);
            e
        })?;
        let user_uuid = Uuid::parse_str(user_id).map_err(|e| {
            error!("Failed to parse user_id: {}", e);
            e
        })?;
        if let Err(e) = self.state.session_manager.lock().await.update_session_context(
            &session_uuid,
            &user_uuid,
            context_name.to_string()
        ).await {
            error!("Failed to update session context: {}", e);
        }

        // Send confirmation back to client
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

    pub async fn process_message(
        &self,
        message: UserMessage,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "Processing message from channel: {}, user: {}, session: {}",
            message.channel, message.user_id, message.session_id
        );
        debug!(
            "Message content: '{}', type: {}",
            message.content, message.message_type
        );

        let user_id = Uuid::parse_str(&message.user_id).map_err(|e| {
            error!("Invalid user ID provided: {}", e);
            e
        })?;

        let bot_id = Uuid::nil();
        let session = {
            let mut sm = self.state.session_manager.lock().await;
            let session_id = Uuid::parse_str(&message.session_id).map_err(|e| {
                error!("Invalid session ID: {}", e);
                e
            })?;

            match sm.get_session_by_id(session_id)? {
                Some(session) => session,
                None => {
                    error!(
                        "Failed to create session for user {} with bot {}",
                        user_id, bot_id
                    );
                    return Err("Failed to create session".into());
                }
            }
        };

        if self.is_waiting_for_input(session.id).await {
            debug!(
                "Session {} is waiting for input, processing as variable input",
                session.id
            );
            if let Some(variable_name) =
                self.handle_user_input(session.id, &message.content).await?
            {
                info!(
                    "Stored user input in variable '{}' for session {}",
                    variable_name, session.id
                );
                if let Some(adapter) = self.state.channels.lock().unwrap().get(&message.channel) {
                    let ack_response = BotResponse {
                        bot_id: message.bot_id.clone(),
                        user_id: message.user_id.clone(),
                        session_id: message.session_id.clone(),
                        channel: message.channel.clone(),
                        content: format!("Input stored in '{}'", variable_name),
                        message_type: 1,
                        stream_token: None,
                        is_complete: true,
                        suggestions: Vec::new(),
                        context_name: None,
                        context_length: 0,
                        context_max_length: 0,
                    };
                    adapter.send_message(ack_response).await?;
                }
            }
            return Ok(());
        }

        if session.answer_mode == 1 && session.current_tool.is_some() {
            self.state.tool_manager.provide_user_response(
                &message.user_id,
                &message.bot_id,
                message.content.clone(),
            )?;
            return Ok(());
        }

        {
            let mut session_manager = self.state.session_manager.lock().await;
            session_manager.save_message(
                session.id,
                user_id,
                1,
                &message.content,
                message.message_type,
            )?;
        }

        let response_content = self.direct_mode_handler(&message, &session).await?;

        {
            let mut session_manager = self.state.session_manager.lock().await;
            session_manager.save_message(session.id, user_id, 2, &response_content, 1)?;
        }

        // Handle context change messages (type 4) first
        if message.message_type == 4 {
            if let Some(context_name) = &message.context_name {
                return self
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

        // Create regular response
        let channel = message.channel.clone();
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

        let current_context_length = 0usize;

        let bot_response = BotResponse {
            bot_id: message.bot_id,
            user_id: message.user_id,
            session_id: message.session_id,
            channel: channel.clone(),
            content: response_content,
            message_type: 1,
            stream_token: None,
            is_complete: true,
            suggestions: Vec::new(),
            context_name: None,
            context_length: current_context_length,
            context_max_length: max_context_size,
        };

        if let Some(adapter) = self.state.channels.lock().unwrap().get(&channel) {
            adapter.send_message(bot_response).await?;
        } else {
            warn!("No channel adapter found for message channel: {}", channel);
        }

        Ok(())
    }

    async fn direct_mode_handler(
        &self,
        message: &UserMessage,
        session: &UserSession,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let system_prompt = std::env::var("SYSTEM_PROMPT").unwrap_or_default();
        let context_data = {
            let session_manager = self.state.session_manager.lock().await;
            session_manager
                .get_session_context_data(&session.id, &session.user_id)
                .await?
        };

        let mut prompt = String::new();
        if !system_prompt.is_empty() {
            prompt.push_str(&format!("System: {}\n", system_prompt));
        }
        if !context_data.is_empty() {
            prompt.push_str(&format!("Context: {}\n", context_data));
        }

        let history = {
            let mut session_manager = self.state.session_manager.lock().await;
            session_manager.get_conversation_history(session.id, session.user_id)?
        };

        // Deduplicate consecutive messages from same role
        let mut deduped_history: Vec<(String, String)> = Vec::new();
        let mut last_role = None;
        for (role, content) in history.iter() {
            if last_role != Some(role) || !deduped_history.is_empty() && 
               content != &deduped_history.last().unwrap().1 {
                deduped_history.push((role.clone(), content.clone()));
                last_role = Some(role);
            }
        }

        let recent_history = if deduped_history.len() > 10 {
            &deduped_history[deduped_history.len() - 10..]
        } else {
            &deduped_history[..]
        };

        for (role, content) in recent_history {
            prompt.push_str(&format!("{}: {}\n", role, content));
        }

        prompt.push_str(&format!("User: {}\nAssistant:", message.content));

        let use_langcache = std::env::var("LLM_CACHE")
            .unwrap_or_else(|_| "false".to_string())
            .eq_ignore_ascii_case("true");

        if use_langcache {
            ensure_collection_exists(&self.state, "semantic_cache").await?;
            let langcache_client = get_langcache_client()?;
            let isolated_question = message.content.trim().to_string();
            let question_embeddings = generate_embeddings(vec![isolated_question.clone()]).await?;
            let question_embedding = question_embeddings
                .get(0)
                .ok_or_else(|| "Failed to generate embedding for question")?
                .clone();

            let search_results = langcache_client
                .search("semantic_cache", question_embedding.clone(), 1)
                .await?;

            if let Some(result) = search_results.first() {
                let payload = &result.payload;
                if let Some(resp) = payload.get("response").and_then(|v| v.as_str()) {
                    return Ok(resp.to_string());
                }
            }

            let response = self
                .state
                .llm_provider
                .generate(&prompt, &serde_json::Value::Null)
                .await?;

            let point = QdrantPoint {
                id: uuid::Uuid::new_v4().to_string(),
                vector: question_embedding,
                payload: serde_json::json!({
                    "question": isolated_question,
                    "prompt": prompt,
                    "response": response
                }),
            };

            langcache_client
                .upsert_points("semantic_cache", vec![point])
                .await?;

            Ok(response)
        } else {
            ensure_collection_exists(&self.state, "semantic_cache").await?;
            let qdrant_client = get_qdrant_client(&self.state)?;
            let embeddings = generate_embeddings(vec![prompt.clone()]).await?;
            let embedding = embeddings
                .get(0)
                .ok_or_else(|| "Failed to generate embedding")?
                .clone();

            let search_results = qdrant_client
                .search("semantic_cache", embedding.clone(), 1)
                .await?;

            if let Some(result) = search_results.first() {
                if let Some(payload) = &result.payload {
                    if let Some(resp) = payload.get("response").and_then(|v| v.as_str()) {
                        return Ok(resp.to_string());
                    }
                }
            }

            let response = self
                .state
                .llm_provider
                .generate(&prompt, &serde_json::Value::Null)
                .await?;

            let point = QdrantPoint {
                id: uuid::Uuid::new_v4().to_string(),
                vector: embedding,
                payload: serde_json::json!({
                    "prompt": prompt,
                    "response": response
                }),
            };

            qdrant_client
                .upsert_points("semantic_cache", vec![point])
                .await?;

            Ok(response)
        }
    }

    pub async fn stream_response(
        &self,
        message: UserMessage,
        response_tx: mpsc::Sender<BotResponse>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "Streaming response for user: {}, session: {}",
            message.user_id, message.session_id
        );

        // Get suggestions from Redis
        let suggestions = if let Some(redis) = &self.state.cache {
            let mut conn = redis.get_multiplexed_async_connection().await?;
            let redis_key = format!("suggestions:{}:{}", message.user_id, message.session_id);
            let suggestions: Vec<String> = redis::cmd("LRANGE")
                .arg(&redis_key)
                .arg(0)
                .arg(-1)
                .query_async(&mut conn)
                .await?;

            // Filter out duplicate suggestions
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

        let session = {
            let mut sm = self.state.session_manager.lock().await;
            let session_id = Uuid::parse_str(&message.session_id).map_err(|e| {
                error!("Invalid session ID: {}", e);
                e
            })?;

            match sm.get_session_by_id(session_id)? {
                Some(sess) => sess,
                None => {
                    error!("Failed to create session for streaming");
                    return Err("Failed to create session".into());
                }
            }
        };

        if session.answer_mode == 1 && session.current_tool.is_some() {
            self.state.tool_manager.provide_user_response(
                &message.user_id,
                &message.bot_id,
                message.content.clone(),
            )?;
            return Ok(());
        }

        {
            let mut sm = self.state.session_manager.lock().await;
            sm.save_message(
                session.id,
                user_id,
                1,
                &message.content,
                message.message_type,
            )?;
        }

        let system_prompt = std::env::var("SYSTEM_PROMPT").unwrap_or_default();
        let context_data = {
            let session_manager = self.state.session_manager.lock().await;
            session_manager
                .get_session_context_data(&session.id, &session.user_id)
                .await?
        };

        let prompt = {
            let mut sm = self.state.session_manager.lock().await;
            let history = sm.get_conversation_history(session.id, user_id)?;
            let mut p = String::new();

            if !system_prompt.is_empty() {
                p.push_str(&format!("System: {}\n", system_prompt));
            }
            if !context_data.is_empty() {
                p.push_str(&format!("Context: {}\n", context_data));
            }

            for (role, content) in &history {
                p.push_str(&format!("{}: {}\n", role, content));
            }

            p.push_str(&format!("User: {}\nAssistant:", message.content));
            info!(
                "Stream prompt constructed with {} history entries",
                history.len()
            );
            p
        };

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

        tokio::spawn(async move {
            if let Err(e) = llm
                .generate_stream(&prompt, &serde_json::Value::Null, stream_tx)
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

        let config_manager = ConfigManager::new(Arc::clone(&self.state.conn));
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

            // Check for analysis markers
            if handler.has_analysis_markers(&analysis_buffer) && !in_analysis {
                in_analysis = true;
            }

            // Check if analysis is complete
            if in_analysis && handler.is_analysis_complete(&analysis_buffer) {
                info!("Analysis section completed");
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
                    warn!("Response channel closed, stopping stream processing");
                    break;
                }
            }
        }

        info!(
            "Stream processing completed, {} chunks processed",
            chunk_count
        );

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

        let current_context_length = 0usize;

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
        info!(
            "Getting conversation history for session {} user {}",
            session_id, user_id
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
        info!(
            "Running start script for session: {} with token: {:?}",
            session.id, token
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

        info!(
            "Start script content for session {}: {}",
            session.id, start_script
        );

        let session_clone = session.clone();
        let state_clone = state.clone();
        let script_service = crate::basic::ScriptService::new(state_clone, session_clone.clone());

        if let Some(_token_id_value) = token {}

        match script_service
            .compile(&start_script)
            .and_then(|ast| script_service.run(&ast))
        {
            Ok(result) => {
                info!(
                    "Start script executed successfully for session {}, result: {}",
                    session_clone.id, result
                );
                Ok(true)
            }
            Err(e) => {
                error!(
                    "Failed to run start script for session {}: {}",
                    session_clone.id, e
                );
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

        if channel == "web" {
            self.send_event(
                "system",
                "system",
                session_id,
                channel,
                "warn",
                serde_json::json!({
                    "message": message,
                    "timestamp": Utc::now().to_rfc3339()
                }),
            )
            .await
        } else {
            if let Some(adapter) = self.state.channels.lock().unwrap().get(channel) {
                let warn_response = BotResponse {
                    bot_id: "system".to_string(),
                    user_id: "system".to_string(),
                    session_id: session_id.to_string(),
                    channel: channel.to_string(),
                    content: format!("⚠️ WARNING: {}", message),
                    message_type: 1,
                    stream_token: None,
                    is_complete: true,
                    suggestions: Vec::new(),
                    context_name: None,
                    context_length: 0,
                    context_max_length: 0,
                };
                adapter.send_message(warn_response).await
            } else {
                warn!(
                    "No channel adapter found for warning on channel: {}",
                    channel
                );
                Ok(())
            }
        }
    }

    pub async fn trigger_auto_welcome(
        &self,
        session_id: &str,
        user_id: &str,
        _bot_id: &str,
        token: Option<String>,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "Triggering auto welcome for user: {}, session: {}, token: {:?}",
            user_id, session_id, token
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

        let result = Self::run_start_script(&session, Arc::clone(&self.state), token).await?;
        info!(
            "Auto welcome completed for session: {} with result: {}",
            session_id, result
        );
        Ok(result)
    }
}

pub fn bot_from_url(
    db_conn: &mut PgConnection,
    path: &str,
) -> Result<(Uuid, String), HttpResponse> {
    use crate::shared::models::schema::bots::dsl::*;
    use diesel::prelude::*;

    // Extract bot name from first path segment
    if let Some(bot_name) = path.split('/').nth(1).filter(|s| !s.is_empty()) {
        match bots
            .filter(name.eq(bot_name))
            .filter(is_active.eq(true))
            .select((id, name))
            .first::<(Uuid, String)>(db_conn)
            .optional()
        {
            Ok(Some((bot_id, bot_name))) => return Ok((bot_id, bot_name)),
            Ok(None) => warn!("No active bot found with name: {}", bot_name),
            Err(e) => error!("Failed to query bot by name: {}", e),
        }
    }

    // Fall back to default bot
    let (bot_id, bot_name) = get_default_bot(db_conn);
    log::info!("Using default bot: {} ({})", bot_id, bot_name);
    Ok((bot_id, bot_name))
}

impl Default for BotOrchestrator {
    fn default() -> Self {
        Self {
            state: Arc::new(AppState::default()),
            mounted_bots: Arc::new(AsyncMutex::new(HashMap::new())),
        }
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

    let user_id = {
        let user_uuid = Uuid::parse_str(&user_id_string).unwrap_or_else(|_| Uuid::new_v4());
        let mut sm = data.session_manager.lock().await;
        match sm.get_or_create_anonymous_user(Some(user_uuid)) {
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
                error!("No active bots found in database for WebSocket");
                return Err(actix_web::error::ErrorServiceUnavailable(
                    "No bots available",
                ));
            }
            Err(e) => {
                error!("Failed to query bots for WebSocket: {}", e);
                return Err(actix_web::error::ErrorInternalServerError("Database error"));
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
        if let Err(e) = orchestrator_clone
            .trigger_auto_welcome(&session_id_welcome, &user_id_welcome, &bot_id_welcome, None)
            .await
        {
            warn!("Failed to trigger auto welcome: {}", e);
        }
    });

    let web_adapter = data.web_adapter.clone();
    let session_id_clone1 = session_id.clone();
    let session_id_clone2 = session_id.clone();
    let user_id_clone = user_id.clone();

    actix_web::rt::spawn(async move {
        info!(
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
        info!(
            "WebSocket sender terminated for session {}, sent {} messages",
            session_id_clone1, message_count
        );
    });

    actix_web::rt::spawn(async move {
        info!(
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
                                error!("No active bots found");
                                continue;
                            }
                            Err(e) => {
                                error!("Failed to query bots: {}", e);
                                continue;
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
                        message_type: json_value["message_type"]
                            .as_u64()
                            .unwrap_or(1) as i32,
                        media_url: None,
                        timestamp: Utc::now(),
                        context_name: json_value["context_name"]
                            .as_str()
                            .map(|s| s.to_string()),
                    };

                    // First try processing as a regular message
                    match orchestrator.process_message(user_message.clone()).await {
                        Ok(_) => (),
                        Err(e) => {
                        error!("Failed to process message: {}", e);
                            // Fall back to streaming if processing fails
                            if let Err(e) = orchestrator.stream_response(user_message, tx.clone()).await {
                                error!("Failed to stream response: {}", e);
                            }
                        }
                    }
                }
                WsMessage::Close(reason) => {
                    debug!(
                        "WebSocket closing for session {} - reason: {:?}",
                        session_id_clone2, reason
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

                    debug!("Sending session_end event for {}", session_id_clone2);
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

                    debug!("Removing WebSocket connection for {}", session_id_clone2);
                    web_adapter.remove_connection(&session_id_clone2).await;

                    debug!("Unregistering response channel for {}", session_id_clone2);
                    orchestrator
                        .unregister_response_channel(&session_id_clone2)
                        .await;

                    // Cancel any ongoing LLM jobs for this session
                    if let Err(e) = data.llm_provider.cancel_job(&session_id_clone2).await {
                        warn!(
                            "Failed to cancel LLM job for session {}: {}",
                            session_id_clone2, e
                        );
                    }

                    info!("WebSocket fully closed for session {}", session_id_clone2);
                    break;
                }
                _ => {}
            }
        }
        info!(
            "WebSocket receiver terminated for session {}, processed {} messages",
            session_id_clone2, message_count
        );
    });

    info!(
        "WebSocket handler setup completed for session {}",
        session_id
    );
    Ok(res)
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

    info!(
        "Sending warning via API - session: {}, channel: {}",
        session_id, channel
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
