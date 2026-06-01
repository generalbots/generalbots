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
) -> axum::response::Response {
    let session_id = params.session_id.and_then(|s| Uuid::parse_str(&s).ok()).unwrap_or_else(Uuid::new_v4);
    let user_id = params.user_id.and_then(|s| Uuid::parse_str(&s).ok()).unwrap_or_else(Uuid::new_v4);
    let raw_bot_name = params.bot_name.clone().unwrap_or_else(|| "default".to_string());
    let bot_name = botcore::shared::utils::sanitize_path_component(&raw_bot_name);
    
    if let Err(e) = super::check_bot_access(&state, &bot_name, user_id).await {
        log::warn!("WS access denied for bot {}: {}", bot_name, e);
        return axum::http::StatusCode::FORBIDDEN.into_response();
    }

    let bot_uuid = lookup_bot_id(&state, &bot_name);
    info!("WebSocket: bot={}, session={}, user={}", bot_name, session_id, user_id);
    ws.on_upgrade(move |socket| handle_ws(socket, state, session_id, user_id, bot_uuid, bot_name)).into_response()
}

pub async fn websocket_handler_with_bot(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    axum::extract::Path(bot_name): axum::extract::Path<String>,
    Query(mut params): Query<WsQuery>,
) -> axum::response::Response {
    let raw_bot_name = if bot_name.is_empty() {
        params.bot_name.clone().unwrap_or_else(|| "default".to_string())
    } else {
        bot_name
    };
    let bot_name = botcore::shared::utils::sanitize_path_component(&raw_bot_name);
    params.bot_name = Some(bot_name.clone());
    
    let user_id = params.user_id.as_ref().and_then(|s| Uuid::parse_str(s).ok()).unwrap_or_else(Uuid::new_v4);
    
    if let Err(e) = super::check_bot_access(&state, &bot_name, user_id).await {
        log::warn!("WS access denied for bot {}: {}", bot_name, e);
        return axum::http::StatusCode::FORBIDDEN.into_response();
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
    // TODO(#500): sanitize bot_name to prevent path traversal
    let work_dir = botcore::shared::utils::get_work_path();
    let gbot_dir = format!("{}/{}.gbai/{}.gbot/", work_dir, bot_name, bot_name);

    // TODO(#500): bot_name comes from user-controlled query param — sanitize before filesystem access
    let prompt_from_file = std::fs::read_to_string(format!("{}PROMPT.md", gbot_dir))
        .or_else(|_| std::fs::read_to_string(format!("{}prompt.md", gbot_dir)))
        .or_else(|_| std::fs::read_to_string(format!("{}PROMPT.txt", gbot_dir)))
        .or_else(|_| std::fs::read_to_string(format!("{}prompt.txt", gbot_dir)));

    if let Ok(p) = prompt_from_file {
        return p;
    }

    let now = chrono::Utc::now().format("%B %d, %Y").to_string();
    format!("Today is {now}.\n\nYou are a helpful assistant. Respond only with valid HTML fragments. Do not use markdown. Do not use code blocks. Use only: <p>, <h3>, <ul>, <li>, <strong>, <em>. Every tag you open MUST be properly closed. Start your response directly with an HTML tag, never with plain text.")
}

fn load_bot_styles_css(bot_name: &str) -> String {
    // TODO(#500): sanitize bot_name to prevent path traversal
    let work_dir = botcore::shared::utils::get_work_path();
    let gbot_dir = format!("{}/{}.gbai/{}.gbot/", work_dir, bot_name, bot_name);

    // Load global.css (Issue #508)
    let global_css_path = format!("{}global.css", gbot_dir);
    let mut combined_css = match std::fs::read_to_string(&global_css_path) {
        Ok(c) => {
            info!("global.css loaded from {} ({} bytes)", global_css_path, c.len());
            c
        }
        Err(_) => String::new(),
    };

    let css_path = format!("{}styles.css", gbot_dir);

    // TODO(#500): bot_name comes from user-controlled query param — sanitize before filesystem access
    let local_css = match std::fs::read_to_string(&css_path) {
        Ok(c) => {
            info!("styles.css loaded from {} ({} bytes)", css_path, c.len());
            c
        }
        Err(e1) => {
            let alt_path = format!("{}style.css", gbot_dir);
            match std::fs::read_to_string(&alt_path) {
                Ok(c) => {
                    info!("style.css loaded from {} ({} bytes)", alt_path, c.len());
                    c
                }
                Err(e2) => {
                    warn!("No styles.css/ style.css found at {} or {}: {}, {}", css_path, alt_path, e1, e2);
                    String::new()
                }
            }
        }
    };

    if !local_css.is_empty() {
        if !combined_css.is_empty() {
            combined_css.push_str("\n");
        }
        combined_css.push_str(&local_css);
    }

    combined_css
}

async fn send_start_suggestions(
    state: &Arc<AppState>,
    ws_sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
    bot_uuid: Uuid,
    session_id: Uuid,
    user_id: Uuid,
) {
    let suggestions = {
        #[cfg(feature = "chat")]
        {
            crate::basic::keywords::add_suggestion::get_suggestions(
                state.cache.as_ref(),
                &bot_uuid.to_string(),
                &session_id.to_string(),
            )
        }
        #[cfg(not(feature = "chat"))]
        Vec::new()
    };
    let switchers = {
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
    if !suggestions.is_empty() || !switchers.is_empty() {
        info!("ws_handler: sending {} suggestions and {} switchers on reconnect", suggestions.len(), switchers.len());
        let _ = ws_sender.send(Message::Text(serde_json::json!({
            "bot_id": bot_uuid.to_string(),
            "user_id": user_id.to_string(),
            "session_id": session_id.to_string(),
            "channel": "web",
            "content": "",
            "message_type": 2,
            "is_complete": true,
            "suggestions": suggestions,
            "switchers": switchers,
            "context_length": 0,
            "context_max_length": 0,
        }).to_string().into())).await;
    }
}

async fn run_start_bas_on_connect(
    state: &Arc<AppState>,
    ws_sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
    rx: &mut mpsc::Receiver<botlib::models::BotResponse>,
    bot_uuid: Uuid,
    session_id: Uuid,
    user_id: Uuid,
    bot_name: &str,
) -> bool {
    // Check in-memory guard first (fast path)
    {
        let guards = state.start_bas_guards.lock().await;
        if guards.contains_key(&session_id) {
            info!("start.bas execution skipped: session already initialized (in-memory guard)");
            // Session already initialized - send stored suggestions/switchers for reconnect
            send_start_suggestions(state, ws_sender, bot_uuid, session_id, user_id).await;
            return false;
        }
        info!("start.bas execution proceeding: session not found in guard");
    }

    // Clean any stale Redis key (from previous botserver instance that set it before file check)
    // TODO(#477): Pass org when available (UserSession does not carry org yet)
    let session_init_key = botlib::key_utils::build_key("", &["start_bas_executed", &bot_uuid.to_string(), &session_id.to_string()]);
    if let Some(ref cache) = state.cache {
        if let Ok(mut conn) = cache.get_multiplexed_async_connection().await {
            let _: Result<(), _> = redis::cmd("DEL").arg(&session_init_key).query_async(&mut conn).await;
        }
    }

    // TODO(#500): sanitize bot_name to prevent path traversal
    let work_path = botcore::shared::utils::get_work_path();
    let ast_path = format!("{}/{}.gbai/{}.gbdialog/start.ast", work_path, bot_name, bot_name);
    // TODO(#500): bot_name is user-controlled — use sanitize_filename before building path
    let ast_content = match tokio::fs::read_to_string(&ast_path).await {
        Ok(c) if !c.is_empty() => c,
        _ => {
            let bas_path = ast_path.replace(".ast", ".bas");
            tokio::fs::read_to_string(&bas_path).await.unwrap_or_default()
        }
    };

    if ast_content.is_empty() {
        // Files not ready yet (DriveMonitor still syncing) - caller retries on first message
        return false;
    }

    // Mark as executed in memory (prevents re-execution within same process)
    {
        let mut guards = state.start_bas_guards.lock().await;
        guards.insert(session_id, true);
    }

    let state_for_bas = state.clone();
    let bot_id_for_bas = bot_uuid;
    let _bot_name_owned = bot_name.to_string();
    let session_for_bas = botlib::models::UserSession {
        id: session_id, user_id, bot_id: bot_id_for_bas,
        title: String::new(),
        context_data: serde_json::Value::Null,
        current_tool: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    info!("start.bas: DEBUG BEFORE execute_script");
    let exec_result = crate::basic::ScriptService::execute_script(
        state_for_bas.clone(),
        session_for_bas.clone(),
        &ast_content,
    ).await;
    info!("start.bas: DEBUG AFTER execute_script");
    match exec_result {
        Ok(result) => info!("start.bas: execution result (len={}): {}", result.to_string().len(), result),
        Err(e) => warn!("start.bas: execution error: {}", e),
    }

    for i in 0..50 {
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

    send_start_suggestions(state, ws_sender, bot_uuid, session_id, user_id).await;
    true
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

    // Run start.bas immediately on connect (not waiting for first user message)
    let start_bas_ran = run_start_bas_on_connect(&state, &mut ws_sender, &mut rx, bot_uuid, session_id, user_id, &bot_name).await;

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
                            let raw_tool_name = user_text.trim().to_string();
                            let tool_name = botcore::shared::utils::sanitize_path_component(&raw_tool_name);
                            if !tool_name.is_empty() {
                                info!("TOOL_EXEC: Direct tool execution: {} (sanitized from: {})", tool_name, raw_tool_name);
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
                                for _ in 0..50 {
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
                            // HEAR consumed the message. TALKs from the unblocked script are
                            // forwarded to WebSocket via rx.recv() in the main select! (line 882).
                            continue;
                        }

                        // Fallback: run start.bas now if it didn't run on connect (DriveMonitor wasn't ready)
                        if !start_bas_ran {
                            let guards = state.start_bas_guards.lock().await;
                            if !guards.contains_key(&session_id) {
                                drop(guards);
                                run_start_bas_on_connect(
                                    &state, &mut ws_sender, &mut rx, bot_uuid,
                                    session_id, user_id, &bot_name,
                                ).await;
                            }
                        }
                        let mut guards = state.start_bas_guards.lock().await;
                        guards.entry(session_id).or_insert(true);

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
                            let sm = state.session_manager.lock().await;
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
                                Some((Arc::new(crate::llm::BotlibLLMProviderWrapper::new(provider, llm_model.clone(), llm_key.clone())) as Arc<dyn botlib::traits::LLMProvider>, llm_key, llm_model))
                            } else {
                                None
                            }
                        };

                        // Check answer mode (data/chart) — bypass normal LLM streaming
                        let answer_mode = {
                            #[cfg(feature = "chat")]
                            {
                                crate::core::bot::answer_mode::get_answer_mode(
                                    &state, &session_id,
                                ).await
                            }
                            #[cfg(not(feature = "chat"))]
                            {
                                crate::core::bot::answer_mode::AnswerMode::Default
                            }
                        };
                        if answer_mode != crate::core::bot::answer_mode::AnswerMode::Default {
                            let mode_response = match answer_mode {
                                crate::core::bot::answer_mode::AnswerMode::Data => {
                                    crate::core::bot::answer_mode::generate_data_response(
                                        &state, &user_text, bot_uuid, &bot_name, session_id, user_id,
                                    ).await
                                }
                                crate::core::bot::answer_mode::AnswerMode::Chart => {
                                    crate::core::bot::answer_mode::generate_chart_response(
                                        &state, &user_text, bot_uuid, &bot_name, session_id, user_id,
                                    ).await
                                }
                                _ => unreachable!(),
                            };
                            match mode_response {
                                Ok(resp) => {
                                    if let Ok(json) = serde_json::to_string(&resp) {
                                        let _ = ws_sender.send(Message::Text(json.into())).await;
                                    }
                                    let mut sm = state.session_manager.lock().await;
                                    let _ = sm.save_message(session_id, user_id, 2, &resp.content, 2);
                                }
                                Err(e) => {
                                    let err_resp = serde_json::json!({
                                        "bot_id": bot_uuid.to_string(),
                                        "user_id": user_id.to_string(),
                                        "session_id": session_id.to_string(),
                                        "channel": "web",
                                        "content": format!("<p>Error: {}</p>", e),
                                        "message_type": 2, "is_complete": true,
                                        "suggestions": [], "switchers": [],
                                        "context_length": 0, "context_max_length": 0,
                                    });
                                    let _ = ws_sender.send(Message::Text(err_resp.to_string().into())).await;
                                }
                            }
                            continue;
                        }

                        match bot_llm_provider.or_else(|| state.llm_provider.clone().map(|p| (p, String::new(), String::new()))) {
                            Some((ref llm, ref llm_key, ref llm_model)) => {
                                let state_clone = state.clone();
                                let prompt_clone = full_prompt.clone();
                                let llm = llm.clone();
                                let llm_key_clone = llm_key.clone();
                                let llm_model_clone = llm_model.clone();
                                let bot_uuid_s = bot_uuid.to_string();
                                let session_id_s = session_id.to_string();
                                let bot_name_clone = bot_name.clone();

                                // Suggestions already sent at message receipt time (see early_suggestions above)

                                // Inject bot styles.css into full_response (sent with first streaming chunk)
                                let style_css = load_bot_styles_css(&bot_name_clone);
                                if !style_css.is_empty() {
                                    let style_tag = format!("<style>\n{}</style>\n", style_css);
                                    full_response.push_str(&style_tag);
                                }

                                // Load session tools for LLM function calling
                                let session_tools = {
                                    let sid: Uuid = match session_id_s.parse() {
                                        Ok(id) => id,
                                        Err(_) => Uuid::new_v4(),
                                    };
                                    crate::core::bot::tool_context::get_session_tools(
                                        &state_clone.conn, &bot_name_clone, &sid,
                                    ).ok().unwrap_or_default()
                                };
                                info!("Loaded {} tools for LLM session {}", session_tools.len(), session_id_s);

                                // Spawn LLM streaming task
                                let _stream_handle = tokio::spawn(async move {
                                    info!("LLM spawn task starting: model={}, key_len={}", llm_model_clone, llm_key_clone.len());
                                    let tools_arg = if session_tools.is_empty() { None } else { Some(session_tools) };
                                    if let Err(e) = llm.generate_stream(&prompt_clone, &serde_json::Value::Null, stream_tx, &llm_model_clone, &llm_key_clone, tools_arg.as_ref()).await {
                                        error!("LLM stream error: {}", e);
                                    } else {
                                        info!("LLM spawn task completed successfully");
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
                                let mut stream_started = false;
                                loop {
                                    tokio::select! {
                                        chunk = stream_rx.recv() => {
                                             match chunk {
                                                  Some(chunk) => {
                                                      stream_started = true;
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
                                            if !stream_started {
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

                                 // Save assistant response to history (async, after is_complete sent)
                                 {
                                     let mut sm = state_clone.session_manager.lock().await;
                                     let _ = sm.save_message(session_id, user_id, 2, &full_response, 2);
                                 }
                            }
                            None => {
                                info!("No LLM provider");
                                let fallback = format!("Recebi: \"{}\"", user_text);
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
    {
        let mut guards = state.start_bas_guards.lock().await;
        guards.remove(&session_id);
    }
    info!("WS disconnected: session={}", session_id);
}
