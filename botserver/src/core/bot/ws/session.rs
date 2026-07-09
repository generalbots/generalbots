use std::sync::Arc;
use axum::extract::ws::{Message, WebSocket};
use botcore::shared::state::AppState;
use futures_util::{SinkExt, StreamExt};
use log::{error, info, warn};
use tokio::sync::mpsc;
use uuid::Uuid;

use super::handler::validate_bot_name;
use super::handler::verify_path_within_workdir;
use super::message::load_system_prompt;
use super::message::run_start_bas_on_connect;
use botcore::shared::utils::current_org_id;

pub async fn handle_ws(
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
    let _ = ws_sender.send(Message::Text(welcome.to_string())).await;

    let mut start_bas_ran = run_start_bas_on_connect(
        &state, &mut ws_sender, &mut rx, bot_uuid, session_id, user_id, &bot_name,
    ).await;

    loop {
        tokio::select! {
            response = rx.recv() => {
                if let Some(response) = response {
                    if let Ok(json) = serde_json::to_string(&response) {
                        if ws_sender.send(Message::Text(json)).await.is_err() {
                            break;
                        }
                    }
                } else {
                    break;
                }
            }
            msg = ws_receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        handle_text_message(
                            &mut ws_sender, &mut rx, &state,
                            session_id, user_id, bot_uuid, &bot_name,
                            &mut start_bas_ran, &text,
                        ).await;
                    }
                    Some(Ok(Message::Ping(data))) => {
                        let _ = ws_sender.send(Message::Pong(data)).await;
                    }
                    Some(Ok(Message::Pong(_))) => {}
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Err(e)) => { error!("WS err: {}", e); break; }
                    _ => {}
                }
            }
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

async fn handle_text_message(
    ws_sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
    rx: &mut mpsc::Receiver<botlib::models::BotResponse>,
    state: &Arc<AppState>,
    session_id: Uuid,
    user_id: Uuid,
    bot_uuid: Uuid,
    bot_name: &str,
    start_bas_ran: &mut bool,
    text: &str,
) {
    info!("WS msg: {}", text);
    let parsed: serde_json::Value = serde_json::from_str(text).unwrap_or_default();
    let mut user_text = parsed.get("text")
        .and_then(|v| v.as_str())
        .or_else(|| parsed.get("content").and_then(|v| v.as_str()))
        .unwrap_or("").to_string();
    let mut msg_type = parsed.get("message_type").and_then(|v| v.as_i64()).unwrap_or(1);
    let active_switchers: Vec<String> = parsed.get("active_switchers")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();

    {
        let mut sm = state.session_manager.lock().await;
        let _ = sm.get_or_create_session_by_id(session_id, user_id, bot_uuid, "");
    }

    if msg_type == 7 || msg_type == 0 {
        return;
    }

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
            return;
        }
    }

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
            let _ = ws_sender.send(Message::Text(resp.to_string())).await;
        }
        return;
    }

    if msg_type == 6 {
        let raw_tool_name = user_text.trim().to_string();
        let tool_name = match validate_bot_name(&raw_tool_name) {
            Ok(n) => n,
            Err(e) => {
                warn!("TOOL_EXEC: invalid tool name '{}': {}", raw_tool_name, e);
                let resp = serde_json::json!({
                    "bot_id": bot_uuid.to_string(),
                    "user_id": user_id.to_string(),
                    "session_id": session_id.to_string(),
                    "channel": "web",
                    "content": format!("<p>Invalid tool name: {}</p>", raw_tool_name),
                    "message_type": 2, "is_complete": true,
                    "suggestions": [], "switchers": [],
                    "context_length": 0, "context_max_length": 0,
                });
                let _ = ws_sender.send(Message::Text(resp.to_string())).await;
                return;
            }
        };

        if !tool_name.is_empty() {
            info!("TOOL_EXEC: Direct tool execution: {} (validated from: {})", tool_name, raw_tool_name);
            let work_path = botcore::shared::utils::get_work_path();
            let org_id = current_org_id();

            // Helper: read tool from a path
            let read_tool = |gbdialog_dir: &str| -> Option<String> {
                let ast_p = format!("{}/{}.ast", gbdialog_dir, tool_name);
                let bas_p = ast_p.replace(".ast", ".bas");
                std::fs::read_to_string(&ast_p).ok()
                    .or_else(|| std::fs::read_to_string(&bas_p).ok())
            };

            // Primary path: {bot_name}.gborg/{bot_name}.gbai/{bot_name}.gbdialog/ (mirrors MinIO)
            let primary_rel = format!("{bot_name}.gborg/{bot_name}.gbai/{bot_name}.gbdialog/");
            let tool_content = if verify_path_within_workdir(&primary_rel) {
                let gbdialog_dir = format!("{work_path}/{bot_name}.gborg/{bot_name}.gbai/{bot_name}.gbdialog/");
                read_tool(&gbdialog_dir)
            } else {
                None
            };

            // Fallback: {org_id}.gborg/ path
            let tool_content = tool_content.or_else(|| {
                let fallback_rel = format!("{org_id}.gborg/{bot_name}.gbai/{bot_name}.gbdialog/", org_id = org_id);
                if org_id != uuid::Uuid::nil() && verify_path_within_workdir(&fallback_rel) {
                    let gbdialog_dir = format!("{work_path}/{org_id}.gborg/{bot_name}.gbai/{bot_name}.gbdialog/", org_id = org_id);
                    read_tool(&gbdialog_dir)
                } else {
                    None
                }
            });

            let ast_content = tool_content.unwrap_or_default();

            if !ast_content.is_empty() {
                let state_for_tool = state.clone();
                let tool_name_clone = tool_name.clone();
                let session_for_tool = botlib::models::UserSession {
                    id: session_id, user_id, branch_id: Uuid::nil(), bot_id: bot_uuid,
                    title: String::new(),
                    context_data: serde_json::Value::Null,
                    current_tool: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                };
                tokio::task::spawn_blocking(move || {
                    let mut svc = crate::basic::ScriptService::new(
                        state_for_tool.clone(), session_for_tool,
                    );
                    svc.load_bot_config_params(&state_for_tool, bot_uuid);
                    if let Err(e) = svc.run(&ast_content) {
                        warn!("Tool '{}' execution error: {}", tool_name_clone, e);
                    }
                });
            }
        }
        return;
    }

    let runtime: Arc<dyn botbasic_types::BasicRuntime> =
        Arc::new(crate::basic::AppStateBasicRuntime(state.clone()));
    let delivered = crate::basic::keywords::hearing::deliver_hear_input(
        &runtime, session_id, user_text.clone(),
    );

    info!("ws_handler: delivered={}, user_text='{}'", delivered, user_text);
    if delivered {
        return;
    }

    if !*start_bas_ran {
        let guards = state.start_bas_guards.lock().await;
        if !guards.contains_key(&session_id) {
            drop(guards);
            *start_bas_ran = run_start_bas_on_connect(
                state, ws_sender, rx, bot_uuid,
                session_id, user_id, bot_name,
            ).await;
        }
    }
    let mut guards = state.start_bas_guards.lock().await;
    guards.entry(session_id).or_insert(true);

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
        }).to_string())).await;
    }

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
    }).to_string())).await;

    let base_system_prompt = load_system_prompt(bot_name);
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
    let session_context = {
        let sm = state.session_manager.lock().await;
        sm.get_session_context_data(&session_id, &user_id).ok().unwrap_or_default()
    };

    let mut messages = vec![
        serde_json::json!({"role": "system", "content": system_prompt.clone()})
    ];

    if !session_context.is_empty() {
        messages.push(serde_json::json!({
            "role": "system", "content": format!("Contexto da conversa:\n{}", session_context)
        }));
    }

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
    }).to_string())).await;

    let user_query = user_text.clone();
    let mut messages_val = serde_json::Value::Array(messages.clone());
    info!("ws_handler: injecting KB context for session {}", session_id);
    if tokio::time::timeout(
        std::time::Duration::from_secs(30),
        crate::core::bot::kb_context::inject_kb_context(
            &state.conn,
            session_id,
            bot_uuid,
            &user_query,
            &mut messages_val,
            4000,
        ),
    ).await.is_err() {
        warn!("ws_handler: inject_kb_context TIMEOUT after 30s for session {}", session_id);
    }
    info!("ws_handler: KB context injection completed for session {}", session_id);

    if !is_switcher_replay {
        let mut sm = state.session_manager.lock().await;
        let _ = sm.save_message(session_id, user_id, 1, &user_text, 1);
    }

    if let Some(arr) = messages_val.as_array_mut() {
        arr.push(serde_json::json!({
            "role": "user",
            "content": user_text
        }));
    }

    info!("ws_handler: calling process_llm_response for session {}", session_id);
    if tokio::time::timeout(
        std::time::Duration::from_secs(300),
        super::stream::process_llm_response(
            ws_sender, rx, state, bot_uuid, session_id, user_id, bot_name,
            &messages_val, &user_text,
        ),
    ).await.is_err() {
        warn!("ws_handler: process_llm_response TIMEOUT after 300s for session {}", session_id);
    }
    info!("ws_handler: process_llm_response finished for session {}", session_id);
}
