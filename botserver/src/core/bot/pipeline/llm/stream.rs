//! Turn orchestration: provider resolution, bounded retries, UI plans, API
//! calls and the tool follow-up loop.

use std::time::Duration;

use super::super::api_call::handle_api_call;
use super::super::types::{LlmTarget, PipelineResult, SessionIdentity};
use super::attempt::{stream_attempt, Attempt, AttemptOutcome};
use super::markdown::convert_markdown;
use super::{
    get_session_tools_with_timeout, is_web_channel, llm_empty_message, LLM_RETRY_DELAY,
    LLM_RETRY_MAX,
};

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
        ..
    } = *identity;
    let bot_llm_provider = match super::super::enriched_llm::resolve_bot_llm_provider(
        state,
        bot_uuid,
        user_id,
        session_id,
        model_preference,
    )
    .await
    {
        Ok(provider) => provider,
        Err(message) => {
            // #1654 — a requested subscription without a linked account is
            // an actionable error, never a silent fallback to another key.
            sink.send_error(&session_id.to_string(), &message).await?;
            return Ok(());
        }
    };

    if super::super::enriched_llm::respond_in_answer_mode(
        sink, state, user_text, bot_uuid, bot_name, session_id, user_id,
    )
    .await?
    {
        return Ok(());
    }

    let env_key = std::env::var("LLM_KEY").unwrap_or_default();
    let env_model = std::env::var("LLM_MODEL").unwrap_or_default();
    match bot_llm_provider.or_else(|| {
        state
            .llm_provider
            .clone()
            .map(move |p| (p, env_key.clone(), env_model.clone()))
    }) {
        Some((ref llm, ref llm_key, ref llm_model)) => {
            log::info!(
                "LLM REQUEST: {} messages, model={}",
                messages.as_array().map(|a| a.len()).unwrap_or(0),
                llm_model
            );

            let prompt_clone = user_text.to_string();
            let messages_clone = messages.clone();
            let bot_uuid_s = bot_uuid.to_string();
            let session_id_s = session_id.to_string();
            let bot_name_clone = bot_name.to_string();

            let style_css = crate::core::bot::ws::message::load_bot_styles_css(&bot_name_clone);

            let mut session_tools =
                get_session_tools_with_timeout(state, &bot_name_clone, &session_id_s).await;
            // #1711 — the native structured tools must be callable through the
            // provider's tool API, not only through a raw `__tool_call__`
            // marker printed in prose.
            if is_web_channel(sink) && super::super::event_sink::enabled() {
                session_tools.extend(super::super::surface_catalog::llm_tools());
            }

            let tools_arg = if session_tools.is_empty() {
                None
            } else {
                Some(session_tools)
            };

            // #1696 — per-run caps must gate THIS path too: the web chat
            // streams through the pipeline, not the legacy ws loop.
            let caps =
                crate::core::bot::runs::caps_resolve::resolve_for_run(state, bot_uuid, user_id);
            let mut budget = crate::core::bot::runs::caps::RunBudget::start(caps);

            for attempt in 0..=LLM_RETRY_MAX {
                let outcome = stream_attempt(Attempt {
                    sink,
                    state,
                    bot_uuid,
                    bot_uuid_s: &bot_uuid_s,
                    session_id,
                    session_id_s: &session_id_s,
                    user_id,
                    llm,
                    prompt: &prompt_clone,
                    messages: &messages_clone,
                    model: llm_model.as_str(),
                    key: llm_key.as_str(),
                    tools: &tools_arg,
                    style_css: &style_css,
                    budget: &mut budget,
                    attempt,
                })
                .await;
                let (full_response, mut content_buffer, reasoning_accumulated) = match outcome {
                    AttemptOutcome::Stopped | AttemptOutcome::Capped => return Ok(()),
                    AttemptOutcome::Completed {
                        full_response,
                        content_buffer,
                        reasoning,
                    } => (full_response, content_buffer, reasoning),
                };

                let has_tool_call = full_response.contains("\"__tool_call__\":");
                let has_ui_plan =
                    full_response.contains(crate::main_module::ui_plan::UI_PLAN_TRIGGER);
                let has_api_call =
                    crate::core::bot::api_catalog::is_api_call_trigger(&full_response);
                log::info!("LLM RESPONSE end: {} bytes total, {} bytes content_buffer, has_tool_call={}, has_ui_plan={}, has_api_call={}", full_response.len(), content_buffer.len(), has_tool_call, has_ui_plan, has_api_call);

                let llm_target = LlmTarget {
                    provider: Some(llm),
                    model: llm_model,
                    key: llm_key,
                };
                if has_api_call
                    && handle_api_call(identity, &llm_target, &full_response, user_text, rx).await
                {
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
                    if let Some(plan) =
                        crate::main_module::ui_plan::extract_and_validate_plan(&full_response)
                    {
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
                        log::warn!(
                            "LLM empty response on attempt {}/{}, retrying in {}s",
                            attempt + 1,
                            LLM_RETRY_MAX,
                            LLM_RETRY_DELAY
                        );
                        tokio::time::sleep(Duration::from_secs(LLM_RETRY_DELAY)).await;
                        continue;
                    }
                    log::warn!(
                        "LLM empty response after {} retries, sending fallback",
                        LLM_RETRY_MAX
                    );
                    let locale = botcore::i18n::locale_for_user_pool(&state.conn, user_id);
                    let err_msg = llm_empty_message(&locale);
                    let err_resp = botlib::models::BotResponse::new(
                        &bot_uuid_s,
                        &session_id_s,
                        user_id.to_string(),
                        &err_msg,
                        "web",
                    );
                    if let Err(error) = sink.send_bot_response(&err_resp).await {
                        log::error!("empty-response notice not delivered: {error}");
                    }
                    {
                        let mut sm = state.session_manager.lock().await;
                        let _ = sm.save_message(session_id, user_id, 2, &full_response, 2);
                        let _ = sm.record_message_model(session_id, llm_model);
                    }
                    return Ok(());
                }

                if !has_tool_call && !content_buffer.is_empty() {
                    // #2108 — the drawing contract is enforced here, not only
                    // asked for in the prompt: a reply that carries an inline,
                    // HTML-escaped or unfenced SVG leaves as exactly one ```svg
                    // block, so the renderer previews it instead of the
                    // sanitizer escaping the markup into the bubble.
                    let drawing_turn =
                        super::super::system_prompt::wants_drawing(user_text);
                    let content =
                        super::svg_contract::enforce(&content_buffer, drawing_turn);
                    let converted = convert_markdown(&content);
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
                    // #2084 — the reply is the whole point of the turn: a
                    // transport error here used to be discarded by `let _`,
                    // so the client waited forever with nothing in the log.
                    if let Err(error) = sink.send_bot_response(&final_resp).await {
                        log::error!(
                            "final LLM reply not delivered ({} bytes): {error}",
                            final_resp.content.len()
                        );
                    }
                }

                if has_tool_call {
                    let is_deep = messages.as_array().map(|a| a.len() > 3).unwrap_or(false);
                    if is_web_channel(sink)
                        && !is_deep
                        && super::super::tool_exec::is_generic_greeting(user_text)
                    {
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
                            log::info!(
                                "Greeting guard: tool_call-only response, sending generic greeting"
                            );
                            let msg = "Ola! Como posso ajudar voce hoje?";
                            let final_resp = botlib::models::BotResponse::new(
                                &bot_uuid_s,
                                &session_id_s,
                                user_id.to_string(),
                                msg,
                                "web",
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
                                    super::super::tool_exec::run_llm_tool_call(
                                        identity, rx, call_chunk, user_text,
                                    )
                                    .await,
                                );
                            }
                            parts.join("\n\n")
                        } else {
                            super::super::tool_exec::run_llm_tool_call(
                                identity,
                                rx,
                                &full_response,
                                user_text,
                            )
                            .await
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
                            if let Some(steer) =
                                crate::core::bot::runs::bus::bus().take_steering(&session_id_s)
                            {
                                botlog::safe_warn!(
                                    "steering applied before follow-up (session {}, {} chars)",
                                    session_id_s,
                                    steer.chars().count()
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
                    user_id,
                    uuid::Uuid::nil(),
                    session_id,
                    user_text,
                    &full_response,
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
                bot_uuid.to_string(),
                session_id.to_string(),
                user_id.to_string(),
                &fallback,
                "web",
            );
            let _ = sink.send_bot_response(&resp).await;
        }
    }

    Ok(())
}
