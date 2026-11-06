use std::time::Duration;

use uuid::Uuid;

use super::sink::ChannelSink;
use super::api_call::handle_api_call;
use super::types::{LlmTarget, PipelineResult, SessionIdentity};



/// Loads the session's tool set with a 10 s timeout so a slow DB cannot stall
/// the streaming LLM loop; any failure yields an empty tool list.
async fn get_session_tools_with_timeout(
    state: &botcore::shared::state::AppState,
    bot_name: &str,
    session_id_s: &str,
) -> Vec<serde_json::Value> {
    let sid: Uuid = match session_id_s.parse() {
        Ok(id) => id,
        Err(_) => Uuid::new_v4(),
    };
    let pool = state.conn.clone();
    let name = bot_name.to_string();
    let sid_for_block = sid;
    match tokio::time::timeout(
        std::time::Duration::from_secs(10),
        tokio::task::spawn_blocking(move || {
            crate::core::bot::tool_context::get_session_tools(&pool, &name, &sid_for_block)
        }),
    ).await {
        Ok(Ok(Ok(tools))) => tools,
        Ok(Ok(Err(e))) => {
            botlog::safe_warn!("get_session_tools error for {session_id_s}: {e}");
            Vec::new()
        }
        Ok(Err(e)) => {
            botlog::safe_warn!("get_session_tools spawn_blocking join error for {session_id_s}: {e}");
            Vec::new()
        }
        Err(_) => {
            botlog::safe_warn!("get_session_tools TIMEOUT after 10s for session {session_id_s}");
            Vec::new()
        }
    }
}

fn is_web_channel(sink: &dyn ChannelSink) -> bool {
    sink.channel_type() == "web"
}

const LLM_RETRY_MAX: u32 = 3;
const LLM_RETRY_DELAY: u64 = 3;

fn llm_empty_message() -> String {
    let locale = botcore::i18n::Locale::new("pt-BR").unwrap_or_default();
    let msg = botcore::i18n::t(&locale, "error-llm-empty-response");
    if msg.starts_with('[') && msg.ends_with(']') {
        "Sorry, I could not process your message right now. Please try again in a few seconds.".to_string()
    } else {
        msg
    }
}

pub async fn stream_llm_response(
    identity: &SessionIdentity<'_>,
    rx: &mut tokio::sync::mpsc::Receiver<botlib::models::BotResponse>,
    messages: &serde_json::Value,
    user_text: &str,
    model_preference: &str,
) -> PipelineResult<()> {
    stream_llm_response_inner(identity, rx, messages, user_text, model_preference, 0).await
}

/// Bounded tool loop: after a tool call the result is fed back to the model
/// for a final answer. Before this, the pipeline broke immediately after the
/// tool ran, so the chat ended on the raw output (an `ls -la` listing, a
/// JSON blob, …) instead of an answer.
const MAX_TOOL_FOLLOWUPS: usize = 2;

async fn stream_llm_response_inner(
    identity: &SessionIdentity<'_>,
    rx: &mut tokio::sync::mpsc::Receiver<botlib::models::BotResponse>,
    messages: &serde_json::Value,
    user_text: &str,
    model_preference: &str,
    tool_depth: usize,
) -> PipelineResult<()> {
    let SessionIdentity {
        sink,
        state,
        bot_uuid,
        session_id,
        user_id,
        bot_name,
    } = *identity;
    let bot_llm_provider =
        match super::enriched_llm::resolve_bot_llm_provider(
            state, bot_uuid, user_id, session_id, model_preference,
        ).await {
            Ok(provider) => provider,
            Err(message) => {
                // #1654 — a requested subscription without a linked account is
                // an actionable error, never a silent fallback to another key.
                sink.send_error(&session_id.to_string(), &message).await?;
                return Ok(());
            }
        };

    if super::enriched_llm::respond_in_answer_mode(
        sink, state, user_text, bot_uuid, bot_name, session_id, user_id,
    ).await? {
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

            let mut session_tools =
                get_session_tools_with_timeout(&state_clone, &bot_name_clone, &session_id_s).await;
            // #1711 — the native structured tools must be callable through the
            // provider's tool API, not only through a raw `__tool_call__`
            // marker printed in prose.
            if is_web_channel(sink) && super::event_sink::enabled() {
                session_tools.extend(super::surface_catalog::llm_tools());
            }

            let tools_arg = if session_tools.is_empty() { None } else { Some(session_tools) };

            let mut full_response = String::new();
            let reasoning_accumulated: String;

            // #1696 — per-run caps must gate THIS path too: the web chat
            // streams through the pipeline, not the legacy ws loop.
            let caps = crate::core::bot::runs::caps_resolve::resolve_for_run(
                state, bot_uuid, user_id,
            );
            let mut budget = crate::core::bot::runs::caps::RunBudget::start(caps);

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

                // #1700 — first content chunk latency for this attempt.
                let llm_started_at = std::time::Instant::now();
                let mut ttft_logged = false;
                let mut keepalive = tokio::time::interval(Duration::from_millis(800));
                keepalive.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
                loop {
                    // #1666 — Stop is cooperative: the owner checks between
                    // chunks and ends the turn without running tool calls.
                    if crate::core::bot::runs::bus::bus().is_cancelled(&session_id_s) {
                        let cancelled = botlib::models::BotResponse::new(
                            &bot_uuid_s,
                            &session_id_s,
                            user_id.to_string(),
                            "Stopped.",
                            "web",
                        );
                        let _ = sink.send_bot_response(&cancelled).await;
                        return Ok(());
                    }
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
                                        // Each __tool_call__ chunk arrives as one
                                        // complete JSON object (one per parallel
                                        // tool call). Track them separately so the
                                        // executor below runs every call with its
                                        // own arguments; concatenating them merged
                                        // distinct argument objects ("{..}{..}")
                                        // and broke parsing.
                                        if !full_response.is_empty() && !full_response.ends_with('\n') {
                                            full_response.push('\n');
                                        }
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
                                    budget.observe_chars(full_response.chars().count());
                                    if let Some(breach) = budget.check() {
                                        let notice = format!(
                                            "Run stopped — {}.",
                                            crate::core::bot::runs::caps::RunBudget::describe(&breach)
                                        );
                                        log::warn!(
                                            "run cap: {} (session {})",
                                            crate::core::bot::runs::caps::RunBudget::describe(&breach),
                                            session_id_s
                                        );
                                        {
                                            let mut sm = state.session_manager.lock().await;
                                            let _ = sm.save_message(session_id, user_id, 2, &notice, 2);
                                        }
                                        let capped = serde_json::json!({
                                            "bot_id": bot_uuid_s,
                                            "user_id": user_id.to_string(),
                                            "session_id": session_id_s,
                                            "channel": "web",
                                            "content": notice,
                                            "message_type": 2,
                                            "is_complete": true,
                                            "suggestions": [],
                                            "switchers": [],
                                            "context_length": 0,
                                            "context_max_length": 0,
                                        });
                                        let _ = sink.send_raw_json(&capped).await;
                                        return Ok(());
                                    }
                                    if !ttft_logged {
                                        ttft_logged = true;
                                        let ttft_ms = i64::try_from(
                                            llm_started_at.elapsed().as_millis(),
                                        )
                                        .unwrap_or(i64::MAX);
                                        log::info!(
                                            "llm_ttft: {} ms (pipeline, model {})",
                                            ttft_ms,
                                            llm_model
                                        );
                                        crate::core::bot::runs::ttft::record_first_token(
                                            &state.conn,
                                            session_id,
                                            bot_uuid,
                                            &llm_model,
                                            ttft_ms,
                                        );
                                    }
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

                let llm_target = LlmTarget {
                    provider: Some(&llm),
                    model: llm_model,
                    key: llm_key,
                };
                if has_api_call && handle_api_call(
                    identity, &llm_target, &full_response, user_text, rx,
                ).await {
                    {
                        let mut sm = state.session_manager.lock().await;
                        let _ = sm.save_message(session_id, user_id, 2, &full_response, 2);
                        let _ = sm.record_message_model(session_id, llm_model);
                    }
                    break;
                }

                if has_ui_plan {
                    content_buffer = crate::main_module::ui_plan::scrub_plan_fragments(
                        &crate::main_module::ui_plan::strip_plan_json(&content_buffer),
                    );
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
                        &bot_uuid_s, &session_id_s, user_id.to_string(),
                        &err_msg, "web",
                    );
                    let _ = sink.send_bot_response(&err_resp).await;
                    {
                        let mut sm = state.session_manager.lock().await;
                        let _ = sm.save_message(session_id, user_id, 2, &full_response, 2);
                        let _ = sm.record_message_model(session_id, llm_model);
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
                                &bot_uuid_s, &session_id_s, user_id.to_string(), msg, "web",
                            );
                            let _ = sink.send_bot_response(&final_resp).await;
                        }
                    } else {
                        // Execute every __tool_call__ JSON object in the reply,
                        // not only the first: parallel calls (e.g. two media
                        // items classified in one turn) must each run with
                        // their own arguments.
                        let call_chunks: Vec<&str> = full_response
                            .split('\n')
                            .filter(|c| c.contains("\"__tool_call__\":"))
                            .collect();
                        let tool_output = if call_chunks.len() > 1 {
                            log::info!("Executing {} parallel tool calls", call_chunks.len());
                            let mut parts = Vec::new();
                            for call_chunk in call_chunks {
                                parts.push(
                                    super::tool_exec::run_llm_tool_call(
                                        identity, rx, call_chunk, user_text,
                                    ).await,
                                );
                            }
                            parts.join("\n\n")
                        } else {
                            super::tool_exec::run_llm_tool_call(
                                identity, rx, &full_response, user_text,
                            ).await
                        };
                        if tool_depth < MAX_TOOL_FOLLOWUPS && !tool_output.trim().is_empty() {
                            // Keep the follow-up inside a sane prompt budget;
                            // a runaway listing must not crowd out the answer.
                            let follow_output: String = tool_output.chars().take(12_000).collect();
                            let mut follow_messages = messages.clone();
                            // #1701 — a message sent while the step runs is
                            // honored at the step boundary in THIS path too
                            // (web chat streams through the pipeline, so the
                            // ws-loop steering queue alone was dead code).
                            if let Some(steer) = crate::core::bot::runs::bus::bus()
                                .take_steering(&session_id_s)
                            {
                                log::warn!(
                                    "steering applied before follow-up (session {}): {}",
                                    session_id_s, steer
                                );
                                let notice = serde_json::json!({
                                    "type": "gb.steer",
                                    "session_id": session_id_s,
                                    "message": steer,
                                });
                                let _ = sink.send_raw_json(&notice).await;
                                if let Some(items) = follow_messages.as_array_mut() {
                                    items.push(serde_json::json!({
                                        "role": "user",
                                        "content": crate::core::bot::runs::steering::instruction(&steer),
                                    }));
                                }
                            }
                            if let Some(items) = follow_messages.as_array_mut() {
                                items.push(serde_json::json!({
                                    "role": "assistant",
                                    "content": full_response,
                                }));
                                items.push(serde_json::json!({
                                    "role": "user",
                                    "content": format!(
                                        "The tool finished with this result:\n\n{follow_output}\n\nAnswer the user's original request now using this result. Do not call another tool unless it is strictly necessary."
                                    ),
                                }));
                            }
                            log::info!(
                                "tool follow-up: depth {}/{} after {} bytes of tool output",
                                tool_depth + 1,
                                MAX_TOOL_FOLLOWUPS,
                                tool_output.len()
                            );
                            return Box::pin(stream_llm_response_inner(
                                identity,
                                rx,
                                &follow_messages,
                                user_text,
                                model_preference,
                                tool_depth + 1,
                            ))
                            .await;
                        }
                    }
                }

                {
                    let mut sm = state.session_manager.lock().await;
                    let _ = sm.save_message(session_id, user_id, 2, &full_response, 2);
                    let _ = sm.record_message_model(session_id, llm_model);
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
