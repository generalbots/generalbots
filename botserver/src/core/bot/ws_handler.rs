use std::sync::Arc;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use botcore::shared::state::AppState;
use futures_util::{SinkExt, StreamExt};
use log::{error, info, warn};
use serde::Deserialize;
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct WsQuery {
    pub session_id: Option<String>,
    pub user_id: Option<String>,
    pub bot_name: Option<String>,
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Query(params): Query<WsQuery>,
) -> impl IntoResponse {
    let session_id = params.session_id.and_then(|s| Uuid::parse_str(&s).ok()).unwrap_or_else(Uuid::new_v4);
    let user_id = params.user_id.and_then(|s| Uuid::parse_str(&s).ok()).unwrap_or_else(Uuid::new_v4);
    let bot_name = params.bot_name.unwrap_or_else(|| "default".to_string());
    let bot_uuid = lookup_bot_id(&state, &bot_name);
    info!("WebSocket: bot={}, session={}, user={}", bot_name, session_id, user_id);
    ws.on_upgrade(move |socket| handle_ws(socket, state, session_id, user_id, bot_uuid, bot_name))
}

pub async fn websocket_handler_with_bot(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    axum::extract::Path(bot_name): axum::extract::Path<String>,
    Query(mut params): Query<WsQuery>,
) -> impl IntoResponse {
    if params.bot_name.is_none() && !bot_name.is_empty() {
        params.bot_name = Some(bot_name);
    }
    websocket_handler(ws, State(state), Query(params)).await
}

fn lookup_bot_id(state: &Arc<AppState>, bot_name: &str) -> Uuid {
    let mut conn = match state.conn.get() {
        Ok(c) => c,
        Err(e) => {
            warn!("DB conn: {}", e);
            return Uuid::nil();
        }
    };

    use botcorebot::schema::bots::dsl::{bots, id, name};
    use diesel::prelude::*;

    if let Ok(uuid) = Uuid::parse_str(bot_name) {
        bots.filter(id.eq(uuid))
            .select(id)
            .first::<Uuid>(&mut conn)
            .unwrap_or(Uuid::nil())
    } else {
        bots.filter(name.eq(bot_name))
            .select(id)
            .first::<Uuid>(&mut conn)
            .unwrap_or_else(|_| {
                warn!("Bot not found: {}", bot_name);
                Uuid::nil()
            })
    }
}

fn load_system_prompt(bot_name: &str) -> String {
    let work_dir = botcore::shared::utils::get_work_path();
    let gbot_dir = format!("{}/{}.gbai/{}.gbot/", work_dir, bot_name, bot_name);

    let prompt_from_file = std::fs::read_to_string(format!("{}PROMPT.md", gbot_dir))
        .or_else(|_| std::fs::read_to_string(format!("{}prompt.md", gbot_dir)))
        .or_else(|_| std::fs::read_to_string(format!("{}PROMPT.txt", gbot_dir)))
        .or_else(|_| std::fs::read_to_string(format!("{}prompt.txt", gbot_dir)));

    if let Ok(p) = prompt_from_file {
        return p;
    }

    "You are a helpful assistant. Responda APENAS com fragmentos HTML válidos. Não use markdown. Não use blocos de código. Use apenas: <p>, <h3>, <ul>, <li>, <strong>, <em>. Cada tag que você abrir DEVE ser fechada corretamente. Comece sua resposta diretamente com uma tag HTML, nunca com texto puro.".to_string()
}

async fn handle_ws(
    socket: WebSocket,
    state: Arc<AppState>,
    session_id: Uuid,
    user_id: Uuid,
    bot_uuid: Uuid,
    bot_name: String,
) {
    let (mut ws_sender, mut ws_receiver) = socket.split();
    let (tx, mut rx) = mpsc::channel::<botlib::models::BotResponse>(100);
    {
        let mut channels = state.response_channels.lock().await;
        channels.insert(session_id.to_string(), tx);
    }
    info!("WebSocket connected: bot={}, session={}", bot_name, session_id);

    let welcome = serde_json::json!({
        "type": "connected", "session_id": session_id, "user_id": user_id,
        "bot_id": bot_uuid, "message": "Connected to bot server", "tools": []
    });
    let _ = ws_sender.send(Message::Text(welcome.to_string().into())).await;

    // Message loop
    loop {
        tokio::select! {
            msg = ws_receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        info!("WS msg: {}", text);
                        let parsed: serde_json::Value = serde_json::from_str(&text).unwrap_or_default();
                        let mut user_text = parsed.get("text")
                            .and_then(|v| v.as_str())
                            .or_else(|| parsed.get("content").and_then(|v| v.as_str()))
                            .unwrap_or("").to_string();
                        let mut msg_type = parsed.get("message_type").and_then(|v| v.as_i64()).unwrap_or(1);
                        let active_switchers: Vec<String> = parsed.get("active_switchers")
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                            .unwrap_or_default();

                        // Ensure session exists in DB for FK constraints (KB, etc.)
                        {
                            let mut sm = state.session_manager.lock().await;
                            let _ = sm.get_or_create_session_by_id(session_id, user_id, bot_uuid, "");
                        }

                        // Handle SYSTEM messages (type 7) - deprecated, just acknowledge
                        if msg_type == 7 {
                            continue;
                        }

                        // Handle SWITCHER_TOGGLE (type 8) - re-process last message with switcher active
                        let mut is_switcher_replay = false;
                        if msg_type == 8 {
                            let last_user_msg = {
                                let mut sm = state.session_manager.lock().await;
                                let history = sm.get_conversation_history(session_id, user_id, Some(1)).ok();
                                history.and_then(|h| h.into_iter().find(|(role, _)| role == "user").map(|(_, c)| c))
                            };
                            if let Some(last_content) = last_user_msg {
                                user_text = last_content;
                                is_switcher_replay = true;
                                msg_type = 1;
                            } else {
                                continue;
                            }
                        }

                        // Legacy: Direct tool invocation via __TOOL__: prefix
                        if user_text.starts_with("__TOOL__:") {
                            let tool_name = user_text.trim_start_matches("__TOOL__:").trim().to_string();
                            if !tool_name.is_empty() {
                                let resp = serde_json::json!({
                                    "bot_id": bot_uuid.to_string(),
                                    "user_id": user_id.to_string(),
                                    "session_id": session_id.to_string(),
                                    "channel": "web",
                                    "content": format!("Tool '{}' not implemented via legacy path", tool_name),
                                    "message_type": 2, "is_complete": true,
                                    "suggestions": [], "switchers": [],
                                    "context_length": 0, "context_max_length": 0,
                                });
                                let _ = ws_sender.send(Message::Text(resp.to_string().into())).await;
                            }
                            continue;
                        }

                        // Handle TOOL_EXEC (type 6) - bypass LLM
                        if msg_type == 6 {
                            let tool_name = user_text.trim().to_string();
                            if !tool_name.is_empty() {
                                info!("TOOL_EXEC: Direct tool execution: {}", tool_name);
                                let work_path = botcore::shared::utils::get_work_path();
                                let ast_path = format!("{}/{}.gbai/{}.gbdialog/{}.ast", work_path, bot_name, bot_name, tool_name);
                                let ast_content = match tokio::fs::read_to_string(&ast_path).await {
                                    Ok(c) if !c.is_empty() => c,
                                    _ => {
                                        let bas_path = ast_path.replace(".ast", ".bas");
                                        tokio::fs::read_to_string(&bas_path).await.unwrap_or_default()
                                    }
                                };

                                if !ast_content.is_empty() {
                                    let state_for_tool = state.clone();
                                    let tool_name_clone = tool_name.clone();
                                    let session_for_tool = botlib::models::UserSession {
                                        id: session_id, user_id, bot_id: bot_uuid,
                                        title: String::new(),
                                        context_data: serde_json::Value::Null,
                                        current_tool: None,
                                        created_at: chrono::Utc::now(),
                                        updated_at: chrono::Utc::now(),
                                    };
                                    let _ = tokio::task::spawn_blocking(move || {
                                        let mut svc = crate::basic::ScriptService::new(
                                            state_for_tool.clone(), session_for_tool,
                                        );
                                        svc.load_bot_config_params(&state_for_tool, bot_uuid);
                                        if let Err(e) = svc.run(&ast_content) {
                                            warn!("Tool '{}' execution error: {}", tool_name_clone, e);
                                        }
                                    }).await;
                                }

                                // Drain any TALK responses from tool execution
                                for _ in 0..20 {
                                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                                    match rx.try_recv() {
                                        Ok(response) => {
                                            if let Ok(json) = serde_json::to_string(&response) {
                                                let _ = ws_sender.send(Message::Text(json.into())).await;
                                            }
                                        }
                                        Err(tokio::sync::mpsc::error::TryRecvError::Empty) => continue,
                                        Err(_) => break,
                                    }
                                }
                            }
                            continue;
                        }

                        // Try to deliver to a waiting HEAR keyword first
                        let runtime: Arc<dyn botbasic_types::BasicRuntime> =
                            Arc::new(crate::basic::AppStateBasicRuntime(state.clone()));
                        let delivered = crate::basic::keywords::hearing::deliver_hear_input(
                            &runtime, session_id, user_text.clone(),
                        );

                        info!("ws_handler: delivered={}, user_text='{}'", delivered, user_text);
                        if delivered {
                            continue;
                        }

                        // Execute start.bas on first user message
                        let session_init_key = format!("start_bas_executed:{}:{}", bot_uuid, session_id);
                        let already_executed = {
                            let guards = state.start_bas_guards.lock().await;
                            guards.get(&session_id).copied().unwrap_or(false)
                        };
                        let should_execute = if already_executed {
                            false
                        } else if let Some(ref cache) = state.cache {
                            use redis::AsyncCommands;
                            if let Ok(mut conn) = cache.get_multiplexed_async_connection().await {
                                let was_set: Option<String> = redis::cmd("SET")
                                    .arg(&session_init_key).arg("1").arg("NX").arg("EX").arg(86400)
                                    .query_async(&mut conn).await.ok();
                                was_set.is_some()
                            } else {
                                false
                            }
                        } else {
                            true
                        };

                        if should_execute {
                            let work_path = botcore::shared::utils::get_work_path();
                            let ast_path = format!("{}/{}.gbai/{}.gbdialog/start.ast", work_path, bot_name, bot_name);
                            let ast_content = match tokio::fs::read_to_string(&ast_path).await {
                                Ok(c) if !c.is_empty() => c,
                                _ => {
                                    let bas_path = ast_path.replace(".ast", ".bas");
                                    tokio::fs::read_to_string(&bas_path).await.unwrap_or_default()
                                }
                            };

                            if !ast_content.is_empty() {
                                let state_for_bas = state.clone();
                                let bot_id_for_bas = bot_uuid;
                                tokio::task::spawn_blocking(move || {
                                    let session_for_bas = botlib::models::UserSession {
                                        id: session_id, user_id, bot_id: bot_id_for_bas,
                                        title: String::new(),
                                        context_data: serde_json::Value::Null,
                                        current_tool: None,
                                        created_at: chrono::Utc::now(),
                                        updated_at: chrono::Utc::now(),
                                    };
                                    let mut svc = crate::basic::ScriptService::new(
                                        state_for_bas.clone(), session_for_bas,
                                    );
                                    svc.load_bot_config_params(&state_for_bas, bot_id_for_bas);
                                    if let Err(e) = svc.run(&ast_content) {
                                        warn!("start.bas execution error: {}", e);
                                    }
                                }).await;
                            }
                            // Wait briefly for TALK's tokio::spawn task to deliver the response
                            for i in 0..20 {
                                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                                match rx.try_recv() {
                                    Ok(response) => {
                                        info!("start.bas: drained BotResponse: content={}", response.content.chars().take(80).collect::<String>());
                                        if let Ok(json) = serde_json::to_string(&response) {
                                            let _ = ws_sender.send(Message::Text(json.into())).await;
                                        }
                                    }
                                    Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                                        if i == 0 { info!("start.bas: rx empty, waiting..."); }
                                        continue;
                                    }
                                    Err(e) => {
                                        info!("start.bas: rx done: {:?}", e);
                                        break;
                                    }
                                }
                            }
                            let mut guards = state.start_bas_guards.lock().await;
                            guards.insert(session_id, true);
                        }

                        // Send suggestions AFTER start.bas has run (so suggestions are in Redis)
                        // but BEFORE KB embedding (to prevent connection timeout)
                        let post_start_suggestions = {
                            #[cfg(feature = "chat")]
                            {
                                let suggs = crate::basic::keywords::add_suggestion::get_suggestions(
                                    state.cache.as_ref(),
                                    &bot_uuid.to_string(),
                                    &session_id.to_string(),
                                );
                                info!("ws_handler: post_start_suggestions: {} found", suggs.len());
                                suggs
                            }
                            #[cfg(not(feature = "chat"))]
                            Vec::new()
                        };
                        let post_start_switchers = {
                            #[cfg(feature = "chat")]
                            {
                                crate::basic::keywords::switcher::get_switchers(
                                    state.cache.as_ref(),
                                    &bot_uuid.to_string(),
                                    &session_id.to_string(),
                                )
                            }
                            #[cfg(not(feature = "chat"))]
                            Vec::new()
                        };
                        if !post_start_suggestions.is_empty() || !post_start_switchers.is_empty() {
                            info!("ws_handler: sending {} post-start suggestions", post_start_suggestions.len());
                            let _ = ws_sender.send(Message::Text(serde_json::json!({
                                "bot_id": bot_uuid.to_string(),
                                "user_id": user_id.to_string(),
                                "session_id": session_id.to_string(),
                                "channel": "web",
                                "content": "",
                                "message_type": 2,
                                "is_complete": true,
                                "suggestions": post_start_suggestions,
                                "switchers": post_start_switchers,
                                "context_length": 0,
                                "context_max_length": 0,
                            }).to_string().into())).await;
                        }

                        // Send keepalive before KB embedding to prevent browser timeout
                        let _ = ws_sender.send(Message::Text(serde_json::json!({
                            "bot_id": bot_uuid.to_string(),
                            "user_id": user_id.to_string(),
                            "session_id": session_id.to_string(),
                            "channel": "web",
                            "content": "",
                            "message_type": 2,
                            "is_complete": false,
                            "suggestions": [],
                            "switchers": [],
                            "context_length": 0,
                            "context_max_length": 0,
                        }).to_string().into())).await;

                        // Build messages array: system prompt + KB context + history + user message
                        let base_system_prompt = load_system_prompt(&bot_name);
                        let system_prompt = if !active_switchers.is_empty() {
                            let switcher_prompts = crate::basic::keywords::switcher::resolve_active_switchers(
                                state.cache.as_ref(),
                                &bot_uuid.to_string(),
                                &session_id.to_string(),
                                &active_switchers,
                            );
                            if switcher_prompts.is_empty() {
                                base_system_prompt
                            } else {
                                format!("{}\n\n{}", base_system_prompt, switcher_prompts)
                            }
                        } else {
                            base_system_prompt
                        };
                        // Inject session context data (bot memory)
                        let session_context = {
                            let mut sm = state.session_manager.lock().await;
                            sm.get_session_context_data(&session_id, &user_id).ok().unwrap_or_default()
                        };

                        let mut messages = vec![
                            serde_json::json!({"role": "system", "content": system_prompt.clone()})
                        ];

                        // Add session context as system message if non-empty
                        if !session_context.is_empty() {
                            messages.push(serde_json::json!({
                                "role": "system", "content": format!("Contexto da conversa:\n{}", session_context)
                            }));
                        }

                        // Load recent conversation history (limit from bot config)
                        let history_limit: i64 = {
                            use botcore::config::ConfigManager;
                            let cfg = ConfigManager::new(state.conn.clone());
                            cfg.get_config(&bot_uuid, "history-limit", Some("10"))
                                .ok().and_then(|v| v.parse().ok()).unwrap_or(10)
                        };
                        {
                            let mut sm = state.session_manager.lock().await;
                            if let Ok(history) = sm.get_conversation_history(session_id, user_id, Some(history_limit)) {
                                for (role, content) in history.iter() {
                                    let api_role = match role.as_str() {
                                        "user" => "user",
                                        "assistant" | "bot" => "assistant",
                                        _ => "system",
                                    };
                                    messages.push(serde_json::json!({
                                        "role": api_role,
                                        "content": content
                                    }));
                                }
                            }
                        }

                        // Send immediate keepalive BEFORE KB embedding to prevent browser
                        // from closing connection during the embedding API calls (1-2s)
                        let _ = ws_sender.send(Message::Text(serde_json::json!({
                            "bot_id": bot_uuid.to_string(),
                            "user_id": user_id.to_string(),
                            "session_id": session_id.to_string(),
                            "channel": "web",
                            "content": "",
                            "message_type": 2,
                            "is_complete": false,
                            "suggestions": [],
                            "switchers": [],
                            "context_length": 0,
                            "context_max_length": 0,
                        }).to_string().into())).await;

                        // Inject KB and website context via Qdrant search
                        let user_query = user_text.clone();
                        let mut messages_val = serde_json::Value::Array(messages.clone());
    crate::core::bot::kb_context::inject_kb_context(
        &state.conn,
        session_id,
        bot_uuid,
        &user_query,
        &mut messages_val,
        4000,
    ).await;
                        if let Some(arr) = messages_val.as_array() {
                            messages = arr.clone();
                        }

                        // Save user message to history (skip for switcher replays)
                        if !is_switcher_replay {
                            let mut sm = state.session_manager.lock().await;
                            let _ = sm.save_message(session_id, user_id, 1, &user_text, 1);
                        }

                        // Build flat prompt from messages for streaming
                        let mut full_prompt = String::new();
                        for msg in &messages {
                            let role = msg["role"].as_str().unwrap_or("user");
                            let content = msg["content"].as_str().unwrap_or("");
                            match role {
                                "system" => full_prompt.push_str(&format!("System: {}\n\n", content)),
                                "user" => full_prompt.push_str(&format!("User: {}\n", content)),
                                "assistant" => full_prompt.push_str(&format!("Assistant: {}\n", content)),
                                _ => full_prompt.push_str(&format!("{}: {}\n", role, content)),
                            }
                        }
                        full_prompt.push_str(&format!("\nUser: {}", user_text));
                        full_prompt.push_str("\nAssistant: ");

                        // Stream LLM response chunk by chunk
                        let (stream_tx, mut stream_rx) = mpsc::channel::<String>(100);
                        let suggestions: Vec<botlib::models::Suggestion>;
                        let switchers: Vec<botlib::models::Switcher>;
                        let mut full_response = String::new();

                        // Look up bot-specific LLM config and create provider
                        let bot_llm_provider: Option<(Arc<dyn botlib::traits::LLMProvider>, String, String)> = {
                            use botcore::config::ConfigManager;
                            let cfg = ConfigManager::new(state.conn.clone());
                            let llm_url = cfg.get_config(&bot_uuid, "llm-url", Some("")).unwrap_or_default();
                            let llm_key = cfg.get_config(&bot_uuid, "llm-key", Some("")).unwrap_or_default();
                            let llm_model = cfg.get_config(&bot_uuid, "llm-model", Some("")).unwrap_or_default();
                            if !llm_url.is_empty() {
                                let provider = crate::llm::create_llm_provider_from_url(&llm_url, if llm_model.is_empty() { None } else { Some(llm_model.clone()) }, None, None);
                                Some((Arc::new(crate::llm::BotlibLLMProviderWrapper(provider)) as Arc<dyn botlib::traits::LLMProvider>, llm_key, llm_model))
                            } else {
                                None
                            }
                        };

                        match bot_llm_provider.or_else(|| state.llm_provider.clone().map(|p| (p, String::new(), String::new()))) {
                            Some((ref llm, ref llm_key, ref llm_model)) => {
                                let state_clone = state.clone();
                                let prompt_clone = full_prompt.clone();
                                let llm = llm.clone();
                                let llm_key_clone = llm_key.clone();
                                let llm_model_clone = llm_model.clone();
                                let bot_uuid_s = bot_uuid.to_string();
                                let session_id_s = session_id.to_string();

                                // Suggestions already sent at message receipt time (see early_suggestions above)

                                // Spawn LLM streaming task
                                let _stream_handle = tokio::spawn(async move {
                                    if let Err(e) = llm.generate_stream(&prompt_clone, &serde_json::Value::Null, stream_tx, &llm_model_clone, &llm_key_clone, None).await {
                                        warn!("LLM stream error: {}", e);
                                    }
                                });

                                // Stream chunks to WebSocket with periodic keepalive
                                // Send immediate thinking indicator before entering the loop
                                let _ = ws_sender.send(Message::Text(serde_json::json!({
                                    "bot_id": bot_uuid_s,
                                    "user_id": user_id.to_string(),
                                    "session_id": session_id_s,
                                    "channel": "web",
                                    "content": "",
                                    "message_type": 2,
                                    "is_complete": false,
                                    "thinking": true,
                                    "suggestions": [],
                                    "switchers": [],
                                    "context_length": 0,
                                    "context_max_length": 0,
                                }).to_string().into())).await;

                                let mut keepalive_interval = tokio::time::interval(std::time::Duration::from_millis(2000));
                                keepalive_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
                                loop {
                                    tokio::select! {
                                        chunk = stream_rx.recv() => {
                                            match chunk {
                                                Some(chunk) => {
                                                    full_response.push_str(&chunk);
                                                    let chunk_resp = serde_json::json!({
                                                        "bot_id": bot_uuid_s,
                                                        "user_id": user_id.to_string(),
                                                        "session_id": session_id_s,
                                                        "channel": "web",
                                                        "content": chunk,
                                                        "message_type": 2,
                                                        "is_complete": false,
                                                        "suggestions": [],
                                                        "switchers": [],
                                                        "context_length": 0,
                                                        "context_max_length": 0,
                                                    });
                                                    if ws_sender.send(Message::Text(chunk_resp.to_string().into())).await.is_err() {
                                                        break;
                                                    }
                                                }
                                                None => break,
                                            }
                                        }
                                        _ = keepalive_interval.tick() => {
                                            let _ = ws_sender.send(Message::Text(serde_json::json!({
                                                "bot_id": bot_uuid_s,
                                                "user_id": user_id.to_string(),
                                                "session_id": session_id_s,
                                                "channel": "web",
                                                "content": "",
                                                "message_type": 2,
                                                "is_complete": false,
                                                "thinking": true,
                                                "suggestions": [],
                                                "switchers": [],
                                                "context_length": 0,
                                                "context_max_length": 0,
                                            }).to_string().into())).await;
                                        }
                                    }
                                }

                                // Send is_complete IMMEDIATELY after stream ends (before any other ops)
                                // This prevents browser from closing connection between stream end and final message
                                let final_resp = serde_json::json!({
                                    "bot_id": bot_uuid_s,
                                    "user_id": user_id.to_string(),
                                    "session_id": session_id_s,
                                    "channel": "web",
                                    "content": "",
                                    "message_type": 2,
                                    "is_complete": true,
                                    "suggestions": [],
                                    "switchers": [],
                                    "context_length": 0,
                                    "context_max_length": 0,
                                });
                                let _ = ws_sender.send(Message::Text(final_resp.to_string().into())).await;

                                // Suggestions already sent at message receipt time
                                suggestions = Vec::new();
                                switchers = Vec::new();

                                // Save assistant response to history (async, after is_complete sent)
                                {
                                    let mut sm = state_clone.session_manager.lock().await;
                                    let _ = sm.save_message(session_id, user_id, 2, &full_response, 2);
                                }
                            }
                            None => {
                                info!("No LLM provider");
                                let fallback = format!("Recebi: \"{}\"", user_text);
                                suggestions = Vec::new();
                                switchers = Vec::new();
                                {
                                    let mut sm = state.session_manager.lock().await;
                                    let _ = sm.save_message(session_id, user_id, 2, &fallback, 2);
                                }
                                // Send fallback response
                                let _ = ws_sender.send(Message::Text(serde_json::json!({
                                    "bot_id": bot_uuid.to_string(),
                                    "user_id": user_id.to_string(),
                                    "session_id": session_id.to_string(),
                                    "channel": "web",
                                    "content": fallback,
                                    "message_type": 2,
                                    "is_complete": true,
                                    "suggestions": [],
                                    "switchers": [],
                                    "context_length": 0,
                                    "context_max_length": 0,
                                }).to_string().into())).await;
                            }
                        };
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Err(e)) => { error!("WS err: {}", e); break; }
                    _ => {}
                }
            }
            Some(response) = rx.recv() => {
                if let Ok(json) = serde_json::to_string(&response) {
                    let _ = ws_sender.send(Message::Text(json.into())).await;
                }
            }
            else => break,
        }
    }

    {
        let mut channels = state.response_channels.lock().await;
        channels.remove(&session_id.to_string());
    }
    info!("WS disconnected: session={}", session_id);
}
