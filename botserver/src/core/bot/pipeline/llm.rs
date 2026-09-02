use std::sync::Arc;
use std::time::Duration;

use botcore::shared::state::AppState;
use uuid::Uuid;

use super::sink::ChannelSink;
use super::types::PipelineResult;

fn is_web_channel(sink: &dyn ChannelSink) -> bool {
    sink.channel_type() == "web"
}

const LLM_RETRY_MAX: u32 = 3;
const LLM_RETRY_DELAY: u64 = 3;

fn llm_empty_message() -> String {
    let locale = botcore::i18n::Locale::new("pt-BR").unwrap_or_default();
    let msg = botcore::i18n::t(&locale, "error-llm-empty-response");
    if msg.starts_with('[') && msg.ends_with(']') {
        "Desculpe, nao consegui processar sua mensagem agora. Tente novamente em alguns segundos.".to_string()
    } else {
        msg
    }
}

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
    let bot_llm_provider: Option<(Arc<dyn botlib::traits::LLMProvider>, String, String)> = {
        use botcore::config::ConfigManager;
        let cfg = ConfigManager::new(state.conn.clone());
        let mut llm_url = cfg.get_config(&bot_uuid, "llm-url", Some("")).unwrap_or_default();
        let mut llm_key = cfg.get_config(&bot_uuid, "llm-key", Some("")).unwrap_or_default();
        let mut llm_model = cfg.get_config(&bot_uuid, "llm-model", Some("")).unwrap_or_default();
        let llm_provider = cfg.get_config(&bot_uuid, "llm-provider", Some("")).unwrap_or_default();
        if let Ok(val) = std::env::var("LLM_URL") { if !val.is_empty() { llm_url = val; } }
        if let Ok(val) = std::env::var("LLM_KEY") { if !val.is_empty() { llm_key = val; } }
        if let Ok(val) = std::env::var("LLM_MODEL") { if !val.is_empty() { llm_model = val; } }
        log::info!("BOT {} LLM CONFIG: url=[{}] key_len=[{}] model=[{}] provider=[{}]", bot_uuid, llm_url, llm_key.len(), llm_model, llm_provider);
        if !llm_url.is_empty() {
            let explicit = if llm_provider.is_empty() { None } else { crate::llm::llm_provider_type_from_name(&llm_provider) };
            let provider = crate::llm::create_llm_provider_from_url(
                &llm_url,
                if llm_model.is_empty() { None } else { Some(llm_model.clone()) },
                None, explicit,
            );
            Some((
                Arc::new(crate::llm::BotlibLLMProviderWrapper::new(
                    provider, llm_model.clone(), llm_key.clone(),
                )) as Arc<dyn botlib::traits::LLMProvider>,
                llm_key, llm_model,
            ))
        } else {
            log::warn!("CRISTO BOT: llm-url is EMPTY, falling back to global provider");
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

            let tools_arg = if session_tools.is_empty() { None } else { Some(session_tools) };

            let mut full_response = String::new();
            let reasoning_accumulated: String;

            for attempt in 0..=LLM_RETRY_MAX {
                full_response.clear();
                let mut content_buffer = String::new();
                let mut attempt_reasoning = String::new();

                let (retry_tx, mut retry_rx) = tokio::sync::mpsc::channel::<String>(100);

                let spawn_llm = llm.clone();
                let spawn_prompt = prompt_clone.clone();
                let spawn_messages = messages_clone.clone();
                let spawn_model = llm_model_clone.clone();
                let spawn_key = llm_key_clone.clone();
                let spawn_tools = tools_arg.clone();
                let _handle = tokio::spawn(async move {
                    if let Err(e) = spawn_llm.generate_stream(
                        &spawn_prompt, &spawn_messages, retry_tx,
                        &spawn_model, &spawn_key, spawn_tools.as_ref(),
                    ).await {
                        log::error!("LLM stream error: {e}");
                    }
                });

                if attempt == 0 {
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
                } else {
                    log::warn!("LLM retry attempt {}/{}", attempt, LLM_RETRY_MAX);
                }

                let mut keepalive = tokio::time::interval(Duration::from_millis(800));
                keepalive.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
                loop {
                    tokio::select! {
                        chunk = retry_rx.recv() => {
                            match chunk {
                                Some(chunk) => {
                                    if chunk.contains("\"__reasoning__\"") {
                                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&chunk) {
                                            if let Some(r) = val.get("__reasoning__").and_then(|v| v.as_str()) {
                                                attempt_reasoning.push_str(r);
                                                let rc = botlib::models::BotResponse {
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
                                                let _ = sink.send_bot_response(&rc).await;
                                            }
                                        }
                                        continue;
                                    }
                                    if chunk.contains("\"__tool_call__\"") {
                                        full_response.push_str(&chunk);
                                        continue;
                                    }
                                    if crate::core::bot::api_catalog::is_api_call_trigger(&chunk) {
                                        full_response.push_str(&chunk);
                                        continue;
                                    }
                                    if chunk.contains(crate::main_module::ui_plan::UI_PLAN_TRIGGER) {
                                        full_response.push_str(&chunk);
                                        continue;
                                    }
                                    full_response.push_str(&chunk);
                                    content_buffer.push_str(&chunk);
                                }
                                None => break,
                            }
                        }
                        _ = keepalive.tick() => {
                            let ka = serde_json::json!({
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
                            let _ = sink.send_raw_json(&ka).await;
                        }
                    }
                }

                let has_tool_call = full_response.contains("\"__tool_call__\":");
                let has_ui_plan = full_response.contains(crate::main_module::ui_plan::UI_PLAN_TRIGGER);
                let has_api_call = crate::core::bot::api_catalog::is_api_call_trigger(&full_response);
                log::info!("LLM RESPONSE end: {} bytes total, {} bytes content_buffer, has_tool_call={}, has_ui_plan={}, has_api_call={}", full_response.len(), content_buffer.len(), has_tool_call, has_ui_plan, has_api_call);

                if has_api_call && handle_api_call(
                    sink, state, &llm, llm_model, llm_key,
                    bot_uuid, session_id, user_id, bot_name,
                    &full_response, user_text,
                ).await {
                    {
                        let mut sm = state.session_manager.lock().await;
                        let _ = sm.save_message(session_id, user_id, 2, &full_response, 2);
                    }
                    break;
                }

                if has_ui_plan {
                    content_buffer = crate::main_module::ui_plan::strip_plan_json(&content_buffer);
                    if let Some(plan) = crate::main_module::ui_plan::extract_and_validate_plan(&full_response) {
                        match plan {
                            Ok(validated) => {
                                log::info!(
                                    "UI plan validated: {} steps for app {:?}",
                                    validated.steps.len(),
                                    validated.app
                                );
                                let plan_frame = serde_json::json!({
                                    "bot_id": bot_uuid_s,
                                    "user_id": user_id.to_string(),
                                    "session_id": session_id_s,
                                    "channel": "web",
                                    "content": "",
                                    "message_type": botlib::message_types::MessageType::UI_ACTION,
                                    "plan": validated,
                                    "is_complete": true,
                                    "suggestions": [],
                                    "switchers": [],
                                    "context_length": 0,
                                    "context_max_length": 0,
                                });
                                let _ = sink.send_raw_json(&plan_frame).await;
                            }
                            Err(e) => {
                                log::warn!("Rejected UI plan: {e}");
                            }
                        }
                    }
                }

                if !has_tool_call && content_buffer.is_empty() {
                    if attempt < LLM_RETRY_MAX {
                        log::warn!("LLM empty response on attempt {}/{}, retrying in {}s", attempt + 1, LLM_RETRY_MAX, LLM_RETRY_DELAY);
                        tokio::time::sleep(Duration::from_secs(LLM_RETRY_DELAY)).await;
                        continue;
                    }
                    log::warn!("LLM empty response after {} retries, sending fallback", LLM_RETRY_MAX);
                    let err_msg = llm_empty_message();
                    let err_resp = botlib::models::BotResponse::new(
                        &bot_uuid_s, &session_id_s, &user_id.to_string(),
                        &err_msg, "web",
                    );
                    let _ = sink.send_bot_response(&err_resp).await;
                    {
                        let mut sm = state.session_manager.lock().await;
                        let _ = sm.save_message(session_id, user_id, 2, &full_response, 2);
                    }
                    return Ok(());
                }

                reasoning_accumulated = attempt_reasoning;

                if !has_tool_call && !content_buffer.is_empty() {
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
                let is_deep = messages.as_array().map(|a| a.len() > 3).unwrap_or(false);
                if is_web_channel(sink) && !is_deep && super::tool_exec::is_generic_greeting(user_text) {
                        if !content_buffer.is_empty() {
                            log::info!("Greeting guard: skipping tool_call, sending buffered content instead");
                            let final_resp = botlib::models::BotResponse {
                                bot_id: bot_uuid_s.clone(),
                                user_id: user_id.to_string(),
                                session_id: session_id_s.clone(),
                                channel: "web".to_string(),
                                content: content_buffer.clone(),
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
                        } else {
                            log::info!("Greeting guard: tool_call-only response, sending generic greeting");
                            let msg = "Ola! Como posso ajudar voce hoje?";
                            let final_resp = botlib::models::BotResponse::new(
                                &bot_uuid_s, &session_id_s, &user_id.to_string(), msg, "web",
                            );
                            let _ = sink.send_bot_response(&final_resp).await;
                        }
                    } else {
                        super::tool_exec::run_llm_tool_call(
                            sink, state, bot_uuid, session_id, user_id, bot_name,
                            &full_response, rx, user_text,
                        ).await;
                    }
                }

                {
                    let mut sm = state.session_manager.lock().await;
                    let _ = sm.save_message(session_id, user_id, 2, &full_response, 2);
                }
                #[cfg(feature = "memory-os")]
                crate::core::bot::memory_hook::extract_from_turn(
                    user_id, uuid::Uuid::nil(), session_id,
                    user_text, &full_response,
                )
                .await;
                break;
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

/// Executes an `{"__api_call__": {"name", "params", "compose"}}` block found
/// in the LLM reply. Returns true when the call was handled (even on error),
/// so the caller skips the regular rendering path.
async fn handle_api_call(
    sink: &dyn ChannelSink,
    state: &Arc<AppState>,
    llm: &Arc<dyn botlib::traits::LLMProvider>,
    model: &str,
    key: &str,
    bot_uuid: Uuid,
    session_id: Uuid,
    user_id: Uuid,
    bot_name: &str,
    full_response: &str,
    user_text: &str,
) -> bool {
    use crate::core::bot::api_catalog;
    let payload = match extract_api_call_payload(full_response) {
        Some(p) => p,
        None => return false,
    };
    let name = payload.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    if name.is_empty() {
        return false;
    }
    let params = payload.get("params").cloned().unwrap_or(serde_json::Value::Null);
    let compose = payload.get("compose").and_then(|v| v.as_bool()).unwrap_or(false);
    let channel = sink.channel_type().to_string();

    // The LLM may have spoken a short "working on it..." line before the
    // __api_call__ block. Surface it to the user so the tool call feels
    // natural, then run the command. The JSON block itself is always
    // stripped out of the leading text (never shown to the user).
    let stripped = strip_api_call_block(full_response);
    let leading = stripped.trim();
    if !leading.is_empty() {
        let progress = botlib::models::BotResponse::new(
            bot_uuid.to_string(), session_id.to_string(), user_id.to_string(),
            leading, &channel,
        );
        let _ = sink.send_bot_response(&progress).await;
    }

    // Tax commands: models frequently omit the `value` param even when the
    // user stated an amount. Recover it from the user's message (issue #722).
    let params = if matches!(name.as_str(), "service.tax" | "tax.calculate") {
        inject_tax_value_from_user(&params, user_text)
    } else {
        params
    };

    #[cfg(feature = "consent")]
    {
        let app_id = crate::apps::commands::command_by_name(&name)
            .map(|c| c.app.to_string())
            .unwrap_or_else(|| name.clone());
        let gate = crate::core::bot::consent_gate::check(
            user_id,
            &app_id,
            "update",
            serde_json::json!({ "command": name, "params": params }),
        )
        .await;
        if !gate.allowed {
            let message = match gate.pending_request {
                Some(ref request) => {
                    crate::core::bot::consent_gate::marker_content(request)
                }
                None => crate::core::bot::consent_gate::deny_message(&app_id),
            };
            let resp = botlib::models::BotResponse::new(
                bot_uuid.to_string(), session_id.to_string(),
                user_id.to_string(), &message, &channel,
            );
            let _ = sink.send_bot_response(&resp).await;
            return true;
        }
    }

    match api_catalog::execute_command(state, bot_uuid, bot_name, user_id, &name, &params).await {
        Ok(result) => {
            if compose {
                let deep_links = extract_deep_links(&result);
                let prompt = format!(
                    "You are the assistant of bot '{bot_name}'. The user asked: \"{user_text}\".\n\
                     You ran the command '{name}' and received this data:\n{json}\n\n\
                     Write a concise, friendly answer for the user in the language of the user's message, \
                     using the data. Never mention JSON, commands or internal details.\n\
                     {deep_link_instruction}",
                    json = serde_json::to_string_pretty(&result).unwrap_or_default(),
                    deep_link_instruction = if deep_links.is_empty() {
                        String::new()
                    } else {
                        format!(
                            "The result references specific records. End your answer with a clickable \
                             link to the most relevant record, in the exact form provided (it starts \
                             with app://), e.g. '{first}'.",
                            first = deep_links[0],
                        )
                    },
                );
                match llm.generate(&prompt, &serde_json::json!({}), model, key).await {
                    Ok(text) => {
                        let resp = botlib::models::BotResponse::new(
                            bot_uuid.to_string(), session_id.to_string(), user_id.to_string(),
                            &text, &channel,
                        );
                        let _ = sink.send_bot_response(&resp).await;
                    }
                    Err(e) => {
                        log::error!("api_call compose LLM error: {e}");
                        let fallback = if deep_links.is_empty() {
                            "Dados obtidos, mas falhei ao redigir a resposta.".to_string()
                        } else {
                            format!(
                                "Here is the record you asked about: {link}",
                                link = deep_links[0],
                            )
                        };
                        let resp = botlib::models::BotResponse::new(
                            bot_uuid.to_string(), session_id.to_string(), user_id.to_string(),
                            &fallback, &channel,
                        );
                        let _ = sink.send_bot_response(&resp).await;
                    }
                }
            } else {
                let text = serde_json::to_string_pretty(&result).unwrap_or_default();
                let text = if text.len() > 2000 {
                    format!("{}...", &text[..2000])
                } else {
                    text
                };
                let resp = botlib::models::BotResponse::new(
                    bot_uuid.to_string(), session_id.to_string(), user_id.to_string(),
                    &text, &channel,
                );
                let _ = sink.send_bot_response(&resp).await;
            }
            true
        }
        Err(e) => {
            log::warn!("api_call '{name}' failed: {e}");
            let resp = botlib::models::BotResponse::new(
                bot_uuid.to_string(), session_id.to_string(), user_id.to_string(),
                &format!("Falha ao executar {name}: {e}"), &channel,
            );
            let _ = sink.send_bot_response(&resp).await;
            true
        }
    }
}

/// Collects every `app://...` deep-link string present anywhere in a command
/// result so the final answer can surface the most relevant one even when the
/// compose LLM call fails.
fn extract_deep_links(value: &serde_json::Value) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![value.clone()];
    while let Some(v) = stack.pop() {
        match v {
            serde_json::Value::String(s) => {
                if s.starts_with("app://") {
                    out.push(s.clone());
                }
            }
            serde_json::Value::Array(items) => {
                for item in items {
                    stack.push(item.clone());
                }
            }
            serde_json::Value::Object(map) => {
                for val in map.values() {
                    stack.push(val.clone());
                }
            }
            _ => {}
        }
    }
    out
}

/// Injects a `value` param for tax commands when the model omitted it but the
/// user's message contains a monetary amount ("R$ 10.000", "10000 reais").
fn inject_tax_value_from_user(params: &serde_json::Value, user_text: &str) -> serde_json::Value {
    let has_value = params
        .get("value")
        .and_then(|v| v.as_str())
        .is_some_and(|v| !v.trim().is_empty());
    if has_value {
        return params.clone();
    }
    let amount = extract_money_amount(user_text);
    match amount {
        Some(value) => {
            let mut params = params.clone();
            if let serde_json::Value::Object(map) = &mut params {
                map.insert("value".to_string(), serde_json::Value::String(value));
            }
            params
        }
        None => params.clone(),
    }
}

/// Extracts a BRL amount from free text, e.g. "R$ 10.000,50" or "10.000 reais".
/// Returns a normalized decimal string ("10000.50").
fn extract_money_amount(text: &str) -> Option<String> {
    let re = regex::Regex::new(r"(?i)R\$\s*([\d.,]+)|\b([\d.,]+)\s*reais").ok()?;
    let caps = re.captures(text)?;
    let raw = caps.get(1).or_else(|| caps.get(2))?.as_str();
    let cleaned = raw.replace('.', "");
    let normalized = if cleaned.contains(',') {
        cleaned.replace(',', ".")
    } else {
        cleaned
    };
    if normalized.parse::<f64>().is_ok() {
        Some(normalized)
    } else {
        None
    }
}

/// Extracts the first `{"__api_call__": {...}}` object from a response,
/// returning its payload (the object's `__api_call__` value, or the whole
/// object when it is not nested).
fn extract_api_call_payload(full_response: &str) -> Option<serde_json::Value> {
    use crate::core::bot::api_catalog::find_api_call;
    let pos = find_api_call(full_response)?;
    let obj_start = full_response[..pos].rfind('{')?;
    let bytes = full_response.as_bytes();
    let mut depth = 0i32;
    let mut in_string = false;
    for (i, &b) in bytes[obj_start..].iter().enumerate() {
        match b {
            b'"' => in_string = !in_string,
            b'{' if !in_string => depth += 1,
            b'}' if !in_string => {
                depth -= 1;
                if depth == 0 {
                    let end = obj_start + i + 1;
                    let parsed: serde_json::Value = match serde_json::from_str(&full_response[obj_start..end]) {
                        Ok(v) => v,
                        Err(_) => return None,
                    };
                    let inner = parsed.get("__api_call__").or_else(|| parsed.get("api_call")).or_else(|| parsed.get("_api_call_")).cloned().unwrap_or(parsed);
                    return Some(inner);
                }
            }
            _ => {}
        }
    }
    None
}

/// Removes the `{"__api_call__": ...}` JSON block (the model's tool-call
/// preamble) from a response, leaving just the user-facing prose before and
/// after it. The block is matched with brace balancing so it is stripped in
/// full no matter which key variant the model emitted.
fn strip_api_call_block(text: &str) -> String {
    use crate::core::bot::api_catalog::find_api_call;
    let Some(pos) = find_api_call(text) else {
        return text.to_string();
    };
    let Some(obj_start) = text[..pos].rfind('{') else {
        return text.to_string();
    };
    let bytes = text.as_bytes();
    let mut depth = 0i32;
    let mut in_string = false;
    let mut end = None;
    for (i, &b) in bytes[obj_start..].iter().enumerate() {
        match b {
            b'"' => in_string = !in_string,
            b'{' if !in_string => depth += 1,
            b'}' if !in_string => {
                depth -= 1;
                if depth == 0 {
                    end = Some(obj_start + i + 1);
                    break;
                }
            }
            _ => {}
        }
    }
    match end {
        Some(e) => {
            let mut out = String::with_capacity(text.len());
            out.push_str(&text[..obj_start]);
            out.push_str(&text[e..]);
            out
        }
        None => text.to_string(),
    }
}