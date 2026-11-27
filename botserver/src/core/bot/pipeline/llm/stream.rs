//! Turn orchestration: provider resolution, bounded retries, UI plans, API
//! calls and the tool follow-up loop.

use std::time::Duration;

use super::super::api_call::handle_api_call;
use super::super::types::{LlmTarget, PipelineResult, SessionIdentity};
use super::attempt::{stream_attempt, Attempt, AttemptOutcome};
use super::markdown::for_shell;
use super::{get_session_tools_with_timeout, is_web_channel, LLM_RETRY_DELAY, LLM_RETRY_MAX};

pub async fn stream_llm_response(
    identity: &SessionIdentity<'_>,
    rx: &mut tokio::sync::mpsc::Receiver<botlib::models::BotResponse>,
    messages: &serde_json::Value,
    user_text: &str,
    model_preference: &str,
) -> PipelineResult<()> {
    stream_llm_response_inner(identity, rx, messages, user_text, model_preference, 0, false).await
}

async fn stream_llm_response_inner(
    identity: &SessionIdentity<'_>,
    rx: &mut tokio::sync::mpsc::Receiver<botlib::models::BotResponse>,
    messages: &serde_json::Value,
    user_text: &str,
    model_preference: &str,
    tool_depth: usize,
    final_round: bool,
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
    // Start the subscription workspace visibly and within a bound.
    #[cfg(feature = "agent-vm")]
    if tool_depth == 0 {
        if let Err(refusal) =
            super::super::workspace_start::before_subscription_turn(identity, model_preference).await
        {
            super::stream_errors::send_refusal_error(sink, identity, &refusal).await;
            return Ok(());
        }
    }
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
        Err(refusal) => {
            super::stream_errors::send_refusal_error(sink, identity, &refusal).await;
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

    // Classify engine type once for per-engine usage grouping. Kept here,
    // before the provider match, so every retry and every persisted turn uses
    // the same classification as the attempt that produced the answer.
    let (engine_type, engine_provider) = {
        let cfg = botcore::config::ConfigManager::new(state.conn.clone());
        let config_value = |key: &str| -> String {
            match cfg.get_config(&bot_uuid, key, Some("")) {
                Ok(value) => value,
                Err(error) => {
                    log::warn!("engine classification: {key} could not be read: {error}");
                    String::new()
                }
            }
        };
        let bp = config_value("llm-provider");
        let mut eu = config_value("llm-url");
        let mut em = config_value("llm-model");
        if let Ok(v) = std::env::var("LLM_URL") {
            if !v.is_empty() {
                eu = v;
            }
        }
        if let Ok(v) = std::env::var("LLM_MODEL") {
            if !v.is_empty() {
                em = v;
            }
        }
        let cls = botsettings::usage_engine::classify_preference(model_preference, &bp, &eu, &em);
        (cls.as_ref().map(|(e, _)| *e), cls.map(|(_, p)| p))
    };

    let env_key = std::env::var("LLM_KEY").unwrap_or(Default::default());
    let env_model = std::env::var("LLM_MODEL").unwrap_or(Default::default());
    match bot_llm_provider.or_else(|| {
        state
            .llm_provider
            .clone()
            .map(move |p| (p, env_key.clone(), env_model.clone()))
    }) {
        Some((ref llm, ref llm_key, ref llm_model)) => {
            log::info!(
                "LLM REQUEST: {} messages, model={}, sizes: {}",
                messages.as_array().map(|a| a.len()).unwrap_or(0),
                llm_model,
                super::prompt_sizes::summarize(&messages)
            );

            let prompt_clone = user_text.to_string();
            let mut messages_clone = messages.clone();
            let bot_uuid_s = bot_uuid.to_string();
            let session_id_s = session_id.to_string();
            let bot_name_clone = bot_name.to_string();

            let style_css = crate::core::bot::ws::message::load_bot_styles_css(&bot_name_clone);

            let mut session_tools =
                get_session_tools_with_timeout(state, &bot_name_clone, &session_id_s).await;
            // #1711 — structured tools through the provider's tool API.
            if is_web_channel(sink) && super::super::event_sink::enabled() {
                session_tools.extend(super::super::surface_catalog::llm_tools());
            }

            // The bounded tool loop's last round offers no tools (turn_end).
            let tools_arg = super::turn_end::tools_for_round(session_tools, final_round);
            if engine_type == Some(botsettings::usage_engine::ENGINE_LOCAL) {
                super::local_profile::apply(&mut messages_clone, tools_arg.is_some());
            }

            // #1696 — per-run caps gate the web chat stream path.
            let caps =
                crate::core::bot::runs::caps_resolve::resolve_for_run(state, bot_uuid, user_id);
            let mut budget = crate::core::bot::runs::caps::RunBudget::start(caps);
            #[cfg(feature = "gblite")]
            let mut local_recovered = false;
            // Preflight: a dead local engine is noticed here, not after timeouts.
            #[cfg(feature = "gblite")]
            if engine_type == Some(botsettings::usage_engine::ENGINE_LOCAL) {
                super::local_recovery::preflight(identity, state, &session_id_s).await;
            }
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
                    engine_type,
                    engine_provider: engine_provider.as_deref(),
                })
                .await;
                let (full_response, mut content_buffer, reasoning_accumulated) = match outcome {
                    AttemptOutcome::Stopped | AttemptOutcome::Capped => return Ok(()),
                    AttemptOutcome::Refused(refusal) => return super::stream_errors::refuse(sink, identity, &refusal).await,
                    AttemptOutcome::Failed { code, detail } => {
                        log::warn!("LLM attempt {}/{} failed: {code:?} — {detail}", attempt + 1, LLM_RETRY_MAX + 1);
                        // ModelUnreachable: inline recovery before normal retry.
                        #[cfg(feature = "gblite")]
                        if matches!(code, super::error_classify::LlmErrorCode::ModelUnreachable)
                            && !local_recovered
                            && attempt < LLM_RETRY_MAX
                        {
                            if super::local_recovery::try_recover_with_progress(
                                identity, state, &session_id_s,
                            ).await {
                                local_recovered = true;
                                continue;
                            }
                        }
                        // Free-model 404: self-heal from the gateway list.
                        #[cfg(feature = "gblite")]
                        match super::free_model_fallback::try_replace_if_free(
                            state, engine_type, &detail, llm_model,
                        ).await {
                            super::free_model_fallback::FallbackResult::Replaced => {
                                return Box::pin(stream_llm_response_inner(
                                    identity, rx, messages, user_text, model_preference, tool_depth, final_round,
                                )).await;
                            }
                            super::free_model_fallback::FallbackResult::Unavailable => {
                                super::stream_errors::send_refusal_error(sink, identity,
                                    &super::refusal::RefusalError::new(
                                        super::refusal::FREE_MODEL_UNAVAILABLE,
                                        "The free model is unavailable right now.",
                                    )).await;
                                return Ok(());
                            }
                            super::free_model_fallback::FallbackResult::NotApplicable => {}
                        }
                        if code.is_retryable() && attempt < LLM_RETRY_MAX {
                            tokio::time::sleep(Duration::from_secs(LLM_RETRY_DELAY)).await;
                            continue;
                        }
                        // Last attempt: error to client, background recovery.
                        #[cfg(feature = "gblite")]
                        if matches!(code, super::error_classify::LlmErrorCode::ModelUnreachable) {
                            super::local_recovery::maybe_recover(state);
                        }
                        super::stream_errors::send_classified_error(sink, identity, code).await;
                        return Ok(());
                    }
                    AttemptOutcome::Completed {
                        full_response,
                        content_buffer,
                        reasoning,
                    } => (full_response, content_buffer, reasoning),
                };

                let has_tool_call = super::turn_end::runs_tools(&full_response, final_round);
                content_buffer = super::turn_end::answer_text(&content_buffer, final_round);
                let has_ui_plan =
                    full_response.contains(crate::main_module::ui_plan::UI_PLAN_TRIGGER);
                let has_api_call =
                    crate::core::bot::api_catalog::is_api_call_trigger(&full_response);
                log::info!("LLM RESPONSE end: {} bytes total, {} bytes content_buffer, has_tool_call={}, has_ui_plan={}, has_api_call={}, final_round={final_round}", full_response.len(), content_buffer.len(), has_tool_call, has_ui_plan, has_api_call);

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
                        // The actual user-facing reply was persisted inside
                        // handle_api_call (marker, composed answer, ...); do
                        // not store the raw __api_call__ envelope here.
                        if let Err(e) = sm.record_message_model(session_id, llm_model) {
                            log::warn!("stream: model not recorded: {e}");
                        }
                    }
                    break;
                }

                if has_api_call {
                    // #2583 — the block was detected but no command ran. The
                    // model's surrounding note is a promise, not a result: it
                    // must never be shown as one. End the turn with an explicit
                    // failure frame so the streaming bubble (which can contain
                    // JSON fragments emitted before the trigger was complete)
                    // is finalized and replaced by an honest answer.
                    log::warn!("api-call block detected but not executed; refusing to render");
                    let locale =
                        botcore::i18n::locale_for_user_pool(&state.conn, user_id);
                    let err_msg = super::llm_empty_message(&locale);
                    {
                        let mut sm = state.session_manager.lock().await;
                        if let Err(e) = sm.save_message(session_id, user_id, 2, &err_msg, 2) {
                            log::warn!("stream: empty-response notice not persisted: {e}");
                        }
                        if let Err(e) = sm.record_message_model(session_id, llm_model) {
                            log::warn!("stream: model not recorded: {e}");
                        }
                    }
                    super::stream_errors::send_empty_response_error(sink, identity).await;
                    return Ok(());
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
                                // #2194 — recover the landing tab.
                                let validated = crate::main_module::ui_plan::apply_open_hints(
                                    validated, user_text,
                                );
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
                                if let Err(e) = sink.send_raw_json(&plan_frame).await {
                                    log::warn!("stream: UI plan frame not delivered: {e}");
                                }
                            }
                            Err(e) => {
                                log::warn!("Rejected UI plan: {e}");
                            }
                        }
                    }
                }

                if !has_tool_call && content_buffer.is_empty() {
                    // A last round that only called a tool again is not retried.
                    if attempt < LLM_RETRY_MAX && !full_response.contains("\"__tool_call__\":") {
                        log::warn!(
                            "LLM empty response on attempt {}/{}, retrying in {}s",
                            attempt + 1,
                            LLM_RETRY_MAX + 1,
                            LLM_RETRY_DELAY
                        );
                        tokio::time::sleep(Duration::from_secs(LLM_RETRY_DELAY)).await;
                        continue;
                    }
                    log::warn!("LLM empty response after {LLM_RETRY_MAX} retries, using fallback");
                    let saved = if final_round {
                        super::turn_end::send_unfinished(identity).await.to_string()
                    } else {
                        super::stream_errors::send_empty_response_error(sink, identity).await;
                        let locale = botcore::i18n::locale_for_user_pool(&state.conn, user_id);
                        super::llm_empty_message(&locale)
                    };
                    {
                        let mut sm = state.session_manager.lock().await;
                        if let Err(e) = sm.save_message(session_id, user_id, 2, &saved, 2) {
                            log::warn!("stream: empty-response notice not persisted: {e}");
                        }
                        if let Err(e) = sm.record_message_model(session_id, llm_model) {
                            log::warn!("stream: model not recorded: {e}");
                        }
                    }
                    return Ok(());
                }

                if !has_tool_call && !content_buffer.is_empty() {
                    // #2108 — enforce the SVG drawing contract.
                    let drawing_turn =
                        super::super::system_prompt::wants_drawing(user_text);
                    let content =
                        super::svg_contract::enforce(&content_buffer, drawing_turn);
                    super::turn_end::send_text(identity, for_shell(&content), &reasoning_accumulated).await;
                }

                if has_tool_call {
                    let is_deep = messages.as_array().map(|a| a.len() > 3).unwrap_or(false);
                    if is_web_channel(sink)
                        && !is_deep
                        && super::super::tool_exec::is_generic_greeting(user_text)
                    {
                        if !content_buffer.is_empty() {
                            log::info!("Greeting guard: sending buffered content");
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
                            if let Err(e) = sink.send_bot_response(&final_resp).await {
                                log::warn!("stream: final response not delivered: {e}");
                            }
                        } else {
                            log::info!("Greeting guard: generic greeting");
                            let msg = "Ola! Como posso ajudar voce hoje?";
                            let final_resp = botlib::models::BotResponse::new(
                                &bot_uuid_s,
                                &session_id_s,
                                user_id.to_string(),
                                msg,
                                "web",
                            );
                            if let Err(e) = sink.send_bot_response(&final_resp).await {
                                log::warn!("stream: final response not delivered: {e}");
                            }
                        }
                    } else {
                        #[cfg(feature = "agent-vm")]
                        crate::core::bot::pipeline::native_tools::vm_turn_engine::remember(
                            session_id, llm, llm_model, llm_key, engine_type, user_text,
                        );
                        match super::stream_tools::execute_and_prepare_followup(
                            identity,
                            rx,
                            &full_response,
                            user_text,
                            messages,
                            tool_depth,
                        )
                        .await?
                        {
                            Some((follow_messages, next_final)) => {
                                return Box::pin(stream_llm_response_inner(
                                    identity, rx, &follow_messages,
                                    user_text, model_preference, tool_depth + 1, next_final,
                                )).await;
                            }
                            None => {}
                        }
                    }
                }

                {
                    // #2562 — persist reasoning alongside the message.
                    let store_content = if reasoning_accumulated.trim().is_empty() {
                        full_response.clone()
                    } else {
                        super::message_meta::encode(
                            &full_response,
                            &super::message_meta::MessageMeta {
                                reasoning: reasoning_accumulated.trim().to_string(),
                                ..Default::default()
                            },
                        )
                    };
                    let mut sm = state.session_manager.lock().await;
                    if let Err(e) = sm.save_message(session_id, user_id, 2, &store_content, 2) {
                        log::warn!("stream: turn not persisted: {e}");
                    }
                    if let Err(e) = sm.record_message_model(session_id, llm_model) {
                        log::warn!("stream: turn model not recorded: {e}");
                    }
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
                if let Err(e) = sm.save_message(session_id, user_id, 2, &fallback, 2) {
                    log::warn!("stream: fallback not persisted: {e}");
                }
            }
            let resp = botlib::models::BotResponse::new(
                bot_uuid.to_string(),
                session_id.to_string(),
                user_id.to_string(),
                &fallback,
                "web",
            );
            if let Err(e) = sink.send_bot_response(&resp).await {
                log::warn!("stream: fallback response not delivered: {e}");
            }
        }
    }

    Ok(())
}
