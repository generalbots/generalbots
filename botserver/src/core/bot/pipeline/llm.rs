use std::sync::Arc;

use botcore::shared::state::AppState;
use uuid::Uuid;

use super::sink::ChannelSink;
use super::types::{PipelineError, PipelineResult};

pub async fn stream_llm_response(
    sink: &dyn ChannelSink,
    rx: &mut tokio::sync::mpsc::Receiver<botlib::models::BotResponse>,
    state: &Arc<AppState>,
    bot_uuid: Uuid,
    session_id: Uuid,
    user_id: Uuid,
    bot_name: &str,
    messages: &serde_json::Value,
    user_text: &str,
) -> PipelineResult<()> {
    let (stream_tx, mut stream_rx) = tokio::sync::mpsc::channel::<String>(100);
    let mut full_response = String::new();

    let bot_llm_provider: Option<(Arc<dyn botlib::traits::LLMProvider>, String, String)> = {
        use botcore::config::ConfigManager;
        let cfg = ConfigManager::new(state.conn.clone());
        let mut llm_url = cfg.get_config(&bot_uuid, "llm-url", Some("")).unwrap_or_default();
        let mut llm_key = cfg.get_config(&bot_uuid, "llm-key", Some("")).unwrap_or_default();
        let mut llm_model = cfg.get_config(&bot_uuid, "llm-model", Some("")).unwrap_or_default();
        if let Ok(val) = std::env::var("LLM_URL") { if !val.is_empty() { llm_url = val; } }
        if let Ok(val) = std::env::var("LLM_KEY") { if !val.is_empty() { llm_key = val; } }
        if let Ok(val) = std::env::var("LLM_MODEL") { if !val.is_empty() { llm_model = val; } }
        if !llm_url.is_empty() {
            let provider = crate::llm::create_llm_provider_from_url(
                &llm_url,
                if llm_model.is_empty() { None } else { Some(llm_model.clone()) },
                None, None,
            );
            Some((
                Arc::new(crate::llm::BotlibLLMProviderWrapper::new(
                    provider, llm_model.clone(), llm_key.clone(),
                )) as Arc<dyn botlib::traits::LLMProvider>,
                llm_key, llm_model,
            ))
        } else {
            None
        }
    };

    let answer_mode = {
        #[cfg(feature = "chat")]
        {
            crate::core::bot::answer_mode::get_answer_mode(state, &session_id).await
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
                    state, user_text, bot_uuid, bot_name, session_id, user_id,
                ).await
            }
            crate::core::bot::answer_mode::AnswerMode::Chart => {
                crate::core::bot::answer_mode::generate_chart_response(
                    state, user_text, bot_uuid, bot_name, session_id, user_id,
                ).await
            }
            _ => return Ok(()),
        };
        match mode_response {
            Ok(resp) => {
                let _ = sink.send_bot_response(&resp).await;
                let mut sm = state.session_manager.lock().await;
                let _ = sm.save_message(session_id, user_id, 2, &resp.content, 2);
            }
            Err(e) => {
                let err_resp = botlib::models::BotResponse::new(
                    bot_uuid.to_string(), session_id.to_string(),
                    user_id.to_string(),
                    format!("<p>Error: {e}</p>"), "web",
                );
                let _ = sink.send_bot_response(&err_resp).await;
            }
        }
        return Ok(());
    }

    let env_key = std::env::var("LLM_KEY").unwrap_or_default();
    let env_model = std::env::var("LLM_MODEL").unwrap_or_default();
    match bot_llm_provider.or_else(|| {
        state.llm_provider.clone().map(move |p| (p, env_key.clone(), env_model.clone()))
    }) {
        Some((ref llm, ref llm_key, ref llm_model)) => {
            // Trace the actual messages sent to the LLM
            log::info!("LLM REQUEST: {} messages, model={}", messages.as_array().map(|a| a.len()).unwrap_or(0), llm_model);

            let state_clone = state.clone();
            let prompt_clone = user_text.to_string();
            let messages_clone = messages.clone();
            let llm = llm.clone();
            let llm_key_clone = llm_key.clone();
            let llm_model_clone = llm_model.clone();
            let bot_uuid_s = bot_uuid.to_string();
            let session_id_s = session_id.to_string();
            let bot_name_clone = bot_name.to_string();

            let style_css = crate::core::bot::ws::message::load_bot_styles_css(&bot_name_clone);

            let session_tools = {
                let sid: Uuid = match session_id_s.parse() {
                    Ok(id) => id,
                    Err(_) => Uuid::new_v4(),
                };
                let pool = state_clone.conn.clone();
                let name = bot_name_clone.clone();
                let sid_for_block = sid;
                match tokio::time::timeout(
                    std::time::Duration::from_secs(10),
                    tokio::task::spawn_blocking(move || {
                        crate::core::bot::tool_context::get_session_tools(
                            &pool, &name, &sid_for_block,
                        )
                    }),
                ).await {
                    Ok(Ok(Ok(tools))) => tools,
                    Ok(Ok(Err(e))) => {
                        log::warn!("get_session_tools error for {}: {e}", session_id_s);
                        Vec::new()
                    }
                    Ok(Err(e)) => {
                        log::warn!("get_session_tools spawn_blocking join error for {}: {e}", session_id_s);
                        Vec::new()
                    }
                    Err(_) => {
                        log::warn!("get_session_tools TIMEOUT after 10s for session {session_id_s}");
                        Vec::new()
                    }
                }
            };

            let _stream_handle = tokio::spawn(async move {
                let tools_arg = if session_tools.is_empty() { None } else { Some(session_tools) };
                if let Err(e) = llm.generate_stream(
                    &prompt_clone, &messages_clone, stream_tx,
                    &llm_model_clone, &llm_key_clone, tools_arg.as_ref(),
                ).await {
                    log::error!("LLM stream error: {e}");
                }
            });

            {
                let mut init_msg = serde_json::json!({
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
                });
                if !style_css.is_empty() {
                    init_msg["css"] = serde_json::Value::String(style_css.clone());
                }
                let _ = sink.send_raw_json(&init_msg).await;
            }

            let mut reasoning_accumulated = String::new();
            let mut content_buffer = String::new();

            let mut keepalive_interval = tokio::time::interval(std::time::Duration::from_millis(800));
            keepalive_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                tokio::select! {
                    chunk = stream_rx.recv() => {
                        match chunk {
                            Some(chunk) => {
                                if chunk.contains("\"__reasoning__\"") {
                                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&chunk) {
                                        if let Some(r) = val.get("__reasoning__").and_then(|v| v.as_str()) {
                                            reasoning_accumulated.push_str(r);
                                            let reasoning_chunk = botlib::models::BotResponse {
                                                bot_id: bot_uuid_s.clone(),
                                                user_id: user_id.to_string(),
                                                session_id: session_id_s.clone(),
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
                                                reasoning: r.to_string(),
                                            };
                                            let _ = sink.send_bot_response(&reasoning_chunk).await;
                                        }
                                    }
                                    continue;
                                }
                                if chunk.contains("\"__tool_call__\"") {
                                    full_response.push_str(&chunk);
                                    continue;
                                }
                                full_response.push_str(&chunk);
                                // Buffer content chunks instead of sending immediately.
                                // If a tool_call is detected later, buffered content will be discarded.
                                content_buffer.push_str(&chunk);
                            }
                            None => break,
                        }
                    }
                    _ = keepalive_interval.tick() => {
                        let keepalive = serde_json::json!({
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
                        });
                        let _ = sink.send_raw_json(&keepalive).await;
                    }
                }
            }

            // Send buffered content only if no tool was called
            let has_tool_call = full_response.contains("\"__tool_call__\":");
            log::info!("LLM RESPONSE end: {} bytes total, {} bytes content_buffer, has_tool_call={}", full_response.len(), content_buffer.len(), has_tool_call);
            if content_buffer.len() > 0 {
                log::info!("LLM CONTENT BUFFER (first 300): {}", &content_buffer[..content_buffer.len().min(300)]);
            }

            if !has_tool_call && !content_buffer.is_empty() {
                // Convert markdown **bold** to <strong>bold</strong> (UTF-8 safe)
                let mut converted = content_buffer
                    .split("**")
                    .enumerate()
                    .map(|(i, part)| {
                        if i % 2 == 1 {
                            format!("<strong>{}</strong>", part)
                        } else {
                            part.to_string()
                        }
                    })
                    .collect::<Vec<_>>()
                    .concat();
                // Convert "- item" lines to <li> wrapped in <ul>
                let lines: Vec<&str> = converted.lines().collect();
                let mut out = Vec::new();
                let mut i = 0;
                while i < lines.len() {
                    let trimmed = lines[i].trim_start();
                    if trimmed.starts_with("- ") {
                        out.push("<ul>".to_string());
                        while i < lines.len() && lines[i].trim_start().starts_with("- ") {
                            let li_content = lines[i].trim_start().strip_prefix("- ").unwrap_or(lines[i].trim_start());
                            out.push(format!("<li>{}</li>", li_content));
                            i += 1;
                        }
                        out.push("</ul>".to_string());
                    } else {
                        out.push(lines[i].to_string());
                        i += 1;
                    }
                }
                converted = out.join("\n");
                let final_resp = botlib::models::BotResponse {
                    bot_id: bot_uuid_s.clone(),
                    user_id: user_id.to_string(),
                    session_id: session_id_s.clone(),
                    channel: "web".to_string(),
                    content: converted,
                    message_type: botlib::message_types::MessageType::BOT_RESPONSE,
                    stream_token: None,
                    is_complete: true,
                    suggestions: Vec::new(),
                    switchers: Vec::new(),
                    context_name: None,
                    context_length: 0,
                    context_max_length: 0,
                    reasoning: reasoning_accumulated.trim().to_string(),
                };
                let _ = sink.send_bot_response(&final_resp).await;
            }

            if has_tool_call {
                super::tool_exec::run_llm_tool_call(
                    sink, state, bot_uuid, session_id, user_id, bot_name,
                    &full_response, rx, user_text,
                ).await;
            }

            if !has_tool_call && content_buffer.is_empty() {
                let err_msg = "Desculpe, nao consegui processar sua mensagem agora (servico ocupado). Tente novamente em alguns segundos.";
                log::warn!("LLM empty response, sending fallback message to user");
                let err_resp = botlib::models::BotResponse::new(
                    &bot_uuid_s, &session_id_s, &user_id.to_string(),
                    err_msg, "web",
                );
                let _ = sink.send_bot_response(&err_resp).await;
            }

            {
                let mut sm = state.session_manager.lock().await;
                let _ = sm.save_message(session_id, user_id, 2, &full_response, 2);
            }
        }
        None => {
            let fallback = format!("Recebi: \"{user_text}\"");
            {
                let mut sm = state.session_manager.lock().await;
                let _ = sm.save_message(session_id, user_id, 2, &fallback, 2);
            }
            let resp = botlib::models::BotResponse::new(
                bot_uuid.to_string(), session_id.to_string(),
                user_id.to_string(), &fallback, "web",
            );
            let _ = sink.send_bot_response(&resp).await;
        }
    }

    Ok(())
}