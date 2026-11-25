//! One streaming attempt: init/keepalive frames, cooperative stop, run caps
//! and the first-token metric.

use std::sync::Arc;
use std::time::Duration;

use botcore::shared::state::AppState;

use crate::core::bot::pipeline::sink::ChannelSink;
use crate::core::bot::runs::caps::RunBudget;

pub(super) enum AttemptOutcome {
    /// The user stopped the run; the "Stopped." reply was already sent.
    Stopped,
    /// The per-run cap was hit and reported; the run ends.
    Capped,
    /// The LLM provider returned an error (classified for the client).
    Failed {
        code: super::error_classify::LlmErrorCode,
        detail: String,
    },
    /// The stream finished; the reply is ready for the caller.
    Completed {
        full_response: String,
        content_buffer: String,
        reasoning: String,
    },
}

pub(super) struct Attempt<'a> {
    pub sink: &'a dyn ChannelSink,
    pub state: &'a Arc<AppState>,
    pub bot_uuid: uuid::Uuid,
    pub bot_uuid_s: &'a str,
    pub session_id: uuid::Uuid,
    pub session_id_s: &'a str,
    pub user_id: uuid::Uuid,
    pub llm: &'a Arc<dyn botlib::traits::LLMProvider>,
    pub prompt: &'a str,
    pub messages: &'a serde_json::Value,
    pub model: &'a str,
    pub key: &'a str,
    pub tools: &'a Option<Vec<serde_json::Value>>,
    pub style_css: &'a str,
    pub budget: &'a mut RunBudget,
    pub attempt: usize,
}

/// Aborts a spawned task when dropped, ensuring the LLM HTTP request and the
/// process-wide `call_gate` permit are released when the caller's future is
/// dropped (turn timeout, user Stop, socket closed).
struct AbortOnDrop(tokio::task::JoinHandle<()>);

impl Drop for AbortOnDrop {
    fn drop(&mut self) {
        self.0.abort();
    }
}

pub(super) async fn stream_attempt(request: Attempt<'_>) -> AttemptOutcome {
    let Attempt {
        sink,
        state,
        bot_uuid,
        bot_uuid_s,
        session_id,
        session_id_s,
        user_id,
        llm,
        prompt,
        messages,
        model,
        key,
        tools,
        style_css,
        budget,
        attempt,
    } = request;

    let mut full_response = String::new();
    let mut content_buffer = String::new();
    let mut attempt_reasoning = String::new();
    // #2583 — streaming providers split `{"__api_call__":` across chunks. Hold
    // back a suspected envelope prefix until we know whether the turn really
    // is a command call; once the trigger is complete, no further chunk may
    // be shown as chat text (the rest is machine payload).
    let mut api_call_detected = false;
    let mut held_chunks = String::new();

    let (retry_tx, mut retry_rx) = tokio::sync::mpsc::channel::<String>(100);

    let spawn_llm = llm.clone();
    let spawn_prompt = prompt.to_string();
    let spawn_messages = messages.clone();
    let spawn_model = model.to_string();
    let spawn_key = key.to_string();
    let spawn_tools = tools.clone();
    let _abort_guard = AbortOnDrop(tokio::spawn(async move {
        if let Err(e) = spawn_llm
            .generate_stream(
                &spawn_prompt,
                &spawn_messages,
                retry_tx.clone(),
                &spawn_model,
                &spawn_key,
                spawn_tools.as_ref(),
            )
            .await
        {
            log::error!("LLM stream error: {e}");
            // #2562 — send the error through the channel so the attempt
            // loop can classify it and report to the client, instead of
            // silently producing an empty response.
            let sentinel = super::error_classify::sentinel_message(&e.to_string());
            if let Err(e) = retry_tx.send(sentinel).await {
                log::warn!("attempt: LLM error sentinel not queued: {e}");
            }
        }
    }));

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
            init_msg["css"] = serde_json::Value::String(style_css.to_string());
        }
        if let Err(e) = sink.send_raw_json(&init_msg).await {
            log::warn!("attempt: init frame not delivered: {e}");
        }
    } else {
        log::warn!("LLM retry attempt {}/{}", attempt, super::LLM_RETRY_MAX);
    }

    // #1700 — first content chunk latency for this attempt.
    let llm_started_at = std::time::Instant::now();
    let mut ttft_logged = false;
    let mut keepalive = tokio::time::interval(Duration::from_millis(800));
    keepalive.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        // #1666 — Stop is cooperative: the owner checks between
        // chunks and ends the turn without running tool calls.
        if crate::core::bot::runs::bus::bus().is_cancelled(session_id_s) {
            // What was on screen stays in the conversation as the answer of
            // this turn. With nothing stored, the next question reached the
            // model together with the stopped one, and it answered both.
            let kept =
                super::message_meta::stopped_answer(&content_buffer, &attempt_reasoning);
            {
                let mut sm = state.session_manager.lock().await;
                if let Err(e) = sm.save_message(session_id, user_id, 2, &kept, 2) {
                    log::warn!("attempt: stopped answer not persisted: {e}");
                }
                if let Err(e) = sm.record_message_model(session_id, model) {
                    log::warn!("attempt: stopped answer model not recorded: {e}");
                }
            }
            let cancelled = botlib::models::BotResponse::new(
                bot_uuid_s,
                session_id_s,
                user_id.to_string(),
                "Stopped.",
                "web",
            );
            if let Err(e) = sink.send_bot_response(&cancelled).await {
                log::warn!("attempt: cancelled notice not delivered: {e}");
            }
            return AttemptOutcome::Stopped;
        }
        tokio::select! {
            chunk = retry_rx.recv() => {
                match chunk {
                    Some(chunk) => {
                        // #2562 — detect error sentinel from the LLM task.
                        if let Some(err_msg) = super::error_classify::parse_sentinel(&chunk) {
                            let code = super::error_classify::classify(err_msg);
                            return AttemptOutcome::Failed {
                                code,
                                detail: err_msg.to_string(),
                            };
                        }
                        if chunk.contains("\"__reasoning__\"") {
                            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&chunk) {
                                if let Some(r) = val.get("__reasoning__").and_then(|v| v.as_str()) {
                                    attempt_reasoning.push_str(r);
                                    let rc = botlib::models::BotResponse {
                                        bot_id: bot_uuid_s.to_string(),
                                        user_id: user_id.to_string(),
                                        session_id: session_id_s.to_string(),
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
                                    if let Err(e) = sink.send_bot_response(&rc).await {
                                        log::warn!("attempt: reasoning frame not delivered: {e}");
                                    }
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
                            api_call_detected = true;
                            full_response.push_str(&chunk);
                            continue;
                        }
                        if chunk.contains(crate::main_module::ui_plan::UI_PLAN_TRIGGER) {
                            full_response.push_str(&chunk);
                            continue;
                        }
                        full_response.push_str(&chunk);
                        if crate::core::bot::api_catalog::is_api_call_trigger(&full_response) {
                            api_call_detected = true;
                        }
                        if api_call_detected {
                            continue;
                        }
                        if crate::core::bot::api_catalog::is_api_call_trigger_start(&full_response)
                        {
                            held_chunks.push_str(&chunk);
                            continue;
                        }
                        let streamed = if held_chunks.is_empty() {
                            chunk.clone()
                        } else {
                            let pending = std::mem::take(&mut held_chunks);
                            format!("{pending}{chunk}")
                        };
                        content_buffer.push_str(&streamed);
                        // #2227 — stream partial content to the client so drawings
                        // (and text) render incrementally in the live preview
                        // instead of appearing only at completion.
                        {
                            let partial = botlib::models::BotResponse {
                                bot_id: bot_uuid_s.to_string(),
                                user_id: user_id.to_string(),
                                session_id: session_id_s.to_string(),
                                channel: "web".to_string(),
                                content: streamed,
                                message_type: botlib::message_types::MessageType::BOT_RESPONSE,
                                stream_token: None,
                                is_complete: false,
                                suggestions: Vec::new(),
                                switchers: Vec::new(),
                                context_name: None,
                                context_length: 0,
                                context_max_length: 0,
                                reasoning: String::new(),
                            };
                            if let Err(e) = sink.send_bot_response(&partial).await {
                                log::warn!("attempt: partial frame not delivered: {e}");
                            }
                        }
                        budget.observe_chars(full_response.chars().count());
                        if let Some(breach) = budget.check() {
                            let notice = format!(
                                "Run stopped — {}.",
                                crate::core::bot::runs::caps::RunBudget::describe(&breach)
                            );
                            botlog::safe_warn!(
                                "run cap: {} (session {})",
                                crate::core::bot::runs::caps::RunBudget::describe(&breach),
                                session_id_s
                            );
                            {
                                let mut sm = state.session_manager.lock().await;
                                if let Err(e) = sm.save_message(session_id, user_id, 2, &notice, 2) {
                                    log::warn!("attempt: cap notice not persisted: {e}");
                                }
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
                            if let Err(e) = sink.send_raw_json(&capped).await {
                                log::warn!("attempt: cap frame not delivered: {e}");
                            }
                            return AttemptOutcome::Capped;
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
                                model
                            );
                            crate::core::bot::runs::ttft::record_first_token(
                                &state.conn,
                                session_id,
                                bot_uuid,
                                model,
                                ttft_ms,
                            );
                        }
                    }
                    None => {
                        // A held prefix that never became an envelope was
                        // ordinary prose: include it in the final answer.
                        if !api_call_detected {
                            content_buffer.push_str(&held_chunks);
                        }
                        break;
                    }
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
                if let Err(e) = sink.send_raw_json(&ka).await {
                    log::warn!("attempt: keepalive frame not delivered: {e}");
                }
            }
        }
    }

    AttemptOutcome::Completed {
        full_response,
        content_buffer,
        reasoning: attempt_reasoning,
    }
}
