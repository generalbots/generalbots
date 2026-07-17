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

            let mut keepalive_interval = tokio::time::interval(std::time::Duration::from_millis(800));
            keepalive_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                tokio::select! {
                    chunk = stream_rx.recv() => {
                        match chunk {
                            Some(chunk) => {
                                full_response.push_str(&chunk);
                                if !chunk.contains("\"__tool_call__\"") {
                                    let chunk_resp = botlib::models::BotResponse {
                                        bot_id: bot_uuid_s.clone(),
                                        user_id: user_id.to_string(),
                                        session_id: session_id_s.clone(),
                                        channel: "web".to_string(),
                                        content: chunk,
                                        message_type: botlib::message_types::MessageType::BOT_RESPONSE,
                                        stream_token: None,
                                        is_complete: false,
                                        suggestions: Vec::new(),
                                        switchers: Vec::new(),
                                        context_name: None,
                                        context_length: 0,
                                        context_max_length: 0,
                                    };
                                    if sink.send_bot_response(&chunk_resp).await.is_err() {
                                        break;
                                    }
                                }
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

            let mut final_content = full_response.trim_end().to_string();
            if let Some(pos) = final_content.rfind("{\"__tool_call__\":") {
                final_content = final_content[..pos].trim_end().to_string();
            }

            let final_resp = botlib::models::BotResponse::new(
                &bot_uuid_s, &session_id_s, &user_id.to_string(),
                &final_content, "web",
            );
            if sink.send_bot_response(&final_resp).await.is_err() {
                let mut pending = state.pending_stream_responses.lock().await;
                pending.insert(session_id_s.clone(), final_content.clone());
            }

            let tool_call_trigger = "\"__tool_call__\":".to_string();
            if full_response.contains(&tool_call_trigger) {
                super::tool_exec::run_llm_tool_call(
                    sink, state, bot_uuid, session_id, user_id, bot_name,
                    &full_response, rx,
                ).await;
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