use std::sync::Arc;

use botcore::shared::state::AppState;
use uuid::Uuid;

use super::sink::ChannelSink;
use super::types::PipelineResult;

pub type PipelineFn = std::sync::Arc<
    dyn Fn(
            botlib::models::UserMessage,
            tokio::sync::mpsc::Sender<botlib::models::BotResponse>,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = PipelineResult<()>> + Send>>
        + Send
        + Sync,
>;

pub async fn process_message_internal(
    sink: &dyn ChannelSink,
    rx: &mut tokio::sync::mpsc::Receiver<botlib::models::BotResponse>,
    state: &Arc<AppState>,
    session_id: Uuid,
    user_id: Uuid,
    bot_uuid: Uuid,
    bot_name: &str,
    start_bas_ran: &mut bool,
    text: &str,
) -> PipelineResult<()> {
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
        return Ok(());
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
            return Ok(());
        }
    }

    if user_text.starts_with("__TOOL__:") {
        let tool_name = user_text.trim_start_matches("__TOOL__:").trim().to_string();
        if !tool_name.is_empty() {
            let resp = botlib::models::BotResponse::new(
                bot_uuid.to_string(), session_id.to_string(),
                user_id.to_string(),
                format!("Tool '{tool_name}' not implemented via legacy path"), "web",
            );
            let _ = sink.send_bot_response(&resp).await;
        }
        return Ok(());
    }

    if msg_type == 6 {
        let raw_tool_name = user_text.trim().to_string();
        let tool_name = match crate::core::bot::ws::handler::validate_bot_name(&raw_tool_name) {
            Ok(n) => n,
            Err(e) => {
                log::warn!("TOOL_EXEC: invalid tool name '{}': {}", raw_tool_name, e);
                let resp = botlib::models::BotResponse::new(
                    bot_uuid.to_string(), session_id.to_string(),
                    user_id.to_string(),
                    format!("<p>Invalid tool name: {raw_tool_name}</p>"), "web",
                );
                let _ = sink.send_bot_response(&resp).await;
                return Ok(());
            }
        };

        if !tool_name.is_empty() {
            super::tool_exec::run_tool_exec(
                state, bot_uuid, session_id, user_id, bot_name, &tool_name, sink.channel_type(),
            ).await;
        }
        return Ok(());
    }

    let runtime: Arc<dyn botbasic_types::BasicRuntime> =
        Arc::new(crate::basic::AppStateBasicRuntime(state.clone()));
    let delivered = crate::basic::keywords::hearing::deliver_hear_input(
        &runtime, session_id, user_text.clone(),
    );
    if delivered {
        return Ok(());
    }

    if !*start_bas_ran {
        let guards = state.start_bas_guards.lock().await;
        if !guards.contains_key(&session_id) {
            drop(guards);
            *start_bas_ran = super::start_bas::run_start_bas(
                sink, state, bot_uuid, session_id, user_id, bot_name, rx,
            ).await.unwrap_or(false);
        }
    }
    let mut guards = state.start_bas_guards.lock().await;
    guards.entry(session_id).or_insert(true);

    let post_start_suggestions = {
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
        let resp = botlib::models::BotResponse {
            bot_id: bot_uuid.to_string(),
            user_id: user_id.to_string(),
            session_id: session_id.to_string(),
            channel: "web".to_string(),
            content: String::new(),
            message_type: botlib::message_types::MessageType::BOT_RESPONSE,
            stream_token: None,
            is_complete: true,
            suggestions: post_start_suggestions.into_iter()
                .map(|s| botlib::models::Suggestion::new(s.text)).collect(),
            switchers: post_start_switchers,
            context_name: None,
            context_length: 0,
            context_max_length: 0,
            reasoning: String::new(),
        };
        let _ = sink.send_bot_response(&resp).await;
    }

    let _ = sink.send_bot_response(&botlib::models::BotResponse {
        bot_id: bot_uuid.to_string(),
        user_id: user_id.to_string(),
        session_id: session_id.to_string(),
        channel: "web".to_string(),
        content: String::new(),
        message_type: botlib::message_types::MessageType::BOT_RESPONSE,
        stream_token: None,
        is_complete: false,
        suggestions: Vec::new(),
        switchers: Vec::new(),
        context_name: None,
        context_length: 0,
        context_max_length: 0,
        reasoning: String::new(),
    }).await;

    let channel = sink.channel_type();
    let base_system_prompt = crate::core::bot::ws::message::load_system_prompt_for_channel(bot_name, channel);
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
            format!("{base_system_prompt}\n\n{switcher_prompts}")
        }
    } else {
        base_system_prompt
    };
    let system_prompt = if channel == "web" {
        format!(
            "{system_prompt}\n\n{}\n\n{}",
            crate::core::bot::api_catalog::api_command_instructions(),
            crate::main_module::ui_plan::ui_automation_instructions(),
        )
    } else if channel == "whatsapp" {
        format!(
            "{system_prompt}\n\n---\nThis conversation is on WhatsApp.\n\n{}",
            crate::core::bot::api_catalog::api_command_instructions(),
        )
    } else {
        format!(
            "{system_prompt}\n\n{}",
            crate::core::bot::api_catalog::api_command_instructions(),
        )
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
            "role": "system",
            "content": format!("Contexto da conversa:\n{session_context}")
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

    let _ = sink.send_bot_response(&botlib::models::BotResponse {
        bot_id: bot_uuid.to_string(),
        user_id: user_id.to_string(),
        session_id: session_id.to_string(),
        channel: "web".to_string(),
        content: String::new(),
        message_type: botlib::message_types::MessageType::BOT_RESPONSE,
        stream_token: None,
        is_complete: false,
        suggestions: Vec::new(),
        switchers: Vec::new(),
        context_name: None,
        context_length: 0,
        context_max_length: 0,
        reasoning: String::new(),
    }).await;

    let user_query = user_text.clone();
    let mut messages_val = serde_json::Value::Array(messages.clone());
    if tokio::time::timeout(
        std::time::Duration::from_secs(30),
        super::kb::inject_kb(
            state, bot_uuid, session_id, user_id, bot_name,
            &user_query, &mut messages_val,
        ),
    ).await.is_err() {
        log::warn!("ws_handler: inject_kb_context TIMEOUT after 30s for session {session_id}");
    }

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

    if tokio::time::timeout(
        std::time::Duration::from_secs(300),
        super::llm::stream_llm_response(
            sink, rx, state, bot_uuid, session_id, user_id, bot_name,
            &messages_val, &user_query,
        ),
    ).await.is_err() {
        log::warn!("stream_llm_response TIMEOUT after 300s for session {session_id}");
    }

    Ok(())
}

pub async fn run_pipeline_for_channel(
    state: &Arc<AppState>,
    msg: &botlib::models::UserMessage,
    sink: &dyn ChannelSink,
) -> PipelineResult<()> {
    let bot_name = msg.bot_id.clone();
    let user_text = msg.content.clone();
    let session_id = Uuid::parse_str(&msg.session_id).unwrap_or_else(|_| Uuid::new_v4());
    let user_id = Uuid::parse_str(&msg.user_id).unwrap_or_else(|_| Uuid::nil());

    let bot_uuid = resolve_bot_uuid(&state.conn, &bot_name).await;

    let response_key = format!("{}_{}", session_id, Uuid::new_v4());
    let (tx_internal, mut rx_internal) = tokio::sync::mpsc::channel::<botlib::models::BotResponse>(100);
    {
        let mut channels = state.response_channels.lock().await;
        channels.insert(response_key.clone(), tx_internal);
    }

    let json_msg = serde_json::json!({
        "text": user_text,
        "content": user_text,
        "message_type": i32::from(msg.message_type),
    }).to_string();

    let mut start_bas_ran = false;
    let result = process_message_internal(
        sink, &mut rx_internal, state,
        session_id, user_id, bot_uuid, &bot_name,
        &mut start_bas_ran, &json_msg,
    ).await;

    {
        let mut channels = state.response_channels.lock().await;
        channels.remove(&response_key);
    }

    result
}

async fn resolve_bot_uuid(pool: &botcore::shared::utils::DbPool, bot_name: &str) -> uuid::Uuid {
    if let Ok(uuid) = uuid::Uuid::parse_str(bot_name) {
        return uuid;
    }
    use diesel::RunQueryDsl;
    if let Ok(mut conn) = pool.get_timeout(std::time::Duration::from_secs(3)) {
        #[derive(diesel::QueryableByName)]
        struct BotId {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: uuid::Uuid,
        }
        diesel::sql_query("SELECT id FROM bots WHERE name = $1 AND is_active = true LIMIT 1")
            .bind::<diesel::sql_types::Text, _>(bot_name)
            .get_result::<BotId>(&mut conn)
            .ok()
            .map(|r| r.id)
            .unwrap_or_default()
    } else {
        uuid::Uuid::nil()
    }
}