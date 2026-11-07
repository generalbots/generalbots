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

    let (retry_tx, mut retry_rx) = tokio::sync::mpsc::channel::<String>(100);

    let spawn_llm = llm.clone();
    let spawn_prompt = prompt.to_string();
    let spawn_messages = messages.clone();
    let spawn_model = model.to_string();
    let spawn_key = key.to_string();
    let spawn_tools = tools.clone();
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
            init_msg["css"] = serde_json::Value::String(style_css.to_string());
        }
        let _ = sink.send_raw_json(&init_msg).await;
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
            let cancelled = botlib::models::BotResponse::new(
                bot_uuid_s,
                session_id_s,
                user_id.to_string(),
                "Stopped.",
                "web",
            );
            let _ = sink.send_bot_response(&cancelled).await;
            return AttemptOutcome::Stopped;
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
                            botlog::safe_warn!(
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

    AttemptOutcome::Completed {
        full_response,
        content_buffer,
        reasoning: attempt_reasoning,
    }
}
